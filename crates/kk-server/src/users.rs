use axum::{extract::State, Json};
use kk_core::ApiError;
use serde::Deserialize;
use serde_json::{json, Value};

use crate::{error::R, http::JsonBody, AppState};

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
