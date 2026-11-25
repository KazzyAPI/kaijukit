//! Classification levels: the site's list, a level per content row and a default per space.
//! `[V]` the seeded levels are ours; the reference has none until an admin defines them.

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use kk_core::ApiError;
use serde::Deserialize;
use serde_json::{json, Value};

use crate::{error::R, http::JsonBody, kinds, target, AppState};

type Level = (String, i32, String, String, String, String, String);

const COLS: &str = "id, position, name, description, guideline, color, status";

fn render(l: Level) -> Value {
    json!({"id": l.0, "order": l.1, "name": l.2, "description": l.3, "guideline": l.4, "color": l.5, "status": l.6})
}

async fn level(s: &AppState, id: &str) -> Result<Level, ApiError> {
    sqlx::query_as(&format!(
        "SELECT {COLS} FROM classification_levels WHERE id = $1"
    ))
    .bind(id)
    .fetch_optional(&s.db)
    .await
    .map_err(|e| crate::error::db_err(e, ""))?
    .ok_or_else(|| ApiError::BadRequest("unknown classification level".into()))
}

pub async fn list(State(s): State<AppState>) -> R<Json<Value>> {
    let rows: Vec<Level> = sqlx::query_as(&format!(
        "SELECT {COLS} FROM classification_levels ORDER BY position"
    ))
    .fetch_all(&s.db)
    .await?;
    Ok(Json(Value::Array(rows.into_iter().map(render).collect())))
}

/// Own level, else the space default, else the site default.
pub async fn get<const K: u8>(State(s): State<AppState>, Path(id): Path<String>) -> R<Json<Value>> {
    let id = target::owner(&s.db, &id, kinds::of(K).ty).await?;
    let l: Level = sqlx::query_as(&format!(
        "SELECT {COLS} FROM classification_levels WHERE id = COALESCE(\
           (SELECT level_id FROM content_classification WHERE content_id = $1), \
           (SELECT sc.level_id FROM space_classification sc JOIN content c ON c.space_id = sc.space_id WHERE c.id = $1), \
           (SELECT id FROM classification_levels WHERE is_default))"
    ))
    .bind(id)
    .fetch_one(&s.db)
    .await?;
    Ok(Json(render(l)))
}

#[derive(Deserialize)]
pub struct Put {
    id: String,
}

pub async fn put<const K: u8>(
    State(s): State<AppState>,
    Path(id): Path<String>,
    JsonBody(b): JsonBody<Put>,
) -> R<StatusCode> {
    let id = target::owner(&s.db, &id, kinds::of(K).ty).await?;
    level(&s, &b.id).await?;
    sqlx::query(
        "INSERT INTO content_classification (content_id, level_id) VALUES ($1, $2) \
         ON CONFLICT (content_id) DO UPDATE SET level_id = $2",
    )
    .bind(id)
    .bind(&b.id)
    .execute(&s.db)
    .await?;
    Ok(StatusCode::NO_CONTENT)
}

#[derive(Deserialize)]
pub struct Reset {
    #[allow(dead_code)]
    status: String,
}

pub async fn reset<const K: u8>(
    State(s): State<AppState>,
    Path(id): Path<String>,
    JsonBody(_): JsonBody<Reset>,
) -> R<StatusCode> {
    let id = target::owner(&s.db, &id, kinds::of(K).ty).await?;
    sqlx::query("DELETE FROM content_classification WHERE content_id = $1")
        .bind(id)
        .execute(&s.db)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn space_get(State(s): State<AppState>, Path(id): Path<String>) -> R<Json<Value>> {
    let id = target::space(&s.db, &id).await?;
    let l: Level = sqlx::query_as(&format!(
        "SELECT {COLS} FROM classification_levels WHERE id = COALESCE(\
           (SELECT level_id FROM space_classification WHERE space_id = $1), \
           (SELECT id FROM classification_levels WHERE is_default))"
    ))
    .bind(id)
    .fetch_one(&s.db)
    .await?;
    Ok(Json(render(l)))
}

pub async fn space_put(
    State(s): State<AppState>,
    Path(id): Path<String>,
    JsonBody(b): JsonBody<Put>,
) -> R<StatusCode> {
    let id = target::space(&s.db, &id).await?;
    level(&s, &b.id).await?;
    sqlx::query(
        "INSERT INTO space_classification (space_id, level_id) VALUES ($1, $2) \
         ON CONFLICT (space_id) DO UPDATE SET level_id = $2",
    )
    .bind(id)
    .bind(&b.id)
    .execute(&s.db)
    .await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn space_delete(State(s): State<AppState>, Path(id): Path<String>) -> R<StatusCode> {
    let id = target::space(&s.db, &id).await?;
    sqlx::query("DELETE FROM space_classification WHERE space_id = $1")
        .bind(id)
        .execute(&s.db)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}
