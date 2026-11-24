use kk_adf::{from_storage, to_storage, Doc};
use kk_core::ApiError;
use serde_json::{json, Value};

pub const PRIMARY: &[&str] = &["storage", "atlas_doc_format"];
pub const SINGLE: &[&str] = &[
    "storage",
    "atlas_doc_format",
    "view",
    "export_view",
    "anonymous_export_view",
    "styled_view",
    "editor",
];

pub fn format<'a>(q: &'a crate::http::Q, allowed: &[&str]) -> Result<Option<&'a str>, ApiError> {
    match q.get("body-format") {
        Some(f) if !allowed.contains(&f) => {
            Err(ApiError::BadRequest(format!("invalid body-format: {f}")))
        }
        f => Ok(f),
    }
}

/// `{ <repr>: { representation, value } }` for stored ADF. Rendered views are the storage XML
/// until macro rendering exists.
pub fn render(adf: &Value, repr: &str) -> Result<Value, ApiError> {
    let value = if repr == "atlas_doc_format" {
        adf.to_string()
    } else {
        let doc: Doc = serde_json::from_value(adf.clone())
            .map_err(|e| ApiError::Internal(format!("stored body is not ADF: {e}")))?;
        to_storage(&doc)
    };
    Ok(json!({ repr: {"representation": repr, "value": value} }))
}

/// Accepts `{representation, value}` or `{storage|atlas_doc_format: {representation, value}}`.
pub fn parse_write(body: &Value) -> Result<(Doc, &'static str), ApiError> {
    let flat = body["representation"].as_str().zip(body["value"].as_str());
    let (repr, value) = flat
        .or_else(|| {
            body.as_object()?
                .iter()
                .find_map(|(k, v)| Some((k.as_str(), v["value"].as_str()?)))
        })
        .ok_or_else(|| ApiError::BadRequest("body needs a representation and a value".into()))?;
    match repr {
        "storage" => from_storage(value)
            .map(|d| (d, "storage"))
            .map_err(|e| ApiError::BadRequest(format!("invalid storage body: {e}"))),
        "atlas_doc_format" => Doc::parse(value)
            .map(|d| (d, "atlas_doc_format"))
            .map_err(ApiError::BadRequest),
        other => Err(ApiError::BadRequest(format!(
            "unsupported body representation: {other}"
        ))),
    }
}
