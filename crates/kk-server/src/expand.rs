//! `include-*` expansions on single-content reads: labels, properties, operations, likes and
//! versions as `{results, meta, _links}` blocks.

use kk_core::ApiError;
use serde_json::{json, Value};

use crate::{
    authz::Access,
    error::db_err,
    http::{ts, Q},
    AppState,
};

type PropRow = (
    i64,
    String,
    Value,
    i32,
    String,
    i64,
    chrono::DateTime<chrono::Utc>,
);

/// Items per expanded block; `meta.hasMore` says whether more exist.
const BLOCK: i64 = 50;

fn block(mut rows: Vec<Value>, self_link: String) -> Value {
    let more = rows.len() as i64 > BLOCK;
    rows.truncate(BLOCK as usize);
    json!({"results": rows, "meta": {"hasMore": more}, "_links": {"self": self_link}})
}

fn version(
    n: i32,
    author: i64,
    message: &str,
    minor: bool,
    at: chrono::DateTime<chrono::Utc>,
) -> Value {
    json!({"number": n, "authorId": author.to_string(), "message": message, "minorEdit": minor, "createdAt": ts(at)})
}

/// Adds the blocks the query asks for to `v`; `seg` is the content type's path segment.
pub async fn apply(
    s: &AppState,
    a: &Access,
    v: &mut Value,
    id: i64,
    seg: &str,
    q: &Q,
    allowed: &[&str],
) -> Result<(), ApiError> {
    let want = |name: &str| -> Result<bool, ApiError> {
        Ok(allowed.contains(&name) && q.flag(&format!("include-{name}"))?)
    };
    let site = format!("{}/api/v2/{seg}/{id}", s.site());
    if want("labels")? {
        let rows: Vec<(i64, String, String)> = sqlx::query_as(
            "SELECT l.id, l.prefix, l.name FROM content_labels x JOIN labels l ON l.id = x.label_id \
             WHERE x.content_id = $1 ORDER BY l.id LIMIT $2",
        )
        .bind(id)
        .bind(BLOCK + 1)
        .fetch_all(&s.db)
        .await
        .map_err(|e| db_err(e, ""))?;
        let rows = rows
            .iter()
            .map(|(i, p, n)| json!({"id": i.to_string(), "prefix": p, "name": n}))
            .collect();
        v["labels"] = block(rows, format!("{site}/labels"));
    }
    if want("properties")? {
        let rows: Vec<PropRow> = sqlx::query_as(
            "SELECT id, key, value, version_number, message, updated_by, updated_at \
                 FROM content_properties WHERE owner_id = $1 ORDER BY key, id LIMIT $2",
        )
        .bind(id)
        .bind(BLOCK + 1)
        .fetch_all(&s.db)
        .await
        .map_err(|e| db_err(e, ""))?;
        let rows = rows
            .iter()
            .map(|(i, k, val, n, m, by, at)| {
                json!({"id": i.to_string(), "key": k, "value": val, "version": version(*n, *by, m, false, *at)})
            })
            .collect();
        v["properties"] = block(rows, format!("{site}/properties"));
    }
    if want("operations")? {
        let ops = crate::likes::ops_list(s, a, id).await?;
        let rows = ops
            .iter()
            .map(|(o, t)| json!({"operation": o, "targetType": t}))
            .collect();
        v["operations"] = block(rows, format!("{site}/operations"));
    }
    if want("likes")? {
        let ids: Vec<i64> = sqlx::query_scalar(
            "SELECT user_id FROM likes WHERE content_id = $1 ORDER BY created_at, user_id LIMIT $2",
        )
        .bind(id)
        .bind(BLOCK + 1)
        .fetch_all(&s.db)
        .await
        .map_err(|e| db_err(e, ""))?;
        let rows = ids
            .iter()
            .map(|u| json!({"accountId": u.to_string()}))
            .collect();
        v["likes"] = block(rows, format!("{site}/likes/users"));
    }
    if want("versions")? {
        let rows: Vec<(i32, i64, String, bool, chrono::DateTime<chrono::Utc>)> = sqlx::query_as(
            "SELECT number, author_id, message, minor_edit, created_at FROM content_versions \
             WHERE content_id = $1 ORDER BY number LIMIT $2",
        )
        .bind(id)
        .bind(BLOCK + 1)
        .fetch_all(&s.db)
        .await
        .map_err(|e| db_err(e, ""))?;
        let rows = rows
            .iter()
            .map(|(n, by, m, minor, at)| version(*n, *by, m, *minor, *at))
            .collect();
        v["versions"] = block(rows, format!("{site}/versions"));
    }
    if q.get("include-version") == Some("false") {
        if let Some(o) = v.as_object_mut() {
            o.remove("version");
        }
    }
    if allowed.contains(&"favorited-by-current-user-status")
        && q.flag("include-favorited-by-current-user-status")?
    {
        v["isFavoritedByCurrentUser"] = json!(false);
    }
    Ok(())
}

/// Blocks a page read can carry.
pub const PAGE: &[&str] = &[
    "labels",
    "properties",
    "operations",
    "likes",
    "versions",
    "favorited-by-current-user-status",
];
