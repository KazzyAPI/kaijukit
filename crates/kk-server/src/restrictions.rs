//! Content restrictions (v1), the permission check, and the v2 page permission reads.

use std::sync::Arc;

use axum::{
    extract::{Path, RawQuery, State},
    response::Response,
    Extension, Json,
};
use kk_core::{
    perm::{self, Principal},
    ApiError, UserId,
};
use serde::Deserialize;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use sqlx::PgConnection;

use crate::{
    authz::{self, Access, Target},
    error::{db_err, R},
    http::{paged, parse_id, ts, JsonBody, Q},
    v1, AppState,
};

type Res<T> = Result<T, ApiError>;

/// A live page the caller can see; 404 otherwise.
async fn page_of(s: &AppState, a: &Access, raw: &str) -> Res<(i64, Target)> {
    let id = parse_id(raw)?;
    let live: Option<i64> = sqlx::query_scalar(
        "SELECT id FROM content WHERE id = $1 AND type = 'page' AND status NOT IN ('deleted', 'historical')",
    )
    .bind(id)
    .fetch_optional(&s.db)
    .await
    .map_err(|e| db_err(e, ""))?;
    live.ok_or_else(|| ApiError::NotFound("content not found".into()))?;
    let (t, _) = a.visible(&s.db, id).await?;
    Ok((id, t))
}

async fn require_restrict(s: &AppState, a: &Access, t: &Target, id: i64) -> Res<()> {
    let chain = authz::chain(&s.db, id).await?;
    if perm::page_can("restrict_content", &a.p, &a.grants(t.space_id), &chain) {
        Ok(())
    } else {
        Err(ApiError::Forbidden(
            "you may not change restrictions of this page".into(),
        ))
    }
}

fn op_key(raw: &str) -> Res<&str> {
    match raw {
        "read" | "update" => Ok(raw),
        _ => Err(ApiError::BadRequest(
            "operationKey must be read or update".into(),
        )),
    }
}

type Row = (String, String, i64);

async fn subjects(db: impl sqlx::PgExecutor<'_>, id: i64) -> Res<Vec<Row>> {
    sqlx::query_as(
        "SELECT operation, subject_type, subject_id FROM content_restrictions WHERE content_id = $1 \
         ORDER BY operation, subject_type, subject_id",
    )
    .bind(id)
    .fetch_all(db)
    .await
    .map_err(|e| db_err(e, ""))
}

fn wants(expand: &[&str], op: &str, kind: &str) -> bool {
    expand
        .iter()
        .any(|e| *e == format!("restrictions.{kind}") || *e == format!("{op}.restrictions.{kind}"))
}

async fn item(s: &AppState, id: i64, op: &str, rows: &[Row], expand: &[&str]) -> Res<Value> {
    let base = format!(
        "{}/rest/api/content/{id}/restriction/byOperation/{op}",
        s.site()
    );
    let mut v = json!({
        "operation": op,
        "_expandable": {"restrictions": "", "content": format!("/rest/api/content/{id}")},
        "_links": {"self": base, "base": s.site(), "context": "/wiki"},
    });
    let mine = |k: &str| {
        rows.iter()
            .filter(|r| r.0 == op && r.1 == k)
            .map(|r| r.2)
            .collect::<Vec<_>>()
    };
    let (wu, wg) = (wants(expand, op, "user"), wants(expand, op, "group"));
    if wu || wg {
        let mut r = json!({"_expandable": {}});
        if wu {
            let u = v1::users(&s.db, &mine("user")).await?;
            r["user"] = json!({"results": u.clone(), "start": 0, "limit": 200, "size": u.len()});
        } else {
            r["_expandable"]["user"] = json!(format!("{base}/user"));
        }
        if wg {
            let g = v1::groups(&s.db, &s.site(), &mine("group")).await?;
            r["group"] = json!({"results": g.clone(), "start": 0, "limit": 200, "size": g.len()});
        } else {
            r["_expandable"]["group"] = json!(format!("{base}/group"));
        }
        v["restrictions"] = r;
    } else {
        v["_expandable"]["restrictions"] = json!(format!("{base}/restrictions"));
    }
    Ok(v)
}

fn expand_of(q: &Q) -> Vec<&str> {
    q.list("expand")
}

async fn array(s: &AppState, id: i64, q: &Q, path: &str) -> Res<Value> {
    let rows = subjects(&s.db, id).await?;
    let expand = expand_of(q);
    let (start, limit) = v1::window(q, 100)?;
    let mut results = Vec::new();
    for op in ["read", "update"] {
        results.push(item(s, id, op, &rows, &expand).await?);
    }
    let hash: String = Sha256::digest(format!("{rows:?}").as_bytes())
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect();
    let window: Vec<Value> = results
        .into_iter()
        .skip(start as usize)
        .take(limit as usize)
        .collect();
    Ok(json!({
        "results": window.clone(), "start": start, "limit": limit, "size": window.len(),
        "restrictionsHash": hash,
        "_links": {"base": s.site(), "context": "/wiki", "self": format!("{}{path}", s.site().trim_end_matches("/wiki"))},
    }))
}

pub async fn get(
    State(s): State<AppState>,
    Extension(a): Extension<Arc<Access>>,
    Path(id): Path<String>,
    RawQuery(raw): RawQuery,
) -> R<Json<Value>> {
    let q = Q::parse(raw);
    let (id, _) = page_of(&s, &a, &id).await?;
    Ok(Json(
        array(&s, id, &q, &format!("/rest/api/content/{id}/restriction")).await?,
    ))
}

pub async fn by_operation(
    State(s): State<AppState>,
    Extension(a): Extension<Arc<Access>>,
    Path(id): Path<String>,
    RawQuery(raw): RawQuery,
) -> R<Json<Value>> {
    let q = Q::parse(raw);
    let (id, _) = page_of(&s, &a, &id).await?;
    let (rows, expand) = (subjects(&s.db, id).await?, expand_of(&q));
    Ok(Json(json!({
        "read": item(&s, id, "read", &rows, &expand).await?,
        "update": item(&s, id, "update", &rows, &expand).await?,
    })))
}

pub async fn for_operation(
    State(s): State<AppState>,
    Extension(a): Extension<Arc<Access>>,
    Path((id, op)): Path<(String, String)>,
    RawQuery(raw): RawQuery,
) -> R<Json<Value>> {
    let q = Q::parse(raw);
    let op = op_key(&op)?;
    let (id, _) = page_of(&s, &a, &id).await?;
    let rows = subjects(&s.db, id).await?;
    Ok(Json(item(&s, id, op, &rows, &expand_of(&q)).await?))
}

#[derive(Deserialize)]
#[serde(untagged)]
pub enum Many<T> {
    Wrapped { results: Vec<T> },
    List(Vec<T>),
}

impl<T> Many<T> {
    fn into_vec(self) -> Vec<T> {
        match self {
            Many::Wrapped { results } | Many::List(results) => results,
        }
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserIn {
    account_id: String,
}

#[derive(Deserialize)]
pub struct GroupIn {
    id: String,
}

#[derive(Deserialize)]
pub struct Subjects {
    user: Option<Many<UserIn>>,
    group: Option<Many<GroupIn>>,
}

#[derive(Deserialize)]
pub struct ContentRef {
    id: String,
}

#[derive(Deserialize)]
pub struct Entry {
    operation: String,
    restrictions: Subjects,
    content: Option<ContentRef>,
}

pub type Body = Many<Entry>;

async fn exists(tx: &mut PgConnection, table: &str, raw: &str) -> Res<i64> {
    let id = parse_id(raw).map_err(|_| ApiError::BadRequest(format!("unknown subject {raw}")))?;
    sqlx::query_scalar::<_, i64>(&format!("SELECT id FROM {table} WHERE id = $1"))
        .bind(id)
        .fetch_optional(tx)
        .await
        .map_err(|e| db_err(e, ""))?
        .ok_or_else(|| ApiError::BadRequest(format!("unknown subject {raw}")))
}

async fn add_row(
    tx: &mut PgConnection,
    id: i64,
    op: &str,
    kind: &str,
    subject: i64,
    by: i64,
) -> Res<()> {
    sqlx::query(
        "INSERT INTO content_restrictions (content_id, operation, subject_type, subject_id, created_by) \
         VALUES ($1, $2, $3, $4, $5) ON CONFLICT DO NOTHING",
    )
    .bind(id)
    .bind(op)
    .bind(kind)
    .bind(subject)
    .bind(by)
    .execute(tx)
    .await
    .map(|_| ())
    .map_err(|e| db_err(e, ""))
}

/// Rejects a change that takes the caller's own view or edit access away.
async fn no_lockout(
    tx: &mut PgConnection,
    a: &Access,
    space: i64,
    id: i64,
    before: (bool, bool),
) -> Res<()> {
    let chain = authz::chain(&mut *tx, id).await?;
    let g = a.grants(space);
    let after = (
        perm::can_view(&a.p, &g, &chain),
        perm::can_update(&a.p, &g, &chain),
    );
    if (before.0 && !after.0) || (before.1 && !after.1) {
        return Err(ApiError::BadRequest(
            "this change would remove your own access to the page".into(),
        ));
    }
    Ok(())
}

async fn access_now(tx: &mut PgConnection, a: &Access, space: i64, id: i64) -> Res<(bool, bool)> {
    let chain = authz::chain(&mut *tx, id).await?;
    let g = a.grants(space);
    Ok((
        perm::can_view(&a.p, &g, &chain),
        perm::can_update(&a.p, &g, &chain),
    ))
}

async fn change(
    s: &AppState,
    a: &Access,
    user: UserId,
    raw_id: &str,
    q: &Q,
    body: Body,
    replace: bool,
) -> R<Json<Value>> {
    let (id, t) = page_of(s, a, raw_id).await?;
    require_restrict(s, a, &t, id).await?;
    let entries = body.into_vec();
    if entries.is_empty() {
        return Err(ApiError::BadRequest("supply at least one restriction".into()).into());
    }
    let mut seen = Vec::new();
    let mut tx = s.db.begin().await?;
    let before = access_now(&mut tx, a, t.space_id, id).await?;
    for e in entries {
        let op = op_key(&e.operation)?.to_string();
        if seen.contains(&op) {
            return Err(ApiError::BadRequest("one entry per operation".into()).into());
        }
        seen.push(op.clone());
        if e.content.is_some_and(|c| c.id != id.to_string()) {
            return Err(ApiError::BadRequest("content.id does not match the path".into()).into());
        }
        if e.restrictions.user.is_none() && e.restrictions.group.is_none() {
            return Err(ApiError::BadRequest("restrictions needs user or group".into()).into());
        }
        for (kind, ids) in [
            (
                "user",
                e.restrictions.user.map(|m| {
                    m.into_vec()
                        .into_iter()
                        .map(|u| u.account_id)
                        .collect::<Vec<_>>()
                }),
            ),
            (
                "group",
                e.restrictions
                    .group
                    .map(|m| m.into_vec().into_iter().map(|g| g.id).collect()),
            ),
        ] {
            let Some(ids) = ids else { continue };
            if replace {
                sqlx::query("DELETE FROM content_restrictions WHERE content_id = $1 AND operation = $2 AND subject_type = $3")
                    .bind(id)
                    .bind(&op)
                    .bind(kind)
                    .execute(&mut *tx)
                    .await?;
            }
            for raw in ids {
                let sid = exists(
                    &mut tx,
                    if kind == "user" { "users" } else { "groups" },
                    &raw,
                )
                .await?;
                add_row(&mut tx, id, &op, kind, sid, user.0).await?;
            }
        }
    }
    no_lockout(&mut tx, a, t.space_id, id, before).await?;
    tx.commit().await?;
    Ok(Json(
        array(s, id, q, &format!("/rest/api/content/{id}/restriction")).await?,
    ))
}

pub async fn add(
    State(s): State<AppState>,
    Extension(a): Extension<Arc<Access>>,
    Extension(user): Extension<UserId>,
    Path(id): Path<String>,
    RawQuery(raw): RawQuery,
    JsonBody(body): JsonBody<Body>,
) -> R<Json<Value>> {
    change(&s, &a, user, &id, &Q::parse(raw), body, false).await
}

pub async fn update(
    State(s): State<AppState>,
    Extension(a): Extension<Arc<Access>>,
    Extension(user): Extension<UserId>,
    Path(id): Path<String>,
    RawQuery(raw): RawQuery,
    JsonBody(body): JsonBody<Body>,
) -> R<Json<Value>> {
    change(&s, &a, user, &id, &Q::parse(raw), body, true).await
}

pub async fn delete_all(
    State(s): State<AppState>,
    Extension(a): Extension<Arc<Access>>,
    Path(id): Path<String>,
    RawQuery(raw): RawQuery,
) -> R<Json<Value>> {
    let q = Q::parse(raw);
    let (id, t) = page_of(&s, &a, &id).await?;
    require_restrict(&s, &a, &t, id).await?;
    sqlx::query("DELETE FROM content_restrictions WHERE content_id = $1")
        .bind(id)
        .execute(&s.db)
        .await?;
    Ok(Json(
        array(&s, id, &q, &format!("/rest/api/content/{id}/restriction")).await?,
    ))
}

/// One user or group on one operation: add, remove or look up.
#[allow(clippy::too_many_arguments)]
async fn one(
    s: &AppState,
    a: &Access,
    user: UserId,
    raw_id: &str,
    op: &str,
    kind: &str,
    subject: &str,
    verb: &str,
) -> R<axum::http::StatusCode> {
    let op = op_key(op)?;
    let (id, t) = page_of(s, a, raw_id).await?;
    if verb == "get" {
        let sid = parse_id(subject).map_err(|_| ApiError::NotFound("subject not found".into()))?;
        let hit: Option<i64> = sqlx::query_scalar(
            "SELECT subject_id FROM content_restrictions WHERE content_id = $1 AND operation = $2 AND subject_type = $3 AND subject_id = $4",
        )
        .bind(id)
        .bind(op)
        .bind(kind)
        .bind(sid)
        .fetch_optional(&s.db)
        .await?;
        return hit
            .map(|_| axum::http::StatusCode::OK)
            .ok_or_else(|| ApiError::NotFound("not restricted".into()).into());
    }
    require_restrict(s, a, &t, id).await?;
    let mut tx = s.db.begin().await?;
    let before = access_now(&mut tx, a, t.space_id, id).await?;
    if verb == "put" {
        let sid = exists(
            &mut tx,
            if kind == "user" { "users" } else { "groups" },
            subject,
        )
        .await?;
        add_row(&mut tx, id, op, kind, sid, user.0).await?;
    } else {
        let sid = parse_id(subject).map_err(|_| ApiError::BadRequest("invalid subject".into()))?;
        sqlx::query("DELETE FROM content_restrictions WHERE content_id = $1 AND operation = $2 AND subject_type = $3 AND subject_id = $4")
            .bind(id)
            .bind(op)
            .bind(kind)
            .bind(sid)
            .execute(&mut *tx)
            .await?;
    }
    no_lockout(&mut tx, a, t.space_id, id, before).await?;
    tx.commit().await?;
    Ok(axum::http::StatusCode::OK)
}

fn account(q: &Q) -> Res<&str> {
    q.get("accountId").ok_or_else(|| {
        ApiError::BadRequest("accountId is required (key and username are not supported)".into())
    })
}

macro_rules! user_route {
    ($name:ident, $verb:literal) => {
        pub async fn $name(
            State(s): State<AppState>,
            Extension(a): Extension<Arc<Access>>,
            Extension(user): Extension<UserId>,
            Path((id, op)): Path<(String, String)>,
            RawQuery(raw): RawQuery,
        ) -> R<axum::http::StatusCode> {
            let q = Q::parse(raw);
            one(&s, &a, user, &id, &op, "user", account(&q)?, $verb).await
        }
    };
}
user_route!(user_get, "get");
user_route!(user_put, "put");
user_route!(user_delete, "delete");

macro_rules! group_route {
    ($name:ident, $verb:literal) => {
        pub async fn $name(
            State(s): State<AppState>,
            Extension(a): Extension<Arc<Access>>,
            Extension(user): Extension<UserId>,
            Path((id, op, gid)): Path<(String, String, String)>,
        ) -> R<axum::http::StatusCode> {
            one(&s, &a, user, &id, &op, "group", &gid, $verb).await
        }
    };
}
group_route!(group_get, "get");
group_route!(group_put, "put");
group_route!(group_delete, "delete");

#[derive(Deserialize)]
pub struct CheckReq {
    subject: SubjectIn,
    operation: String,
}

#[derive(Deserialize)]
pub struct SubjectIn {
    #[serde(rename = "type")]
    kind: String,
    identifier: String,
}

fn message(text: &str) -> Value {
    json!({"translation": text, "args": []})
}

pub async fn check(
    State(s): State<AppState>,
    Extension(a): Extension<Arc<Access>>,
    Path(id): Path<String>,
    JsonBody(req): JsonBody<CheckReq>,
) -> R<Json<Value>> {
    let id = parse_id(&id)?;
    let (t, chain) = a.visible(&s.db, id).await?;
    if !["read", "update", "delete"].contains(&req.operation.as_str()) {
        return Err(ApiError::BadRequest("operation must be read, update or delete".into()).into());
    }
    let who = if req.subject.kind == "group" {
        "Group"
    } else {
        "User"
    };
    let answer = |ok: bool, what: &str| {
        let errors = if ok {
            vec![]
        } else {
            vec![message(&format!(
                "{who} does not have permission to the {what}"
            ))]
        };
        Json(json!({"hasPermission": ok, "errors": errors, "_links": {"base": s.site()}}))
    };
    let subject_id = match (req.subject.kind.as_str(), req.subject.identifier.as_str()) {
        ("user", "anonymous") => {
            return Ok(Json(json!({
                "hasPermission": false,
                "errors": [message("Anonymous users are not allowed to use Confluence")],
            })))
        }
        ("user" | "group", raw) => {
            parse_id(raw).map_err(|_| ApiError::BadRequest("identifier must be an id".into()))?
        }
        _ => return Err(ApiError::BadRequest("subject.type must be user or group".into()).into()),
    };
    let (p, g) = if req.subject.kind == "group" {
        sqlx::query_scalar::<_, i64>("SELECT id FROM groups WHERE id = $1")
            .bind(subject_id)
            .fetch_optional(&s.db)
            .await?
            .ok_or_else(|| ApiError::NotFound("group not found".into()))?;
        let p = Principal {
            id: 0,
            admin: false,
            groups: [subject_id].into(),
        };
        let g = Access::grants_for_group(&s.db, subject_id, t.space_id).await?;
        (p, g)
    } else {
        sqlx::query_scalar::<_, i64>("SELECT id FROM users WHERE id = $1")
            .bind(subject_id)
            .fetch_optional(&s.db)
            .await?
            .ok_or_else(|| ApiError::NotFound("user not found".into()))?;
        let other = Access::load(&s.db, UserId(subject_id)).await?;
        let g = other.grants(t.space_id);
        (other.p, g)
    };
    if !p.admin && !g.has("read", "space") {
        return Ok(answer(false, "space"));
    }
    let ok = match req.operation.as_str() {
        "read" => perm::can_view(&p, &g, &chain),
        op => perm::page_can(op, &p, &g, &chain),
    };
    // a failure with the space grants alone is a space problem, otherwise a content problem
    let space_ok = perm::page_can(&req.operation, &p, &g, &[]);
    Ok(answer(ok, if space_ok { "content" } else { "space" }))
}

pub async fn page_permissions(
    State(s): State<AppState>,
    Extension(a): Extension<Arc<Access>>,
    Path(id): Path<String>,
    RawQuery(raw): RawQuery,
) -> R<Response> {
    let q = Q::parse(raw);
    let (limit, offset) = (q.limit()?, q.offset()?);
    let (id, _) = page_of(&s, &a, &id).await?;
    #[allow(clippy::type_complexity)]
    let rows: Vec<(String, String, i64, chrono::DateTime<chrono::Utc>, Option<i64>)> = sqlx::query_as(
        "SELECT operation, subject_type, subject_id, created_at, created_by FROM content_restrictions \
         WHERE content_id = $1 AND ($2::text IS NULL OR operation = $2) \
         AND ($3::text IS NULL OR subject_type = $3) AND ($4::text IS NULL OR subject_id::text = $4) \
         ORDER BY operation, subject_type, subject_id LIMIT $5 OFFSET $6",
    )
    .bind(id)
    .bind(q.get("operation"))
    .bind(q.get("principal-type"))
    .bind(q.get("principal-id"))
    .bind(limit + 1)
    .bind(offset)
    .fetch_all(&s.db)
    .await?;
    let rows = rows
        .iter()
        .map(|(op, kind, sid, at, by)| {
            let mut v = json!({
                "operation": op,
                "principal": {"principalType": kind, "principalId": sid.to_string()},
                "createdAt": ts(*at),
            });
            if let Some(b) = by {
                v["createdBy"] = json!(b.to_string());
                v["createdByAaid"] = json!(b.to_string());
            }
            v
        })
        .collect();
    Ok(paged(
        &q,
        &format!("/wiki/api/v2/pages/{id}/permissions"),
        &s.site(),
        rows,
        limit,
        offset,
    ))
}

pub async fn restriction_state(
    State(s): State<AppState>,
    Extension(a): Extension<Arc<Access>>,
    Path(id): Path<String>,
) -> R<Json<Value>> {
    let (id, _) = page_of(&s, &a, &id).await?;
    let last: Option<(String, chrono::DateTime<chrono::Utc>, Option<i64>)> = sqlx::query_as(
        "SELECT CASE WHEN bool_or(operation = 'read') THEN 'VIEW_RESTRICTED' ELSE 'EDIT_RESTRICTED' END, \
         max(created_at), (array_agg(created_by ORDER BY created_at DESC))[1] \
         FROM content_restrictions WHERE content_id = $1 HAVING count(*) > 0",
    )
    .bind(id)
    .fetch_optional(&s.db)
    .await?;
    Ok(Json(match last {
        None => json!({"state": "OPEN"}),
        Some((state, at, by)) => {
            let mut v = json!({"state": state, "createdAt": ts(at)});
            if let Some(b) = by {
                v["createdBy"] = json!(b.to_string());
                v["createdByAaid"] = json!(b.to_string());
            }
            v
        }
    }))
}
