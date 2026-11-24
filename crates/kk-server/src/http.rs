use axum::{
    body::Bytes,
    extract::{FromRequest, Request},
    http::{header, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use kk_core::ApiError;
use serde::de::DeserializeOwned;
use serde_json::{json, Value};

use crate::error::E;

pub const MAX_BODY: usize = 5 * 1024 * 1024;

pub fn parse_id(s: &str) -> Result<i64, ApiError> {
    s.parse::<kk_core::ContentId>().map(|i| i.0)
}

/// Attachment ids are exposed with the `att` prefix of the reference; both forms are accepted.
pub fn public_id(ty: &str, id: i64) -> String {
    if ty == "attachment" {
        format!("att{id}")
    } else {
        id.to_string()
    }
}

pub fn parse_attachment_id(s: &str) -> Result<i64, ApiError> {
    parse_id(s.strip_prefix("att").unwrap_or(s))
}

pub fn ts(t: chrono::DateTime<chrono::Utc>) -> String {
    t.format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string()
}

/// JSON request body with reference-shaped 400/413 errors.
pub struct JsonBody<T>(pub T);

impl<T: DeserializeOwned, S: Send + Sync> FromRequest<S> for JsonBody<T> {
    type Rejection = E;

    async fn from_request(req: Request, state: &S) -> Result<Self, E> {
        let bytes = Bytes::from_request(req, state).await.map_err(|e| {
            if e.status() == StatusCode::PAYLOAD_TOO_LARGE {
                ApiError::PayloadTooLarge
            } else {
                ApiError::BadRequest(e.body_text())
            }
        })?;
        serde_json::from_slice(&bytes)
            .map(JsonBody)
            .map_err(|e| E(ApiError::BadRequest(e.to_string())))
    }
}

/// Parsed query string. Array params accept repeats and comma lists.
pub struct Q(Vec<(String, String)>);

impl Q {
    pub fn parse(raw: Option<String>) -> Self {
        Q(form_urlencoded::parse(raw.unwrap_or_default().as_bytes())
            .into_owned()
            .collect())
    }

    pub fn get(&self, k: &str) -> Option<&str> {
        self.0.iter().find(|(n, _)| n == k).map(|(_, v)| v.as_str())
    }

    pub fn list(&self, k: &str) -> Vec<&str> {
        self.0
            .iter()
            .filter(|(n, _)| n == k)
            .flat_map(|(_, v)| v.split(','))
            .filter(|s| !s.is_empty())
            .collect()
    }

    pub fn ids(&self, k: &str) -> Result<Option<Vec<i64>>, ApiError> {
        let l = self.list(k);
        if l.is_empty() {
            return Ok(None);
        }
        l.into_iter()
            .map(crate::http::parse_id)
            .collect::<Result<_, _>>()
            .map(Some)
    }

    pub fn int(&self, k: &str) -> Result<Option<i32>, ApiError> {
        self.get(k)
            .map(|v| {
                v.parse()
                    .map_err(|_| ApiError::BadRequest(format!("invalid {k}")))
            })
            .transpose()
    }

    pub fn flag(&self, k: &str) -> Result<bool, ApiError> {
        match self.get(k) {
            None => Ok(false),
            Some("true") => Ok(true),
            Some("false") => Ok(false),
            Some(_) => Err(ApiError::BadRequest(format!("invalid {k}"))),
        }
    }

    pub fn limit(&self) -> Result<i64, ApiError> {
        match self.int("limit")? {
            None => Ok(25),
            Some(n) if (1..=250).contains(&n) => Ok(n.into()),
            Some(_) => Err(ApiError::BadRequest(
                "limit must be between 1 and 250".into(),
            )),
        }
    }

    /// Cursors are opaque to clients; ours encodes the row offset.
    pub fn offset(&self) -> Result<i64, ApiError> {
        let Some(c) = self.get("cursor") else {
            return Ok(0);
        };
        URL_SAFE_NO_PAD
            .decode(c)
            .ok()
            .and_then(|b| String::from_utf8(b).ok())
            .and_then(|s| s.strip_prefix("o:")?.parse().ok())
            .filter(|o| *o >= 0)
            .ok_or_else(|| ApiError::BadRequest("invalid cursor".into()))
    }

    fn next_url(&self, path: &str, offset: i64) -> String {
        let mut s = form_urlencoded::Serializer::new(String::new());
        for (k, v) in self.0.iter().filter(|(k, _)| k != "cursor") {
            s.append_pair(k, v);
        }
        s.append_pair("cursor", &URL_SAFE_NO_PAD.encode(format!("o:{offset}")));
        format!("{path}?{}", s.finish())
    }

    /// `ORDER BY` body from a `sort` param (`field` or `-field`); `allowed` maps field -> column.
    pub fn order_by(&self, allowed: &[(&str, &str)], tie: &str) -> Result<String, ApiError> {
        let (name, desc) = match self.get("sort") {
            None => (allowed[0].0, false),
            Some(s) => s.strip_prefix('-').map_or((s, false), |n| (n, true)),
        };
        let col = allowed
            .iter()
            .find(|(k, _)| *k == name)
            .ok_or_else(|| ApiError::BadRequest("invalid sort".into()))?
            .1;
        let dir = if desc { "DESC" } else { "ASC" };
        Ok(format!("{col} {dir}, {tie} {dir}"))
    }
}

/// `{results, _links}` response; `rows` holds up to `limit + 1` items.
pub fn paged(
    q: &Q,
    path: &str,
    base: &str,
    mut rows: Vec<Value>,
    limit: i64,
    offset: i64,
) -> Response {
    let more = rows.len() as i64 > limit;
    rows.truncate(limit as usize);
    let mut links = json!({"base": base});
    let mut link_header = None;
    if more {
        let next = q.next_url(path, offset + limit);
        link_header = HeaderValue::from_str(&format!("<{next}>; rel=\"next\"")).ok();
        links["next"] = json!(next);
    }
    let mut res = Json(json!({"results": rows, "_links": links})).into_response();
    if let Some(h) = link_header {
        res.headers_mut().insert(header::LINK, h);
    }
    res
}
