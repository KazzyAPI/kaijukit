//! v1 content operations that act on the page tree and on version history: move, version
//! delete and restore.

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Extension, Json,
};
use kk_core::{ApiError, UserId};
use serde::Deserialize;
use serde_json::{json, Value};
use std::sync::Arc;

use crate::{
    attachments::blocking,
    authz::Access,
    error::{db_err, R},
    http::{parse_attachment_id, parse_id, ts, JsonBody},
    pages::{lock, put_version, NewVersion},
    AppState,
};

/// Content types that sit in the page tree and so share a sibling order.
const TREE: &str = "('page', 'folder', 'whiteboard', 'database', 'embed')";

fn gone() -> ApiError {
    ApiError::NotFound("content not found".into())
}

/// `PUT /content/{id}/move/{position}/{target}`: `append` makes the page the last child of the
/// target; `before` and `after` make it a sibling. Moving into another space takes the subtree along.
pub async fn move_page(
    State(s): State<AppState>,
    Extension(a): Extension<Arc<Access>>,
    Path((id, position, target)): Path<(String, String, String)>,
) -> R<Json<Value>> {
    let (id, target) = (parse_id(&id)?, parse_id(&target)?);
    if !["before", "after", "append"].contains(&position.as_str()) {
        return Err(ApiError::BadRequest("position must be before, after or append".into()).into());
    }
    if id == target {
        return Err(ApiError::BadRequest("a page cannot be moved onto itself".into()).into());
    }
    let mut tx = s.db.begin().await?;
    lock(&mut tx, id).await.map_err(|_| gone())?;
    let (space, parent): (i64, Option<i64>) = sqlx::query_as(
        "SELECT space_id, parent_id FROM content WHERE id = $1 AND status IN ('current', 'draft', 'archived')",
    )
    .bind(target)
    .fetch_optional(&mut *tx)
    .await?
    .ok_or_else(gone)?;
    a.visible(&s.db, target).await.map_err(|_| gone())?;
    let (old_space,): (i64,) = sqlx::query_as("SELECT space_id FROM content WHERE id = $1")
        .bind(id)
        .fetch_one(&mut *tx)
        .await?;
    if old_space != space {
        a.require_space(space, "create", "page")?;
    }
    let home: Option<i64> = sqlx::query_scalar("SELECT id FROM spaces WHERE homepage_id = $1")
        .bind(id)
        .fetch_optional(&mut *tx)
        .await?;
    if home.is_some() {
        return Err(ApiError::BadRequest("a space homepage cannot be moved".into()).into());
    }
    let inside: Option<i64> = sqlx::query_scalar(
        "WITH RECURSIVE d AS (SELECT id FROM content WHERE id = $1 \
         UNION SELECT c.id FROM content c JOIN d ON c.parent_id = d.id) SELECT id FROM d WHERE id = $2",
    )
    .bind(id)
    .bind(target)
    .fetch_optional(&mut *tx)
    .await?;
    if inside.is_some() {
        return Err(ApiError::BadRequest("a page cannot be moved below itself".into()).into());
    }
    let new_parent = if position == "append" {
        Some(target)
    } else {
        parent
    };
    let mut order: Vec<i64> = sqlx::query_scalar(&format!(
        "SELECT id FROM content WHERE parent_id IS NOT DISTINCT FROM $1 AND space_id = $2 AND id <> $3 \
         AND type IN {TREE} AND status IN ('current', 'draft', 'archived') ORDER BY position, id"
    ))
    .bind(new_parent)
    .bind(space)
    .bind(id)
    .fetch_all(&mut *tx)
    .await?;
    let at = match position.as_str() {
        "append" => order.len(),
        p => {
            let i = order.iter().position(|x| *x == target).unwrap_or(0);
            i + usize::from(p == "after")
        }
    };
    order.insert(at, id);
    sqlx::query("UPDATE content SET parent_id = $2 WHERE id = $1")
        .bind(id)
        .bind(new_parent)
        .execute(&mut *tx)
        .await
        .map_err(|e| db_err(e, ""))?;
    let positions: Vec<i32> = (0..order.len() as i32).collect();
    sqlx::query(
        "UPDATE content SET position = u.pos FROM unnest($1::bigint[], $2::int[]) AS u(id, pos) WHERE content.id = u.id",
    )
    .bind(&order)
    .bind(&positions)
    .execute(&mut *tx)
    .await?;
    if old_space != space {
        sqlx::query(
            "WITH RECURSIVE d AS (SELECT id FROM content WHERE id = $1 \
             UNION SELECT c.id FROM d JOIN content c ON c.parent_id = d.id \
             OR EXISTS (SELECT 1 FROM attachments a WHERE a.content_id = c.id AND a.container_id = d.id) \
             OR EXISTS (SELECT 1 FROM comments k WHERE k.content_id = c.id AND k.container_id = d.id)) \
             UPDATE content SET space_id = $2 WHERE id IN (SELECT id FROM d)",
        )
        .bind(id)
        .bind(space)
        .execute(&mut *tx)
        .await
        .map_err(|e| db_err(e, "A page with this title already exists in the target space"))?;
    }
    tx.commit().await?;
    Ok(Json(json!({"pageId": id.to_string()})))
}

/// `DELETE /content/{id}/version/{n}`: removes a historical version; the current one stays.
pub async fn delete_version(
    State(s): State<AppState>,
    Path((id, number)): Path<(String, i32)>,
) -> R<StatusCode> {
    let id = parse_attachment_id(&id)?;
    let mut tx = s.db.begin().await?;
    let (ty, current): (String, i32) = sqlx::query_as(
        "SELECT type, version_number FROM content WHERE id = $1 \
         AND status NOT IN ('deleted', 'historical') FOR UPDATE",
    )
    .bind(id)
    .fetch_optional(&mut *tx)
    .await?
    .ok_or_else(gone)?;
    if number == current {
        return Err(ApiError::BadRequest("the current version cannot be deleted".into()).into());
    }
    let key: Option<String> = sqlx::query_scalar(
        "SELECT blob_key FROM attachment_versions WHERE content_id = $1 AND number = $2",
    )
    .bind(id)
    .bind(number)
    .fetch_optional(&mut *tx)
    .await?;
    let n = sqlx::query("DELETE FROM content_versions WHERE content_id = $1 AND number = $2")
        .bind(id)
        .bind(number)
        .execute(&mut *tx)
        .await?
        .rows_affected();
    if n == 0 {
        return Err(ApiError::NotFound("version not found".into()).into());
    }
    sqlx::query("DELETE FROM attachment_versions WHERE content_id = $1 AND number = $2")
        .bind(id)
        .bind(number)
        .execute(&mut *tx)
        .await?;
    let orphan = match &key {
        Some(k) => {
            !sqlx::query_scalar::<_, bool>(
                "SELECT EXISTS (SELECT 1 FROM attachment_versions WHERE blob_key = $1)",
            )
            .bind(k)
            .fetch_one(&mut *tx)
            .await?
        }
        None => false,
    };
    tx.commit().await?;
    if let (true, Some(k)) = (orphan, key) {
        blocking(&s.blobs, move |b| b.delete(&k)).await?;
    }
    let _ = ty;
    Ok(StatusCode::NO_CONTENT)
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RestoreReq {
    operation_key: String,
    params: RestoreParams,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RestoreParams {
    version_number: i32,
    message: String,
    restore_title: Option<bool>,
}

/// `POST /content/{id}/version`: appends a new version that copies an older one.
pub async fn restore_version(
    State(s): State<AppState>,
    Extension(user): Extension<UserId>,
    Path(id): Path<String>,
    JsonBody(req): JsonBody<RestoreReq>,
) -> R<Json<Value>> {
    if req.operation_key != "restore" {
        return Err(ApiError::BadRequest("operationKey must be restore".into()).into());
    }
    let id = parse_attachment_id(&id)?;
    let p = req.params;
    let mut tx = s.db.begin().await?;
    let (ty, status, current, title): (String, String, i32, String) = sqlx::query_as(
        "SELECT type, status, version_number, title FROM content WHERE id = $1 \
         AND status NOT IN ('deleted', 'historical', 'trashed') FOR UPDATE",
    )
    .bind(id)
    .fetch_optional(&mut *tx)
    .await?
    .ok_or_else(gone)?;
    let (old_title, body): (String, Value) = sqlx::query_as(
        "SELECT title, body FROM content_versions WHERE content_id = $1 AND number = $2",
    )
    .bind(id)
    .bind(p.version_number)
    .fetch_optional(&mut *tx)
    .await?
    .ok_or_else(|| ApiError::NotFound("version not found".into()))?;
    let number = current + 1;
    let new_title = if p.restore_title.unwrap_or(false) || ty == "attachment" {
        old_title
    } else {
        title
    };
    if ty == "attachment" {
        sqlx::query(
            "INSERT INTO content_versions (content_id, number, title, body, author_id, message) \
             VALUES ($1, $2, $3, $4, $5, $6)",
        )
        .bind(id)
        .bind(number)
        .bind(&new_title)
        .bind(&body)
        .bind(user.0)
        .bind(&p.message)
        .execute(&mut *tx)
        .await?;
        sqlx::query(
            "INSERT INTO attachment_versions (content_id, number, blob_key, media_type, file_size, comment) \
             SELECT content_id, $3, blob_key, media_type, file_size, $4 FROM attachment_versions \
             WHERE content_id = $1 AND number = $2",
        )
        .bind(id)
        .bind(p.version_number)
        .bind(number)
        .bind(&p.message)
        .execute(&mut *tx)
        .await?;
        sqlx::query("UPDATE content SET version_number = $2 WHERE id = $1")
            .bind(id)
            .bind(number)
            .execute(&mut *tx)
            .await?;
    } else {
        let v = NewVersion {
            number,
            status: &status,
            title: &new_title,
            adf: &body,
            user: user.0,
            message: &p.message,
        };
        put_version(&mut tx, id, &status, v).await?;
    }
    let (at,): (chrono::DateTime<chrono::Utc>,) = sqlx::query_as(
        "SELECT created_at FROM content_versions WHERE content_id = $1 AND number = $2",
    )
    .bind(id)
    .bind(number)
    .fetch_one(&mut *tx)
    .await?;
    let by = crate::v1::users(&s.db, &[user.0]).await?;
    tx.commit().await?;
    Ok(Json(json!({
        "by": by.first(), "when": ts(at), "message": p.message, "number": number, "minorEdit": false,
    })))
}
