//! Existence checks shared by every endpoint that hangs data off a content row.

use kk_core::ApiError;
use sqlx::PgExecutor;

use crate::error::db_err;

const LIVE: &str = "('current', 'draft', 'archived')";

/// `(space_id, title)` of a live content row of type `ty`, else 404.
pub async fn live<'e>(
    ex: impl PgExecutor<'e>,
    id: i64,
    ty: &str,
) -> Result<(i64, String), ApiError> {
    sqlx::query_as(&format!(
        "SELECT space_id, title FROM content WHERE id = $1 AND type = $2 AND status IN {LIVE}"
    ))
    .bind(id)
    .bind(ty)
    .fetch_optional(ex)
    .await
    .map_err(|e| db_err(e, ""))?
    .ok_or_else(|| ApiError::NotFound(format!("{ty} not found")))
}

/// Parses `raw` (attachment ids may carry the `att` prefix) and checks the live row exists.
pub async fn owner<'e>(ex: impl PgExecutor<'e>, raw: &str, ty: &str) -> Result<i64, ApiError> {
    let id = if ty == "attachment" {
        crate::http::parse_attachment_id(raw)?
    } else {
        crate::http::parse_id(raw)?
    };
    live(ex, id, ty).await?;
    Ok(id)
}

pub async fn space<'e>(ex: impl PgExecutor<'e>, raw: &str) -> Result<i64, ApiError> {
    let id = crate::http::parse_id(raw)?;
    sqlx::query_scalar("SELECT id FROM spaces WHERE id = $1 AND status = 'current'")
        .bind(id)
        .fetch_optional(ex)
        .await
        .map_err(|e| db_err(e, ""))?
        .ok_or_else(|| ApiError::NotFound("space not found".into()))
}
