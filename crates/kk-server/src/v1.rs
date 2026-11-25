//! Shared shapes of the v1 (`/wiki/rest/api`) API: offset paging and the user and group objects.

use axum::{response::IntoResponse, response::Response, Json};
use kk_core::ApiError;
use serde_json::{json, Value};
use sqlx::PgPool;

use crate::{error::db_err, http::Q};

/// `start` and `limit` with the endpoint's default; `limit` may be 0.
pub fn window(q: &Q, default_limit: i64) -> Result<(i64, i64), ApiError> {
    let start = q.int("start")?.unwrap_or(0);
    let limit = q.int("limit")?.unwrap_or(default_limit as i32);
    if start < 0 || !(0..=1000).contains(&limit) {
        return Err(ApiError::BadRequest("start or limit out of range".into()));
    }
    Ok((start.into(), limit.into()))
}

/// `{results, start, limit, size, _links}`; `rows` holds up to `limit + 1` items.
pub fn page(q: &Q, path: &str, site: &str, mut rows: Vec<Value>, start: i64, limit: i64) -> Value {
    let more = rows.len() as i64 > limit;
    rows.truncate(limit as usize);
    let mut links = json!({"base": site, "context": "/wiki"});
    if more {
        let mut s = form_urlencoded::Serializer::new(String::new());
        for (k, v) in q.without(&["start", "limit"]) {
            s.append_pair(&k, &v);
        }
        s.append_pair("start", &(start + limit).to_string());
        s.append_pair("limit", &limit.to_string());
        links["next"] = json!(format!("{path}?{}", s.finish()));
    }
    json!({"results": rows.clone(), "start": start, "limit": limit, "size": rows.len(), "_links": links})
}

pub fn ok(v: Value) -> Response {
    Json(v).into_response()
}

pub fn user_json(id: i64, name: &str, email: &str, account_type: &str) -> Value {
    json!({
        "type": "known",
        "accountId": id.to_string(),
        "accountType": account_type,
        "email": email,
        "publicName": name,
        "displayName": name,
    })
}

pub async fn users(db: &PgPool, ids: &[i64]) -> Result<Vec<Value>, ApiError> {
    let rows: Vec<(i64, String, String, String)> = sqlx::query_as(
        "SELECT id, display_name, email, account_type FROM users WHERE id = ANY($1) ORDER BY id",
    )
    .bind(ids)
    .fetch_all(db)
    .await
    .map_err(|e| db_err(e, ""))?;
    Ok(rows
        .iter()
        .map(|(i, n, e, t)| user_json(*i, n, e, t))
        .collect())
}

pub fn group_json(id: i64, name: &str, site: &str) -> Value {
    json!({
        "type": "group",
        "name": name,
        "id": id.to_string(),
        "usageType": "USERBASE_GROUP",
        "managedBy": "ADMINS",
        "_links": {"self": format!("{site}/rest/api/group/by-id?id={id}")},
    })
}

pub async fn groups(db: &PgPool, site: &str, ids: &[i64]) -> Result<Vec<Value>, ApiError> {
    let rows: Vec<(i64, String)> =
        sqlx::query_as("SELECT id, name FROM groups WHERE id = ANY($1) ORDER BY lower(name), id")
            .bind(ids)
            .fetch_all(db)
            .await
            .map_err(|e| db_err(e, ""))?;
    Ok(rows.iter().map(|(i, n)| group_json(*i, n, site)).collect())
}
