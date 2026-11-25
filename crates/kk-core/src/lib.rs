//! Pure domain types: ids, statuses, content types, API errors. No IO.

pub mod cql;
pub mod perm;

use std::{fmt, str::FromStr};

use serde_json::{json, Value};

macro_rules! id {
    ($name:ident) => {
        #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
        pub struct $name(pub i64);

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                self.0.fmt(f)
            }
        }

        impl FromStr for $name {
            type Err = ApiError;

            fn from_str(s: &str) -> Result<Self, ApiError> {
                if s.is_empty() || !s.bytes().all(|b| b.is_ascii_digit()) {
                    return Err(ApiError::BadRequest(format!("invalid id: {s:?}")));
                }
                s.parse()
                    .map($name)
                    .map_err(|_| ApiError::BadRequest(format!("invalid id: {s:?}")))
            }
        }
    };
}

id!(SpaceId);
id!(ContentId);
id!(UserId);

macro_rules! text_enum {
    ($name:ident { $($variant:ident => $text:literal),+ $(,)? }) => {
        #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
        pub enum $name { $($variant),+ }

        impl $name {
            pub fn as_str(self) -> &'static str {
                match self { $(Self::$variant => $text),+ }
            }
        }

        impl FromStr for $name {
            type Err = ApiError;

            fn from_str(s: &str) -> Result<Self, ApiError> {
                match s {
                    $($text => Ok(Self::$variant),)+
                    _ => Err(ApiError::BadRequest(format!("invalid {}: {s:?}", stringify!($name)))),
                }
            }
        }
    };
}

text_enum!(ContentStatus {
    Current => "current",
    Draft => "draft",
    Archived => "archived",
    Historical => "historical",
    Trashed => "trashed",
    Deleted => "deleted",
});

text_enum!(ContentType {
    Page => "page",
    Folder => "folder",
    Whiteboard => "whiteboard",
    Database => "database",
    Embed => "embed",
    Blogpost => "blogpost",
});

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ApiError {
    BadRequest(String),
    Unauthorized,
    Forbidden(String),
    NotFound(String),
    Conflict(String),
    PayloadTooLarge,
    Internal(String),
}

impl ApiError {
    pub fn status(&self) -> u16 {
        match self {
            Self::BadRequest(_) => 400,
            Self::Unauthorized => 401,
            Self::Forbidden(_) => 403,
            Self::NotFound(_) => 404,
            Self::Conflict(_) => 409,
            Self::PayloadTooLarge => 413,
            Self::Internal(_) => 500,
        }
    }

    pub fn code(&self) -> &'static str {
        match self {
            Self::BadRequest(_) => "INVALID_REQUEST_PARAMETER",
            Self::Unauthorized => "UNAUTHORIZED",
            Self::Forbidden(_) => "FORBIDDEN",
            Self::NotFound(_) => "NOT_FOUND",
            Self::Conflict(_) => "CONFLICT",
            Self::PayloadTooLarge => "REQUEST_ENTITY_TOO_LARGE",
            Self::Internal(_) => "INTERNAL_SERVER_ERROR",
        }
    }

    pub fn title(&self) -> &'static str {
        match self {
            Self::BadRequest(_) => "Bad Request",
            Self::Unauthorized => "Unauthorized",
            Self::Forbidden(_) => "Forbidden",
            Self::NotFound(_) => "Not Found",
            Self::Conflict(_) => "Conflict",
            Self::PayloadTooLarge => "Payload Too Large",
            Self::Internal(_) => "Internal Server Error",
        }
    }

    pub fn detail(&self) -> Option<&str> {
        match self {
            Self::BadRequest(d) | Self::Forbidden(d) | Self::NotFound(d) | Self::Conflict(d) => {
                Some(d)
            }
            _ => None,
        }
    }

    /// Reference-shaped v2 error body.
    pub fn body(&self) -> Value {
        json!({"errors": [{
            "status": self.status(),
            "code": self.code(),
            "title": self.title(),
            "detail": self.detail(),
        }]})
    }
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}: {}",
            self.status(),
            self.detail().unwrap_or(self.title())
        )
    }
}

impl std::error::Error for ApiError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ids_parse_digits_only() {
        assert_eq!("123".parse::<ContentId>(), Ok(ContentId(123)));
        assert!("-1".parse::<ContentId>().is_err());
        assert!("".parse::<SpaceId>().is_err());
        assert!("99999999999999999999".parse::<UserId>().is_err());
        assert_eq!(ContentId(7).to_string(), "7");
    }

    #[test]
    fn enums_round_trip() {
        assert_eq!(
            "trashed".parse::<ContentStatus>(),
            Ok(ContentStatus::Trashed)
        );
        assert_eq!(ContentType::Blogpost.as_str(), "blogpost");
        assert!("nope".parse::<ContentStatus>().is_err());
    }

    #[test]
    fn error_body_shape() {
        assert_eq!(
            ApiError::Conflict("stale".into()).body(),
            json!({"errors": [{"status": 409, "code": "CONFLICT", "title": "Conflict", "detail": "stale"}]})
        );
        assert_eq!(
            ApiError::Unauthorized.body()["errors"][0]["detail"],
            Value::Null
        );
    }
}
