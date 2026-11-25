//! v2 tasks: rows mirror `taskItem` nodes of published page bodies; status changes write back to the body.

use axum::{
    extract::{Path, RawQuery, State},
    response::Response,
    Extension, Json,
};
use chrono::{DateTime, TimeZone, Utc};
use kk_adf::{Doc, Node};
use kk_core::{ApiError, UserId};
use serde::Deserialize;
use serde_json::{json, Value};
use sqlx::PgConnection;
use std::sync::Arc;

use crate::{
    authz::Access,
    body,
    error::{db_err, R},
    http::{paged, parse_id, ts, JsonBody, Q},
    pages, AppState,
};

fn blank(content: &[Node]) -> bool {
    content
        .iter()
        .all(|n| n.text.as_deref().is_some_and(|t| t.trim().is_empty()))
}

/// Brings the task rows of a page in line with its newly published body.
pub async fn sync(
    tx: &mut PgConnection,
    page: i64,
    adf: &Value,
    user: i64,
) -> Result<(), sqlx::Error> {
    let Ok(doc) = serde_json::from_value::<Doc>(adf.clone()) else {
        return Ok(());
    };
    let found = kk_adf::tasks(&doc);
    let ids: Vec<&str> = found.iter().map(|t| t.local_id.as_str()).collect();
    sqlx::query("DELETE FROM tasks WHERE page_id = $1 AND local_id <> ALL($2)")
        .bind(page)
        .bind(&ids)
        .execute(&mut *tx)
        .await?;
    for t in &found {
        let body = serde_json::to_value(Doc::new(vec![
            Node::new("paragraph").content(t.content.clone())
        ]))
        .unwrap_or_default();
        let due: Option<DateTime<Utc>> = t.due.and_then(|ms| Utc.timestamp_millis_opt(ms).single());
        let assignee: Option<i64> = t.assignee.as_deref().and_then(|a| a.parse().ok());
        let status = if t.done { "complete" } else { "incomplete" };
        sqlx::query(
            "INSERT INTO tasks (page_id, space_id, local_id, status, body, blank, created_by, assigned_to, due_at, \
             completed_by, completed_at) \
             SELECT $1, space_id, $2, $3, $4, $5, $6, (SELECT id FROM users WHERE id = $7), $8, \
             CASE WHEN $3 = 'complete' THEN $6 END, CASE WHEN $3 = 'complete' THEN now() END \
             FROM content WHERE id = $1 \
             ON CONFLICT (page_id, local_id) DO UPDATE SET body = EXCLUDED.body, blank = EXCLUDED.blank, \
             assigned_to = EXCLUDED.assigned_to, due_at = EXCLUDED.due_at, \
             updated_at = CASE WHEN tasks.status <> EXCLUDED.status OR tasks.body <> EXCLUDED.body \
                 THEN now() ELSE tasks.updated_at END, \
             completed_by = CASE WHEN EXCLUDED.status <> 'complete' THEN NULL \
                 WHEN tasks.status <> 'complete' THEN $6 ELSE tasks.completed_by END, \
             completed_at = CASE WHEN EXCLUDED.status <> 'complete' THEN NULL \
                 WHEN tasks.status <> 'complete' THEN now() ELSE tasks.completed_at END, \
             status = EXCLUDED.status",
        )
        .bind(page)
        .bind(&t.local_id)
        .bind(status)
        .bind(body)
        .bind(blank(&t.content))
        .bind(user)
        .bind(assignee)
        .bind(due)
        .execute(&mut *tx)
        .await?;
    }
    Ok(())
}

#[derive(sqlx::FromRow)]
struct Row {
    id: i64,
    page_id: i64,
    space_id: i64,
    local_id: String,
    status: String,
    body: Value,
    created_by: i64,
    assigned_to: Option<i64>,
    completed_by: Option<i64>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    due_at: Option<DateTime<Utc>>,
    completed_at: Option<DateTime<Utc>>,
}

const SELECT: &str =
    "SELECT t.id, t.page_id, t.space_id, t.local_id, t.status, t.body, t.created_by, \
    t.assigned_to, t.completed_by, t.created_at, t.updated_at, t.due_at, t.completed_at \
    FROM tasks t JOIN content c ON c.id = t.page_id AND c.status = 'current'";

fn render(r: &Row, repr: Option<&str>) -> Result<Value, ApiError> {
    let mut v = json!({
        "id": r.id.to_string(), "localId": r.local_id, "spaceId": r.space_id.to_string(),
        "pageId": r.page_id.to_string(), "status": r.status,
        "body": match repr { Some(f) => body::render(&r.body, f)?, None => json!({}) },
        "createdBy": r.created_by.to_string(),
        "createdAt": ts(r.created_at), "updatedAt": ts(r.updated_at),
    });
    let opt = [
        ("assignedTo", r.assigned_to.map(|u| u.to_string())),
        ("completedBy", r.completed_by.map(|u| u.to_string())),
        ("dueAt", r.due_at.map(ts)),
        ("completedAt", r.completed_at.map(ts)),
    ];
    for (k, x) in opt {
        if let Some(x) = x {
            v[k] = json!(x);
        }
    }
    Ok(v)
}

fn range(q: &Q, from: &str, to: &str) -> Result<(Option<i64>, Option<i64>), ApiError> {
    let n = |k: &str| {
        q.get(k)
            .map(|v| {
                v.parse::<i64>()
                    .map_err(|_| ApiError::BadRequest(format!("invalid {k}")))
            })
            .transpose()
    };
    Ok((n(from)?, n(to)?))
}

pub async fn list(
    State(s): State<AppState>,
    Extension(a): Extension<Arc<Access>>,
    RawQuery(raw): RawQuery,
) -> R<Response> {
    let q = Q::parse(raw);
    let (limit, offset) = (q.limit()?, q.offset()?);
    let repr = body::format(&q, body::PRIMARY)?;
    let status = match q.get("status") {
        None => None,
        Some(v @ ("complete" | "incomplete")) => Some(v),
        Some(_) => return Err(ApiError::BadRequest("invalid status".into()).into()),
    };
    let (created, due, done) = (
        range(&q, "created-at-from", "created-at-to")?,
        range(&q, "due-at-from", "due-at-to")?,
        range(&q, "completed-at-from", "completed-at-to")?,
    );
    let ms = |c: &str, a: usize| {
        format!("(${a}::bigint IS NULL OR extract(epoch from {c}) * 1000 >= ${a}) AND (${b}::bigint IS NULL OR extract(epoch from {c}) * 1000 <= ${b})", b = a + 1)
    };
    let sql = format!(
        "{SELECT} WHERE ($1::text IS NULL OR t.status = $1) AND ($2::bigint[] IS NULL OR t.id = ANY($2)) \
         AND ($3::bigint[] IS NULL OR t.space_id = ANY($3)) AND ($4::bigint[] IS NULL OR t.page_id = ANY($4)) \
         AND ($5::bigint[] IS NULL OR t.created_by = ANY($5)) AND ($6::bigint[] IS NULL OR t.assigned_to = ANY($6)) \
         AND ($7::bigint[] IS NULL OR t.completed_by = ANY($7)) AND ($8 OR NOT t.blank) AND NOT $9 \
         AND {} AND {} AND {} AND NOT (t.page_id = ANY($18)) \
         AND ($19::bigint[] IS NULL OR t.space_id = ANY($19)) ORDER BY t.id LIMIT $16 OFFSET $17",
        ms("t.created_at", 10),
        ms("t.due_at", 12),
        ms("t.completed_at", 14),
    );
    let blogs = q.ids("blogpost-id")?.is_some();
    let rows: Vec<Row> = sqlx::query_as(&sql)
        .bind(status)
        .bind(q.ids("task-id")?)
        .bind(q.ids("space-id")?)
        .bind(q.ids("page-id")?)
        .bind(q.ids("created-by")?)
        .bind(q.ids("assigned-to")?)
        .bind(q.ids("completed-by")?)
        .bind(q.flag("include-blank-tasks")?)
        .bind(blogs)
        .bind(created.0)
        .bind(created.1)
        .bind(due.0)
        .bind(due.1)
        .bind(done.0)
        .bind(done.1)
        .bind(limit + 1)
        .bind(offset)
        .bind(a.hidden(&s.db).await?)
        .bind(a.readable_spaces())
        .fetch_all(&s.db)
        .await?;
    let rows = rows
        .iter()
        .map(|r| render(r, repr))
        .collect::<Result<_, _>>()?;
    Ok(paged(
        &q,
        "/wiki/api/v2/tasks",
        &s.site(),
        rows,
        limit,
        offset,
    ))
}

async fn load(conn: &mut PgConnection, id: i64, lock: bool) -> Result<Row, ApiError> {
    let suffix = if lock { " FOR UPDATE OF t" } else { "" };
    sqlx::query_as(&format!("{SELECT} WHERE t.id = $1{suffix}"))
        .bind(id)
        .fetch_optional(conn)
        .await
        .map_err(|e| db_err(e, ""))?
        .ok_or_else(|| ApiError::NotFound("task not found".into()))
}

pub async fn get(
    State(s): State<AppState>,
    Path(id): Path<String>,
    RawQuery(raw): RawQuery,
) -> R<Json<Value>> {
    let q = Q::parse(raw);
    let repr = body::format(&q, body::PRIMARY)?;
    let row = load(&mut *s.db.acquire().await?, parse_id(&id)?, false).await?;
    let mut v = render(&row, repr)?;
    v["_links"] = json!({"base": s.site()});
    Ok(Json(v))
}

#[derive(Deserialize)]
pub struct UpdateReq {
    status: String,
}

pub async fn update(
    State(s): State<AppState>,
    Extension(user): Extension<UserId>,
    Path(id): Path<String>,
    RawQuery(raw): RawQuery,
    JsonBody(req): JsonBody<UpdateReq>,
) -> R<Json<Value>> {
    let q = Q::parse(raw);
    let repr = body::format(&q, body::PRIMARY)?;
    let done = match req.status.as_str() {
        "complete" => true,
        "incomplete" => false,
        _ => {
            return Err(ApiError::BadRequest("status must be complete or incomplete".into()).into())
        }
    };
    let id = parse_id(&id)?;
    let mut tx = s.db.begin().await?;
    let task = load(&mut tx, id, true).await?;
    if task.status != req.status {
        pages::write_back(&mut tx, task.page_id, user.0, |doc| {
            kk_adf::set_task_done(doc, &task.local_id, done)
        })
        .await?;
    }
    let row = load(&mut tx, id, false).await?;
    tx.commit().await?;
    let mut v = render(&row, repr)?;
    v["_links"] = json!({"base": s.site()});
    Ok(Json(v))
}
