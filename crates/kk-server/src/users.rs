use std::sync::Arc;

use axum::{extract::State, http::StatusCode, Extension, Json};
use kk_core::ApiError;
use serde::Deserialize;
use serde_json::{json, Value};

use crate::{authz::Access, error::R, http::JsonBody, AppState};

const MAX_LOOKUP: usize = 100;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Bulk {
    account_ids: Vec<String>,
}

pub async fn bulk(State(s): State<AppState>, JsonBody(req): JsonBody<Bulk>) -> R<Json<Value>> {
    if req.account_ids.is_empty() || req.account_ids.len() > MAX_LOOKUP {
        return Err(
            ApiError::BadRequest(format!("accountIds must hold 1 to {MAX_LOOKUP} ids")).into(),
        );
    }
    // Ids that are not numeric cannot belong to anyone, so they are skipped like unknown ones.
    let ids: Vec<i64> = req
        .account_ids
        .iter()
        .filter_map(|i| i.parse().ok())
        .collect();
    let rows: Vec<(i64, String, String, String)> = sqlx::query_as(
        "SELECT id, display_name, email, account_type FROM users WHERE id = ANY($1) ORDER BY id",
    )
    .bind(ids)
    .fetch_all(&s.db)
    .await?;
    let results: Vec<Value> = rows
        .into_iter()
        .map(|(id, name, email, kind)| {
            json!({
                "accountId": id.to_string(),
                "displayName": name,
                "publicName": name,
                "email": email,
                "accountType": kind,
                "accountStatus": "active",
                "isExternalCollaborator": false,
            })
        })
        .collect();
    Ok(Json(
        json!({"results": results, "_links": {"base": s.site()}}),
    ))
}

#[derive(Deserialize)]
pub struct Emails {
    emails: Vec<String>,
}

pub async fn check_access(
    State(s): State<AppState>,
    JsonBody(req): JsonBody<Emails>,
) -> R<Json<Value>> {
    let (invalid, valid): (Vec<String>, Vec<String>) =
        req.emails.into_iter().partition(|e| !e.contains('@'));
    let known: Vec<String> =
        sqlx::query_scalar("SELECT lower(email) FROM users WHERE lower(email) = ANY($1)")
            .bind(valid.iter().map(|e| e.to_lowercase()).collect::<Vec<_>>())
            .fetch_all(&s.db)
            .await?;
    let without: Vec<String> = valid
        .into_iter()
        .filter(|e| !known.contains(&e.to_lowercase()))
        .collect();
    Ok(Json(
        json!({"emailsWithoutAccess": without, "invalidEmails": invalid}),
    ))
}

/// v1 `getCurrentUser`.
pub async fn current(
    State(s): State<AppState>,
    Extension(a): Extension<Arc<Access>>,
) -> R<Json<Value>> {
    Ok(Json(crate::v1::users(&s.db, &[a.p.id]).await?.remove(0)))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewUser {
    email: String,
    display_name: Option<String>,
    #[serde(default)]
    admin: bool,
}

/// kaijukit extension: an administrator creates a user and receives its API token once.
pub async fn create(
    State(s): State<AppState>,
    Extension(a): Extension<Arc<Access>>,
    JsonBody(req): JsonBody<NewUser>,
) -> R<(StatusCode, Json<Value>)> {
    if !a.p.admin {
        return Err(ApiError::Forbidden("requires a site administrator".into()).into());
    }
    let email = req.email.trim();
    let name = req.display_name.as_deref().unwrap_or(email).trim();
    if !valid_email(email) || name.is_empty() || name.chars().count() > 255 {
        return Err(ApiError::BadRequest(
            "a valid email and a displayName of 1 to 255 characters are required".into(),
        )
        .into());
    }
    let token: String = sqlx::query_scalar(
        "SELECT 'kk_' || replace(gen_random_uuid()::text || gen_random_uuid()::text, '-', '')",
    )
    .fetch_one(&s.db)
    .await?;
    let mut tx = s.db.begin().await?;
    let id: i64 = sqlx::query_scalar(
        "INSERT INTO users (email, display_name, is_admin) VALUES ($1, $2, $3) RETURNING id",
    )
    .bind(email)
    .bind(name)
    .bind(req.admin)
    .fetch_one(&mut *tx)
    .await
    .map_err(|e| crate::error::db_err(e, "A user with this email already exists"))?;
    sqlx::query("INSERT INTO api_tokens (user_id, token_hash, label) VALUES ($1, $2, 'created')")
        .bind(id)
        .bind(crate::auth::hash(&token))
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;
    Ok((
        StatusCode::CREATED,
        Json(
            json!({"accountId": id.to_string(), "email": email, "displayName": name, "admin": req.admin, "token": token}),
        ),
    ))
}

/// One `@`, text on both sides, no whitespace; real validation is a delivery problem.
pub fn valid_email(e: &str) -> bool {
    e.split_once('@').is_some_and(|(l, d)| {
        !l.is_empty() && d.contains('.') && !e.contains(char::is_whitespace) && !d.contains('@')
    })
}
