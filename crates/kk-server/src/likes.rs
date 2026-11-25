//! Likes (v2 reads, plus a v1-style write) and permitted-operation lists.

use axum::{
    extract::{Path, RawQuery, State},
    http::StatusCode,
    response::Response,
    Extension, Json,
};
use kk_core::{ApiError, UserId};
use serde_json::{json, Value};

use crate::{
    error::R,
    http::{paged, Q},
    target, AppState,
};

/// Liked content: pages and comments. `TY`: 0 page, 1 footer comment, 2 inline comment.
async fn liked<const TY: u8>(s: &AppState, raw: &str) -> Result<i64, ApiError> {
    let id = crate::http::parse_id(raw)?;
    let (kind, ty) = match TY {
        0 => (None, "page"),
        1 => (Some("footer"), "comment"),
        _ => (Some("inline"), "comment"),
    };
    target::live(&s.db, id, ty).await?;
    if let Some(kind) = kind {
        let ok: Option<i64> = sqlx::query_scalar(
            "SELECT content_id FROM comments WHERE content_id = $1 AND kind = $2",
        )
        .bind(id)
        .bind(kind)
        .fetch_optional(&s.db)
        .await
        .map_err(|e| crate::error::db_err(e, ""))?;
        ok.ok_or_else(|| ApiError::NotFound(format!("{kind} comment not found")))?;
    }
    Ok(id)
}

pub async fn count<const TY: u8>(
    State(s): State<AppState>,
    Path(id): Path<String>,
) -> R<Json<Value>> {
    let id = liked::<TY>(&s, &id).await?;
    let n: i64 = sqlx::query_scalar("SELECT count(*) FROM likes WHERE content_id = $1")
        .bind(id)
        .fetch_one(&s.db)
        .await?;
    Ok(Json(json!({"count": n})))
}

pub async fn users<const TY: u8>(
    State(s): State<AppState>,
    Path(id): Path<String>,
    RawQuery(raw): RawQuery,
) -> R<Response> {
    let q = Q::parse(raw);
    let (limit, offset) = (q.limit()?, q.offset()?);
    let id = liked::<TY>(&s, &id).await?;
    let ids: Vec<i64> = sqlx::query_scalar(
        "SELECT user_id FROM likes WHERE content_id = $1 ORDER BY created_at, user_id LIMIT $2 OFFSET $3",
    )
    .bind(id)
    .bind(limit + 1)
    .bind(offset)
    .fetch_all(&s.db)
    .await?;
    let rows = ids
        .iter()
        .map(|u| json!({"accountId": u.to_string()}))
        .collect();
    let seg = ["pages", "footer-comments", "inline-comments"][TY as usize];
    let path = format!("/wiki/api/v2/{seg}/{id}/likes/users");
    Ok(paged(&q, &path, &s.site(), rows, limit, offset))
}

/// Not part of the reference's public API; clients need some way to like and unlike.
pub async fn set(
    State(s): State<AppState>,
    Extension(user): Extension<UserId>,
    Path(id): Path<String>,
    method: axum::http::Method,
) -> R<StatusCode> {
    let id = crate::http::parse_id(&id)?;
    let ty: Option<String> = sqlx::query_scalar(
        "SELECT type FROM content WHERE id = $1 AND status = 'current' AND type IN ('page', 'comment')",
    )
    .bind(id)
    .fetch_optional(&s.db)
    .await?;
    ty.ok_or_else(|| ApiError::NotFound("page or comment not found".into()))?;
    let sql = if method == axum::http::Method::PUT {
        "INSERT INTO likes (content_id, user_id) VALUES ($1, $2) ON CONFLICT DO NOTHING"
    } else {
        "DELETE FROM likes WHERE content_id = $1 AND user_id = $2"
    };
    sqlx::query(sql)
        .bind(id)
        .bind(user.0)
        .execute(&s.db)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

const PAGE: &[(&str, &str)] = &[
    ("read", "page"),
    ("update", "page"),
    ("delete", "page"),
    ("copy", "page"),
    ("move", "page"),
    ("archive", "page"),
    ("restrict_content", "page"),
    ("create", "comment"),
    ("create", "attachment"),
];
const SPACE: &[(&str, &str)] = &[
    ("read", "space"),
    ("create", "page"),
    ("create", "blogpost"),
    ("create", "comment"),
    ("create", "attachment"),
    ("delete", "space"),
    ("export", "space"),
    ("administer", "space"),
    ("archive", "page"),
    ("restrict_content", "space"),
];
const COMMENT: &[(&str, &str)] = &[
    ("read", "comment"),
    ("update", "comment"),
    ("delete", "comment"),
];
const ATTACHMENT: &[(&str, &str)] = &[
    ("read", "attachment"),
    ("update", "attachment"),
    ("delete", "attachment"),
];

fn operations(ops: &[(&str, &str)]) -> Json<Value> {
    let ops: Vec<_> = ops
        .iter()
        .map(|(o, t)| json!({"operation": o, "targetType": t}))
        .collect();
    Json(json!({"operations": ops}))
}

/// Until permissions exist every authenticated user holds every operation.
pub async fn page_ops(State(s): State<AppState>, Path(id): Path<String>) -> R<Json<Value>> {
    target::owner(&s.db, &id, "page").await?;
    Ok(operations(PAGE))
}

pub async fn space_ops(State(s): State<AppState>, Path(id): Path<String>) -> R<Json<Value>> {
    target::space(&s.db, &id).await?;
    Ok(operations(SPACE))
}

pub async fn attachment_ops(State(s): State<AppState>, Path(id): Path<String>) -> R<Json<Value>> {
    target::owner(&s.db, &id, "attachment").await?;
    Ok(operations(ATTACHMENT))
}

pub async fn comment_ops<const TY: u8>(
    State(s): State<AppState>,
    Path(id): Path<String>,
) -> R<Json<Value>> {
    liked::<TY>(&s, &id).await?;
    Ok(operations(COMMENT))
}
