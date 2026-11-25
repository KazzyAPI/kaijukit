//! kaijukit extension: read API over the `notifications` rows the job worker writes.

use std::sync::Arc;

use axum::{
    extract::{Path, RawQuery, State},
    http::StatusCode,
    response::Response,
    Extension, Json,
};
use kk_core::ApiError;
use serde_json::{json, Value};

use crate::{
    authz::Access,
    error::{db_err, R},
    http::{parse_id, ts, Q},
    v1, AppState,
};

type Row = (
    i64,
    String,
    bool,
    chrono::DateTime<chrono::Utc>,
    Option<i64>,
    Option<String>,
    i64,
    String,
    String,
    String,
);

pub async fn list(
    State(s): State<AppState>,
    Extension(a): Extension<Arc<Access>>,
    RawQuery(raw): RawQuery,
) -> R<Response> {
    let q = Q::parse(raw);
    let (start, limit) = v1::window(&q, 25)?;
    let unread = q.flag("unread")?;
    let rows: Vec<Row> = sqlx::query_as(
        "SELECT n.id, n.kind, n.read, n.created_at, n.actor_id, au.display_name, \
                c.id, c.type, c.title, sp.key \
         FROM notifications n JOIN content c ON c.id = n.content_id \
         JOIN spaces sp ON sp.id = c.space_id LEFT JOIN users au ON au.id = n.actor_id \
         WHERE n.user_id = $1 AND (NOT $2 OR NOT n.read) AND c.status <> 'trashed' \
         ORDER BY n.id DESC LIMIT $3 OFFSET $4",
    )
    .bind(a.p.id)
    .bind(unread)
    .bind(limit + 1)
    .bind(start)
    .fetch_all(&s.db)
    .await
    .map_err(|e| db_err(e, ""))?;
    let mut out = Vec::with_capacity(rows.len());
    for (id, kind, read, at, actor, actor_name, cid, ty, title, key) in rows {
        // permissions are checked at read time, as the worker does at write time
        if a.visible(&s.db, cid).await.is_err() {
            continue;
        }
        out.push(json!({
            "id": id.to_string(),
            "kind": kind,
            "read": read,
            "createdAt": ts(at),
            "actor": actor.map(|u| json!({"accountId": u.to_string(), "displayName": actor_name})),
            "content": {
                "id": cid.to_string(), "type": ty, "title": title, "spaceKey": key,
                "_links": {"webui": format!("/spaces/{key}/pages/{cid}")},
            },
        }));
    }
    Ok(v1::ok(v1::page(
        &q,
        "/wiki/rest/kk/v1/notifications",
        &s.site(),
        out,
        start,
        limit,
    )))
}

pub async fn unread_count(
    State(s): State<AppState>,
    Extension(a): Extension<Arc<Access>>,
) -> R<Json<Value>> {
    let n: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM notifications n JOIN content c ON c.id = n.content_id \
         WHERE n.user_id = $1 AND NOT n.read AND c.status <> 'trashed'",
    )
    .bind(a.p.id)
    .fetch_one(&s.db)
    .await?;
    Ok(Json(json!({"count": n})))
}

pub async fn mark_read(
    State(s): State<AppState>,
    Extension(a): Extension<Arc<Access>>,
    Path(nid): Path<String>,
) -> R<StatusCode> {
    let missing = || ApiError::NotFound("notification not found".into());
    let id = parse_id(&nid).map_err(|_| missing())?;
    let n = sqlx::query("UPDATE notifications SET read = true WHERE id = $1 AND user_id = $2")
        .bind(id)
        .bind(a.p.id)
        .execute(&s.db)
        .await?
        .rows_affected();
    if n == 0 {
        return Err(missing().into());
    }
    Ok(StatusCode::NO_CONTENT)
}

pub async fn mark_all_read(
    State(s): State<AppState>,
    Extension(a): Extension<Arc<Access>>,
) -> R<StatusCode> {
    sqlx::query("UPDATE notifications SET read = true WHERE user_id = $1 AND NOT read")
        .bind(a.p.id)
        .execute(&s.db)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}
