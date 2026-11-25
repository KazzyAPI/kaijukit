//! Labels on pages, attachments, comments and spaces. Reads are v2; writes are the v1 label endpoints.

use axum::{
    extract::{Path, RawQuery, State},
    http::StatusCode,
    response::Response,
    Json,
};
use kk_core::ApiError;
use serde::Deserialize;
use serde_json::{json, Value};
use sqlx::PgPool;

use crate::{
    error::{db_err, R},
    http::{paged, Q},
    target, AppState,
};

const PREFIXES: [&str; 4] = ["my", "team", "global", "system"];
const SORTS: [(&str, &str); 3] = [
    ("id", "l.id"),
    ("name", "l.name"),
    ("created-date", "x.created_at"),
];

#[derive(Clone, Copy)]
pub enum Scope {
    Content,
    Space,
    SpaceContent,
    All,
}

impl Scope {
    fn from(self) -> &'static str {
        match self {
            Scope::Content => "content_labels x JOIN labels l ON l.id = x.label_id WHERE x.content_id = $1",
            Scope::Space => "space_labels x JOIN labels l ON l.id = x.label_id WHERE x.space_id = $1",
            Scope::SpaceContent => {
                "labels l JOIN (SELECT cl.label_id, min(cl.created_at) AS created_at FROM content_labels cl \
                 JOIN content c ON c.id = cl.content_id WHERE c.space_id = $1 \
                 AND c.status IN ('current', 'draft', 'archived') GROUP BY cl.label_id) x \
                 ON x.label_id = l.id WHERE true"
            }
            Scope::All => "labels l JOIN (SELECT id, now() AS created_at FROM labels) x ON x.id = l.id WHERE $1::bigint IS NULL",
        }
    }
}

fn label_json(id: i64, prefix: &str, name: &str) -> Value {
    json!({"id": id.to_string(), "prefix": prefix, "name": name})
}

async fn list(s: &AppState, q: Q, scope: Scope, id: Option<i64>, path: &str) -> R<Response> {
    let (limit, offset) = (q.limit()?, q.offset()?);
    let order = q.order_by(&SORTS, "l.id")?;
    let prefixes: Vec<String> = q.list("prefix").into_iter().map(String::from).collect();
    if let Some(bad) = prefixes.iter().find(|p| !PREFIXES.contains(&p.as_str())) {
        return Err(ApiError::BadRequest(format!("invalid prefix: {bad}")).into());
    }
    let ids = q.ids("label-id")?.unwrap_or_default();
    let sql = format!(
        "SELECT l.id, l.prefix, l.name FROM {} AND (cardinality($2::text[]) = 0 OR l.prefix = ANY($2)) \
         AND (cardinality($3::bigint[]) = 0 OR l.id = ANY($3)) ORDER BY {order} LIMIT $4 OFFSET $5",
        scope.from()
    );
    let rows: Vec<(i64, String, String)> = sqlx::query_as(&sql)
        .bind(id)
        .bind(prefixes)
        .bind(ids)
        .bind(limit + 1)
        .bind(offset)
        .fetch_all(&s.db)
        .await?;
    let rows = rows.iter().map(|(i, p, n)| label_json(*i, p, n)).collect();
    Ok(paged(&q, path, &s.site(), rows, limit, offset))
}

pub async fn of_content<const TY: u8>(
    State(s): State<AppState>,
    Path(id): Path<String>,
    RawQuery(raw): RawQuery,
) -> R<Response> {
    let k = crate::kinds::of(TY);
    let (ty, seg) = (k.ty, k.seg);
    let id = target::owner(&s.db, &id, ty).await?;
    let path = format!(
        "/wiki/api/v2/{seg}/{}/labels",
        crate::http::public_id(ty, id)
    );
    list(&s, Q::parse(raw), Scope::Content, Some(id), &path).await
}

pub async fn of_space(
    State(s): State<AppState>,
    Path(id): Path<String>,
    RawQuery(raw): RawQuery,
) -> R<Response> {
    let id = target::space(&s.db, &id).await?;
    list(
        &s,
        Q::parse(raw),
        Scope::Space,
        Some(id),
        &format!("/wiki/api/v2/spaces/{id}/labels"),
    )
    .await
}

pub async fn of_space_content(
    State(s): State<AppState>,
    Path(id): Path<String>,
    RawQuery(raw): RawQuery,
) -> R<Response> {
    let id = target::space(&s.db, &id).await?;
    let path = format!("/wiki/api/v2/spaces/{id}/content/labels");
    list(&s, Q::parse(raw), Scope::SpaceContent, Some(id), &path).await
}

pub async fn all(State(s): State<AppState>, RawQuery(raw): RawQuery) -> R<Response> {
    list(&s, Q::parse(raw), Scope::All, None, "/wiki/api/v2/labels").await
}

#[derive(Deserialize)]
pub struct LabelIn {
    prefix: Option<String>,
    name: String,
}

#[derive(Deserialize)]
#[serde(untagged)]
pub enum AddReq {
    Many(Vec<LabelIn>),
    One(LabelIn),
}

fn valid(l: &LabelIn, default_prefix: &str) -> Result<(String, String), ApiError> {
    let prefix = l.prefix.clone().unwrap_or_else(|| default_prefix.into());
    let name = l.name.trim().to_lowercase();
    if !PREFIXES.contains(&prefix.as_str()) {
        return Err(ApiError::BadRequest(format!("invalid prefix: {prefix}")));
    }
    if name.is_empty() || name.chars().count() > 255 || name.contains(char::is_whitespace) {
        return Err(ApiError::BadRequest(
            "label name must be 1 to 255 characters without whitespace".into(),
        ));
    }
    Ok((prefix, name))
}

async fn v1_list(db: &PgPool, table: &str, col: &str, id: i64, base: &str) -> R<Json<Value>> {
    let rows: Vec<(i64, String, String)> = sqlx::query_as(&format!(
        "SELECT l.id, l.prefix, l.name FROM {table} x JOIN labels l ON l.id = x.label_id \
         WHERE x.{col} = $1 ORDER BY l.id"
    ))
    .bind(id)
    .fetch_all(db)
    .await?;
    let results: Vec<_> = rows
        .iter()
        .map(|(i, p, n)| json!({"id": i.to_string(), "prefix": p, "name": n, "label": n}))
        .collect();
    Ok(Json(json!({
        "results": results, "start": 0, "limit": results.len(), "size": results.len(),
        "_links": {"base": base, "context": "/wiki"},
    })))
}

async fn add(
    s: &AppState,
    (table, col): (&str, &str),
    id: i64,
    req: AddReq,
    default_prefix: &str,
) -> R<Json<Value>> {
    let labels = match req {
        AddReq::Many(v) => v,
        AddReq::One(l) => vec![l],
    };
    let mut tx = s.db.begin().await?;
    for l in &labels {
        let (prefix, name) = valid(l, default_prefix)?;
        let label: i64 = sqlx::query_scalar(
            "INSERT INTO labels (prefix, name) VALUES ($1, $2) \
             ON CONFLICT (prefix, name) DO UPDATE SET name = EXCLUDED.name RETURNING id",
        )
        .bind(prefix)
        .bind(name)
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| db_err(e, ""))?;
        sqlx::query(&format!(
            "INSERT INTO {table} ({col}, label_id) VALUES ($1, $2) ON CONFLICT DO NOTHING"
        ))
        .bind(id)
        .bind(label)
        .execute(&mut *tx)
        .await?;
    }
    tx.commit().await?;
    v1_list(&s.db, table, col, id, &s.site()).await
}

async fn remove(db: &PgPool, (table, col): (&str, &str), id: i64, name: &str) -> R<StatusCode> {
    let done = sqlx::query(&format!(
        "DELETE FROM {table} WHERE {col} = $1 AND label_id IN \
         (SELECT id FROM labels WHERE name = lower($2))"
    ))
    .bind(id)
    .bind(name)
    .execute(db)
    .await?;
    if done.rows_affected() == 0 {
        return Err(ApiError::NotFound(format!("label {name} not found")).into());
    }
    Ok(StatusCode::NO_CONTENT)
}

const CONTENT: (&str, &str) = ("content_labels", "content_id");
const SPACE: (&str, &str) = ("space_labels", "space_id");

async fn any_content(s: &AppState, raw: &str) -> Result<i64, ApiError> {
    let id = crate::http::parse_attachment_id(raw)?;
    let ty: Option<String> = sqlx::query_scalar(
        "SELECT type FROM content WHERE id = $1 AND status IN ('current', 'draft', 'archived')",
    )
    .bind(id)
    .fetch_optional(&s.db)
    .await
    .map_err(|e| crate::error::db_err(e, ""))?;
    ty.map(|_| id)
        .ok_or_else(|| ApiError::NotFound("content not found".into()))
}

async fn space_by_key(s: &AppState, key: &str) -> Result<i64, ApiError> {
    sqlx::query_scalar("SELECT id FROM spaces WHERE upper(key) = upper($1) AND status = 'current'")
        .bind(key)
        .fetch_optional(&s.db)
        .await
        .map_err(|e| crate::error::db_err(e, ""))?
        .ok_or_else(|| ApiError::NotFound("space not found".into()))
}

pub async fn add_content(
    State(s): State<AppState>,
    Path(id): Path<String>,
    crate::http::JsonBody(req): crate::http::JsonBody<AddReq>,
) -> R<Json<Value>> {
    add(&s, CONTENT, any_content(&s, &id).await?, req, "global").await
}

#[derive(Deserialize)]
pub struct NameQ {
    name: String,
}

pub async fn remove_content_by_query(
    State(s): State<AppState>,
    Path(id): Path<String>,
    axum::extract::Query(q): axum::extract::Query<NameQ>,
) -> R<StatusCode> {
    remove(&s.db, CONTENT, any_content(&s, &id).await?, &q.name).await
}

pub async fn remove_content(
    State(s): State<AppState>,
    Path((id, name)): Path<(String, String)>,
) -> R<StatusCode> {
    remove(&s.db, CONTENT, any_content(&s, &id).await?, &name).await
}

pub async fn add_space(
    State(s): State<AppState>,
    Path(key): Path<String>,
    crate::http::JsonBody(req): crate::http::JsonBody<AddReq>,
) -> R<Json<Value>> {
    add(&s, SPACE, space_by_key(&s, &key).await?, req, "team").await
}

pub async fn remove_space(
    State(s): State<AppState>,
    Path(key): Path<String>,
    axum::extract::Query(q): axum::extract::Query<NameQ>,
) -> R<StatusCode> {
    remove(&s.db, SPACE, space_by_key(&s, &key).await?, &q.name).await
}
