//! Background jobs: a Postgres queue claimed with `FOR UPDATE SKIP LOCKED`.
//! A failed job is retried with exponential backoff and marked `failed` after `MAX_ATTEMPTS`.

use std::time::Duration;

use kk_core::UserId;
use serde_json::{json, Value};
use sqlx::{PgConnection, PgPool};

use crate::authz::Access;

const MAX_ATTEMPTS: i32 = 5;

pub async fn enqueue(tx: &mut PgConnection, kind: &str, payload: Value) -> Result<(), sqlx::Error> {
    sqlx::query("INSERT INTO jobs (kind, payload) VALUES ($1, $2)")
        .bind(kind)
        .bind(payload)
        .execute(tx)
        .await
        .map(|_| ())
}

/// Tells watchers that a page was created or changed.
pub async fn notify_content(
    tx: &mut PgConnection,
    content: i64,
    actor: i64,
    event: &str,
) -> Result<(), sqlx::Error> {
    enqueue(
        tx,
        "notify.content",
        json!({"content_id": content, "actor": actor, "event": event}),
    )
    .await
}

async fn handle(db: &PgPool, kind: &str, p: &Value) -> Result<(), String> {
    match kind {
        "notify.content" => {
            let id = p["content_id"].as_i64().ok_or("missing content_id")?;
            let actor = p["actor"].as_i64().unwrap_or(0);
            let event = p["event"].as_str().unwrap_or("updated");
            let watchers: Vec<i64> = sqlx::query_scalar(
                "SELECT DISTINCT w.user_id FROM watches w, content c WHERE c.id = $1 AND w.user_id <> $2 \
                 AND ((w.target_type = 'content' AND w.target_id = c.id) \
                   OR (w.target_type = 'space' AND w.target_id = c.space_id))",
            )
            .bind(id)
            .bind(actor)
            .fetch_all(db)
            .await
            .map_err(|e| e.to_string())?;
            for u in watchers {
                // permissions are checked now, not when the watch was made
                let access = Access::load(db, UserId(u))
                    .await
                    .map_err(|e| format!("{e:?}"))?;
                if access.visible(db, id).await.is_err() {
                    continue;
                }
                sqlx::query("INSERT INTO notifications (user_id, content_id, kind, actor_id) VALUES ($1, $2, $3, NULLIF($4, 0))")
                    .bind(u)
                    .bind(id)
                    .bind(format!("content.{event}"))
                    .bind(actor)
                    .execute(db)
                    .await
                    .map_err(|e| e.to_string())?;
            }
            Ok(())
        }
        other => Err(format!("unknown job kind {other}")),
    }
}

/// Runs every job that is due; returns how many were processed.
pub async fn run_due(db: &PgPool) -> Result<usize, sqlx::Error> {
    let mut n = 0;
    loop {
        let mut tx = db.begin().await?;
        let job: Option<(i64, String, Value, i32)> = sqlx::query_as(
            "SELECT id, kind, payload, attempts FROM jobs WHERE state = 'queued' AND run_at <= now() \
             ORDER BY run_at, id LIMIT 1 FOR UPDATE SKIP LOCKED",
        )
        .fetch_optional(&mut *tx)
        .await?;
        let Some((id, kind, payload, attempts)) = job else {
            return Ok(n);
        };
        match handle(db, &kind, &payload).await {
            Ok(()) => {
                sqlx::query("UPDATE jobs SET state = 'done', attempts = $2 WHERE id = $1")
                    .bind(id)
                    .bind(attempts + 1)
                    .execute(&mut *tx)
                    .await?;
            }
            Err(e) => {
                let attempts = attempts + 1;
                tracing::warn!(job = id, %kind, attempts, error = %e, "job failed");
                let state = if attempts >= MAX_ATTEMPTS {
                    "failed"
                } else {
                    "queued"
                };
                sqlx::query(
                    "UPDATE jobs SET state = $2, attempts = $3, last_error = $4, \
                     run_at = now() + make_interval(secs => 5 * power(2, $3)) WHERE id = $1",
                )
                .bind(id)
                .bind(state)
                .bind(attempts)
                .bind(e)
                .execute(&mut *tx)
                .await?;
            }
        }
        tx.commit().await?;
        n += 1;
    }
}

/// Polls the queue until the process exits.
pub fn spawn(db: PgPool) {
    tokio::spawn(async move {
        loop {
            if let Err(e) = run_due(&db).await {
                tracing::error!(error = %e, "job runner");
            }
            tokio::time::sleep(Duration::from_secs(2)).await;
        }
    });
}
