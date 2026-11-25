//! Space permission grants: v1 add and remove, v2 reads.

use axum::{
    extract::{Path, RawQuery, State},
    http::StatusCode,
    response::Response,
    Json,
};
use kk_core::{perm, ApiError};
use serde::Deserialize;
use serde_json::{json, Value};

use crate::{
    authz,
    error::{db_err, R},
    http::{paged, parse_id, JsonBody, Q},
    AppState,
};

#[derive(Deserialize)]
pub struct Req {
    subject: Subject,
    operation: Operation,
}

#[derive(Deserialize)]
pub struct Subject {
    #[serde(rename = "type")]
    kind: String,
    identifier: String,
}

#[derive(Deserialize)]
pub struct Operation {
    key: String,
    target: String,
}

pub async fn add(
    State(s): State<AppState>,
    Path(key): Path<String>,
    JsonBody(req): JsonBody<Req>,
) -> R<Json<Value>> {
    let space = authz::space_by_key(&s.db, &key).await?;
    let Operation { key: op, target } = req.operation;
    if !perm::valid_space_perm(&op, &target) {
        return Err(
            ApiError::BadRequest(format!("{op} on {target} is not a space permission")).into(),
        );
    }
    let (table, kind) = match req.subject.kind.as_str() {
        "user" => ("users", "user"),
        "group" => ("groups", "group"),
        _ => return Err(ApiError::BadRequest("subject.type must be user or group".into()).into()),
    };
    let bad = || ApiError::BadRequest(format!("unknown {kind} {}", req.subject.identifier));
    let sid = parse_id(&req.subject.identifier).map_err(|_| bad())?;
    sqlx::query_scalar::<_, i64>(&format!("SELECT id FROM {table} WHERE id = $1"))
        .bind(sid)
        .fetch_optional(&s.db)
        .await?
        .ok_or_else(bad)?;
    let id: i64 = sqlx::query_scalar(
        "INSERT INTO space_permissions (space_id, subject_type, subject_id, operation, target) \
         VALUES ($1, $2, $3, $4, $5) RETURNING id",
    )
    .bind(space)
    .bind(kind)
    .bind(sid)
    .bind(&op)
    .bind(&target)
    .fetch_one(&s.db)
    .await
    .map_err(|e| db_err(e, "This permission is already granted"))?;
    Ok(Json(json!({
        "id": id,
        "subject": {"type": kind, "identifier": sid.to_string()},
        "operation": {"key": op, "target": target},
        "_links": {"base": s.site(), "context": "/wiki"},
    })))
}

pub async fn remove(
    State(s): State<AppState>,
    Path((key, id)): Path<(String, String)>,
) -> R<StatusCode> {
    let space = authz::space_by_key(&s.db, &key).await?;
    let id = parse_id(&id)?;
    let gone = sqlx::query("DELETE FROM space_permissions WHERE id = $1 AND space_id = $2")
        .bind(id)
        .bind(space)
        .execute(&s.db)
        .await?;
    if gone.rows_affected() == 0 {
        return Err(ApiError::NotFound("permission not found".into()).into());
    }
    Ok(StatusCode::NO_CONTENT)
}

pub async fn available(State(s): State<AppState>, RawQuery(raw): RawQuery) -> R<Response> {
    let q = Q::parse(raw);
    let (limit, offset) = (q.limit()?, q.offset()?);
    let rows: Vec<Value> = perm::SPACE_PERMS
        .iter()
        .skip(offset as usize)
        .take(limit as usize + 1)
        .map(|(k, t)| {
            json!({
                "id": format!("{k}/{t}"),
                "displayName": format!("{k} {t}").replace('_', " "),
                "description": format!("Allows the operation {k} on {t}."),
                "requiredPermissionIds": if (*k, *t) == ("read", "space") { vec![] } else { vec!["read/space"] },
            })
        })
        .collect();
    Ok(paged(
        &q,
        "/wiki/api/v2/space-permissions",
        &s.site(),
        rows,
        limit,
        offset,
    ))
}

pub async fn assignments(
    State(s): State<AppState>,
    Path(id): Path<String>,
    RawQuery(raw): RawQuery,
) -> R<Response> {
    let q = Q::parse(raw);
    let (limit, offset) = (q.limit()?, q.offset()?);
    let space = crate::target::space(&s.db, &id).await?;
    let rows: Vec<(i64, String, i64, String, String)> = sqlx::query_as(
        "SELECT id, subject_type, subject_id, operation, target FROM space_permissions \
         WHERE space_id = $1 ORDER BY id LIMIT $2 OFFSET $3",
    )
    .bind(space)
    .bind(limit + 1)
    .bind(offset)
    .fetch_all(&s.db)
    .await?;
    let rows = rows
        .iter()
        .map(|(i, kind, sid, op, target)| {
            json!({
                "id": i.to_string(),
                "principal": {"type": kind, "id": sid.to_string()},
                "operation": {"key": op, "targetType": target},
            })
        })
        .collect();
    Ok(paged(
        &q,
        &format!("/wiki/api/v2/spaces/{space}/permissions"),
        &s.site(),
        rows,
        limit,
        offset,
    ))
}
