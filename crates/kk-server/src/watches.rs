//! v1 content and space watches for the calling user.

use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Extension, Json,
};
use kk_core::ApiError;
use serde_json::{json, Value};

use crate::{
    authz::{self, Access},
    error::R,
    http::parse_id,
    AppState,
};

/// `(type, id)` of what is watched; the guard already hid unreadable content, spaces are checked here.
async fn space_target(s: &AppState, a: &Access, key: &str) -> Result<i64, ApiError> {
    let id = authz::space_by_key(&s.db, key).await?;
    if a.grants(id).has("read", "space") {
        Ok(id)
    } else {
        Err(ApiError::NotFound("space not found".into()))
    }
}

async fn set(s: &AppState, a: &Access, kind: &str, id: i64, on: bool) -> R<StatusCode> {
    let sql = if on {
        "INSERT INTO watches (user_id, target_type, target_id) VALUES ($1, $2, $3) ON CONFLICT DO NOTHING"
    } else {
        "DELETE FROM watches WHERE user_id = $1 AND target_type = $2 AND target_id = $3"
    };
    sqlx::query(sql)
        .bind(a.p.id)
        .bind(kind)
        .bind(id)
        .execute(&s.db)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn status(s: &AppState, a: &Access, kind: &str, id: i64) -> R<Json<Value>> {
    let on: bool = sqlx::query_scalar(
        "SELECT EXISTS (SELECT 1 FROM watches WHERE user_id = $1 AND target_type = $2 AND target_id = $3)",
    )
    .bind(a.p.id)
    .bind(kind)
    .bind(id)
    .fetch_one(&s.db)
    .await?;
    Ok(Json(json!({"watching": on})))
}

async fn content_id(s: &AppState, raw: &str) -> Result<i64, ApiError> {
    let id = parse_id(raw)?;
    authz::target(&s.db, id).await?;
    Ok(id)
}

pub async fn content_get(
    State(s): State<AppState>,
    Extension(a): Extension<Arc<Access>>,
    Path(id): Path<String>,
) -> R<Json<Value>> {
    status(&s, &a, "content", content_id(&s, &id).await?).await
}

pub async fn content_add(
    State(s): State<AppState>,
    Extension(a): Extension<Arc<Access>>,
    Path(id): Path<String>,
) -> R<StatusCode> {
    set(&s, &a, "content", content_id(&s, &id).await?, true).await
}

pub async fn content_remove(
    State(s): State<AppState>,
    Extension(a): Extension<Arc<Access>>,
    Path(id): Path<String>,
) -> R<StatusCode> {
    set(&s, &a, "content", content_id(&s, &id).await?, false).await
}

pub async fn space_get(
    State(s): State<AppState>,
    Extension(a): Extension<Arc<Access>>,
    Path(key): Path<String>,
) -> R<Json<Value>> {
    status(&s, &a, "space", space_target(&s, &a, &key).await?).await
}

pub async fn space_add(
    State(s): State<AppState>,
    Extension(a): Extension<Arc<Access>>,
    Path(key): Path<String>,
) -> R<StatusCode> {
    set(&s, &a, "space", space_target(&s, &a, &key).await?, true).await
}

pub async fn space_remove(
    State(s): State<AppState>,
    Extension(a): Extension<Arc<Access>>,
    Path(key): Path<String>,
) -> R<StatusCode> {
    set(&s, &a, "space", space_target(&s, &a, &key).await?, false).await
}
