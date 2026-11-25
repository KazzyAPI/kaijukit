//! Likes (v2 reads, plus a v1-style write) and permitted-operation lists.

use axum::{
    extract::{Path, RawQuery, State},
    http::StatusCode,
    response::Response,
    Extension, Json,
};
use kk_core::{perm, ApiError, UserId};
use serde_json::{json, Value};
use std::sync::Arc;

use crate::{
    authz::Access,
    error::R,
    http::{paged, Q},
    target, AppState,
};

/// Liked content: pages, comments and blog posts. `TY`: 0 page, 1 footer comment, 2 inline comment, 3 blog post.
async fn liked<const TY: u8>(s: &AppState, raw: &str) -> Result<i64, ApiError> {
    let id = crate::http::parse_id(raw)?;
    let (kind, ty) = match TY {
        0 => (None, "page"),
        1 => (Some("footer"), "comment"),
        3 => (None, "blogpost"),
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
    let seg = ["pages", "footer-comments", "inline-comments", "blogposts"][TY as usize];
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
        "SELECT type FROM content WHERE id = $1 AND status = 'current' AND type IN ('page', 'comment', 'blogpost')",
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

fn operations(ops: Vec<(&str, &str)>) -> Json<Value> {
    let ops: Vec<_> = ops
        .iter()
        .map(|(o, t)| json!({"operation": o, "targetType": t}))
        .collect();
    Json(json!({"operations": ops}))
}

/// Operations the caller holds on a content row, from their permissions and restrictions.
pub(crate) async fn ops_list(
    s: &AppState,
    a: &Access,
    id: i64,
) -> Result<Vec<(&'static str, &'static str)>, ApiError> {
    let (t, chain) = a.visible(&s.db, id).await?;
    let g = a.grants(t.space_id);
    let own = t.author_id == a.p.id;
    Ok(match t.ty.as_str() {
        "comment" => perm::comment_ops(&a.p, &g, &chain, own),
        "attachment" => perm::attachment_ops(&a.p, &g, &chain),
        // the page operations name their own target; other types report themselves
        "blogpost" => perm::content_ops("blogpost", &a.p, &g, &chain),
        ty => {
            let label = crate::kinds::target_label(ty);
            perm::page_ops(&a.p, &g, &chain)
                .into_iter()
                .map(|(o, t)| (o, if t == "page" { label } else { t }))
                .collect()
        }
    })
}

async fn content_ops(s: &AppState, a: &Access, id: i64) -> R<Json<Value>> {
    Ok(operations(ops_list(s, a, id).await?))
}

/// Operations on any content type that is checked like a page.
pub async fn ops<const K: u8>(
    State(s): State<AppState>,
    Extension(a): Extension<Arc<Access>>,
    Path(id): Path<String>,
) -> R<Json<Value>> {
    let id = target::owner(&s.db, &id, crate::kinds::of(K).ty).await?;
    content_ops(&s, &a, id).await
}

pub async fn page_ops(
    State(s): State<AppState>,
    Extension(a): Extension<Arc<Access>>,
    Path(id): Path<String>,
) -> R<Json<Value>> {
    let id = target::owner(&s.db, &id, "page").await?;
    content_ops(&s, &a, id).await
}

pub async fn space_ops(
    State(s): State<AppState>,
    Extension(a): Extension<Arc<Access>>,
    Path(id): Path<String>,
) -> R<Json<Value>> {
    let id = target::space(&s.db, &id).await?;
    Ok(operations(perm::space_ops(&a.grants(id))))
}

pub async fn attachment_ops(
    State(s): State<AppState>,
    Extension(a): Extension<Arc<Access>>,
    Path(id): Path<String>,
) -> R<Json<Value>> {
    let id = target::owner(&s.db, &id, "attachment").await?;
    content_ops(&s, &a, id).await
}

pub async fn comment_ops<const TY: u8>(
    State(s): State<AppState>,
    Extension(a): Extension<Arc<Access>>,
    Path(id): Path<String>,
) -> R<Json<Value>> {
    let id = liked::<TY>(&s, &id).await?;
    content_ops(&s, &a, id).await
}
