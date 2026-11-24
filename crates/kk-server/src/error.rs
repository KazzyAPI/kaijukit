use axum::{
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use kk_core::ApiError;

/// Handler error: `ApiError` rendered as reference-shaped JSON.
pub struct E(pub ApiError);

pub type R<T> = Result<T, E>;

impl From<ApiError> for E {
    fn from(e: ApiError) -> Self {
        E(e)
    }
}

impl From<sqlx::Error> for E {
    fn from(e: sqlx::Error) -> Self {
        E(db_err(e, ""))
    }
}

/// Maps constraint violations to 400s; everything else is a logged 500.
pub fn db_err(e: sqlx::Error, duplicate: &str) -> ApiError {
    match e.as_database_error().and_then(|d| d.code()).as_deref() {
        Some("23505") => ApiError::BadRequest(duplicate.into()),
        Some("23503") => ApiError::BadRequest("content still has children or references".into()),
        _ => {
            tracing::error!(error = %e, "database error");
            ApiError::Internal("database error".into())
        }
    }
}

impl IntoResponse for E {
    fn into_response(self) -> Response {
        let status =
            StatusCode::from_u16(self.0.status()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
        let mut res = (status, Json(self.0.body())).into_response();
        if self.0 == ApiError::Unauthorized {
            res.headers_mut().insert(
                header::WWW_AUTHENTICATE,
                "Basic realm=\"kaijukit\"".parse().unwrap(),
            );
        }
        res
    }
}
