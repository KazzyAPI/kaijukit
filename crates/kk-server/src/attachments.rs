//! Attachments: a content row of type `attachment` (title = filename) under a page, with
//! per-version file metadata and content-addressed blobs.

use std::sync::Arc;

use axum::{
    body::Bytes,
    extract::{Path, RawQuery, State},
    http::{header, HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    Extension, Json,
};
use chrono::{DateTime, Utc};
use kk_core::{ApiError, UserId};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};

use crate::{
    authz::Access,
    blob::BlobStore,
    error::{db_err, R},
    http::{paged, parse_attachment_id, parse_id, ts, Q},
    multipart, target, versions, AppState,
};

const STATUSES: [&str; 3] = ["current", "archived", "trashed"];
const ORDER: [(&str, &str); 2] = [
    ("created-date", "c.created_at"),
    ("modified-date", "v.created_at"),
];

#[derive(sqlx::FromRow)]
struct Row {
    id: i64,
    status: String,
    title: String,
    created_at: DateTime<Utc>,
    container_id: i64,
    file_id: String,
    space_key: String,
    number: i32,
    author_id: i64,
    minor_edit: bool,
    v_created: DateTime<Utc>,
    media_type: String,
    file_size: i64,
    comment: String,
}

const SELECT: &str = "SELECT c.id, c.status, c.title, c.created_at, a.container_id, a.file_id, \
    s.key AS space_key, v.number, v.author_id, v.minor_edit, v.created_at AS v_created, \
    av.media_type, av.file_size, av.comment \
    FROM content c JOIN attachments a ON a.content_id = c.id \
    JOIN content p ON p.id = a.container_id AND p.status IN ('current', 'draft', 'archived') \
    JOIN spaces s ON s.id = c.space_id \
    JOIN content_versions v ON v.content_id = c.id AND v.number = COALESCE($1, c.version_number) \
    JOIN attachment_versions av ON av.content_id = c.id AND av.number = v.number \
    WHERE c.type = 'attachment'";

fn download_link(r: &Row) -> String {
    format!(
        "/download/attachments/{}/{}?version={}&modificationDate={}&api=v2",
        r.container_id,
        form_urlencoded::byte_serialize(r.title.as_bytes())
            .collect::<String>()
            .replace('+', "%20"),
        r.number,
        r.v_created.timestamp_millis()
    )
}

fn webui_link(r: &Row) -> String {
    format!(
        "/spaces/{}/pages/{}?preview=%2F{}%2Fatt{}%2F{}",
        r.space_key,
        r.container_id,
        r.container_id,
        r.id,
        form_urlencoded::byte_serialize(r.title.as_bytes()).collect::<String>()
    )
}

fn render(r: &Row) -> Value {
    let (webui, download) = (webui_link(r), download_link(r));
    json!({
        "id": format!("att{}", r.id), "status": r.status, "title": r.title,
        "createdAt": ts(r.created_at), "pageId": r.container_id.to_string(),
        "mediaType": r.media_type, "comment": r.comment, "fileId": r.file_id,
        "fileSize": r.file_size, "webuiLink": webui, "downloadLink": download,
        "version": {"number": r.number, "message": r.comment, "minorEdit": r.minor_edit,
                    "authorId": r.author_id.to_string(), "createdAt": ts(r.v_created)},
        "_links": {"webui": webui, "download": download},
    })
}

async fn blocking<T: Send + 'static>(
    blobs: &Arc<dyn BlobStore>,
    f: impl FnOnce(&dyn BlobStore) -> std::io::Result<T> + Send + 'static,
) -> Result<T, ApiError> {
    let blobs = blobs.clone();
    tokio::task::spawn_blocking(move || f(blobs.as_ref()))
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?
        .map_err(|e| {
            tracing::error!(error = %e, "blob store error");
            ApiError::Internal("blob store error".into())
        })
}

async fn list(s: &AppState, a: &Access, q: Q, container: Option<i64>, path: &str) -> R<Response> {
    let (limit, offset) = (q.limit_or(50)?, q.offset()?);
    let order = q.order_by(&ORDER, "c.id")?;
    let status: Vec<String> = match q.list("status") {
        v if v.is_empty() => vec!["current".into()],
        v => v
            .into_iter()
            .map(|x| {
                STATUSES
                    .contains(&x)
                    .then(|| x.to_string())
                    .ok_or_else(|| ApiError::BadRequest(format!("invalid status: {x}")))
            })
            .collect::<Result<_, _>>()?,
    };
    let sql = format!(
        "{SELECT} AND c.status = ANY($2) AND ($3::bigint IS NULL OR a.container_id = $3) \
         AND ($4::text IS NULL OR av.media_type = $4) AND ($5::text IS NULL OR lower(c.title) = lower($5)) \
         AND NOT (c.id = ANY($8)) AND ($9::bigint[] IS NULL OR c.space_id = ANY($9)) \
         ORDER BY {order} LIMIT $6 OFFSET $7"
    );
    let rows: Vec<Row> = sqlx::query_as(&sql)
        .bind(None::<i32>)
        .bind(status)
        .bind(container)
        .bind(q.get("mediaType"))
        .bind(q.get("filename"))
        .bind(limit + 1)
        .bind(offset)
        .bind(a.hidden(&s.db).await?)
        .bind(a.readable_spaces())
        .fetch_all(&s.db)
        .await?;
    Ok(paged(
        &q,
        path,
        &s.site(),
        rows.iter().map(render).collect(),
        limit,
        offset,
    ))
}

pub async fn list_all(
    State(s): State<AppState>,
    Extension(a): Extension<Arc<Access>>,
    RawQuery(raw): RawQuery,
) -> R<Response> {
    list(&s, &a, Q::parse(raw), None, "/wiki/api/v2/attachments").await
}

pub async fn list_page(
    State(s): State<AppState>,
    Extension(a): Extension<Arc<Access>>,
    Path(id): Path<String>,
    RawQuery(raw): RawQuery,
) -> R<Response> {
    let id = target::owner(&s.db, &id, "page").await?;
    list(
        &s,
        &a,
        Q::parse(raw),
        Some(id),
        &format!("/wiki/api/v2/pages/{id}/attachments"),
    )
    .await
}

async fn load(s: &AppState, id: i64, version: Option<i32>) -> Result<Row, ApiError> {
    sqlx::query_as(&format!("{SELECT} AND c.id = $2 AND c.status <> 'deleted'"))
        .bind(version)
        .bind(id)
        .fetch_optional(&s.db)
        .await
        .map_err(|e| db_err(e, ""))?
        .ok_or_else(|| ApiError::NotFound("attachment not found".into()))
}

pub async fn get(
    State(s): State<AppState>,
    Path(id): Path<String>,
    RawQuery(raw): RawQuery,
) -> R<Json<Value>> {
    let q = Q::parse(raw);
    let mut v = render(&load(&s, parse_attachment_id(&id)?, q.int("version")?).await?);
    if q.get("include-version") == Some("false") {
        if let Some(o) = v.as_object_mut() {
            o.remove("version");
        }
    }
    v["_links"]["base"] = json!(s.site());
    Ok(Json(v))
}

pub async fn delete(
    State(s): State<AppState>,
    Path(id): Path<String>,
    RawQuery(raw): RawQuery,
) -> R<StatusCode> {
    let id = parse_attachment_id(&id)?;
    let purge = Q::parse(raw).flag("purge")?;
    load(&s, id, None).await?;
    if !purge {
        sqlx::query("UPDATE content SET status = 'trashed' WHERE id = $1")
            .bind(id)
            .execute(&s.db)
            .await?;
        return Ok(StatusCode::NO_CONTENT);
    }
    let keys: Vec<String> = sqlx::query_scalar(
        "SELECT DISTINCT blob_key FROM attachment_versions WHERE content_id = $1",
    )
    .bind(id)
    .fetch_all(&s.db)
    .await?;
    sqlx::query("DELETE FROM content WHERE id = $1")
        .bind(id)
        .execute(&s.db)
        .await?;
    for key in keys {
        let used: bool = sqlx::query_scalar(
            "SELECT EXISTS (SELECT 1 FROM attachment_versions WHERE blob_key = $1)",
        )
        .bind(&key)
        .fetch_one(&s.db)
        .await?;
        if !used {
            blocking(&s.blobs, move |b| b.delete(&key)).await?;
        }
    }
    Ok(StatusCode::NO_CONTENT)
}

pub async fn version_list(
    State(s): State<AppState>,
    Path(id): Path<String>,
    RawQuery(raw): RawQuery,
) -> R<Response> {
    let id = parse_attachment_id(&id)?;
    load(&s, id, None).await?;
    let path = format!("/wiki/api/v2/attachments/att{id}/versions");
    versions::list_for(&s, Q::parse(raw), id, ("attachment", "attachment"), &path).await
}

pub async fn version_get(
    State(s): State<AppState>,
    Path((id, number)): Path<(String, String)>,
) -> R<Json<Value>> {
    let id = parse_attachment_id(&id)?;
    load(&s, id, None).await?;
    versions::detail(&s, id, &number, "attachment").await
}

/// Media type from content, never from the client; text falls back to the extension.
fn sniff(name: &str, data: &[u8]) -> &'static str {
    const MAGIC: [(&[u8], &str); 7] = [
        (b"\x89PNG\r\n\x1a\n", "image/png"),
        (b"\xff\xd8\xff", "image/jpeg"),
        (b"GIF8", "image/gif"),
        (b"%PDF-", "application/pdf"),
        (b"PK\x03\x04", "application/zip"),
        (b"RIFF", "image/webp"),
        (b"<svg", "image/svg+xml"),
    ];
    if let Some((_, t)) = MAGIC.iter().find(|(m, _)| data.starts_with(m)) {
        if *t != "image/webp" || data.get(8..12) == Some(b"WEBP") {
            return t;
        }
    }
    if data.contains(&0) || std::str::from_utf8(data).is_err() {
        return "application/octet-stream";
    }
    match name
        .rsplit_once('.')
        .map(|(_, e)| e.to_ascii_lowercase())
        .as_deref()
    {
        Some("json") => "application/json",
        Some("csv") => "text/csv",
        Some("md") => "text/markdown",
        _ => "text/plain",
    }
}

fn clean_name(raw: &str) -> Result<String, ApiError> {
    let base = raw.rsplit(['/', '\\']).next().unwrap_or("").trim();
    if base.is_empty() || base.chars().count() > 255 || base.chars().any(char::is_control) {
        return Err(ApiError::BadRequest("invalid filename".into()));
    }
    Ok(base.to_string())
}

struct Upload {
    name: String,
    data: Vec<u8>,
    comment: String,
    minor: bool,
}

fn read_upload(headers: &HeaderMap, body: &Bytes) -> Result<Upload, ApiError> {
    let boundary = headers
        .get(header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .and_then(multipart::boundary)
        .ok_or_else(|| ApiError::BadRequest("expected multipart/form-data".into()))?;
    let parts = multipart::parse(body, &boundary)
        .ok_or_else(|| ApiError::BadRequest("malformed multipart body".into()))?;
    let text = |n: &str| {
        parts
            .iter()
            .find(|p| p.name == n)
            .map(|p| String::from_utf8_lossy(&p.data).into_owned())
    };
    let file = parts
        .iter()
        .find(|p| p.name == "file")
        .ok_or_else(|| ApiError::BadRequest("file part is required".into()))?;
    Ok(Upload {
        name: clean_name(file.filename.as_deref().unwrap_or(""))?,
        data: file.data.clone(),
        comment: text("comment").unwrap_or_default(),
        minor: text("minorEdit").is_some_and(|v| v.trim() == "true"),
    })
}

/// Stores the blob, then adds a new attachment or, for an existing filename, a new version.
async fn store(s: &AppState, user: i64, container: i64, up: Upload, only: Option<i64>) -> R<i64> {
    if up.data.len() > s.max_upload {
        return Err(ApiError::PayloadTooLarge.into());
    }
    let key: String = Sha256::digest(&up.data)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect();
    let media = sniff(&up.name, &up.data);
    let size = up.data.len() as i64;
    let data = up.data;
    let k = key.clone();
    blocking(&s.blobs, move |b| b.put(&k, &data)).await?;

    let mut tx = s.db.begin().await?;
    let (space, _): (i64, String) = sqlx::query_as(
        "SELECT space_id, title FROM content WHERE id = $1 AND type = 'page' \
         AND status IN ('current', 'draft', 'archived') FOR UPDATE",
    )
    .bind(container)
    .fetch_optional(&mut *tx)
    .await?
    .ok_or_else(|| ApiError::NotFound("page not found".into()))?;
    let existing: Option<(i64, i32)> = sqlx::query_as(
        "SELECT c.id, c.version_number FROM content c JOIN attachments a ON a.content_id = c.id \
         WHERE a.container_id = $1 AND c.status = 'current' AND ($3::bigint IS NULL AND lower(c.title) = lower($2) \
         OR c.id = $3)",
    )
    .bind(container)
    .bind(&up.name)
    .bind(only)
    .fetch_optional(&mut *tx)
    .await?;
    let (id, number) = match existing {
        Some((id, n)) => {
            sqlx::query("UPDATE content SET version_number = $2 WHERE id = $1")
                .bind(id)
                .bind(n + 1)
                .execute(&mut *tx)
                .await?;
            (id, n + 1)
        }
        None if only.is_some() => {
            return Err(ApiError::NotFound("attachment not found".into()).into())
        }
        None => {
            let id: i64 = sqlx::query_scalar(
                "INSERT INTO content (type, status, space_id, title, author_id) \
                 VALUES ('attachment', 'current', $1, $2, $3) RETURNING id",
            )
            .bind(space)
            .bind(&up.name)
            .bind(user)
            .fetch_one(&mut *tx)
            .await
            .map_err(|e| db_err(e, ""))?;
            sqlx::query("INSERT INTO attachments (content_id, container_id) VALUES ($1, $2)")
                .bind(id)
                .bind(container)
                .execute(&mut *tx)
                .await?;
            (id, 1)
        }
    };
    sqlx::query(
        "INSERT INTO content_versions (content_id, number, title, body, author_id, message, minor_edit) \
         VALUES ($1, $2, $3, '{}', $4, $5, $6)",
    )
    .bind(id)
    .bind(number)
    .bind(&up.name)
    .bind(user)
    .bind(&up.comment)
    .bind(up.minor)
    .execute(&mut *tx)
    .await?;
    sqlx::query(
        "INSERT INTO attachment_versions (content_id, number, blob_key, media_type, file_size, comment) \
         VALUES ($1, $2, $3, $4, $5, $6)",
    )
    .bind(id)
    .bind(number)
    .bind(&key)
    .bind(media)
    .bind(size)
    .bind(&up.comment)
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;
    Ok(id)
}

fn v1_json(r: &Row) -> Value {
    json!({
        "id": format!("att{}", r.id), "type": "attachment", "status": r.status, "title": r.title,
        "version": {"number": r.number, "message": r.comment, "minorEdit": r.minor_edit,
                    "when": ts(r.v_created)},
        "extensions": {"mediaType": r.media_type, "fileSize": r.file_size,
                       "comment": r.comment, "fileId": r.file_id},
        "container": {"id": r.container_id.to_string(), "type": "page"},
        "_links": {"download": download_link(r),
                   "webui": webui_link(r), "self": format!("/rest/api/content/att{}", r.id)},
    })
}

async fn v1_created(s: &AppState, id: i64) -> R<Json<Value>> {
    let row = load(s, id, None).await?;
    Ok(Json(json!({
        "results": [v1_json(&row)], "start": 0, "limit": 1, "size": 1,
        "_links": {"base": s.site(), "context": "/wiki"},
    })))
}

pub async fn upload(
    State(s): State<AppState>,
    Extension(user): Extension<UserId>,
    Path(id): Path<String>,
    headers: HeaderMap,
    body: Bytes,
) -> R<Json<Value>> {
    let container = parse_id(&id)?;
    let new = store(&s, user.0, container, read_upload(&headers, &body)?, None).await?;
    v1_created(&s, new).await
}

pub async fn update_data(
    State(s): State<AppState>,
    Extension(user): Extension<UserId>,
    Path((id, att)): Path<(String, String)>,
    headers: HeaderMap,
    body: Bytes,
) -> R<Json<Value>> {
    let (container, att) = (parse_id(&id)?, parse_attachment_id(&att)?);
    let id = store(
        &s,
        user.0,
        container,
        read_upload(&headers, &body)?,
        Some(att),
    )
    .await?;
    // the spec returns the single updated content here, not a list
    Ok(Json(v1_json(&load(&s, id, None).await?)))
}

pub async fn v1_download(
    State(s): State<AppState>,
    Path((id, att)): Path<(String, String)>,
) -> R<impl IntoResponse> {
    let row = load(&s, parse_attachment_id(&att)?, None).await?;
    if row.container_id != parse_id(&id)? {
        return Err(ApiError::NotFound("attachment not found".into()).into());
    }
    // the spec documents 302 (axum's helpers only offer 303/307/308)
    Ok((
        StatusCode::FOUND,
        [(header::LOCATION, format!("/wiki{}", download_link(&row)))],
    ))
}

pub async fn download(
    State(s): State<AppState>,
    Path((container, filename)): Path<(String, String)>,
    RawQuery(raw): RawQuery,
) -> R<Response> {
    let version = Q::parse(raw).int("version")?;
    let (container,) = (parse_id(&container)?,);
    let (key, media, title): (String, String, String) = sqlx::query_as(
        "SELECT av.blob_key, av.media_type, c.title FROM content c \
         JOIN attachments a ON a.content_id = c.id \
         JOIN content p ON p.id = a.container_id AND p.status IN ('current', 'draft', 'archived') \
         JOIN attachment_versions av ON av.content_id = c.id AND av.number = COALESCE($3, c.version_number) \
         WHERE a.container_id = $1 AND lower(c.title) = lower($2) AND c.status = 'current'",
    )
    .bind(container)
    .bind(&filename)
    .bind(version)
    .fetch_optional(&s.db)
    .await?
    .ok_or_else(|| ApiError::NotFound("attachment not found".into()))?;
    let data = blocking(&s.blobs, move |b| b.get(&key)).await?;
    let inline =
        media.starts_with("image/") && media != "image/svg+xml" || media == "application/pdf";
    let disposition = format!(
        "{}; filename*=UTF-8''{}",
        if inline { "inline" } else { "attachment" },
        form_urlencoded::byte_serialize(title.as_bytes())
            .collect::<String>()
            .replace('+', "%20")
    );
    Ok((
        [
            (header::CONTENT_TYPE, media),
            (header::CONTENT_DISPOSITION, disposition),
            (header::X_CONTENT_TYPE_OPTIONS, "nosniff".into()),
        ],
        data,
    )
        .into_response())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sniffs_content_over_extension() {
        assert_eq!(sniff("x.txt", b"\x89PNG\r\n\x1a\nrest"), "image/png");
        assert_eq!(sniff("x.json", b"{}"), "application/json");
        assert_eq!(sniff("x.png", b"\x00\x01"), "application/octet-stream");
        assert_eq!(sniff("x", b"RIFF1234WAVE"), "text/plain");
    }

    #[test]
    fn filenames_lose_directories() {
        assert_eq!(clean_name(" a/b\\c.txt ").unwrap(), "c.txt");
        assert!(clean_name("dir/").is_err());
    }
}
