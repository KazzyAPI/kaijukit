mod auth;
mod body;
mod comments;
mod error;
mod http;
mod pages;
mod spaces;
mod target;
mod tree;
mod users;
mod versions;

use axum::{
    extract::{DefaultBodyLimit, Request, State},
    middleware,
    response::IntoResponse,
    routing::{get, post, put},
    Router,
};
use error::{E, R};
use kk_core::ApiError;
use sqlx::PgPool;
use std::path::PathBuf;
use tower::ServiceExt;
use tower_http::{
    services::{ServeDir, ServeFile},
    trace::TraceLayer,
};

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
        .route("/users-bulk", post(users::bulk))
        .route(
            "/user/access/check-access-by-email",
            post(users::check_access),
        )
        .route("/pages/{id}/title", put(pages::update_title))
        .route("/pages/{id}/versions", get(versions::list))
        .route("/pages/{id}/versions/{number}", get(versions::get))
        .route("/pages/{id}/children", get(tree::children))
        .route("/pages/{id}/direct-children", get(tree::direct_children))
        .route("/pages/{id}/ancestors", get(tree::ancestors))
        .route("/pages/{id}/descendants", get(tree::descendants))
        .route(
            "/footer-comments",
            get(comments::list_all::<false>).post(comments::create::<false>),
        )
        .route(
            "/inline-comments",
            get(comments::list_all::<true>).post(comments::create::<true>),
        )
        .route(
            "/footer-comments/{id}",
            get(comments::get::<false>)
                .put(comments::update::<false>)
                .delete(comments::delete::<false>),
        )
        .route(
            "/inline-comments/{id}",
            get(comments::get::<true>)
                .put(comments::update::<true>)
                .delete(comments::delete::<true>),
        )
        .route(
            "/footer-comments/{id}/children",
            get(comments::children::<false>),
        )
        .route(
            "/inline-comments/{id}/children",
            get(comments::children::<true>),
        )
        .route(
            "/footer-comments/{id}/versions",
            get(comments::version_list::<false>),
        )
        .route(
            "/inline-comments/{id}/versions",
            get(comments::version_list::<true>),
        )
        .route(
            "/footer-comments/{id}/versions/{number}",
            get(comments::version_get::<false>),
        )
        .route(
            "/inline-comments/{id}/versions/{number}",
            get(comments::version_get::<true>),
        )
        .route(
            "/pages/{id}/footer-comments",
            get(comments::list_page::<false>),
        )
        .route(
            "/pages/{id}/inline-comments",
            get(comments::list_page::<true>),
        )
        .route(
            "/attachments/{id}/footer-comments",
            get(comments::list_attachment),
        )
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

/// Serves the built web app; unknown paths fall back to `index.html` except under the API prefixes.
pub fn with_web(app: Router, dir: PathBuf) -> Router {
    let files = ServeDir::new(&dir).fallback(ServeFile::new(dir.join("index.html")));
    app.fallback(move |req: Request| async move {
        let path = req.uri().path();
        if path.starts_with("/wiki/api/") || path.starts_with("/wiki/rest/") {
            return E(ApiError::NotFound("no such route".into())).into_response();
        }
        match files.oneshot(req).await {
            Ok(res) => res.into_response(),
            Err(e) => match e {},
        }
    })
}

#[cfg(test)]
mod web_tests {
    use super::*;
    use axum::body::Body;
    use http_body_util::BodyExt;

    async fn get(app: &Router, path: &str) -> (u16, String) {
        let res = app
            .clone()
            .oneshot(Request::get(path).body(Body::empty()).unwrap())
            .await
            .unwrap();
        let status = res.status().as_u16();
        let body = res.into_body().collect().await.unwrap().to_bytes();
        (status, String::from_utf8(body.to_vec()).unwrap())
    }

    #[tokio::test]
    async fn serves_spa_but_keeps_api_404_json() {
        let dir = std::env::temp_dir().join(format!("kk-web-{}", std::process::id()));
        std::fs::create_dir_all(dir.join("assets")).unwrap();
        std::fs::write(dir.join("index.html"), "<html>kk</html>").unwrap();
        std::fs::write(dir.join("assets/a.js"), "x=1").unwrap();
        let app = with_web(Router::new(), dir);
        assert_eq!(
            get(&app, "/wiki/spaces/ENG/overview").await,
            (200, "<html>kk</html>".into())
        );
        assert_eq!(get(&app, "/assets/a.js").await, (200, "x=1".into()));
        let (status, body) = get(&app, "/wiki/api/v2/nope").await;
        assert_eq!(status, 404);
        assert!(body.contains("no such route"), "{body}");
    }
}
