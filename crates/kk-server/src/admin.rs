//! Admin key and data policy reads. `[V]` an admin key is a time-boxed marker row; it grants nothing
//! extra because site administrators already pass every check. Data policies are not built, so
//! nothing is ever blocked.

use std::sync::Arc;

use axum::{
    extract::{RawQuery, State},
    http::StatusCode,
    response::Response,
    Extension, Json,
};
use kk_core::{ApiError, UserId};
use serde::Deserialize;
use serde_json::{json, Value};

use crate::{
    authz::Access,
    error::R,
    http::{paged, ts, Q},
    AppState,
};

fn need_admin(a: &Access) -> Result<(), ApiError> {
    if a.p.admin {
        Ok(())
    } else {
        Err(ApiError::Forbidden(
            "only site administrators use admin keys".into(),
        ))
    }
}

async fn current(s: &AppState, user: i64) -> Result<Option<Value>, ApiError> {
    use crate::error::db_err;
    let at: Option<chrono::DateTime<chrono::Utc>> = sqlx::query_scalar(
        "SELECT expires_at FROM admin_keys WHERE user_id = $1 AND expires_at > now()",
    )
    .bind(user)
    .fetch_optional(&s.db)
    .await
    .map_err(|e| db_err(e, ""))?;
    Ok(at.map(|t| json!({"accountId": user.to_string(), "expirationTime": ts(t)})))
}

pub async fn get_key(
    State(s): State<AppState>,
    Extension(a): Extension<Arc<Access>>,
    Extension(user): Extension<UserId>,
) -> R<Json<Value>> {
    need_admin(&a)?;
    current(&s, user.0)
        .await?
        .map(Json)
        .ok_or_else(|| ApiError::NotFound("no active admin key".into()).into())
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Enable {
    duration_in_minutes: Option<i32>,
}

pub async fn enable_key(
    State(s): State<AppState>,
    Extension(a): Extension<Arc<Access>>,
    Extension(user): Extension<UserId>,
    body: axum::body::Bytes,
) -> R<Json<Value>> {
    need_admin(&a)?;
    let given: Enable = if body.is_empty() {
        Enable::default()
    } else {
        serde_json::from_slice(&body).map_err(|e| ApiError::BadRequest(e.to_string()))?
    };
    let minutes = given.duration_in_minutes.unwrap_or(10);
    if !(1..=60).contains(&minutes) {
        return Err(ApiError::BadRequest("durationInMinutes must be 1 to 60".into()).into());
    }
    sqlx::query(
        "INSERT INTO admin_keys (user_id, expires_at) VALUES ($1, now() + make_interval(mins => $2)) \
         ON CONFLICT (user_id) DO UPDATE SET expires_at = excluded.expires_at",
    )
    .bind(user.0)
    .bind(minutes)
    .execute(&s.db)
    .await?;
    Ok(Json(current(&s, user.0).await?.unwrap_or(Value::Null)))
}

pub async fn disable_key(
    State(s): State<AppState>,
    Extension(a): Extension<Arc<Access>>,
    Extension(user): Extension<UserId>,
) -> R<StatusCode> {
    need_admin(&a)?;
    let n = sqlx::query("DELETE FROM admin_keys WHERE user_id = $1 AND expires_at > now()")
        .bind(user.0)
        .execute(&s.db)
        .await?
        .rows_affected();
    if n == 0 {
        return Err(ApiError::NotFound("no active admin key".into()).into());
    }
    Ok(StatusCode::NO_CONTENT)
}

pub async fn policy_metadata() -> Json<Value> {
    Json(json!({"anyContentBlocked": false}))
}

pub async fn policy_spaces(State(s): State<AppState>, RawQuery(raw): RawQuery) -> R<Response> {
    let q = Q::parse(raw);
    let (limit, offset) = (q.limit_or(25)?, q.offset()?);
    Ok(paged(
        &q,
        "/wiki/api/v2/data-policies/spaces",
        &s.site(),
        vec![],
        limit,
        offset,
    ))
}
