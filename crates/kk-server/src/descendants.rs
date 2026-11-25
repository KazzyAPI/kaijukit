//! v1 `getContentDescendants` and `getDescendantsOfType`.

use std::sync::Arc;

use axum::{
    extract::{Path, RawQuery, State},
    response::Response,
    Extension, Json,
};
use kk_core::ApiError;
use serde_json::{json, Value};

use crate::{
    authz::Access,
    error::{db_err, R},
    http::{parse_id, Q},
    search::{content_json, Hit, DERIVED, HIT_COLS},
    v1, AppState,
};

fn depth(q: &Q) -> Result<i32, ApiError> {
    match q.get("depth") {
        None | Some("all") => Ok(100),
        Some("root") => Ok(1),
        Some(n) => n
            .parse::<i32>()
            .ok()
            .filter(|d| (1..=100).contains(d))
            .ok_or_else(|| ApiError::BadRequest("depth must be all, root or 1 to 100".into())),
    }
}

/// Descendants of one kind: pages by tree depth; comments and attachments of the page and of
/// the pages below it within the same depth.
async fn of_type(
    s: &AppState,
    a: &Access,
    id: i64,
    kind: &str,
    depth: i32,
    start: i64,
    limit: i64,
) -> Result<Vec<Hit>, ApiError> {
    let sql = format!(
        "WITH RECURSIVE t AS (SELECT $3::bigint AS id, 0 AS depth \
           UNION ALL SELECT c.id, t.depth + 1 FROM content c JOIN t ON c.parent_id = t.id \
             WHERE c.type = 'page' AND t.depth < $4) \
         SELECT {HIT_COLS} {DERIVED} AND c.type = $5 AND c.id IN ( \
           SELECT id FROM t WHERE depth > 0 AND $5 = 'page' \
           UNION SELECT k.content_id FROM comments k JOIN t ON k.container_id = t.id WHERE $5 = 'comment' AND t.depth < $4 \
           UNION SELECT k.content_id FROM attachments k JOIN t ON k.container_id = t.id WHERE $5 = 'attachment' AND t.depth < $4) \
         AND NOT (c.id = ANY($2)) ORDER BY c.id LIMIT {} OFFSET {start}",
        limit + 1
    );
    sqlx::query_as(&sql)
        .bind(a.readable_spaces())
        .bind(a.hidden(&s.db).await?)
        .bind(id)
        .bind(depth)
        .bind(kind)
        .fetch_all(&s.db)
        .await
        .map_err(|e| db_err(e, ""))
}

async fn live_page(s: &AppState, raw: &str) -> Result<i64, ApiError> {
    let id = parse_id(raw)?;
    sqlx::query_scalar::<_, i64>("SELECT id FROM content WHERE id = $1 AND type = 'page' AND status IN ('current', 'archived')")
        .bind(id)
        .fetch_optional(&s.db)
        .await
        .map_err(|e| db_err(e, ""))?
        .ok_or_else(|| ApiError::NotFound("content not found".into()))
}

pub async fn all(
    State(s): State<AppState>,
    Extension(a): Extension<Arc<Access>>,
    Path(id): Path<String>,
) -> R<Json<Value>> {
    let id = live_page(&s, &id).await?;
    let site = s.site();
    let mut out = json!({"_expandable": {}, "_links": {"base": site, "context": "/wiki"}});
    for kind in ["page", "comment", "attachment"] {
        let rows = of_type(&s, &a, id, kind, 100, 0, 25).await?;
        let items = rows
            .iter()
            .take(25)
            .map(|h| content_json(h, &site))
            .collect();
        out[kind] = v1::page(
            &Q::parse(None),
            &format!("/wiki/rest/api/content/{id}/descendant/{kind}"),
            &site,
            items,
            0,
            25,
        );
    }
    Ok(Json(out))
}

pub async fn typed(
    State(s): State<AppState>,
    Extension(a): Extension<Arc<Access>>,
    Path((id, kind)): Path<(String, String)>,
    RawQuery(raw): RawQuery,
) -> R<Response> {
    let q = Q::parse(raw);
    if !["page", "comment", "attachment"].contains(&kind.as_str()) {
        return Err(ApiError::BadRequest("type must be page, comment or attachment".into()).into());
    }
    let id = live_page(&s, &id).await?;
    let (start, limit) = v1::window(&q, 25)?;
    let rows = of_type(&s, &a, id, &kind, depth(&q)?, start, limit).await?;
    let site = s.site();
    let items = rows.iter().map(|h| content_json(h, &site)).collect();
    Ok(v1::ok(v1::page(
        &q,
        &format!("/wiki/rest/api/content/{id}/descendant/{kind}"),
        &site,
        items,
        start,
        limit,
    )))
}
