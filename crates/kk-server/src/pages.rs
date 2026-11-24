use axum::{
    extract::{Path, RawQuery, State},
    http::StatusCode,
    response::Response,
    Extension, Json,
};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use chrono::{DateTime, Utc};
use kk_core::{ApiError, ContentStatus, ContentType, UserId};
use serde::Deserialize;
use serde_json::{json, Value};
use sqlx::PgConnection;

use crate::{
    body,
    error::{db_err, R},
    http::{paged, parse_id, ts, JsonBody, Q},
    AppState,
};

pub fn not_found() -> ApiError {
    ApiError::NotFound("page not found".into())
}

#[derive(sqlx::FromRow)]
pub struct PageRow {
    id: i64,
    space_id: i64,
    space_key: String,
    status: String,
    title: String,
    parent_id: Option<i64>,
    parent_type: Option<String>,
    position: i32,
    author_id: i64,
    created_at: DateTime<Utc>,
    v_number: i32,
    v_author: i64,
    v_message: String,
    v_minor: bool,
    v_created: DateTime<Utc>,
    pub body: Value,
}

/// Page row joined with the version selected by `ver` (an SQL expression).
fn select(ver: &str) -> String {
    format!(
        "SELECT c.id, c.space_id, s.key AS space_key, c.status, v.title, c.parent_id, \
         p.type AS parent_type, c.position, c.author_id, c.created_at, v.number AS v_number, \
         v.author_id AS v_author, v.message AS v_message, v.minor_edit AS v_minor, \
         v.created_at AS v_created, v.body \
         FROM content c JOIN spaces s ON s.id = c.space_id \
         JOIN content_versions v ON v.content_id = c.id AND v.number = {ver} \
         LEFT JOIN content p ON p.id = c.parent_id"
    )
}

async fn load(
    conn: &mut PgConnection,
    id: i64,
    ver: Option<i32>,
) -> Result<Option<PageRow>, sqlx::Error> {
    let sql = format!(
        "{} WHERE c.id = $1 AND c.type = 'page'",
        select("COALESCE($2, c.version_number)")
    );
    sqlx::query_as(&sql)
        .bind(id)
        .bind(ver)
        .fetch_optional(conn)
        .await
}

fn tiny(id: i64) -> String {
    let b = id.to_be_bytes();
    let skip = b.iter().take_while(|x| **x == 0).count().min(7);
    URL_SAFE_NO_PAD.encode(&b[skip..])
}

fn page_json(r: &PageRow, repr: Option<&str>) -> Result<Value, ApiError> {
    let body = match repr {
        Some(f) => body::render(&r.body, f)?,
        None => json!({}),
    };
    Ok(json!({
        "id": r.id.to_string(),
        "status": r.status,
        "title": r.title,
        "spaceId": r.space_id.to_string(),
        "parentId": r.parent_id.map(|i| i.to_string()),
        "parentType": r.parent_type,
        "position": r.position,
        "authorId": r.author_id.to_string(),
        "ownerId": r.author_id.to_string(),
        "lastOwnerId": null,
        "createdAt": ts(r.created_at),
        "version": {
            "createdAt": ts(r.v_created),
            "message": r.v_message,
            "number": r.v_number,
            "minorEdit": r.v_minor,
            "authorId": r.v_author.to_string(),
        },
        "body": body,
        "_links": {
            "webui": format!("/spaces/{}/pages/{}/{}", r.space_key, r.id, r.title.replace(' ', "+")),
            "editui": format!("/spaces/{}/pages/edit-v2/{}", r.space_key, r.id),
            "tinyui": format!("/x/{}", tiny(r.id)),
        },
    }))
}

fn single(s: &AppState, r: &PageRow, repr: Option<&str>) -> Result<Json<Value>, ApiError> {
    let mut v = page_json(r, repr)?;
    v["_links"]["base"] = json!(s.site());
    Ok(Json(v))
}

fn statuses(q: &Q, default: &[&str]) -> Result<Vec<String>, ApiError> {
    let given = q.list("status");
    if given.is_empty() {
        return Ok(default.iter().map(|s| s.to_string()).collect());
    }
    given
        .into_iter()
        .map(|s| s.parse::<ContentStatus>().map(|_| s.to_string()))
        .collect()
}

pub async fn get(
    State(s): State<AppState>,
    Path(id): Path<String>,
    RawQuery(raw): RawQuery,
) -> R<Json<Value>> {
    let q = Q::parse(raw);
    let repr = body::format(&q, body::SINGLE)?;
    let allowed = statuses(&q, &["current", "draft", "archived"])?;
    let mut conn = s.db.acquire().await?;
    let row = load(&mut conn, parse_id(&id)?, q.int("version")?)
        .await?
        .filter(|r| allowed.contains(&r.status))
        .ok_or_else(not_found)?;
    Ok(single(&s, &row, repr)?)
}

async fn list_pages(s: &AppState, path: &str, q: Q, space: Option<i64>) -> R<Response> {
    let (limit, offset) = (q.limit()?, q.offset()?);
    let repr = body::format(&q, body::PRIMARY)?;
    let order = q.order_by(
        &[
            ("id", "c.id"),
            ("created-date", "c.created_at"),
            ("modified-date", "v.created_at"),
            ("title", "v.title"),
        ],
        "c.id",
    )?;
    let spaces = match space {
        Some(id) => Some(vec![id]),
        None => q.ids("space-id")?,
    };
    let root_only = match q.get("depth") {
        None | Some("all") => false,
        Some("root") => true,
        Some(_) => return Err(ApiError::BadRequest("invalid depth".into()).into()),
    };
    let sql = format!(
        "{} WHERE c.type = 'page' AND ($1::bigint[] IS NULL OR c.id = ANY($1)) \
         AND ($2::bigint[] IS NULL OR c.space_id = ANY($2)) AND c.status = ANY($3) \
         AND ($4::text IS NULL OR v.title = $4) AND (NOT $5 OR c.parent_id IS NULL) \
         ORDER BY {order} LIMIT $6 OFFSET $7",
        select("c.version_number")
    );
    let rows: Vec<PageRow> = sqlx::query_as(&sql)
        .bind(q.ids("id")?)
        .bind(spaces)
        .bind(statuses(&q, &["current"])?)
        .bind(q.get("title"))
        .bind(root_only)
        .bind(limit + 1)
        .bind(offset)
        .fetch_all(&s.db)
        .await?;
    let rows = rows
        .iter()
        .map(|r| page_json(r, repr))
        .collect::<Result<_, _>>()?;
    Ok(paged(&q, path, &s.site(), rows, limit, offset))
}

pub async fn list(State(s): State<AppState>, RawQuery(raw): RawQuery) -> R<Response> {
    list_pages(&s, "/wiki/api/v2/pages", Q::parse(raw), None).await
}

pub async fn in_space(
    State(s): State<AppState>,
    Path(id): Path<String>,
    RawQuery(raw): RawQuery,
) -> R<Response> {
    let id = parse_id(&id)?;
    sqlx::query_scalar::<_, i64>("SELECT id FROM spaces WHERE id = $1")
        .bind(id)
        .fetch_optional(&s.db)
        .await?
        .ok_or_else(|| ApiError::NotFound("space not found".into()))?;
    list_pages(
        &s,
        &format!("/wiki/api/v2/spaces/{id}/pages"),
        Q::parse(raw),
        Some(id),
    )
    .await
}

/// Appends a page as the last child of `parent` with version 1.
pub async fn insert_page(
    conn: &mut PgConnection,
    space: i64,
    parent: Option<i64>,
    status: &str,
    title: &str,
    adf: &Value,
    user: i64,
) -> Result<i64, ApiError> {
    let dup = "A page with this title already exists in the space";
    let id: i64 = sqlx::query_scalar(
        "INSERT INTO content (type, status, space_id, parent_id, position, title, author_id) \
         VALUES ($1, $2, $3, $4, (SELECT COALESCE(MAX(position) + 1, 0) FROM content \
         WHERE parent_id IS NOT DISTINCT FROM $4), $5, $6) RETURNING id",
    )
    .bind(ContentType::Page.as_str())
    .bind(status)
    .bind(space)
    .bind(parent)
    .bind(title)
    .bind(user)
    .fetch_one(&mut *conn)
    .await
    .map_err(|e| db_err(e, dup))?;
    sqlx::query("INSERT INTO content_versions (content_id, number, title, body, author_id) VALUES ($1, 1, $2, $3, $4)")
        .bind(id)
        .bind(title)
        .bind(adf)
        .bind(user)
        .execute(conn)
        .await
        .map_err(|e| db_err(e, dup))?;
    Ok(id)
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateReq {
    space_id: String,
    status: Option<String>,
    title: Option<String>,
    parent_id: Option<String>,
    body: Option<Value>,
}

fn draft_or_current(s: &str) -> Result<&str, ApiError> {
    match s {
        "current" | "draft" => Ok(s),
        _ => Err(ApiError::BadRequest(
            "status must be current or draft".into(),
        )),
    }
}

pub async fn create(
    State(s): State<AppState>,
    Extension(user): Extension<UserId>,
    RawQuery(raw): RawQuery,
    JsonBody(req): JsonBody<CreateReq>,
) -> R<Json<Value>> {
    let q = Q::parse(raw);
    let root_level = q.flag("root-level")?;
    if root_level && req.parent_id.is_some() {
        return Err(
            ApiError::BadRequest("root-level cannot be combined with parentId".into()).into(),
        );
    }
    let status = draft_or_current(req.status.as_deref().unwrap_or("current"))?;
    let title = req.title.unwrap_or_default();
    if status == "current" && title.trim().is_empty() {
        return Err(ApiError::BadRequest("title is required for published pages".into()).into());
    }
    let (doc, repr) = match &req.body {
        Some(b) => {
            let (d, r) = body::parse_write(b)?;
            (d, Some(r))
        }
        None => (kk_adf::Doc::new(vec![]), None),
    };
    let space = parse_id(&req.space_id)?;
    let mut tx = s.db.begin().await?;
    let home: Option<i64> =
        sqlx::query_scalar("SELECT homepage_id FROM spaces WHERE id = $1 AND status = 'current'")
            .bind(space)
            .fetch_optional(&mut *tx)
            .await?
            .ok_or_else(|| ApiError::NotFound("space not found".into()))?;
    let parent = match req.parent_id.as_deref().map(parse_id).transpose()? {
        Some(p) => {
            let ok = sqlx::query_scalar::<_, i64>(
                "SELECT id FROM content WHERE id = $1 AND space_id = $2 AND status IN ('current', 'draft', 'archived')",
            )
            .bind(p)
            .bind(space)
            .fetch_optional(&mut *tx)
            .await?;
            Some(ok.ok_or_else(|| ApiError::NotFound("parent not found in space".into()))?)
        }
        None if root_level => None,
        None => home,
    };
    let adf = serde_json::to_value(&doc).map_err(|e| ApiError::Internal(e.to_string()))?;
    let id = insert_page(&mut tx, space, parent, status, title.trim(), &adf, user.0).await?;
    let row = load(&mut tx, id, None).await?.ok_or_else(not_found)?;
    tx.commit().await?;
    Ok(single(&s, &row, repr)?)
}

#[derive(Deserialize)]
pub struct UpdateReq {
    id: String,
    status: String,
    title: String,
    body: Value,
    version: VersionReq,
}

#[derive(Deserialize)]
pub struct VersionReq {
    number: Option<i32>,
    message: Option<String>,
}

async fn lock(tx: &mut PgConnection, id: i64) -> Result<(String, i32), ApiError> {
    sqlx::query_as::<_, (String, i32)>(
        "SELECT status, version_number FROM content WHERE id = $1 AND type = 'page' \
         AND status NOT IN ('deleted', 'historical') FOR UPDATE",
    )
    .bind(id)
    .fetch_optional(tx)
    .await
    .map_err(|e| db_err(e, ""))?
    .ok_or_else(not_found)
}

struct NewVersion<'a> {
    number: i32,
    status: &'a str,
    title: &'a str,
    adf: &'a Value,
    user: i64,
    message: &'a str,
}

/// Appends a version, restoring the page and its trashed subtree when it was trashed.
async fn put_version(
    tx: &mut PgConnection,
    id: i64,
    was: &str,
    v: NewVersion<'_>,
) -> Result<(), ApiError> {
    let dup = "A page with this title already exists in the space";
    if was == "trashed" {
        sqlx::query(
            "UPDATE content SET status = 'current', trashed_with = NULL WHERE trashed_with = $1",
        )
        .bind(id)
        .execute(&mut *tx)
        .await
        .map_err(|e| db_err(e, dup))?;
    }
    sqlx::query(
        "INSERT INTO content_versions (content_id, number, title, body, author_id, message) \
         VALUES ($1, $2, $3, $4, $5, $6)",
    )
    .bind(id)
    .bind(v.number)
    .bind(v.title)
    .bind(v.adf)
    .bind(v.user)
    .bind(v.message)
    .execute(&mut *tx)
    .await
    .map_err(|e| db_err(e, dup))?;
    sqlx::query("UPDATE content SET title = $2, status = $3, version_number = $4 WHERE id = $1")
        .bind(id)
        .bind(v.title)
        .bind(v.status)
        .bind(v.number)
        .execute(tx)
        .await
        .map_err(|e| db_err(e, dup))?;
    Ok(())
}

/// Shared by full and title-only updates: checks the status transition.
fn transition(was: &str, to: &str) -> Result<(), ApiError> {
    match (was, to) {
        ("current", "draft") => Err(ApiError::BadRequest(
            "a published page cannot become a draft".into(),
        )),
        ("trashed", "draft") => Err(not_found()),
        _ => Ok(()),
    }
}

pub async fn update(
    State(s): State<AppState>,
    Extension(user): Extension<UserId>,
    Path(id): Path<String>,
    JsonBody(req): JsonBody<UpdateReq>,
) -> R<Json<Value>> {
    let id = parse_id(&id)?;
    if parse_id(&req.id)? != id {
        return Err(ApiError::BadRequest("id in body does not match the path".into()).into());
    }
    let status = draft_or_current(&req.status)?;
    let title = req.title.trim();
    if status == "current" && title.is_empty() {
        return Err(ApiError::BadRequest("title is required for published pages".into()).into());
    }
    let number = req
        .version
        .number
        .ok_or_else(|| ApiError::BadRequest("version.number is required".into()))?;
    let (doc, repr) = body::parse_write(&req.body)?;
    let adf = serde_json::to_value(&doc).map_err(|e| ApiError::Internal(e.to_string()))?;
    let mut tx = s.db.begin().await?;
    let (was, current) = lock(&mut tx, id).await?;
    transition(&was, status)?;
    if number != current + 1 {
        return Err(ApiError::Conflict(format!(
            "version.number must be {}, got {number}",
            current + 1
        ))
        .into());
    }
    let message = req.version.message.unwrap_or_default();
    let v = NewVersion {
        number,
        status,
        title,
        adf: &adf,
        user: user.0,
        message: &message,
    };
    put_version(&mut tx, id, &was, v).await?;
    let row = load(&mut tx, id, None).await?.ok_or_else(not_found)?;
    tx.commit().await?;
    Ok(single(&s, &row, Some(repr))?)
}

#[derive(Deserialize)]
pub struct TitleReq {
    status: String,
    title: String,
}

pub async fn update_title(
    State(s): State<AppState>,
    Extension(user): Extension<UserId>,
    Path(id): Path<String>,
    JsonBody(req): JsonBody<TitleReq>,
) -> R<Json<Value>> {
    let id = parse_id(&id)?;
    let status = draft_or_current(&req.status)?;
    let title = req.title.trim();
    if status == "current" && title.is_empty() {
        return Err(ApiError::BadRequest("title is required for published pages".into()).into());
    }
    let mut tx = s.db.begin().await?;
    let (was, current) = lock(&mut tx, id).await?;
    transition(&was, status)?;
    let adf: Value = sqlx::query_scalar(
        "SELECT body FROM content_versions WHERE content_id = $1 AND number = $2",
    )
    .bind(id)
    .bind(current)
    .fetch_one(&mut *tx)
    .await?;
    let v = NewVersion {
        number: current + 1,
        status,
        title,
        adf: &adf,
        user: user.0,
        message: "",
    };
    put_version(&mut tx, id, &was, v).await?;
    let row = load(&mut tx, id, None).await?.ok_or_else(not_found)?;
    tx.commit().await?;
    Ok(single(&s, &row, None)?)
}

pub async fn delete(
    State(s): State<AppState>,
    Path(id): Path<String>,
    RawQuery(raw): RawQuery,
) -> R<StatusCode> {
    let q = Q::parse(raw);
    let (purge, draft) = (q.flag("purge")?, q.flag("draft")?);
    let id = parse_id(&id)?;
    let mut tx = s.db.begin().await?;
    let (status, _) = lock(&mut tx, id).await?;
    if draft {
        if status != "draft" {
            return Err(not_found().into());
        }
        sqlx::query("DELETE FROM content WHERE id = $1")
            .bind(id)
            .execute(&mut *tx)
            .await
            .map_err(|e| db_err(e, ""))?;
    } else if purge {
        if status != "trashed" {
            return Err(ApiError::BadRequest("only trashed pages can be purged".into()).into());
        }
        sqlx::query("DELETE FROM content WHERE trashed_with = $1")
            .bind(id)
            .execute(&mut *tx)
            .await
            .map_err(|e| db_err(e, ""))?;
    } else {
        if status == "trashed" {
            return Err(not_found().into());
        }
        let home = sqlx::query_scalar::<_, i64>("SELECT id FROM spaces WHERE homepage_id = $1")
            .bind(id)
            .fetch_optional(&mut *tx)
            .await?;
        if home.is_some() {
            return Err(ApiError::BadRequest("a space homepage cannot be deleted".into()).into());
        }
        sqlx::query(
            "WITH RECURSIVE t AS (SELECT id FROM content WHERE id = $1 \
             UNION ALL SELECT c.id FROM content c JOIN t ON c.parent_id = t.id \
             WHERE c.status IN ('current', 'draft', 'archived')) \
             UPDATE content SET status = 'trashed', trashed_with = $1 WHERE id IN (SELECT id FROM t)",
        )
        .bind(id)
        .execute(&mut *tx)
        .await?;
    }
    tx.commit().await?;
    Ok(StatusCode::NO_CONTENT)
}
