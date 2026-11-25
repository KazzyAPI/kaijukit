//! v1 content templates. Global ones are managed by site administrators, space ones by space
//! administrators; blueprints do not exist, so the blueprint list is always empty.

use std::sync::Arc;

use axum::{
    extract::{Path, RawQuery, State},
    http::StatusCode,
    response::Response,
    Extension, Json,
};
use kk_core::ApiError;
use serde::Deserialize;
use serde_json::{json, Value};

use crate::{
    authz::{self, Access},
    body,
    error::{db_err, R},
    http::{parse_id, JsonBody, Q},
    v1, AppState,
};

type Row = (
    i64,
    Option<i64>,
    String,
    String,
    Value,
    Value,
    Option<String>,
    Option<String>,
);

const SELECT: &str = "SELECT t.id, t.space_id, t.name, t.description, t.labels, t.body, \
     s.key, s.name FROM templates t LEFT JOIN spaces s ON s.id = t.space_id";

fn json_of(r: Row, with_body: bool, site: &str) -> Result<Value, ApiError> {
    let (id, _, name, description, labels, adf, key, space_name) = r;
    let labels: Vec<Value> = labels
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .map(|n| json!({"prefix": "global", "name": n, "id": "", "label": n}))
        .collect();
    let mut t = json!({
        "templateId": id.to_string(),
        "name": name,
        "description": description,
        "labels": labels,
        "templateType": "page",
        "editorVersion": "v2",
        "_expandable": {"body": ""},
        "_links": {"self": format!("{site}/rest/api/template/{id}")},
    });
    if let Some(k) = key {
        t["space"] = json!({"key": k, "name": space_name});
    }
    if with_body {
        t["body"] = body::render(&adf, "storage")?;
        t["body"]["storage"]["embeddedContent"] = json!([]);
    }
    Ok(t)
}

fn wants_body(q: &Q) -> bool {
    q.list("templateExpand").contains(&"body")
}

/// The caller may manage templates of this space (or global ones when `space` is `None`).
fn can_manage(a: &Access, space: Option<i64>) -> Result<(), ApiError> {
    match space {
        None if a.p.admin => Ok(()),
        None => Err(ApiError::Forbidden(
            "global templates need a site administrator".into(),
        )),
        Some(id) => a.require_space(id, "administer", "space"),
    }
}

async fn fetch(s: &AppState, a: &Access, raw: &str) -> Result<Row, ApiError> {
    let missing = || ApiError::NotFound("template not found".into());
    let id = parse_id(raw).map_err(|_| missing())?;
    let row: Row = sqlx::query_as(&format!("{SELECT} WHERE t.id = $1"))
        .bind(id)
        .fetch_optional(&s.db)
        .await
        .map_err(|e| db_err(e, ""))?
        .ok_or_else(missing)?;
    match row.1 {
        Some(sp) if !a.grants(sp).has("read", "space") => Err(missing()),
        _ => Ok(row),
    }
}

pub async fn get(
    State(s): State<AppState>,
    Extension(a): Extension<Arc<Access>>,
    Path(id): Path<String>,
    RawQuery(raw): RawQuery,
) -> R<Json<Value>> {
    let q = Q::parse(raw);
    Ok(Json(json_of(
        fetch(&s, &a, &id).await?,
        wants_body(&q),
        &s.site(),
    )?))
}

pub async fn list(
    State(s): State<AppState>,
    Extension(a): Extension<Arc<Access>>,
    RawQuery(raw): RawQuery,
) -> R<Response> {
    let q = Q::parse(raw);
    let (start, limit) = v1::window(&q, 25)?;
    // no spaceKey: the global templates
    let space = match q.get("spaceKey") {
        Some(k) => {
            let id = authz::space_by_key(&s.db, k).await?;
            a.require_space(id, "read", "space")?;
            Some(id)
        }
        None => None,
    };
    let rows: Vec<Row> = sqlx::query_as(&format!(
        "{SELECT} WHERE t.space_id IS NOT DISTINCT FROM $1 ORDER BY lower(t.name), t.id LIMIT $2 OFFSET $3"
    ))
    .bind(space)
    .bind(limit + 1)
    .bind(start)
    .fetch_all(&s.db)
    .await
    .map_err(|e| db_err(e, ""))?;
    let with_body = wants_body(&q);
    let rows = rows
        .into_iter()
        .map(|r| json_of(r, with_body, &s.site()))
        .collect::<Result<_, _>>()?;
    Ok(v1::ok(v1::page(
        &q,
        "/wiki/rest/api/template/page",
        &s.site(),
        rows,
        start,
        limit,
    )))
}

pub async fn blueprints(State(s): State<AppState>, RawQuery(raw): RawQuery) -> R<Response> {
    let q = Q::parse(raw);
    let (start, limit) = v1::window(&q, 25)?;
    Ok(v1::ok(v1::page(
        &q,
        "/wiki/rest/api/template/blueprint",
        &s.site(),
        vec![],
        start,
        limit,
    )))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Input {
    template_id: Option<String>,
    name: String,
    template_type: String,
    description: Option<String>,
    #[serde(default)]
    labels: Vec<Value>,
    body: Value,
    space: Option<Value>,
}

struct Parsed {
    name: String,
    description: String,
    labels: Value,
    adf: Value,
    space: Option<String>,
}

fn parse(req: &Input) -> Result<Parsed, ApiError> {
    let bad = |m: &str| ApiError::BadRequest(m.into());
    let name = req.name.trim();
    if name.is_empty() || name.chars().count() > 255 {
        return Err(bad("name must be 1 to 255 characters"));
    }
    if req.template_type != "page" {
        return Err(bad("templateType must be page"));
    }
    let description = req.description.clone().unwrap_or_default();
    if description.chars().count() > 255 {
        return Err(bad("description is at most 255 characters"));
    }
    let mut labels: Vec<String> = vec![];
    for l in &req.labels {
        let n = l["name"]
            .as_str()
            .or(l["label"].as_str())
            .unwrap_or("")
            .trim();
        if n.is_empty() {
            return Err(bad("every label needs a name"));
        }
        if !labels.iter().any(|x| x == n) {
            labels.push(n.to_string());
        }
    }
    let (doc, _) = body::parse_write(&req.body)?;
    Ok(Parsed {
        name: name.into(),
        description,
        labels: json!(labels),
        adf: serde_json::to_value(&doc).map_err(|e| ApiError::Internal(e.to_string()))?,
        space: req
            .space
            .as_ref()
            .and_then(|s| s["key"].as_str())
            .map(str::to_string),
    })
}

async fn space_id(s: &AppState, key: &Option<String>) -> Result<Option<i64>, ApiError> {
    match key {
        Some(k) => Ok(Some(authz::space_by_key(&s.db, k).await?)),
        None => Ok(None),
    }
}

pub async fn create(
    State(s): State<AppState>,
    Extension(a): Extension<Arc<Access>>,
    JsonBody(req): JsonBody<Input>,
) -> R<Json<Value>> {
    let p = parse(&req)?;
    let space = space_id(&s, &p.space).await?;
    can_manage(&a, space)?;
    let id: i64 = sqlx::query_scalar(
        "INSERT INTO templates (space_id, name, description, labels, body, author_id) \
         VALUES ($1, $2, $3, $4, $5, $6) RETURNING id",
    )
    .bind(space)
    .bind(&p.name)
    .bind(&p.description)
    .bind(&p.labels)
    .bind(&p.adf)
    .bind(a.p.id)
    .fetch_one(&s.db)
    .await
    .map_err(|e| db_err(e, ""))?;
    let row = fetch(&s, &a, &id.to_string()).await?;
    Ok(Json(json_of(row, true, &s.site())?))
}

pub async fn update(
    State(s): State<AppState>,
    Extension(a): Extension<Arc<Access>>,
    JsonBody(req): JsonBody<Input>,
) -> R<Json<Value>> {
    let p = parse(&req)?;
    let id = req
        .template_id
        .as_deref()
        .ok_or_else(|| ApiError::BadRequest("templateId is required".into()))?;
    let current = fetch(&s, &a, id).await?;
    can_manage(&a, current.1)?;
    // a template keeps its space; the body's space must agree with it
    if p.space != current.6 {
        return Err(
            ApiError::BadRequest("space.key must be the template's current space".into()).into(),
        );
    }
    sqlx::query(
        "UPDATE templates SET name = $2, description = $3, labels = $4, body = $5, updated_at = now() WHERE id = $1",
    )
    .bind(current.0)
    .bind(&p.name)
    .bind(&p.description)
    .bind(&p.labels)
    .bind(&p.adf)
    .execute(&s.db)
    .await
    .map_err(|e| db_err(e, ""))?;
    let row = fetch(&s, &a, id).await?;
    Ok(Json(json_of(row, true, &s.site())?))
}

pub async fn remove(
    State(s): State<AppState>,
    Extension(a): Extension<Arc<Access>>,
    Path(id): Path<String>,
) -> R<StatusCode> {
    let current = fetch(&s, &a, &id).await?;
    can_manage(&a, current.1)?;
    sqlx::query("DELETE FROM templates WHERE id = $1")
        .bind(current.0)
        .execute(&s.db)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}
