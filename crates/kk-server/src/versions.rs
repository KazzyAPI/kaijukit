use axum::{
    extract::{Path, RawQuery, State},
    response::Response,
    Json,
};
use chrono::{DateTime, Utc};
use kk_core::ApiError;
use serde_json::{json, Value};

use crate::{
    body,
    error::R,
    http::{paged, parse_id, ts, Q},
    pages::not_found,
    AppState,
};

type VersionRow = (i32, i64, String, bool, DateTime<Utc>, String, Value);

pub async fn list(
    State(s): State<AppState>,
    Path(id): Path<String>,
    RawQuery(raw): RawQuery,
) -> R<Response> {
    let q = Q::parse(raw);
    let (limit, offset) = (q.limit()?, q.offset()?);
    let repr = body::format(&q, body::PRIMARY)?;
    let order = q.order_by(&[("modified-date", "v.number")], "v.number")?;
    let id = parse_id(&id)?;
    let sql = format!(
        "SELECT v.number, v.author_id, v.message, v.minor_edit, v.created_at, v.title, v.body \
         FROM content_versions v JOIN content c ON c.id = v.content_id \
         WHERE v.content_id = $1 AND c.type = 'page' ORDER BY {order} LIMIT $2 OFFSET $3"
    );
    let rows: Vec<VersionRow> = sqlx::query_as(&sql)
        .bind(id)
        .bind(limit + 1)
        .bind(offset)
        .fetch_all(&s.db)
        .await?;
    if rows.is_empty() && offset == 0 {
        return Err(not_found().into());
    }
    let rows = rows
        .iter()
        .map(|(number, author, message, minor, created, title, adf)| {
            Ok(json!({
                "number": number,
                "authorId": author.to_string(),
                "message": message,
                "minorEdit": minor,
                "createdAt": ts(*created),
                "page": {
                    "id": id.to_string(),
                    "title": title,
                    "body": repr.map(|f| body::render(adf, f)).transpose()?.unwrap_or_else(|| json!({})),
                },
            }))
        })
        .collect::<Result<_, ApiError>>()?;
    Ok(paged(
        &q,
        &format!("/wiki/api/v2/pages/{id}/versions"),
        &s.site(),
        rows,
        limit,
        offset,
    ))
}

pub async fn get(
    State(s): State<AppState>,
    Path((id, number)): Path<(String, String)>,
) -> R<Json<Value>> {
    let id = parse_id(&id)?;
    let number: i32 = number
        .parse()
        .map_err(|_| ApiError::BadRequest("invalid version number".into()))?;
    let row: (i64, String, bool, DateTime<Utc>, bool) = sqlx::query_as(
        "SELECT v.author_id, v.message, v.minor_edit, v.created_at, \
         EXISTS (SELECT 1 FROM content_versions n WHERE n.content_id = v.content_id AND n.number = v.number + 1) \
         FROM content_versions v JOIN content c ON c.id = v.content_id \
         WHERE v.content_id = $1 AND v.number = $2 AND c.type = 'page'",
    )
    .bind(id)
    .bind(number)
    .fetch_optional(&s.db)
    .await?
    .ok_or_else(|| ApiError::NotFound("version not found".into()))?;
    let mut v = json!({
        "number": number,
        "authorId": row.0.to_string(),
        "message": row.1,
        "minorEdit": row.2,
        "createdAt": ts(row.3),
        "contentTypeModified": false,
        "collaborators": [row.0.to_string()],
    });
    if number > 1 {
        v["prevVersion"] = json!(number - 1);
    }
    if row.4 {
        v["nextVersion"] = json!(number + 1);
    }
    Ok(Json(v))
}
