//! Blog posts, folders, whiteboards, databases, smart links and custom content: one set of
//! handlers over the generic `content` table, selected by a `const K` from `kinds`.

use axum::{
    extract::{Path, RawQuery, State},
    http::StatusCode,
    response::Response,
    Extension, Json,
};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use chrono::{DateTime, Utc};
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
    kinds::{self, BLOG, CUSTOM, EMBED},
    pages::{self, PageRow},
    AppState,
};

fn nf(k: u8) -> ApiError {
    ApiError::NotFound(format!("{} not found", kinds::of(k).ty))
}

fn tiny(id: i64) -> String {
    let b = id.to_be_bytes();
    let skip = b.iter().take_while(|x| **x == 0).count().min(7);
    URL_SAFE_NO_PAD.encode(&b[skip..])
}

/// Custom content keeps an app-defined string body as `{"kkRaw": value}`; everything else is ADF.
pub(crate) fn render_body(adf: &Value, repr: &str) -> Result<Value, ApiError> {
    match adf.get("kkRaw").and_then(Value::as_str) {
        Some(raw) => Ok(json!({repr: {"representation": "raw", "value": raw}})),
        None => body::render(adf, repr),
    }
}

fn parse_body(b: &Value, allow_raw: bool) -> Result<(Value, &'static str), ApiError> {
    let flat = b["representation"].as_str().zip(b["value"].as_str());
    if let (true, Some(("raw", v))) = (allow_raw, flat) {
        return Ok((json!({"kkRaw": v}), "raw"));
    }
    if let (true, Some(v)) = (allow_raw, b["raw"]["value"].as_str()) {
        return Ok((json!({"kkRaw": v}), "raw"));
    }
    let (doc, repr) = body::parse_write(b)?;
    let adf = serde_json::to_value(&doc).map_err(|e| ApiError::Internal(e.to_string()))?;
    Ok((adf, repr))
}

fn row_json(k: u8, r: &PageRow, repr: Option<&str>) -> Result<Value, ApiError> {
    let version = json!({
        "createdAt": ts(r.v_created), "message": r.v_message, "number": r.v_number,
        "minorEdit": r.v_minor, "authorId": r.v_author.to_string(),
    });
    let rendered = match repr {
        Some(f) => render_body(&r.body, f)?,
        None => json!({}),
    };
    let key = &r.space_key;
    Ok(match k {
        BLOG => json!({
            "id": r.id.to_string(), "status": r.status, "title": r.title,
            "spaceId": r.space_id.to_string(), "authorId": r.author_id.to_string(),
            "createdAt": ts(r.created_at), "version": version, "body": rendered,
            "_links": {
                "webui": format!("/spaces/{key}/blog/{}/{}/{}", r.created_at.format("%Y/%m/%d"), r.id,
                                 r.title.replace(' ', "+")),
                "editui": format!("/spaces/{key}/blog/edit-v2/{}", r.id),
                "tinyui": format!("/x/{}", tiny(r.id)),
            },
        }),
        CUSTOM => {
            let mut v = json!({
                "id": r.id.to_string(), "type": r.custom_type, "status": r.status, "title": r.title,
                "spaceId": r.space_id.to_string(), "authorId": r.author_id.to_string(),
                "createdAt": ts(r.created_at), "version": version, "body": rendered,
                "_links": {"webui": format!("/spaces/{key}/custom/{}", r.id)},
            });
            if let Some(p) = r.parent_id {
                let field = match r.parent_type.as_deref() {
                    Some("blogpost") => "blogPostId",
                    Some("custom") => "customContentId",
                    _ => "pageId",
                };
                v[field] = json!(p.to_string());
            }
            v
        }
        _ => {
            let seg = match k {
                EMBED => "smart-link",
                _ => kinds::of(k).ty,
            };
            let mut v = json!({
                "id": r.id.to_string(), "type": r.ty, "status": r.status, "title": r.title,
                "parentId": r.parent_id.map(|i| i.to_string()), "parentType": r.parent_type,
                "position": r.position, "authorId": r.author_id.to_string(),
                "ownerId": r.author_id.to_string(), "createdAt": ts(r.created_at),
                "spaceId": r.space_id.to_string(), "version": version,
                "_links": {"webui": format!("/spaces/{key}/{seg}/{}", r.id)},
            });
            if k == EMBED {
                v["embedUrl"] = json!(r.embed_url);
            }
            v
        }
    })
}

fn expansions(k: u8) -> &'static [&'static str] {
    match k {
        BLOG => crate::expand::PAGE,
        CUSTOM => &["labels", "properties", "operations", "versions"],
        _ => &["properties", "operations"],
    }
}

const SINGLE_BODY: &[&str] = &[
    "storage",
    "atlas_doc_format",
    "view",
    "export_view",
    "anonymous_export_view",
    "styled_view",
    "editor",
];
const CUSTOM_BODY: &[&str] = &[
    "raw",
    "storage",
    "atlas_doc_format",
    "view",
    "export_view",
    "anonymous_export_view",
    "styled_view",
    "editor",
];

fn body_formats(k: u8) -> &'static [&'static str] {
    if k == CUSTOM {
        CUSTOM_BODY
    } else {
        SINGLE_BODY
    }
}

pub async fn get<const K: u8>(
    State(s): State<AppState>,
    Extension(a): Extension<Arc<Access>>,
    Path(id): Path<String>,
    RawQuery(raw): RawQuery,
) -> R<Json<Value>> {
    let q = Q::parse(raw);
    let repr = body::format(&q, body_formats(K))?;
    let allowed: Vec<String> = match q.list("status") {
        v if v.is_empty() => ["current", "draft", "archived"].map(String::from).to_vec(),
        v => v.into_iter().map(String::from).collect(),
    };
    let mut conn = s.db.acquire().await?;
    let version = q.int("version")?;
    let mut row = pages::load_ty(&mut conn, kinds::of(K).ty, parse_id(&id)?, version)
        .await?
        .filter(|r| allowed.contains(&r.status))
        .ok_or_else(|| nf(K))?;
    if K == BLOG && q.flag("get-draft")? && version.is_none() {
        pages::overlay_draft(&mut conn, &mut row).await?;
    }
    let mut v = row_json(K, &row, repr)?;
    v["_links"]["base"] = json!(s.site());
    crate::expand::apply(&s, &a, &mut v, row.id, kinds::of(K).seg, &q, expansions(K)).await?;
    Ok(Json(v))
}

pub async fn delete<const K: u8>(
    State(s): State<AppState>,
    Path(id): Path<String>,
    RawQuery(raw): RawQuery,
) -> R<StatusCode> {
    pages::delete_ty(&s, kinds::of(K).ty, &id, raw).await
}

/// Appends a content row as the last child of `parent` (or at the root) with version 1.
#[allow(clippy::too_many_arguments)]
async fn insert(
    conn: &mut PgConnection,
    k: u8,
    custom_type: Option<&str>,
    embed_url: Option<&str>,
    space: i64,
    parent: Option<i64>,
    status: &str,
    title: &str,
    adf: &Value,
    user: i64,
    created: Option<DateTime<Utc>>,
) -> Result<i64, ApiError> {
    let id: i64 = sqlx::query_scalar(
        "INSERT INTO content (type, status, space_id, parent_id, position, title, author_id, custom_type, embed_url, created_at) \
         VALUES ($1, $2, $3, $4, (SELECT COALESCE(MAX(position) + 1, 0) FROM content \
         WHERE parent_id IS NOT DISTINCT FROM $4 AND space_id = $3 AND type = ANY($10)), $5, $6, $7, $8, COALESCE($9, now())) \
         RETURNING id",
    )
    .bind(kinds::of(k).ty)
    .bind(status)
    .bind(space)
    .bind(parent)
    .bind(title)
    .bind(user)
    .bind(custom_type)
    .bind(embed_url)
    .bind(created)
    .bind(kinds::TREE_TYPES.map(String::from).to_vec())
    .fetch_one(&mut *conn)
    .await
    .map_err(|e| db_err(e, "duplicate title"))?;
    sqlx::query("INSERT INTO content_versions (content_id, number, title, body, author_id, created_at) VALUES ($1, 1, $2, $3, $4, COALESCE($5, now()))")
        .bind(id)
        .bind(title)
        .bind(adf)
        .bind(user)
        .bind(created)
        .execute(&mut *conn)
        .await
        .map_err(|e| db_err(e, ""))?;
    Ok(id)
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateReq {
    space_id: Option<String>,
    status: Option<String>,
    title: Option<String>,
    parent_id: Option<String>,
    body: Option<Value>,
    created_at: Option<String>,
    embed_url: Option<String>,
    #[serde(rename = "type")]
    ty: Option<String>,
    page_id: Option<String>,
    blog_post_id: Option<String>,
    custom_content_id: Option<String>,
}

async fn live_in_space(
    tx: &mut PgConnection,
    id: i64,
    space: i64,
    types: &[&str],
) -> Result<Option<i64>, sqlx::Error> {
    sqlx::query_scalar(
        "SELECT id FROM content WHERE id = $1 AND space_id = $2 AND type = ANY($3) \
         AND status IN ('current', 'draft', 'archived')",
    )
    .bind(id)
    .bind(space)
    .bind(types.iter().map(|t| t.to_string()).collect::<Vec<_>>())
    .fetch_optional(tx)
    .await
}

pub async fn create<const K: u8>(
    State(s): State<AppState>,
    Extension(user): Extension<UserId>,
    Extension(access): Extension<Arc<Access>>,
    JsonBody(req): JsonBody<CreateReq>,
) -> R<(StatusCode, Json<Value>)> {
    let status = pages::draft_or_current(req.status.as_deref().unwrap_or("current"))?;
    let title = req.title.clone().unwrap_or_default();
    let title = title.trim();
    if matches!(K, BLOG | CUSTOM) && status == "current" && title.is_empty() {
        return Err(ApiError::BadRequest("title is required".into()).into());
    }
    let title = if title.is_empty() && !matches!(K, BLOG | CUSTOM) {
        "Untitled"
    } else {
        title
    };
    let mut tx = s.db.begin().await?;
    // the container of custom content decides the space
    let (space, parent) = if K == CUSTOM {
        let given = [&req.page_id, &req.blog_post_id, &req.custom_content_id]
            .iter()
            .filter(|c| c.is_some())
            .count()
            + usize::from(req.space_id.is_some());
        if req.ty.as_deref().is_none_or(str::is_empty) || given != 1 {
            return Err(ApiError::BadRequest(
                "type and exactly one of spaceId, pageId, blogPostId or customContentId are required".into(),
            )
            .into());
        }
        match (&req.page_id, &req.blog_post_id, &req.custom_content_id) {
            (Some(c), _, _) | (_, Some(c), _) | (_, _, Some(c)) => {
                let c = parse_id(c)?;
                let (sp,): (i64,) = sqlx::query_as(
                    "SELECT space_id FROM content WHERE id = $1 AND type IN ('page', 'blogpost', 'custom') \
                     AND status IN ('current', 'draft', 'archived')",
                )
                .bind(c)
                .fetch_optional(&mut *tx)
                .await?
                .ok_or_else(|| ApiError::NotFound("container not found".into()))?;
                access
                    .visible(&s.db, c)
                    .await
                    .map_err(|_| ApiError::NotFound("container not found".into()))?;
                (sp, Some(c))
            }
            _ => (parse_id(req.space_id.as_deref().unwrap_or_default())?, None),
        }
    } else {
        let sid = req
            .space_id
            .as_deref()
            .ok_or_else(|| ApiError::BadRequest("spaceId is required".into()))?;
        let sp = parse_id(sid)?;
        let parent = match req.parent_id.as_deref().map(parse_id).transpose()? {
            Some(p) => {
                let p = live_in_space(&mut tx, p, sp, &kinds::TREE_TYPES).await?;
                let p = p.ok_or_else(|| ApiError::NotFound("parent not found in space".into()))?;
                access
                    .visible(&s.db, p)
                    .await
                    .map_err(|_| ApiError::NotFound("parent not found in space".into()))?;
                Some(p)
            }
            None => None,
        };
        (sp, parent)
    };
    let exists: Option<i64> =
        sqlx::query_scalar("SELECT id FROM spaces WHERE id = $1 AND status = 'current'")
            .bind(space)
            .fetch_optional(&mut *tx)
            .await?;
    exists.ok_or_else(|| ApiError::NotFound("space not found".into()))?;
    access.require_space(space, "create", if K == BLOG { "blogpost" } else { "page" })?;
    let (adf, repr) = match (&req.body, K) {
        (Some(b), BLOG | CUSTOM) => {
            let (v, r) = parse_body(b, K == CUSTOM)?;
            (v, Some(r))
        }
        (None, CUSTOM) => return Err(ApiError::BadRequest("body is required".into()).into()),
        (None, BLOG) => (
            serde_json::to_value(kk_adf::Doc::new(vec![])).unwrap_or_default(),
            None,
        ),
        _ => (json!({}), None),
    };
    let created = req
        .created_at
        .as_deref()
        .map(|c| {
            DateTime::parse_from_rfc3339(c)
                .map(|d| d.with_timezone(&Utc))
                .map_err(|_| ApiError::BadRequest("createdAt must be an RFC 3339 timestamp".into()))
        })
        .transpose()?;
    let embed_url = if K == EMBED {
        let u = req
            .embed_url
            .as_deref()
            .filter(|u| u.starts_with("http://") || u.starts_with("https://"))
            .ok_or_else(|| ApiError::BadRequest("embedUrl must be an http or https URL".into()))?;
        Some(u)
    } else {
        None
    };
    let id = insert(
        &mut tx,
        K,
        req.ty.as_deref(),
        embed_url,
        space,
        parent,
        status,
        title,
        &adf,
        user.0,
        created,
    )
    .await?;
    if K == BLOG && status == "current" {
        crate::jobs::notify_content(&mut tx, id, user.0, "created")
            .await
            .map_err(|e| db_err(e, ""))?;
        crate::tasks::sync(&mut tx, id, &adf, user.0)
            .await
            .map_err(|e| db_err(e, ""))?;
    }
    let row = pages::load_ty(&mut tx, kinds::of(K).ty, id, None)
        .await?
        .ok_or_else(|| nf(K))?;
    tx.commit().await?;
    let mut v = row_json(K, &row, repr)?;
    v["_links"]["base"] = json!(s.site());
    // the spec answers 201 for custom content and 200 for the rest
    Ok((
        if K == CUSTOM {
            StatusCode::CREATED
        } else {
            StatusCode::OK
        },
        Json(v),
    ))
}

#[derive(Deserialize)]
pub struct UpdateReq {
    id: String,
    status: String,
    title: String,
    body: Value,
    version: pages::VersionReq,
}

pub async fn update<const K: u8>(
    State(s): State<AppState>,
    Extension(user): Extension<UserId>,
    Path(id): Path<String>,
    JsonBody(req): JsonBody<UpdateReq>,
) -> R<Json<Value>> {
    let id = parse_id(&id)?;
    if parse_id(&req.id)? != id {
        return Err(ApiError::BadRequest("id in body does not match the path".into()).into());
    }
    let status = if K == CUSTOM && req.status == "current" {
        "current"
    } else {
        pages::draft_or_current(&req.status)?
    };
    let title = req.title.trim();
    if status == "current" && title.is_empty() {
        return Err(ApiError::BadRequest("title is required".into()).into());
    }
    let number = req
        .version
        .number
        .ok_or_else(|| ApiError::BadRequest("version.number is required".into()))?;
    let (adf, repr) = parse_body(&req.body, K == CUSTOM)?;
    let ty = kinds::of(K).ty;
    let mut tx = s.db.begin().await?;
    let (was, current) = pages::lock_ty(&mut tx, ty, id).await.map_err(|_| nf(K))?;
    pages::transition(&was, status).map_err(|_| nf(K))?;
    if K == BLOG && status == "draft" && was != "draft" {
        pages::save_draft(&mut tx, id, title, &adf, user.0).await?;
        let mut row = pages::load_ty(&mut tx, ty, id, None)
            .await?
            .ok_or_else(|| nf(K))?;
        pages::overlay_draft(&mut tx, &mut row).await?;
        tx.commit().await?;
        let mut v = row_json(K, &row, Some(repr))?;
        v["_links"]["base"] = json!(s.site());
        return Ok(Json(v));
    }
    if number != current + 1 {
        return Err(ApiError::Conflict(format!(
            "version.number must be {}, got {number}",
            current + 1
        ))
        .into());
    }
    let message = req.version.message.unwrap_or_default();
    if K == BLOG {
        let v = pages::NewVersion {
            number,
            status,
            title,
            adf: &adf,
            user: user.0,
            message: &message,
        };
        pages::put_version(&mut tx, id, &was, v).await?;
    } else {
        if was == "trashed" {
            sqlx::query("UPDATE content SET status = 'current', trashed_with = NULL WHERE trashed_with = $1")
                .bind(id)
                .execute(&mut *tx)
                .await?;
        }
        sqlx::query("INSERT INTO content_versions (content_id, number, title, body, author_id, message) VALUES ($1, $2, $3, $4, $5, $6)")
            .bind(id)
            .bind(number)
            .bind(title)
            .bind(&adf)
            .bind(user.0)
            .bind(&message)
            .execute(&mut *tx)
            .await?;
        sqlx::query(
            "UPDATE content SET title = $2, status = $3, version_number = $4 WHERE id = $1",
        )
        .bind(id)
        .bind(title)
        .bind(status)
        .bind(number)
        .execute(&mut *tx)
        .await?;
    }
    let row = pages::load_ty(&mut tx, ty, id, None)
        .await?
        .ok_or_else(|| nf(K))?;
    tx.commit().await?;
    let mut v = row_json(K, &row, Some(repr))?;
    v["_links"]["base"] = json!(s.site());
    Ok(Json(v))
}

/// Scope of a list: all rows of the type, or those in a space, under a container or with a label.
#[derive(Default)]
pub struct Scope {
    pub space: Option<i64>,
    pub label: Option<i64>,
    pub parent: Option<i64>,
}

pub async fn list_kind(
    s: &AppState,
    a: &Access,
    k: u8,
    path: &str,
    q: Q,
    scope: Scope,
) -> R<Response> {
    let (limit, offset) = (q.limit()?, q.offset()?);
    let repr = body::format(&q, body_formats(k))?;
    let order = q.order_by(
        &[
            ("id", "c.id"),
            ("created-date", "c.created_at"),
            ("modified-date", "v.created_at"),
            ("title", "v.title"),
        ],
        "c.id",
    )?;
    let custom_type = q.get("type");
    if k == CUSTOM && scope.parent.is_none() && custom_type.is_none() && scope.space.is_none() {
        return Err(ApiError::BadRequest("type is required".into()).into());
    }
    if k == CUSTOM && custom_type.is_none() && scope.parent.is_none() {
        return Err(ApiError::BadRequest("type is required".into()).into());
    }
    let spaces = match scope.space {
        Some(id) => Some(vec![id]),
        None => q.ids("space-id")?,
    };
    let statuses: Vec<String> = match q.list("status") {
        v if v.is_empty() => vec!["current".into()],
        v => v.into_iter().map(String::from).collect(),
    };
    let sql = format!(
        "{} WHERE c.type = $1 AND ($2::bigint[] IS NULL OR c.id = ANY($2)) \
         AND ($3::bigint[] IS NULL OR c.space_id = ANY($3)) AND c.status = ANY($4) \
         AND ($5::text IS NULL OR v.title = $5) AND ($6::text IS NULL OR c.custom_type = $6) \
         AND ($7::bigint IS NULL OR c.parent_id = $7) \
         AND ($8::bigint[] IS NULL OR c.space_id = ANY($8)) AND NOT (c.id = ANY($9)) \
         AND ($10::bigint IS NULL OR EXISTS (SELECT 1 FROM content_labels cl WHERE cl.content_id = c.id AND cl.label_id = $10)) \
         ORDER BY {order} LIMIT $11 OFFSET $12",
        pages::select("c.version_number")
    );
    let rows: Vec<PageRow> = sqlx::query_as(&sql)
        .bind(kinds::of(k).ty)
        .bind(q.ids("id")?)
        .bind(spaces)
        .bind(statuses)
        .bind(q.get("title"))
        .bind(custom_type)
        .bind(scope.parent)
        .bind(a.readable_spaces())
        .bind(a.hidden(&s.db).await?)
        .bind(scope.label)
        .bind(limit + 1)
        .bind(offset)
        .fetch_all(&s.db)
        .await?;
    let rows = rows
        .iter()
        .map(|r| row_json(k, r, repr))
        .collect::<Result<_, _>>()?;
    Ok(paged(&q, path, &s.site(), rows, limit, offset))
}

pub async fn list<const K: u8>(
    State(s): State<AppState>,
    Extension(a): Extension<Arc<Access>>,
    RawQuery(raw): RawQuery,
) -> R<Response> {
    let path = format!("/wiki/api/v2/{}", kinds::of(K).seg);
    list_kind(&s, &a, K, &path, Q::parse(raw), Scope::default()).await
}

/// Lists of type `K` below a space (`SCOPE` 0), a label (1) or a container content row (2).
pub async fn list_in<const K: u8, const SCOPE: u8>(
    State(s): State<AppState>,
    Extension(a): Extension<Arc<Access>>,
    Path(id): Path<String>,
    RawQuery(raw): RawQuery,
) -> R<Response> {
    let id = parse_id(&id)?;
    let (table, seg) = match SCOPE {
        0 => ("spaces", "spaces"),
        1 => ("labels", "labels"),
        _ => ("content", ""),
    };
    let found: Option<i64> = sqlx::query_scalar(&format!("SELECT id FROM {table} WHERE id = $1"))
        .bind(id)
        .fetch_optional(&s.db)
        .await?;
    found.ok_or_else(|| ApiError::NotFound("not found".into()))?;
    let (seg, scope) = match SCOPE {
        0 => (
            seg.to_string(),
            Scope {
                space: Some(id),
                ..Default::default()
            },
        ),
        1 => (
            seg.to_string(),
            Scope {
                label: Some(id),
                ..Default::default()
            },
        ),
        _ => {
            // the container's own segment: the route decides which type it must be
            let ty: String = sqlx::query_scalar("SELECT type FROM content WHERE id = $1")
                .bind(id)
                .fetch_one(&s.db)
                .await?;
            let seg = match ty.as_str() {
                "page" => "pages",
                "blogpost" => "blogposts",
                _ => "custom-content",
            };
            (
                seg.to_string(),
                Scope {
                    parent: Some(id),
                    ..Default::default()
                },
            )
        }
    };
    let path = format!("/wiki/api/v2/{seg}/{id}/{}", kinds::of(K).seg);
    list_kind(&s, &a, K, &path, Q::parse(raw), scope).await
}

/// Databases carry no entries, fields or views yet: the lists are empty and an entry is 404.
pub async fn database_list<const WHAT: u8>(
    State(s): State<AppState>,
    Path(id): Path<String>,
    RawQuery(raw): RawQuery,
) -> R<Json<Value>> {
    let id = crate::target::owner(&s.db, &id, "database").await?;
    let q = Q::parse(raw);
    let limit = q.limit_or(50)?;
    let what = ["entries", "fields", "views"][WHAT as usize];
    let me = format!("/wiki/api/v2/databases/{id}/{what}");
    Ok(Json(json!({
        "results": [],
        "meta": {"pageSize": limit, "hasMore": false},
        "_links": {"self": me, "base": s.site()},
    })))
}

pub async fn database_entry(
    State(s): State<AppState>,
    Path((id, _entry)): Path<(String, String)>,
) -> R<(StatusCode, Json<Value>)> {
    crate::target::owner(&s.db, &id, "database").await?;
    // the spec names this code for a missing entry; databases hold no entries yet
    let err = json!({"errors": [{"code": "ENTRY_NOT_FOUND", "status": 404, "title": "Not Found", "detail": "entry not found"}]});
    Ok((StatusCode::NOT_FOUND, Json(err)))
}
