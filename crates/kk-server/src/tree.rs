use axum::{
    extract::{Path, RawQuery, State},
    response::Response,
    Extension, Json,
};
use kk_core::ApiError;
use serde_json::{json, Value};
use sqlx::PgPool;
use std::sync::Arc;

use crate::{
    authz::Access,
    error::R,
    http::{paged, parse_id, Q},
    pages::not_found,
    AppState,
};

const LIVE: &str = "('current', 'archived')";

async fn require_page(db: &PgPool, id: i64) -> R<()> {
    sqlx::query_scalar::<_, i64>(
        "SELECT id FROM content WHERE id = $1 AND type = 'page' AND status NOT IN ('trashed', 'deleted', 'historical')",
    )
    .bind(id)
    .fetch_optional(db)
    .await?
    .map(|_| ())
    .ok_or_else(|| not_found().into())
}

type Child = (i64, String, String, i64, String, i32);

fn child_json(c: &Child, with_type: bool) -> Value {
    let mut v = json!({
        "id": c.0.to_string(),
        "status": c.1,
        "title": c.2,
        "spaceId": c.3.to_string(),
        "childPosition": c.5,
    });
    if with_type {
        v["type"] = json!(c.4);
    }
    v
}

async fn kids(
    s: &AppState,
    a: &Access,
    id: &str,
    raw: Option<String>,
    route: &str,
    only_pages: bool,
) -> R<Response> {
    let q = Q::parse(raw);
    let (limit, offset) = (q.limit()?, q.offset()?);
    let order = q.order_by(&[("child-position", "c.position")], "c.id")?;
    let id = parse_id(id)?;
    require_page(&s.db, id).await?;
    let sql = format!(
        "SELECT c.id, c.status, c.title, c.space_id, c.type, c.position FROM content c \
         WHERE c.parent_id = $1 AND c.status IN {LIVE} AND (NOT $2 OR c.type = 'page') AND NOT (c.id = ANY($5)) \
         ORDER BY {order} LIMIT $3 OFFSET $4"
    );
    let rows: Vec<Child> = sqlx::query_as(&sql)
        .bind(id)
        .bind(only_pages)
        .bind(limit + 1)
        .bind(offset)
        .bind(a.hidden(&s.db).await?)
        .fetch_all(&s.db)
        .await?;
    let rows = rows.iter().map(|c| child_json(c, !only_pages)).collect();
    Ok(paged(
        &q,
        &format!("/wiki/api/v2/pages/{id}/{route}"),
        &s.site(),
        rows,
        limit,
        offset,
    ))
}

pub async fn children(
    State(s): State<AppState>,
    Extension(a): Extension<Arc<Access>>,
    Path(id): Path<String>,
    RawQuery(raw): RawQuery,
) -> R<Response> {
    kids(&s, &a, &id, raw, "children", true).await
}

pub async fn direct_children(
    State(s): State<AppState>,
    Extension(a): Extension<Arc<Access>>,
    Path(id): Path<String>,
    RawQuery(raw): RawQuery,
) -> R<Response> {
    kids(&s, &a, &id, raw, "direct-children", false).await
}

pub async fn ancestors(
    State(s): State<AppState>,
    Path(id): Path<String>,
    RawQuery(raw): RawQuery,
) -> R<Json<Value>> {
    let q = Q::parse(raw);
    let limit = q.limit()?;
    let id = parse_id(&id)?;
    require_page(&s.db, id).await?;
    let rows: Vec<(i64, String)> = sqlx::query_as(
        "WITH RECURSIVE a AS (SELECT parent_id AS id, 1 AS d FROM content WHERE id = $1 \
         UNION ALL SELECT c.parent_id, a.d + 1 FROM content c JOIN a ON c.id = a.id) \
         SELECT c.id, c.type FROM a JOIN content c ON c.id = a.id ORDER BY a.d DESC LIMIT $2",
    )
    .bind(id)
    .bind(limit)
    .fetch_all(&s.db)
    .await?;
    let results: Vec<Value> = rows
        .iter()
        .map(|(i, t)| json!({"id": i.to_string(), "type": t}))
        .collect();
    Ok(Json(
        json!({"results": results, "_links": {"base": s.site()}}),
    ))
}

pub async fn descendants(
    State(s): State<AppState>,
    Extension(a): Extension<Arc<Access>>,
    Path(id): Path<String>,
    RawQuery(raw): RawQuery,
) -> R<Response> {
    let q = Q::parse(raw);
    let (limit, offset) = (q.limit()?, q.offset()?);
    let depth = q.int("depth")?.unwrap_or(2);
    if !(1..=10).contains(&depth) {
        return Err(ApiError::BadRequest("depth must be between 1 and 10".into()).into());
    }
    let id = parse_id(&id)?;
    require_page(&s.db, id).await?;
    let sql = format!(
        "WITH RECURSIVE d AS (SELECT id, 1 AS depth FROM content WHERE parent_id = $1 AND status IN {LIVE} AND NOT (id = ANY($5)) \
         UNION ALL SELECT c.id, d.depth + 1 FROM content c JOIN d ON c.parent_id = d.id \
         WHERE d.depth < $2 AND c.status IN {LIVE} AND NOT (c.id = ANY($5))) \
         SELECT c.id, c.status, c.title, c.type, c.parent_id, d.depth, c.position FROM d \
         JOIN content c ON c.id = d.id ORDER BY d.depth, c.position, c.id LIMIT $3 OFFSET $4"
    );
    let rows: Vec<(i64, String, String, String, i64, i32, i32)> = sqlx::query_as(&sql)
        .bind(id)
        .bind(depth)
        .bind(limit + 1)
        .bind(offset)
        .bind(a.hidden(&s.db).await?)
        .fetch_all(&s.db)
        .await?;
    let rows = rows
        .iter()
        .map(|r| {
            json!({
                "id": r.0.to_string(), "status": r.1, "title": r.2, "type": r.3,
                "parentId": r.4.to_string(), "depth": r.5, "childPosition": r.6,
            })
        })
        .collect();
    Ok(paged(
        &q,
        &format!("/wiki/api/v2/pages/{id}/descendants"),
        &s.site(),
        rows,
        limit,
        offset,
    ))
}
