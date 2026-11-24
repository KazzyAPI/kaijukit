use axum::{
    extract::{Request, State},
    http::{header, HeaderMap},
    middleware::Next,
    response::{IntoResponse, Response},
};
use base64::{engine::general_purpose::STANDARD, Engine};
use kk_core::{ApiError, UserId};
use sha2::{Digest, Sha256};
use sqlx::PgPool;

use crate::{error::E, AppState};

pub const DEV_TOKEN: &str = "kk-dev-token";

fn hash(token: &str) -> String {
    Sha256::digest(token.as_bytes())
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

async fn authenticate(db: &PgPool, headers: &HeaderMap) -> Result<UserId, ApiError> {
    let creds = headers
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Basic "))
        .and_then(|b| STANDARD.decode(b.trim()).ok())
        .and_then(|b| String::from_utf8(b).ok())
        .ok_or(ApiError::Unauthorized)?;
    let (email, token) = creds.split_once(':').ok_or(ApiError::Unauthorized)?;
    sqlx::query_scalar::<_, i64>(
        "SELECT u.id FROM users u JOIN api_tokens t ON t.user_id = u.id \
         WHERE lower(u.email) = lower($1) AND t.token_hash = $2",
    )
    .bind(email)
    .bind(hash(token))
    .fetch_optional(db)
    .await
    .map_err(|e| crate::error::db_err(e, ""))?
    .map(UserId)
    .ok_or(ApiError::Unauthorized)
}

pub async fn require_auth(State(s): State<AppState>, mut req: Request, next: Next) -> Response {
    match authenticate(&s.db, req.headers()).await {
        Ok(user) => {
            req.extensions_mut().insert(user);
            next.run(req).await
        }
        Err(e) => E(e).into_response(),
    }
}

pub async fn bootstrap_admin(db: &PgPool, email: &str, token: &str) -> Result<(), sqlx::Error> {
    if token == DEV_TOKEN {
        tracing::warn!("admin token is the development default; set KK_ADMIN_TOKEN");
    }
    let id: i64 = sqlx::query_scalar(
        "INSERT INTO users (email, display_name, is_admin) VALUES ($1, 'Admin', true) \
         ON CONFLICT (lower(email)) DO UPDATE SET is_admin = true, \
         display_name = CASE WHEN users.display_name = users.email THEN 'Admin' ELSE users.display_name END \
         RETURNING id",
    )
    .bind(email)
    .fetch_one(db)
    .await?;
    sqlx::query(
        "INSERT INTO api_tokens (user_id, token_hash, label) VALUES ($1, $2, 'bootstrap') \
         ON CONFLICT (token_hash) DO NOTHING",
    )
    .bind(id)
    .bind(hash(token))
    .execute(db)
    .await?;
    Ok(())
}
