//! v1 groups. `kk-users` (the everyone group) is built in and cannot be edited.

use std::sync::Arc;

use axum::{
    extract::{Path, RawQuery, State},
    http::StatusCode,
    response::Response,
    Extension,
};
use kk_core::ApiError;
use serde::Deserialize;
use serde_json::{json, Value};

use crate::{
    authz::Access,
    error::{db_err, R},
    http::{parse_id, JsonBody, Q},
    v1, AppState,
};

fn admin_only(a: &Access) -> Result<(), ApiError> {
    if a.p.admin {
        Ok(())
    } else {
        Err(ApiError::Forbidden("requires a site administrator".into()))
    }
}

/// `(name, everyone)` of a group addressed by a query or path id; 404 when absent.
async fn find(s: &AppState, raw: &str) -> Result<(i64, bool), ApiError> {
    let missing = || ApiError::NotFound("group not found".into());
    let id = parse_id(raw).map_err(|_| missing())?;
    let row: Option<bool> = sqlx::query_scalar("SELECT everyone FROM groups WHERE id = $1")
        .bind(id)
        .fetch_optional(&s.db)
        .await
        .map_err(|e| db_err(e, ""))?;
    Ok((id, row.ok_or_else(missing)?))
}

fn required<'a>(q: &'a Q, k: &str) -> Result<&'a str, ApiError> {
    q.get(k)
        .ok_or_else(|| ApiError::BadRequest(format!("{k} is required")))
}

pub async fn list(State(s): State<AppState>, RawQuery(raw): RawQuery) -> R<Response> {
    let q = Q::parse(raw);
    let (start, limit) = v1::window(&q, 200)?;
    let ids: Vec<i64> =
        sqlx::query_scalar("SELECT id FROM groups ORDER BY lower(name), id LIMIT $1 OFFSET $2")
            .bind(limit + 1)
            .bind(start)
            .fetch_all(&s.db)
            .await?;
    let mut rows = v1::groups(&s.db, &s.site(), &ids).await?;
    rows.sort_by_key(|g| g["name"].as_str().unwrap_or("").to_lowercase());
    Ok(v1::ok(v1::page(
        &q,
        "/wiki/rest/api/group",
        &s.site(),
        rows,
        start,
        limit,
    )))
}

#[derive(Deserialize)]
pub struct Name {
    name: String,
}

pub async fn create(
    State(s): State<AppState>,
    Extension(a): Extension<Arc<Access>>,
    JsonBody(req): JsonBody<Name>,
) -> R<(StatusCode, axum::Json<Value>)> {
    admin_only(&a)?;
    let name = req.name.trim();
    if name.is_empty() || name.chars().count() > 255 {
        return Err(ApiError::BadRequest("name must be 1 to 255 characters".into()).into());
    }
    let id: i64 = sqlx::query_scalar("INSERT INTO groups (name) VALUES ($1) RETURNING id")
        .bind(name)
        .fetch_one(&s.db)
        .await
        .map_err(|e| db_err(e, "A group with this name already exists"))?;
    Ok((
        StatusCode::CREATED,
        axum::Json(v1::group_json(id, name, &s.site())),
    ))
}

pub async fn get(State(s): State<AppState>, RawQuery(raw): RawQuery) -> R<axum::Json<Value>> {
    let q = Q::parse(raw);
    let (id, _) = find(&s, required(&q, "id")?).await?;
    Ok(axum::Json(
        v1::groups(&s.db, &s.site(), &[id]).await?.remove(0),
    ))
}

pub async fn delete(
    State(s): State<AppState>,
    Extension(a): Extension<Arc<Access>>,
    RawQuery(raw): RawQuery,
) -> R<StatusCode> {
    admin_only(&a)?;
    let q = Q::parse(raw);
    let (id, everyone) = find(&s, required(&q, "id")?).await?;
    if everyone {
        return Err(ApiError::BadRequest("the built-in group cannot be deleted".into()).into());
    }
    // grants and restrictions that named the group go with it
    let mut tx = s.db.begin().await?;
    for t in ["space_permissions", "content_restrictions"] {
        sqlx::query(&format!(
            "DELETE FROM {t} WHERE subject_type = 'group' AND subject_id = $1"
        ))
        .bind(id)
        .execute(&mut *tx)
        .await?;
    }
    sqlx::query("DELETE FROM groups WHERE id = $1")
        .bind(id)
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;
    Ok(StatusCode::NO_CONTENT)
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Member {
    account_id: String,
}

async fn user_id(s: &AppState, raw: &str) -> Result<i64, ApiError> {
    let id = parse_id(raw).map_err(|_| ApiError::NotFound("user not found".into()))?;
    sqlx::query_scalar::<_, i64>("SELECT id FROM users WHERE id = $1")
        .bind(id)
        .fetch_optional(&s.db)
        .await
        .map_err(|e| db_err(e, ""))?
        .ok_or_else(|| ApiError::NotFound("user not found".into()))
}

pub async fn add_member(
    State(s): State<AppState>,
    Extension(a): Extension<Arc<Access>>,
    RawQuery(raw): RawQuery,
    JsonBody(req): JsonBody<Member>,
) -> R<StatusCode> {
    admin_only(&a)?;
    let q = Q::parse(raw);
    let (gid, everyone) = find(&s, required(&q, "groupId")?).await?;
    if everyone {
        return Err(
            ApiError::BadRequest("every user is a member of the built-in group".into()).into(),
        );
    }
    let uid = user_id(&s, &req.account_id).await?;
    sqlx::query(
        "INSERT INTO group_members (group_id, user_id) VALUES ($1, $2) ON CONFLICT DO NOTHING",
    )
    .bind(gid)
    .bind(uid)
    .execute(&s.db)
    .await?;
    Ok(StatusCode::CREATED)
}

pub async fn remove_member(
    State(s): State<AppState>,
    Extension(a): Extension<Arc<Access>>,
    RawQuery(raw): RawQuery,
) -> R<StatusCode> {
    admin_only(&a)?;
    let q = Q::parse(raw);
    let (gid, everyone) = find(&s, required(&q, "groupId")?).await?;
    if everyone {
        return Err(
            ApiError::BadRequest("members of the built-in group cannot be removed".into()).into(),
        );
    }
    let uid = user_id(&s, required(&q, "accountId")?).await?;
    let gone = sqlx::query("DELETE FROM group_members WHERE group_id = $1 AND user_id = $2")
        .bind(gid)
        .bind(uid)
        .execute(&s.db)
        .await?;
    if gone.rows_affected() == 0 {
        return Err(ApiError::NotFound("user is not a member".into()).into());
    }
    Ok(StatusCode::NO_CONTENT)
}

pub async fn members(
    State(s): State<AppState>,
    Path(id): Path<String>,
    RawQuery(raw): RawQuery,
) -> R<Response> {
    let q = Q::parse(raw);
    let (start, limit) = v1::window(&q, 200)?;
    let (gid, everyone) = find(&s, &id).await.map_err(|e| match e {
        ApiError::NotFound(_) => ApiError::BadRequest("group not found".into()),
        e => e,
    })?;
    let ids: Vec<i64> = sqlx::query_scalar(
        "SELECT u.id FROM users u WHERE $1 OR EXISTS (SELECT 1 FROM group_members m WHERE m.group_id = $2 AND m.user_id = u.id) \
         ORDER BY u.id LIMIT $3 OFFSET $4",
    )
    .bind(everyone)
    .bind(gid)
    .bind(limit + 1)
    .bind(start)
    .fetch_all(&s.db)
    .await?;
    let rows = v1::users(&s.db, &ids).await?;
    let mut page = v1::page(
        &q,
        &format!("/wiki/rest/api/group/{gid}/membersByGroupId"),
        &s.site(),
        rows,
        start,
        limit,
    );
    if q.flag("shouldReturnTotalSize")? {
        let n: i64 = sqlx::query_scalar(
            "SELECT count(*) FROM users u WHERE $1 OR EXISTS (SELECT 1 FROM group_members m WHERE m.group_id = $2 AND m.user_id = u.id)",
        )
        .bind(everyone)
        .bind(gid)
        .fetch_one(&s.db)
        .await?;
        page["totalSize"] = json!(n);
    }
    Ok(v1::ok(page))
}
