//! v2 footer and inline comments. A comment is a content row of type `comment`, so versions,
//! properties and likes reuse the generic tables; `comments` holds its anchor and thread.

use axum::{
    extract::{Path, RawQuery, State},
    http::StatusCode,
    response::Response,
    Extension, Json,
};
use chrono::{DateTime, Utc};
use kk_adf::Doc;
use kk_core::{perm, ApiError, UserId};
use serde::Deserialize;
use serde_json::{json, Value};
use sqlx::PgConnection;
use std::sync::Arc;

use crate::{
    authz::Access,
    body,
    error::{db_err, R},
    http::{paged, parse_attachment_id, parse_id, ts, JsonBody, Q},
    target, versions, AppState,
};

const STATUSES: [&str; 6] = [
    "current",
    "deleted",
    "trashed",
    "historical",
    "draft",
    "archived",
];
const RESOLUTIONS: [&str; 4] = ["resolved", "open", "dangling", "reopened"];
const ORDER: [(&str, &str); 2] = [
    ("created-date", "c.created_at"),
    ("modified-date", "v.created_at"),
];

fn kind(inline: bool) -> &'static str {
    if inline {
        "inline"
    } else {
        "footer"
    }
}

fn not_found(inline: bool) -> ApiError {
    ApiError::NotFound(format!("{} comment not found", kind(inline)))
}

#[derive(sqlx::FromRow)]
struct Row {
    id: i64,
    status: String,
    title: String,
    space_key: String,
    container_id: i64,
    container_type: String,
    container_title: String,
    parent_id: Option<i64>,
    marker_ref: Option<String>,
    selection: Option<String>,
    resolution: String,
    resolved_by: Option<i64>,
    resolved_at: Option<DateTime<Utc>>,
    v_number: i32,
    v_author: i64,
    author: i64,
    v_message: String,
    v_minor: bool,
    v_created: DateTime<Utc>,
    body: Value,
}

const SELECT: &str = "SELECT c.id, c.status, c.title, s.key AS space_key, k.container_id, \
    ct.type AS container_type, ct.title AS container_title, k.parent_id, k.marker_ref, \
    k.original_selection AS selection, k.resolution, k.resolved_by, k.resolved_at, \
    v.number AS v_number, v.author_id AS v_author, v.message AS v_message, \
    c.author_id AS author, v.minor_edit AS v_minor, v.created_at AS v_created, v.body \
    FROM content c JOIN comments k ON k.content_id = c.id JOIN spaces s ON s.id = c.space_id \
    JOIN content ct ON ct.id = k.container_id \
    JOIN content_versions v ON v.content_id = c.id AND v.number = COALESCE($1, c.version_number)";

pub fn version_json(
    number: i32,
    author: i64,
    message: &str,
    minor: bool,
    at: DateTime<Utc>,
) -> Value {
    json!({"number": number, "authorId": author.to_string(), "message": message,
           "minorEdit": minor, "createdAt": ts(at)})
}

fn render(r: &Row, repr: Option<&str>, inline: bool) -> Result<Value, ApiError> {
    let body = match repr {
        Some(f) => body::render(&r.body, f)?,
        None => json!({}),
    };
    let key = match r.container_type.as_str() {
        "attachment" => "attachmentId",
        _ => "pageId",
    };
    let mut v = json!({
        "id": r.id.to_string(),
        "status": r.status,
        "title": r.title,
        // extension: the version author changes on edit, this is who wrote the comment
        "authorId": r.author.to_string(),
        key: crate::http::public_id(&r.container_type, r.container_id),
        "version": version_json(r.v_number, r.v_author, &r.v_message, r.v_minor, r.v_created),
        "body": body,
        "_links": {"webui": format!(
            "/spaces/{}/pages/{}/{}?focusedCommentId={}",
            r.space_key, r.container_id, r.container_title.replace(' ', "+"), r.id)},
    });
    if let Some(p) = r.parent_id {
        v["parentCommentId"] = json!(p.to_string());
    }
    if inline {
        v["resolutionStatus"] = json!(r.resolution);
        if let (Some(by), Some(at)) = (r.resolved_by, r.resolved_at) {
            v["resolutionLastModifierId"] = json!(by.to_string());
            v["resolutionLastModifiedAt"] = json!(ts(at));
        }
        let mut props = json!({});
        if let Some(m) = &r.marker_ref {
            props["inlineMarkerRef"] = json!(m);
        }
        if let Some(sel) = &r.selection {
            props["inlineOriginalSelection"] = json!(sel);
        }
        v["properties"] = props;
    }
    Ok(v)
}

async fn load(
    conn: &mut PgConnection,
    id: i64,
    inline: bool,
    ver: Option<i32>,
) -> Result<Row, ApiError> {
    sqlx::query_as(&format!(
        "{SELECT} WHERE c.id = $2 AND k.kind = $3 AND c.status = 'current'"
    ))
    .bind(ver)
    .bind(id)
    .bind(kind(inline))
    .fetch_optional(conn)
    .await
    .map_err(|e| db_err(e, ""))?
    .ok_or_else(|| not_found(inline))
}

pub async fn get<const INLINE: bool>(
    State(s): State<AppState>,
    Path(id): Path<String>,
    RawQuery(raw): RawQuery,
) -> R<Json<Value>> {
    let q = Q::parse(raw);
    let repr = body::format(&q, body::SINGLE)?;
    let id = parse_id(&id)?;
    let mut conn = s.db.acquire().await?;
    let row = load(&mut conn, id, INLINE, q.int("version")?).await?;
    let mut v = render(&row, repr, INLINE)?;
    if q.get("include-version") == Some("false") {
        if let Some(o) = v.as_object_mut() {
            o.remove("version");
        }
    }
    if q.flag("include-versions")? {
        let rows: Vec<(i32, i64, String, bool, DateTime<Utc>)> = sqlx::query_as(
            "SELECT number, author_id, message, minor_edit, created_at FROM content_versions \
             WHERE content_id = $1 ORDER BY number",
        )
        .bind(id)
        .fetch_all(&mut *conn)
        .await?;
        let results: Vec<_> = rows
            .iter()
            .map(|(n, a, m, mi, at)| version_json(*n, *a, m, *mi, *at))
            .collect();
        v["versions"] = json!({"results": results, "meta": {"hasMore": false}});
    }
    v["_links"]["base"] = json!(s.site());
    Ok(Json(v))
}

struct Scope<'a> {
    inline: bool,
    path: String,
    /// `WHERE` fragment over `c`, `k`, `v` with `$2` = the id bound for this scope.
    filter: &'a str,
    id: Option<i64>,
}

async fn list(s: &AppState, a: &Access, q: Q, sc: Scope<'_>) -> R<Response> {
    let (limit, offset) = (q.limit()?, q.offset()?);
    let repr = body::format(&q, body::PRIMARY)?;
    let order = q.order_by(&ORDER, "c.id")?;
    let status = enum_list(&q, "status", &STATUSES, &["current"])?;
    let resolution = enum_list(&q, "resolution-status", &RESOLUTIONS, &RESOLUTIONS)?;
    let sql = format!(
        "{SELECT} WHERE k.kind = $3 AND c.status = ANY($4) AND k.resolution = ANY($5) AND {} \
         AND NOT (c.id = ANY($8)) AND ($9::bigint[] IS NULL OR c.space_id = ANY($9)) \
         ORDER BY {order} LIMIT $6 OFFSET $7",
        sc.filter
    );
    let rows: Vec<Row> = sqlx::query_as(&sql)
        .bind(None::<i32>)
        .bind(sc.id)
        .bind(kind(sc.inline))
        .bind(status)
        .bind(resolution)
        .bind(limit + 1)
        .bind(offset)
        .bind(a.hidden(&s.db).await?)
        .bind(a.readable_spaces())
        .fetch_all(&s.db)
        .await?;
    let rows = rows
        .iter()
        .map(|r| render(r, repr, sc.inline))
        .collect::<Result<_, _>>()?;
    Ok(paged(&q, &sc.path, &s.site(), rows, limit, offset))
}

fn enum_list(
    q: &Q,
    key: &str,
    allowed: &[&str],
    default: &[&str],
) -> Result<Vec<String>, ApiError> {
    let given = q.list(key);
    if given.is_empty() {
        return Ok(default.iter().map(|d| d.to_string()).collect());
    }
    given
        .into_iter()
        .map(|v| {
            allowed
                .contains(&v)
                .then(|| v.to_string())
                .ok_or_else(|| ApiError::BadRequest(format!("invalid {key}: {v}")))
        })
        .collect()
}

pub async fn list_all<const INLINE: bool>(
    State(s): State<AppState>,
    Extension(a): Extension<Arc<Access>>,
    RawQuery(raw): RawQuery,
) -> R<Response> {
    let path = format!("/wiki/api/v2/{}-comments", kind(INLINE));
    list(
        &s,
        &a,
        Q::parse(raw),
        Scope {
            inline: INLINE,
            path,
            filter: "$2::bigint IS NULL",
            id: None,
        },
    )
    .await
}

pub async fn list_page<const INLINE: bool>(
    State(s): State<AppState>,
    Extension(a): Extension<Arc<Access>>,
    Path(id): Path<String>,
    RawQuery(raw): RawQuery,
) -> R<Response> {
    let id = parse_id(&id)?;
    target::live(&s.db, id, "page").await?;
    let path = format!("/wiki/api/v2/pages/{id}/{}-comments", kind(INLINE));
    let filter = "k.container_id = $2 AND k.parent_id IS NULL";
    list(
        &s,
        &a,
        Q::parse(raw),
        Scope {
            inline: INLINE,
            path,
            filter,
            id: Some(id),
        },
    )
    .await
}

pub async fn list_attachment(
    State(s): State<AppState>,
    Extension(a): Extension<Arc<Access>>,
    Path(id): Path<String>,
    RawQuery(raw): RawQuery,
) -> R<Response> {
    let id = parse_attachment_id(&id)?;
    target::live(&s.db, id, "attachment").await?;
    let path = format!("/wiki/api/v2/attachments/att{id}/footer-comments");
    let filter = "k.container_id = $2 AND k.parent_id IS NULL";
    list(
        &s,
        &a,
        Q::parse(raw),
        Scope {
            inline: false,
            path,
            filter,
            id: Some(id),
        },
    )
    .await
}

pub async fn children<const INLINE: bool>(
    State(s): State<AppState>,
    Extension(a): Extension<Arc<Access>>,
    Path(id): Path<String>,
    RawQuery(raw): RawQuery,
) -> R<Response> {
    let id = parse_id(&id)?;
    load(&mut *s.db.acquire().await?, id, INLINE, None).await?;
    let path = format!("/wiki/api/v2/{}-comments/{id}/children", kind(INLINE));
    list(
        &s,
        &a,
        Q::parse(raw),
        Scope {
            inline: INLINE,
            path,
            filter: "k.parent_id = $2",
            id: Some(id),
        },
    )
    .await
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateReq {
    page_id: Option<String>,
    attachment_id: Option<String>,
    blog_post_id: Option<String>,
    custom_content_id: Option<String>,
    parent_comment_id: Option<String>,
    body: Option<Value>,
    inline_comment_properties: Option<InlineProps>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct InlineProps {
    text_selection: String,
    text_selection_match_count: usize,
    text_selection_match_index: usize,
}

async fn insert(
    tx: &mut PgConnection,
    space: i64,
    title: &str,
    adf: &Value,
    user: i64,
) -> Result<i64, ApiError> {
    let id: i64 = sqlx::query_scalar(
        "INSERT INTO content (type, status, space_id, title, author_id) \
         VALUES ('comment', 'current', $1, $2, $3) RETURNING id",
    )
    .bind(space)
    .bind(title)
    .bind(user)
    .fetch_one(&mut *tx)
    .await
    .map_err(|e| db_err(e, ""))?;
    sqlx::query(
        "INSERT INTO content_versions (content_id, number, title, body, author_id) VALUES ($1, 1, $2, $3, $4)",
    )
    .bind(id)
    .bind(title)
    .bind(adf)
    .bind(user)
    .execute(tx)
    .await
    .map_err(|e| db_err(e, ""))?;
    Ok(id)
}

/// Anchors the selection in the page's current version body without creating a page version.
async fn anchor(tx: &mut PgConnection, page: i64, p: &InlineProps) -> Result<String, ApiError> {
    let (number, body): (i32, Value) = sqlx::query_as(
        "SELECT v.number, v.body FROM content c JOIN content_versions v \
         ON v.content_id = c.id AND v.number = c.version_number WHERE c.id = $1 FOR UPDATE OF c",
    )
    .bind(page)
    .fetch_one(&mut *tx)
    .await
    .map_err(|e| db_err(e, ""))?;
    let mut doc: Doc = serde_json::from_value(body)
        .map_err(|e| ApiError::Internal(format!("stored body is not ADF: {e}")))?;
    let marker: String = sqlx::query_scalar("SELECT gen_random_uuid()::text")
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| db_err(e, ""))?;
    kk_adf::annotate(
        &mut doc,
        &p.text_selection,
        p.text_selection_match_count,
        p.text_selection_match_index,
        &marker,
    )
    .map_err(ApiError::BadRequest)?;
    sqlx::query("UPDATE content_versions SET body = $3 WHERE content_id = $1 AND number = $2")
        .bind(page)
        .bind(number)
        .bind(serde_json::to_value(&doc).map_err(|e| ApiError::Internal(e.to_string()))?)
        .execute(tx)
        .await
        .map_err(|e| db_err(e, ""))?;
    Ok(marker)
}

pub async fn create<const INLINE: bool>(
    State(s): State<AppState>,
    Extension(user): Extension<UserId>,
    Extension(access): Extension<Arc<Access>>,
    JsonBody(req): JsonBody<CreateReq>,
) -> R<(StatusCode, Json<Value>)> {
    if req.blog_post_id.is_some() || req.custom_content_id.is_some() {
        return Err(
            ApiError::BadRequest("only page and attachment comments are supported".into()).into(),
        );
    }
    let targets = [&req.page_id, &req.attachment_id, &req.parent_comment_id];
    if targets.iter().filter(|t| t.is_some()).count() != 1
        || (INLINE && req.attachment_id.is_some())
    {
        let what = if INLINE {
            "pageId or parentCommentId"
        } else {
            "exactly one of pageId, attachmentId or parentCommentId"
        };
        return Err(ApiError::BadRequest(format!("provide {what}")).into());
    }
    let (doc, repr) = body::parse_write(
        req.body
            .as_ref()
            .ok_or_else(|| ApiError::BadRequest("body is required".into()))?,
    )?;
    let adf = serde_json::to_value(&doc).map_err(|e| ApiError::Internal(e.to_string()))?;
    let mut tx = s.db.begin().await?;
    let (container, container_type, parent) =
        match (&req.page_id, &req.attachment_id, &req.parent_comment_id) {
            (Some(p), _, _) => (parse_id(p)?, "page".to_string(), None),
            (_, Some(a), _) => (parse_attachment_id(a)?, "attachment".to_string(), None),
            (_, _, Some(c)) => {
                let c = parse_id(c)?;
                let parent = load(&mut tx, c, INLINE, None).await?;
                (parent.container_id, parent.container_type, Some(c))
            }
            _ => unreachable!("one target was checked above"),
        };
    let (space, container_title) = target::live(&mut *tx, container, &container_type).await?;
    let (t, chain) = access
        .visible(&s.db, container)
        .await
        .map_err(|_| ApiError::NotFound(format!("{container_type} not found")))?;
    if !perm::page_can(
        "create_comment",
        &access.p,
        &access.grants(t.space_id),
        &chain,
    ) {
        return Err(ApiError::Forbidden("you may not comment here".into()).into());
    }
    let (marker, selection) = match (&req.inline_comment_properties, INLINE, parent) {
        (Some(p), true, None) => (
            Some(anchor(&mut tx, container, p).await?),
            Some(p.text_selection.clone()),
        ),
        (None, true, None) => {
            return Err(ApiError::BadRequest("inlineCommentProperties is required".into()).into())
        }
        _ => (None, None),
    };
    let id = insert(
        &mut tx,
        space,
        &format!("Re: {container_title}"),
        &adf,
        user.0,
    )
    .await?;
    sqlx::query(
        "INSERT INTO comments (content_id, kind, container_id, parent_id, marker_ref, original_selection) \
         VALUES ($1, $2, $3, $4, $5, $6)",
    )
    .bind(id)
    .bind(kind(INLINE))
    .bind(container)
    .bind(parent)
    .bind(marker)
    .bind(selection)
    .execute(&mut *tx)
    .await
    .map_err(|e| db_err(e, ""))?;
    let row = load(&mut tx, id, INLINE, None).await?;
    tx.commit().await?;
    let mut v = render(&row, Some(repr), INLINE)?;
    v["_links"]["base"] = json!(s.site());
    Ok((StatusCode::CREATED, Json(v)))
}

#[derive(Deserialize)]
pub struct UpdateReq {
    version: Option<VersionReq>,
    body: Option<Value>,
    resolved: Option<bool>,
}

#[derive(Deserialize)]
pub struct VersionReq {
    number: Option<i32>,
    message: Option<String>,
}

pub async fn update<const INLINE: bool>(
    State(s): State<AppState>,
    Extension(user): Extension<UserId>,
    Path(id): Path<String>,
    JsonBody(req): JsonBody<UpdateReq>,
) -> R<Json<Value>> {
    let id = parse_id(&id)?;
    if req.resolved.is_some() && !INLINE {
        return Err(ApiError::BadRequest("resolved applies to inline comments only".into()).into());
    }
    let parsed = req.body.as_ref().map(body::parse_write).transpose()?;
    if parsed.is_none() && req.resolved.is_none() {
        return Err(ApiError::BadRequest("body is required".into()).into());
    }
    let mut tx = s.db.begin().await?;
    let row = load(&mut tx, id, INLINE, None).await?;
    if parsed.is_some() {
        let author: i64 = sqlx::query_scalar("SELECT author_id FROM content WHERE id = $1")
            .bind(id)
            .fetch_one(&mut *tx)
            .await?;
        if author != user.0 && !user_is_admin(&mut tx, user.0).await? {
            return Err(ApiError::Forbidden("only the author can edit a comment".into()).into());
        }
    }
    if let Some((doc, _)) = &parsed {
        let number = req
            .version
            .as_ref()
            .and_then(|v| v.number)
            .ok_or_else(|| ApiError::BadRequest("version.number is required".into()))?;
        if number != row.v_number + 1 {
            return Err(ApiError::Conflict(format!(
                "version.number must be {}, got {number}",
                row.v_number + 1
            ))
            .into());
        }
        let adf = serde_json::to_value(doc).map_err(|e| ApiError::Internal(e.to_string()))?;
        let message = req.version.and_then(|v| v.message).unwrap_or_default();
        sqlx::query(
            "INSERT INTO content_versions (content_id, number, title, body, author_id, message) \
             VALUES ($1, $2, $3, $4, $5, $6)",
        )
        .bind(id)
        .bind(number)
        .bind(&row.title)
        .bind(adf)
        .bind(user.0)
        .bind(message)
        .execute(&mut *tx)
        .await?;
        sqlx::query("UPDATE content SET version_number = $2 WHERE id = $1")
            .bind(id)
            .bind(number)
            .execute(&mut *tx)
            .await?;
    }
    if let Some(resolved) = req.resolved {
        let next = match (resolved, row.resolution.as_str()) {
            (_, "dangling") => {
                return Err(ApiError::BadRequest(
                    "a dangling comment cannot be resolved or reopened".into(),
                )
                .into())
            }
            (true, _) => "resolved",
            (false, "resolved") => "reopened",
            (false, other) => other,
        };
        sqlx::query(
            "UPDATE comments SET resolution = $2, resolved_by = $3, resolved_at = now() WHERE content_id = $1",
        )
        .bind(id)
        .bind(next)
        .bind(user.0)
        .execute(&mut *tx)
        .await?;
    }
    let row = load(&mut tx, id, INLINE, None).await?;
    tx.commit().await?;
    let repr = parsed.map(|(_, r)| r);
    let mut v = render(&row, repr, INLINE)?;
    v["_links"]["base"] = json!(s.site());
    Ok(Json(v))
}

pub async fn delete<const INLINE: bool>(
    State(s): State<AppState>,
    Path(id): Path<String>,
) -> R<StatusCode> {
    let id = parse_id(&id)?;
    let mut tx = s.db.begin().await?;
    let row = load(&mut tx, id, INLINE, None).await?;
    if let Some(marker) = &row.marker_ref {
        unanchor(&mut tx, row.container_id, marker).await?;
    }
    sqlx::query("DELETE FROM content WHERE id = $1")
        .bind(id)
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn unanchor(tx: &mut PgConnection, page: i64, marker: &str) -> Result<(), ApiError> {
    let (number, body): (i32, Value) = sqlx::query_as(
        "SELECT v.number, v.body FROM content c JOIN content_versions v \
         ON v.content_id = c.id AND v.number = c.version_number WHERE c.id = $1 FOR UPDATE OF c",
    )
    .bind(page)
    .fetch_one(&mut *tx)
    .await
    .map_err(|e| db_err(e, ""))?;
    if let Ok(mut doc) = serde_json::from_value::<Doc>(body) {
        kk_adf::unannotate(&mut doc, marker);
        sqlx::query("UPDATE content_versions SET body = $3 WHERE content_id = $1 AND number = $2")
            .bind(page)
            .bind(number)
            .bind(serde_json::to_value(&doc).map_err(|e| ApiError::Internal(e.to_string()))?)
            .execute(tx)
            .await
            .map_err(|e| db_err(e, ""))?;
    }
    Ok(())
}

/// Open inline comments whose anchor is missing from a newly published body become dangling.
pub async fn mark_dangling(
    tx: &mut PgConnection,
    page: i64,
    adf: &Value,
) -> Result<(), sqlx::Error> {
    let Ok(doc) = serde_json::from_value::<Doc>(adf.clone()) else {
        return Ok(());
    };
    sqlx::query(
        "UPDATE comments SET resolution = 'dangling' WHERE container_id = $1 AND kind = 'inline' \
         AND marker_ref IS NOT NULL AND marker_ref <> ALL($2) AND resolution IN ('open', 'reopened')",
    )
    .bind(page)
    .bind(kk_adf::annotation_ids(&doc))
    .execute(tx)
    .await
    .map(|_| ())
}

pub async fn version_list<const INLINE: bool>(
    State(s): State<AppState>,
    Path(id): Path<String>,
    RawQuery(raw): RawQuery,
) -> R<Response> {
    let id = parse_id(&id)?;
    load(&mut *s.db.acquire().await?, id, INLINE, None).await?;
    let path = format!("/wiki/api/v2/{}-comments/{id}/versions", kind(INLINE));
    versions::list_for(&s, Q::parse(raw), id, ("comment", "comment"), &path).await
}

pub async fn version_get<const INLINE: bool>(
    State(s): State<AppState>,
    Path((id, number)): Path<(String, String)>,
) -> R<Json<Value>> {
    let id = parse_id(&id)?;
    load(&mut *s.db.acquire().await?, id, INLINE, None).await?;
    versions::detail(&s, id, &number, "comment").await
}

async fn user_is_admin(tx: &mut PgConnection, id: i64) -> Result<bool, sqlx::Error> {
    sqlx::query_scalar("SELECT is_admin FROM users WHERE id = $1")
        .bind(id)
        .fetch_one(tx)
        .await
}
