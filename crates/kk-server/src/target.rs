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
