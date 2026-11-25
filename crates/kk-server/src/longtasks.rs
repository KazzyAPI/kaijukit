//! v1 long-running tasks. Every task runs inline, so its record is created already finished.

use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Extension, Json,
};
use kk_core::ApiError;
use serde_json::{json, Value};
use sqlx::PgConnection;

use crate::{
    authz::Access,
    error::{db_err, R},
    http::parse_id,
    pages, AppState,
};

/// Records a finished task for `user` and returns its id.
pub async fn record(
    db: &mut PgConnection,
    kind: &str,
    user: i64,
    ok: bool,
) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar(
        "INSERT INTO long_tasks (kind, user_id, successful) VALUES ($1, $2, $3) RETURNING id",
    )
    .bind(kind)
    .bind(user)
    .bind(ok)
    .fetch_one(db)
    .await
}

pub async fn get(
    State(s): State<AppState>,
    Extension(a): Extension<Arc<Access>>,
    Path(id): Path<String>,
) -> R<Json<Value>> {
    let missing = || ApiError::NotFound("task not found".into());
    let id = parse_id(&id).map_err(|_| missing())?;
    let (kind, user, ok, ms): (String, i64, bool, i64) = sqlx::query_as(
        "SELECT kind, user_id, successful, \
         (extract(epoch FROM finished_at - created_at) * 1000)::bigint FROM long_tasks WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(&s.db)
    .await
    .map_err(|e| db_err(e, ""))?
    .ok_or_else(missing)?;
    if user != a.p.id && !a.p.admin {
        return Err(missing().into());
    }
    Ok(Json(json!({
        "id": id.to_string(),
        "name": {"key": format!("kk.longtask.{kind}"), "args": []},
        "elapsedTime": ms,
        "percentageComplete": 100,
        "successful": ok,
        "finished": true,
        "status": if ok { "COMPLETE" } else { "FAILED" },
        "messages": [],
        "_links": {"base": s.site(), "context": "/wiki", "status": format!("/rest/api/longtask/{id}")},
    })))
}

/// v1 `deletePageTree`: trashes the page and its live descendants, like the v2 delete.
pub async fn delete_page_tree(
    State(s): State<AppState>,
    Extension(a): Extension<Arc<Access>>,
    Path(id): Path<String>,
) -> R<(StatusCode, Json<Value>)> {
    pages::delete_ty(&s, "page", &id, None).await?;
    let task = record(&mut *s.db.acquire().await?, "deletePageTree", a.p.id, true).await?;
    Ok((
        StatusCode::ACCEPTED,
        Json(
            json!({"id": task.to_string(), "links": {"status": format!("/rest/api/longtask/{task}")}}),
        ),
    ))
}
