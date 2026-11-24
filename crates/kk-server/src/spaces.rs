use axum::{
    extract::{Path, RawQuery, State},
    http::StatusCode,
    response::Response,
    Extension, Json,
};
use chrono::{DateTime, Utc};
use kk_core::{ApiError, UserId};
use serde::Deserialize;
use serde_json::{json, Value};

use crate::{
    error::{db_err, R},
    http::{paged, parse_id, ts, JsonBody, Q},
    pages::insert_page,
    AppState,
};

const COLS: &str =
    "id, key, name, type AS kind, status, description, author_id, homepage_id, created_at";
const TYPES: &[&str] = &[
    "global",
    "collaboration",
    "knowledge_base",
    "personal",
    "system",
    "onboarding",
    "xflow_sample_space",
];
const STATUSES: &[&str] = &["current", "archived", "trashed"];

#[derive(sqlx::FromRow)]
struct Space {
    id: i64,
    key: String,
    name: String,
    kind: String,
    status: String,
    description: String,
    author_id: i64,
    homepage_id: Option<i64>,
    created_at: DateTime<Utc>,
}

fn space_json(s: &Space, desc_format: Option<&str>) -> Value {
    let mut v = json!({
        "id": s.id.to_string(),
        "key": s.key,
        "name": s.name,
        "type": s.kind,
        "status": s.status,
        "authorId": s.author_id.to_string(),
        "spaceOwnerId": s.author_id.to_string(),
        "createdAt": ts(s.created_at),
        "homepageId": s.homepage_id.map(|i| i.to_string()),
        "_links": {"webui": format!("/spaces/{}", s.key)},
    });
    if let Some(f) = desc_format {
        v["description"] = json!({ f: {"representation": f, "value": s.description} });
    }
    v
}

fn desc_format(q: &Q) -> Result<Option<&str>, ApiError> {
    match q.get("description-format") {
        Some(f) if !["plain", "view"].contains(&f) => {
            Err(ApiError::BadRequest("invalid description-format".into()))
        }
        f => Ok(f),
    }
}

fn one_of<'a>(q: &'a Q, k: &str, allowed: &[&str]) -> Result<Option<&'a str>, ApiError> {
    match q.get(k) {
        Some(v) if !allowed.contains(&v) => Err(ApiError::BadRequest(format!("invalid {k}: {v}"))),
        v => Ok(v),
    }
}

pub async fn list(State(s): State<AppState>, RawQuery(raw): RawQuery) -> R<Response> {
    let q = Q::parse(raw);
    let (limit, offset) = (q.limit()?, q.offset()?);
    let order = q.order_by(&[("id", "id"), ("key", "key"), ("name", "name")], "id")?;
    let keys = q.list("keys");
    let sql = format!(
        "SELECT {COLS} FROM spaces WHERE ($1::bigint[] IS NULL OR id = ANY($1)) \
         AND ($2::text[] IS NULL OR key = ANY($2)) AND ($3::text IS NULL OR type = $3) \
         AND status = $4 ORDER BY {order} LIMIT $5 OFFSET $6"
    );
    let fmt = desc_format(&q)?;
    let rows: Vec<Space> = sqlx::query_as(&sql)
        .bind(q.ids("ids")?)
        .bind((!keys.is_empty()).then_some(keys))
        .bind(one_of(&q, "type", TYPES)?)
        .bind(one_of(&q, "status", STATUSES)?.unwrap_or("current"))
        .bind(limit + 1)
        .bind(offset)
        .fetch_all(&s.db)
        .await?;
    let rows = rows.iter().map(|r| space_json(r, fmt)).collect();
    Ok(paged(
        &q,
        "/wiki/api/v2/spaces",
        &s.site(),
        rows,
        limit,
        offset,
    ))
}

pub async fn get(
    State(s): State<AppState>,
    Path(id): Path<String>,
    RawQuery(raw): RawQuery,
) -> R<Json<Value>> {
    let q = Q::parse(raw);
    let row: Space = sqlx::query_as(&format!("SELECT {COLS} FROM spaces WHERE id = $1"))
        .bind(parse_id(&id)?)
        .fetch_optional(&s.db)
        .await?
        .ok_or_else(|| ApiError::NotFound("space not found".into()))?;
    let mut v = space_json(&row, desc_format(&q)?);
    v["_links"]["base"] = json!(s.site());
    Ok(Json(v))
}

#[derive(Deserialize)]
pub struct CreateReq {
    name: String,
    key: Option<String>,
    description: Option<Description>,
}

#[derive(Deserialize)]
pub struct Description {
    value: String,
}

/// Uppercased alphanumerics of the name, at most 10 characters.
fn derive_key(name: &str) -> String {
    let k: String = name
        .chars()
        .filter(char::is_ascii_alphanumeric)
        .take(10)
        .collect();
    if k.is_empty() {
        "SPACE".into()
    } else {
        k.to_uppercase()
    }
}

pub async fn create(
    State(s): State<AppState>,
    Extension(user): Extension<UserId>,
    JsonBody(req): JsonBody<CreateReq>,
) -> R<(StatusCode, Json<Value>)> {
    if req.name.trim().is_empty() {
        return Err(ApiError::BadRequest("name is required".into()).into());
    }
    let key = req.key.unwrap_or_else(|| derive_key(&req.name));
    if key.is_empty() || key.len() > 255 || !key.chars().all(|c| c.is_ascii_alphanumeric()) {
        return Err(ApiError::BadRequest("key must be alphanumeric".into()).into());
    }
    let mut tx = s.db.begin().await?;
    let id: i64 = sqlx::query_scalar(
        "INSERT INTO spaces (key, name, description, author_id) VALUES ($1, $2, $3, $4) RETURNING id",
    )
    .bind(&key)
    .bind(req.name.trim())
    .bind(req.description.map(|d| d.value).unwrap_or_default())
    .bind(user.0)
    .fetch_one(&mut *tx)
    .await
    .map_err(|e| db_err(e, "A space with this key already exists"))?;
    let home = insert_page(
        &mut tx,
        id,
        None,
        "current",
        req.name.trim(),
        &json!({"version": 1, "type": "doc", "content": []}),
        user.0,
    )
    .await?;
    sqlx::query("UPDATE spaces SET homepage_id = $1 WHERE id = $2")
        .bind(home)
        .bind(id)
        .execute(&mut *tx)
        .await?;
    let row: Space = sqlx::query_as(&format!("SELECT {COLS} FROM spaces WHERE id = $1"))
        .bind(id)
        .fetch_one(&mut *tx)
        .await?;
    tx.commit().await?;
    let mut v = space_json(&row, Some("plain"));
    v["_links"]["base"] = json!(s.site());
    Ok((StatusCode::CREATED, Json(v)))
}
