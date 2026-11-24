//! ADF document types and storage-format conversion.
//!
//! Nodes are generic (`kind` string + optional attrs/content/marks/text) so every ADF
//! node and mark, known or not, round-trips losslessly through serde.

mod storage;

pub use storage::{from_storage, to_storage};

use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Doc {
    pub version: u32,
    #[serde(rename = "type")]
    pub kind: String,
    pub content: Vec<Node>,
}

impl Doc {
    pub fn new(content: Vec<Node>) -> Self {
        Self {
            version: 1,
            kind: "doc".into(),
            content,
        }
    }

    pub fn parse(json: &str) -> Result<Self, String> {
        let doc: Doc = serde_json::from_str(json).map_err(|e| format!("invalid ADF: {e}"))?;
        if doc.kind != "doc" || doc.version != 1 {
            return Err("invalid ADF: expected {\"version\":1,\"type\":\"doc\"}".into());
        }
        Ok(doc)
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Node {
    #[serde(rename = "type")]
    pub kind: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub attrs: Option<Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content: Option<Vec<Node>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub marks: Option<Vec<Mark>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Mark {
    #[serde(rename = "type")]
    pub kind: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub attrs: Option<Value>,
}

impl Mark {
    pub fn new(kind: &str, attrs: Option<Value>) -> Self {
        Self {
            kind: kind.into(),
            attrs,
        }
    }
}

impl Node {
    pub fn new(kind: &str) -> Self {
        Self {
            kind: kind.into(),
            attrs: None,
            content: None,
            marks: None,
            text: None,
        }
    }

    pub fn attrs(mut self, attrs: Value) -> Self {
        self.attrs = Some(attrs);
        self
    }

    /// Empty content is omitted, as ADF forbids empty `content` on some nodes.
    pub fn content(mut self, content: Vec<Node>) -> Self {
        self.content = (!content.is_empty()).then_some(content);
        self
    }

    pub fn text(text: &str, marks: &[Mark]) -> Self {
        Self {
            text: Some(text.into()),
            marks: (!marks.is_empty()).then(|| marks.to_vec()),
            ..Self::new("text")
        }
    }
}
