//! v1 content and space watches for the calling user.

use std::sync::Arc;

use axum::{
    extract::{Path, RawQuery, State},
    http::StatusCode,
    response::Response,
    Extension, Json,
};
use kk_core::ApiError;
use serde_json::{json, Value};

use crate::{
    authz::{self, Access},
    error::R,
    http::{parse_id, Q},
    v1, AppState,
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

/// One page of watchers of `(kind, id)`, each wrapped by `wrap`.
async fn watchers(
    s: &AppState,
    q: &Q,
    path: &str,
    kind: &str,
    id: i64,
    wrap: impl Fn(Value) -> Value,
) -> R<Response> {
    let (start, limit) = v1::window(q, 200)?;
    let ids: Vec<i64> = sqlx::query_scalar(
        "SELECT user_id FROM watches WHERE target_type = $1 AND target_id = $2 \
         ORDER BY user_id LIMIT $3 OFFSET $4",
    )
    .bind(kind)
    .bind(id)
    .bind(limit + 1)
    .bind(start)
    .fetch_all(&s.db)
    .await?;
    let rows = v1::full_users(&s.db, &s.site(), &ids)
        .await?
        .into_iter()
        .map(wrap)
        .collect();
    Ok(v1::ok(v1::page(q, path, &s.site(), rows, start, limit)))
}

/// v1 `getWatchesForPage`: who watches this page.
pub async fn of_page(
    State(s): State<AppState>,
    Path(id): Path<String>,
    RawQuery(raw): RawQuery,
) -> R<Response> {
    let q = Q::parse(raw);
    let id = parse_id(&id)?;
    let path = format!("/wiki/rest/api/content/{id}/notification/child-created");
    watchers(
        &s,
        &q,
        &path,
        "content",
        id,
        |w| json!({"type": "watch", "watcher": w, "contentId": id}),
    )
    .await
}

/// v1 `getWatchesForSpace`: who watches the space of this page.
pub async fn of_page_space(
    State(s): State<AppState>,
    Path(id): Path<String>,
    RawQuery(raw): RawQuery,
) -> R<Response> {
    let q = Q::parse(raw);
    let id = parse_id(&id)?;
    let (space, key): (i64, String) = sqlx::query_as(
        "SELECT s.id, s.key FROM content c JOIN spaces s ON s.id = c.space_id WHERE c.id = $1",
    )
    .bind(id)
    .fetch_one(&s.db)
    .await
    .map_err(|_| ApiError::NotFound("content not found".into()))?;
    let path = format!("/wiki/rest/api/content/{id}/notification/created");
    watchers(
        &s,
        &q,
        &path,
        "space",
        space,
        |w| json!({"type": "watch", "watcher": w, "spaceKey": key}),
    )
    .await
}

/// v1 `getWatchersForSpace`.
pub async fn of_space(
    State(s): State<AppState>,
    Extension(a): Extension<Arc<Access>>,
    Path(key): Path<String>,
    RawQuery(raw): RawQuery,
) -> R<Response> {
    let q = Q::parse(raw);
    let id = space_target(&s, &a, &key).await?;
    let name: String = sqlx::query_scalar("SELECT key FROM spaces WHERE id = $1")
        .bind(id)
        .fetch_one(&s.db)
        .await?;
    let path = format!("/wiki/rest/api/space/{key}/watch");
    watchers(
        &s,
        &q,
        &path,
        "space",
        id,
        |w| json!({"type": "watch", "watcher": w, "spaceKey": name}),
    )
    .await
}
