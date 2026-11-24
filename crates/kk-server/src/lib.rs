mod auth;
mod body;
mod error;
mod http;
mod pages;
mod spaces;
mod tree;
mod versions;

use axum::{
    extract::{DefaultBodyLimit, State},
    middleware,
    routing::{get, put},
    Router,
};
use error::{E, R};
use kk_core::ApiError;
use sqlx::PgPool;
use tower_http::trace::TraceLayer;

pub use auth::{bootstrap_admin, DEV_TOKEN};

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    /// Public origin, no trailing slash; `_links.base` is this plus `/wiki`.
    pub base_url: String,
}

impl AppState {
    fn site(&self) -> String {
        format!("{}/wiki", self.base_url)
    }
}

pub async fn migrate(db: &PgPool) -> Result<(), sqlx::migrate::MigrateError> {
    sqlx::migrate!("../../migrations").run(db).await
}

async fn ready(State(s): State<AppState>) -> R<&'static str> {
    sqlx::query("SELECT 1").execute(&s.db).await?;
    Ok("ok")
}

pub fn app(state: AppState) -> Router {
    let v2 = Router::new()
        .route("/spaces", get(spaces::list).post(spaces::create))
        .route("/spaces/{id}", get(spaces::get))
        .route("/spaces/{id}/pages", get(pages::in_space))
        .route("/pages", get(pages::list).post(pages::create))
        .route(
            "/pages/{id}",
            get(pages::get).put(pages::update).delete(pages::delete),
        )
        .route("/pages/{id}/title", put(pages::update_title))
        .route("/pages/{id}/versions", get(versions::list))
        .route("/pages/{id}/versions/{number}", get(versions::get))
        .route("/pages/{id}/children", get(tree::children))
        .route("/pages/{id}/direct-children", get(tree::direct_children))
        .route("/pages/{id}/ancestors", get(tree::ancestors))
        .route("/pages/{id}/descendants", get(tree::descendants))
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            auth::require_auth,
        ));
    Router::new()
        .route("/healthz", get(|| async { "ok" }))
        .route("/readyz", get(ready))
        .nest("/wiki/api/v2", v2)
        .fallback(|| async { E(ApiError::NotFound("no such route".into())) })
        .layer(DefaultBodyLimit::max(http::MAX_BODY))
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}
