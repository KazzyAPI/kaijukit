//! Content properties (pages, attachments, comments), space properties and app properties.

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
    http::{paged, ts, JsonBody, Q},
    target, AppState,
};

/// Serialized value limit.
const MAX_VALUE: usize = 32 * 1024;
const MAX_KEY: usize = 255;

#[derive(Clone, Copy)]
pub struct Kind {
    table: &'static str,
    ty: &'static str,
    seg: &'static str,
    space: bool,
}

pub const PAGE: Kind = Kind {
    table: "content_properties",
    ty: "page",
    seg: "pages",
    space: false,
};
pub const ATTACHMENT: Kind = Kind {
    table: "content_properties",
    ty: "attachment",
    seg: "attachments",
    space: false,
};
pub const COMMENT: Kind = Kind {
    table: "content_properties",
    ty: "comment",
    seg: "comments",
    space: false,
};
pub const SPACE: Kind = Kind {
    table: "space_properties",
    ty: "space",
    seg: "spaces",
    space: true,
};

#[derive(sqlx::FromRow)]
struct Row {
    id: i64,
    key: String,
    value: Value,
    version_number: i32,
    message: String,
    created_by: i64,
    created_at: DateTime<Utc>,
    updated_by: i64,
    updated_at: DateTime<Utc>,
}

const COLS: &str =
    "id, key, value, version_number, message, created_by, created_at, updated_by, updated_at";

impl Row {
    fn json(&self, k: Kind) -> Value {
        if k.space {
            return json!({
                "id": self.id.to_string(), "key": self.key, "value": self.value,
                "createdAt": ts(self.created_at), "createdBy": self.created_by.to_string(),
                "version": {"number": self.version_number, "message": self.message,
                            "createdAt": ts(self.updated_at), "createdBy": self.updated_by.to_string()},
            });
        }
        json!({
            "id": self.id.to_string(), "key": self.key, "value": self.value,
            "version": {"number": self.version_number, "message": self.message, "minorEdit": false,
                        "authorId": self.updated_by.to_string(), "createdAt": ts(self.updated_at)},
        })
    }
}

async fn owner(s: &AppState, k: Kind, raw: &str) -> Result<i64, ApiError> {
    if k.space {
        target::space(&s.db, raw).await
    } else {
        target::owner(&s.db, raw, k.ty).await
    }
}

fn parse_property_id(raw: &str) -> Result<i64, ApiError> {
    crate::http::parse_id(raw)
}

fn check(key: &str, value: &Value) -> Result<(), ApiError> {
    if key.is_empty() || key.chars().count() > MAX_KEY {
        return Err(ApiError::BadRequest(
            "key must be 1 to 255 characters".into(),
        ));
    }
    if value.to_string().len() > MAX_VALUE {
        return Err(ApiError::PayloadTooLarge);
    }
    Ok(())
}

pub async fn list<const K: u8>(
    State(s): State<AppState>,
    Path(id): Path<String>,
    RawQuery(raw): RawQuery,
) -> R<Response> {
    let k = kind(K);
    let q = Q::parse(raw);
    let (limit, offset) = (q.limit()?, q.offset()?);
    let order = q.order_by(&[("key", "key")], "id")?;
    let id = owner(&s, k, &id).await?;
    let rows: Vec<Row> = sqlx::query_as(&format!(
        "SELECT {COLS} FROM {} WHERE owner_id = $1 AND ($2::text IS NULL OR key = $2) \
         ORDER BY {order} LIMIT $3 OFFSET $4",
        k.table
    ))
    .bind(id)
    .bind(q.get("key"))
    .bind(limit + 1)
    .bind(offset)
    .fetch_all(&s.db)
    .await?;
    let path = format!(
        "/wiki/api/v2/{}/{}/properties",
        k.seg,
        crate::http::public_id(k.ty, id)
    );
    Ok(paged(
        &q,
        &path,
        &s.site(),
        rows.iter().map(|r| r.json(k)).collect(),
        limit,
        offset,
    ))
}

fn kind(k: u8) -> Kind {
    [PAGE, ATTACHMENT, COMMENT, SPACE][k as usize]
}

pub async fn get<const K: u8>(
    State(s): State<AppState>,
    Path((id, pid)): Path<(String, String)>,
) -> R<Json<Value>> {
    let k = kind(K);
    let owner = owner(&s, k, &id).await?;
    Ok(Json(
        fetch(&s, k, owner, parse_property_id(&pid)?).await?.json(k),
    ))
}

async fn fetch(s: &AppState, k: Kind, owner: i64, id: i64) -> Result<Row, ApiError> {
    sqlx::query_as(&format!(
        "SELECT {COLS} FROM {} WHERE owner_id = $1 AND id = $2",
        k.table
    ))
    .bind(owner)
    .bind(id)
    .fetch_optional(&s.db)
    .await
    .map_err(|e| db_err(e, ""))?
    .ok_or_else(|| ApiError::NotFound("property not found".into()))
}

#[derive(Deserialize)]
pub struct CreateReq {
    key: String,
    value: Value,
}

pub async fn create<const K: u8>(
    State(s): State<AppState>,
    Extension(user): Extension<UserId>,
    Path(id): Path<String>,
    JsonBody(req): JsonBody<CreateReq>,
) -> R<(StatusCode, Json<Value>)> {
    let k = kind(K);
    let owner = owner(&s, k, &id).await?;
    check(&req.key, &req.value)?;
    let row: Row = sqlx::query_as(&format!(
        "INSERT INTO {} (owner_id, key, value, created_by, updated_by) VALUES ($1, $2, $3, $4, $4) RETURNING {COLS}",
        k.table
    ))
    .bind(owner)
    .bind(&req.key)
    .bind(&req.value)
    .bind(user.0)
    .fetch_one(&s.db)
    .await
    .map_err(|e| match e.as_database_error().and_then(|d| d.code()).as_deref() {
        Some("23505") => ApiError::Conflict(format!("property {} already exists", req.key)),
        _ => db_err(e, ""),
    })?;
    let status = if k.space {
        StatusCode::CREATED
    } else {
        StatusCode::OK
    };
    Ok((status, Json(row.json(k))))
}

#[derive(Deserialize)]
pub struct UpdateReq {
    key: String,
    value: Value,
    version: VersionReq,
}

#[derive(Deserialize)]
struct VersionReq {
    number: i32,
    message: Option<String>,
}

pub async fn update<const K: u8>(
    State(s): State<AppState>,
    Extension(user): Extension<UserId>,
    Path((id, pid)): Path<(String, String)>,
    JsonBody(req): JsonBody<UpdateReq>,
) -> R<Json<Value>> {
    let k = kind(K);
    let owner = owner(&s, k, &id).await?;
    let pid = parse_property_id(&pid)?;
    check(&req.key, &req.value)?;
    let mut tx = s.db.begin().await?;
    let cur: Row = sqlx::query_as(&format!(
        "SELECT {COLS} FROM {} WHERE owner_id = $1 AND id = $2 FOR UPDATE",
        k.table
    ))
    .bind(owner)
    .bind(pid)
    .fetch_optional(&mut *tx)
    .await?
    .ok_or_else(|| ApiError::NotFound("property not found".into()))?;
    if req.key != cur.key {
        return Err(ApiError::BadRequest("key cannot be changed".into()).into());
    }
    if req.version.number != cur.version_number + 1 {
        return Err(ApiError::Conflict(format!(
            "version.number must be {}, got {}",
            cur.version_number + 1,
            req.version.number
        ))
        .into());
    }
    let row: Row = sqlx::query_as(&format!(
        "UPDATE {} SET value = $2, version_number = $3, message = $4, updated_by = $5, updated_at = now() \
         WHERE id = $1 RETURNING {COLS}",
        k.table
    ))
    .bind(pid)
    .bind(&req.value)
    .bind(req.version.number)
    .bind(req.version.message.unwrap_or_default())
    .bind(user.0)
    .fetch_one(&mut *tx)
    .await?;
    tx.commit().await?;
    Ok(Json(row.json(k)))
}

pub async fn delete<const K: u8>(
    State(s): State<AppState>,
    Path((id, pid)): Path<(String, String)>,
) -> R<StatusCode> {
    let k = kind(K);
    let owner = owner(&s, k, &id).await?;
    let done = sqlx::query(&format!(
        "DELETE FROM {} WHERE owner_id = $1 AND id = $2",
        k.table
    ))
    .bind(owner)
    .bind(parse_property_id(&pid)?)
    .execute(&s.db)
    .await?;
    if done.rows_affected() == 0 {
        return Err(ApiError::NotFound("property not found".into()).into());
    }
    Ok(StatusCode::NO_CONTENT)
}

pub async fn app_list(State(s): State<AppState>, RawQuery(raw): RawQuery) -> R<Response> {
    let q = Q::parse(raw);
    let limit = match q.int("limit")? {
        None => 50,
        Some(n) if (1..=250).contains(&n) => i64::from(n),
        Some(_) => {
            return Err(ApiError::BadRequest("limit must be between 1 and 250".into()).into())
        }
    };
    let offset = q.offset()?;
    let rows: Vec<(String, Value)> =
        sqlx::query_as("SELECT key, value FROM app_properties ORDER BY key LIMIT $1 OFFSET $2")
            .bind(limit + 1)
            .bind(offset)
            .fetch_all(&s.db)
            .await?;
    let rows = rows
        .into_iter()
        .map(|(k, v)| json!({"key": k, "value": v}))
        .collect();
    Ok(paged(
        &q,
        "/wiki/api/v2/app/properties",
        &s.site(),
        rows,
        limit,
        offset,
    ))
}

pub async fn app_get(State(s): State<AppState>, Path(key): Path<String>) -> R<Json<Value>> {
    let value: Value = sqlx::query_scalar("SELECT value FROM app_properties WHERE key = $1")
        .bind(&key)
        .fetch_optional(&s.db)
        .await?
        .ok_or_else(|| ApiError::NotFound("property not found".into()))?;
    Ok(Json(json!({"key": key, "value": value})))
}

pub async fn app_put(
    State(s): State<AppState>,
    Path(key): Path<String>,
    JsonBody(value): JsonBody<Value>,
) -> R<StatusCode> {
    check(&key, &value)?;
    let created: bool = sqlx::query_scalar(
        "INSERT INTO app_properties (key, value) VALUES ($1, $2) \
         ON CONFLICT (key) DO UPDATE SET value = EXCLUDED.value, updated_at = now() \
         RETURNING (xmax = 0)",
    )
    .bind(&key)
    .bind(&value)
    .fetch_one(&s.db)
    .await?;
    Ok(if created {
        StatusCode::CREATED
    } else {
        StatusCode::OK
    })
}

pub async fn app_delete(State(s): State<AppState>, Path(key): Path<String>) -> R<StatusCode> {
    sqlx::query("DELETE FROM app_properties WHERE key = $1")
        .bind(key)
        .execute(&s.db)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}
