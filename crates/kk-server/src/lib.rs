mod admin;
mod attachments;
mod auth;
mod authz;
mod blob;
mod body;
mod classification;
mod comments;
mod content;
mod descendants;
mod error;
mod expand;
mod groups;
mod http;
pub mod jobs;
mod kinds;
mod labels;
mod likes;
mod multipart;
mod notifications;
mod pages;
mod props;
mod restrictions;
mod search;
mod space_perms;
mod spaces;
mod target;
mod tasks;
mod tree;
mod users;
mod v1;
mod v1_content;
mod versions;
mod watches;

use axum::{
    extract::{DefaultBodyLimit, Request, State},
    middleware,
    response::IntoResponse,
    routing::{delete, get, post, put},
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
pub use blob::{BlobStore, LocalBlobs};

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    /// Public origin, no trailing slash; `_links.base` is this plus `/wiki`.
    pub base_url: String,
    pub blobs: std::sync::Arc<dyn blob::BlobStore>,
    /// Largest accepted attachment in bytes.
    pub max_upload: usize,
}

impl AppState {
    pub(crate) fn site(&self) -> String {
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
        .route("/labels/{id}/pages", get(pages::of_label))
        .route("/labels/{id}/attachments", get(attachments::of_label))
        .route("/pages", get(pages::list).post(pages::create))
        .route(
            "/pages/{id}",
            get(pages::get).put(pages::update).delete(pages::delete),
        )
        .route("/users-bulk", post(users::bulk))
        .route("/classification-levels", get(classification::list))
        .route(
            "/spaces/{id}/classification-level/default",
            get(classification::space_get)
                .put(classification::space_put)
                .delete(classification::space_delete),
        )
        .route(
            "/admin-key",
            get(admin::get_key)
                .post(admin::enable_key)
                .delete(admin::disable_key),
        )
        .route("/data-policies/metadata", get(admin::policy_metadata))
        .route("/data-policies/spaces", get(admin::policy_spaces))
        .route(
            "/pages/{id}/classification-level",
            get(classification::get::<0>).put(classification::put::<0>),
        )
        .route(
            "/pages/{id}/classification-level/reset",
            post(classification::reset::<0>),
        )
        .route(
            "/blogposts/{id}/classification-level",
            get(classification::get::<4>).put(classification::put::<4>),
        )
        .route(
            "/blogposts/{id}/classification-level/reset",
            post(classification::reset::<4>),
        )
        .route(
            "/whiteboards/{id}/classification-level",
            get(classification::get::<6>).put(classification::put::<6>),
        )
        .route(
            "/whiteboards/{id}/classification-level/reset",
            post(classification::reset::<6>),
        )
        .route(
            "/databases/{id}/classification-level",
            get(classification::get::<7>).put(classification::put::<7>),
        )
        .route(
            "/databases/{id}/classification-level/reset",
            post(classification::reset::<7>),
        )
        .route("/space-permissions", get(space_perms::available))
        .route("/spaces/{id}/permissions", get(space_perms::assignments))
        .route(
            "/pages/{id}/permissions",
            get(restrictions::permissions::<0>)
                .post(restrictions::permissions_add::<0>)
                .put(restrictions::permissions_set::<0>)
                .delete(restrictions::permissions_delete::<0>),
        )
        .route(
            "/pages/{id}/restriction-state",
            get(restrictions::restriction_state::<0>).put(restrictions::set_restriction_state::<0>),
        )
        .route(
            "/blogposts/{id}/permissions",
            get(restrictions::permissions::<4>)
                .post(restrictions::permissions_add::<4>)
                .put(restrictions::permissions_set::<4>)
                .delete(restrictions::permissions_delete::<4>),
        )
        .route(
            "/blogposts/{id}/restriction-state",
            get(restrictions::restriction_state::<4>).put(restrictions::set_restriction_state::<4>),
        )
        .route(
            "/folders/{id}/permissions",
            get(restrictions::permissions::<5>)
                .post(restrictions::permissions_add::<5>)
                .put(restrictions::permissions_set::<5>)
                .delete(restrictions::permissions_delete::<5>),
        )
        .route(
            "/folders/{id}/restriction-state",
            get(restrictions::restriction_state::<5>).put(restrictions::set_restriction_state::<5>),
        )
        .route(
            "/whiteboards/{id}/permissions",
            get(restrictions::permissions::<6>)
                .post(restrictions::permissions_add::<6>)
                .put(restrictions::permissions_set::<6>)
                .delete(restrictions::permissions_delete::<6>),
        )
        .route(
            "/whiteboards/{id}/restriction-state",
            get(restrictions::restriction_state::<6>).put(restrictions::set_restriction_state::<6>),
        )
        .route(
            "/databases/{id}/permissions",
            get(restrictions::permissions::<7>)
                .post(restrictions::permissions_add::<7>)
                .put(restrictions::permissions_set::<7>)
                .delete(restrictions::permissions_delete::<7>),
        )
        .route(
            "/databases/{id}/restriction-state",
            get(restrictions::restriction_state::<7>).put(restrictions::set_restriction_state::<7>),
        )
        .route(
            "/user/access/check-access-by-email",
            post(users::check_access),
        )
        .route("/pages/{id}/title", put(pages::update_title))
        .route("/pages/{id}/versions", get(versions::list))
        .route("/pages/{id}/versions/{number}", get(versions::get))
        .route("/pages/{id}/children", get(tree::children))
        .route(
            "/pages/{id}/direct-children",
            get(tree::direct_children::<0>),
        )
        .route("/pages/{id}/ancestors", get(tree::ancestors::<0>))
        .route("/pages/{id}/descendants", get(tree::descendants::<0>))
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
            get(comments::list_page::<false, 0>),
        )
        .route(
            "/pages/{id}/inline-comments",
            get(comments::list_page::<true, 0>),
        )
        .route(
            "/attachments/{id}/footer-comments",
            get(comments::list_attachment),
        )
        .route("/tasks", get(tasks::list))
        .route("/tasks/{id}", get(tasks::get).put(tasks::update))
        .route("/attachments", get(attachments::list_all))
        .route(
            "/attachments/{id}",
            get(attachments::get).delete(attachments::delete),
        )
        .route("/pages/{id}/attachments", get(attachments::list_page::<0>))
        .route("/attachments/{id}/versions", get(attachments::version_list))
        .route(
            "/attachments/{id}/thumbnail/download",
            get(attachments::thumbnail),
        )
        .route(
            "/attachments/{id}/versions/{number}",
            get(attachments::version_get),
        )
        .route("/labels", get(labels::all))
        .route("/pages/{id}/labels", get(labels::of_content::<0>))
        .route("/attachments/{id}/labels", get(labels::of_content::<1>))
        .route("/spaces/{id}/labels", get(labels::of_space))
        .route("/spaces/{id}/content/labels", get(labels::of_space_content))
        .route(
            "/pages/{id}/properties",
            get(props::list::<0>).post(props::create::<0>),
        )
        .route(
            "/attachments/{id}/properties",
            get(props::list::<1>).post(props::create::<1>),
        )
        .route(
            "/comments/{id}/properties",
            get(props::list::<2>).post(props::create::<2>),
        )
        .route(
            "/spaces/{id}/properties",
            get(props::list::<3>).post(props::create::<3>),
        )
        .route(
            "/pages/{id}/properties/{pid}",
            get(props::get::<0>)
                .put(props::update::<0>)
                .delete(props::delete::<0>),
        )
        .route(
            "/attachments/{id}/properties/{pid}",
            get(props::get::<1>)
                .put(props::update::<1>)
                .delete(props::delete::<1>),
        )
        .route(
            "/comments/{id}/properties/{pid}",
            get(props::get::<2>)
                .put(props::update::<2>)
                .delete(props::delete::<2>),
        )
        .route(
            "/spaces/{id}/properties/{pid}",
            get(props::get::<3>)
                .put(props::update::<3>)
                .delete(props::delete::<3>),
        )
        .route("/app/properties", get(props::app_list))
        .route(
            "/app/properties/{key}",
            get(props::app_get)
                .put(props::app_put)
                .delete(props::app_delete),
        )
        .route("/pages/{id}/likes/count", get(likes::count::<0>))
        .route("/footer-comments/{id}/likes/count", get(likes::count::<1>))
        .route("/inline-comments/{id}/likes/count", get(likes::count::<2>))
        .route("/pages/{id}/likes/users", get(likes::users::<0>))
        .route("/footer-comments/{id}/likes/users", get(likes::users::<1>))
        .route("/inline-comments/{id}/likes/users", get(likes::users::<2>))
        .route("/pages/{id}/operations", get(likes::page_ops))
        .route("/spaces/{id}/operations", get(likes::space_ops))
        .route("/attachments/{id}/operations", get(likes::attachment_ops))
        .route(
            "/footer-comments/{id}/operations",
            get(likes::comment_ops::<1>),
        )
        .route(
            "/inline-comments/{id}/operations",
            get(likes::comment_ops::<2>),
        )
        .route(
            "/blogposts",
            get(content::list::<4>).post(content::create::<4>),
        )
        .route(
            "/blogposts/{id}",
            get(content::get::<4>)
                .put(content::update::<4>)
                .delete(content::delete::<4>),
        )
        .route("/spaces/{id}/blogposts", get(content::list_in::<4, 0>))
        .route("/labels/{id}/blogposts", get(content::list_in::<4, 1>))
        .route("/blogposts/{id}/labels", get(labels::of_content::<4>))
        .route("/blogposts/{id}/likes/count", get(likes::count::<3>))
        .route("/blogposts/{id}/likes/users", get(likes::users::<3>))
        .route("/blogposts/{id}/versions", get(versions::list_k::<4>))
        .route(
            "/blogposts/{id}/versions/{number}",
            get(versions::get_k::<4>),
        )
        .route(
            "/blogposts/{id}/attachments",
            get(attachments::list_page::<4>),
        )
        .route(
            "/blogposts/{id}/footer-comments",
            get(comments::list_page::<false, 4>),
        )
        .route(
            "/blogposts/{id}/inline-comments",
            get(comments::list_page::<true, 4>),
        )
        .route(
            "/blogposts/{id}/custom-content",
            get(content::list_in::<9, 2>),
        )
        .route("/blogposts/{id}/operations", get(likes::ops::<4>))
        .route(
            "/blogposts/{id}/properties",
            get(props::list::<4>).post(props::create::<4>),
        )
        .route(
            "/blogposts/{id}/properties/{pid}",
            get(props::get::<4>)
                .put(props::update::<4>)
                .delete(props::delete::<4>),
        )
        .route("/folders/{id}/ancestors", get(tree::ancestors::<5>))
        .route(
            "/folders/{id}/direct-children",
            get(tree::direct_children::<5>),
        )
        .route("/folders/{id}/descendants", get(tree::descendants::<5>))
        .route("/folders/{id}/operations", get(likes::ops::<5>))
        .route(
            "/folders/{id}/properties",
            get(props::list::<5>).post(props::create::<5>),
        )
        .route(
            "/folders/{id}/properties/{pid}",
            get(props::get::<5>)
                .put(props::update::<5>)
                .delete(props::delete::<5>),
        )
        .route("/whiteboards/{id}/ancestors", get(tree::ancestors::<6>))
        .route(
            "/whiteboards/{id}/direct-children",
            get(tree::direct_children::<6>),
        )
        .route("/whiteboards/{id}/descendants", get(tree::descendants::<6>))
        .route("/whiteboards/{id}/operations", get(likes::ops::<6>))
        .route(
            "/whiteboards/{id}/properties",
            get(props::list::<6>).post(props::create::<6>),
        )
        .route(
            "/whiteboards/{id}/properties/{pid}",
            get(props::get::<6>)
                .put(props::update::<6>)
                .delete(props::delete::<6>),
        )
        .route("/databases/{id}/ancestors", get(tree::ancestors::<7>))
        .route(
            "/databases/{id}/direct-children",
            get(tree::direct_children::<7>),
        )
        .route("/databases/{id}/descendants", get(tree::descendants::<7>))
        .route("/databases/{id}/operations", get(likes::ops::<7>))
        .route(
            "/databases/{id}/properties",
            get(props::list::<7>).post(props::create::<7>),
        )
        .route(
            "/databases/{id}/properties/{pid}",
            get(props::get::<7>)
                .put(props::update::<7>)
                .delete(props::delete::<7>),
        )
        .route("/embeds/{id}/ancestors", get(tree::ancestors::<8>))
        .route(
            "/embeds/{id}/direct-children",
            get(tree::direct_children::<8>),
        )
        .route("/embeds/{id}/descendants", get(tree::descendants::<8>))
        .route("/embeds/{id}/operations", get(likes::ops::<8>))
        .route(
            "/embeds/{id}/properties",
            get(props::list::<8>).post(props::create::<8>),
        )
        .route(
            "/embeds/{id}/properties/{pid}",
            get(props::get::<8>)
                .put(props::update::<8>)
                .delete(props::delete::<8>),
        )
        .route("/folders", post(content::create::<5>))
        .route(
            "/folders/{id}",
            get(content::get::<5>).delete(content::delete::<5>),
        )
        .route("/whiteboards", post(content::create::<6>))
        .route(
            "/whiteboards/{id}",
            get(content::get::<6>).delete(content::delete::<6>),
        )
        .route("/databases", post(content::create::<7>))
        .route(
            "/databases/{id}",
            get(content::get::<7>).delete(content::delete::<7>),
        )
        .route("/databases/{id}/entries", get(content::database_list::<0>))
        .route(
            "/databases/{id}/entries/{entry}",
            get(content::database_entry),
        )
        .route("/databases/{id}/fields", get(content::database_list::<1>))
        .route("/databases/{id}/views", get(content::database_list::<2>))
        .route("/embeds", post(content::create::<8>))
        .route(
            "/embeds/{id}",
            get(content::get::<8>).delete(content::delete::<8>),
        )
        .route(
            "/custom-content",
            get(content::list::<9>).post(content::create::<9>),
        )
        .route(
            "/custom-content/{id}",
            get(content::get::<9>)
                .put(content::update::<9>)
                .delete(content::delete::<9>),
        )
        .route(
            "/custom-content/{id}/children",
            get(content::list_in::<9, 2>),
        )
        .route("/spaces/{id}/custom-content", get(content::list_in::<9, 0>))
        .route("/pages/{id}/custom-content", get(content::list_in::<9, 2>))
        .route("/custom-content/{id}/labels", get(labels::of_content::<9>))
        .route("/custom-content/{id}/versions", get(versions::list_k::<9>))
        .route(
            "/custom-content/{id}/versions/{number}",
            get(versions::get_k::<9>),
        )
        .route(
            "/custom-content/{id}/attachments",
            get(attachments::list_page::<9>),
        )
        .route(
            "/custom-content/{id}/footer-comments",
            get(comments::list_page::<false, 9>),
        )
        .route("/custom-content/{id}/operations", get(likes::ops::<9>))
        .route(
            "/custom-content/{id}/properties",
            get(props::list::<9>).post(props::create::<9>),
        )
        .route(
            "/custom-content/{id}/properties/{pid}",
            get(props::get::<9>)
                .put(props::update::<9>)
                .delete(props::delete::<9>),
        )
        .route_layer(middleware::from_fn_with_state(state.clone(), authz::guard))
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            auth::require_auth,
        ));
    // multipart framing adds a little to the file itself
    let upload_limit = state.max_upload + 64 * 1024;
    let download = Router::new()
        .route(
            "/attachments/{container}/{filename}",
            get(attachments::download),
        )
        .route_layer(middleware::from_fn_with_state(state.clone(), authz::guard))
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            auth::require_auth,
        ));
    let v1 = Router::new()
        .route(
            "/content/{id}/label",
            post(labels::add_content).delete(labels::remove_content_by_query),
        )
        .route("/content/{id}/label/{name}", delete(labels::remove_content))
        .route(
            "/space/{key}/label",
            post(labels::add_space).delete(labels::remove_space),
        )
        .route(
            "/content/{id}/child/attachment",
            post(attachments::upload)
                .put(attachments::upload)
                .layer(DefaultBodyLimit::max(upload_limit)),
        )
        .route(
            "/content/{id}/child/attachment/{att}/data",
            post(attachments::update_data).layer(DefaultBodyLimit::max(upload_limit)),
        )
        .route(
            "/content/{id}/child/attachment/{att}/download",
            get(attachments::v1_download),
        )
        .route("/content/{id}/like", put(likes::set).delete(likes::set))
        .route(
            "/content/{id}/restriction",
            get(restrictions::get)
                .post(restrictions::add)
                .put(restrictions::update)
                .delete(restrictions::delete_all),
        )
        .route(
            "/content/{id}/restriction/byOperation",
            get(restrictions::by_operation),
        )
        .route(
            "/content/{id}/restriction/byOperation/{op}",
            get(restrictions::for_operation),
        )
        .route(
            "/content/{id}/restriction/byOperation/{op}/user",
            get(restrictions::user_get)
                .put(restrictions::user_put)
                .delete(restrictions::user_delete),
        )
        .route(
            "/content/{id}/restriction/byOperation/{op}/byGroupId/{gid}",
            get(restrictions::group_get)
                .put(restrictions::group_put)
                .delete(restrictions::group_delete),
        )
        .route("/content/{id}/permission/check", post(restrictions::check))
        .route("/space/{key}/permission", post(space_perms::add))
        .route("/space/{key}/permission/{id}", delete(space_perms::remove))
        .route(
            "/content/{id}/move/{position}/{target}",
            put(v1_content::move_page),
        )
        .route(
            "/content/{id}/version/{number}",
            delete(v1_content::delete_version),
        )
        .route("/content/{id}/version", post(v1_content::restore_version))
        .route("/user/current", get(users::current))
        .route(
            "/user/watch/content/{contentId}",
            get(watches::content_get)
                .post(watches::content_add)
                .delete(watches::content_remove),
        )
        .route(
            "/user/watch/space/{spaceKey}",
            get(watches::space_get)
                .post(watches::space_add)
                .delete(watches::space_remove),
        )
        .route("/content/{id}/descendant", get(descendants::all))
        .route("/content/{id}/descendant/{type}", get(descendants::typed))
        .route("/search", get(search::search))
        .route("/content/search", get(search::content))
        .route("/search/user", get(search::user))
        .route("/group", get(groups::list).post(groups::create))
        .route("/group/by-id", get(groups::get).delete(groups::delete))
        .route(
            "/group/userByGroupId",
            post(groups::add_member).delete(groups::remove_member),
        )
        .route("/group/{id}/membersByGroupId", get(groups::members))
        .route_layer(middleware::from_fn_with_state(state.clone(), authz::guard))
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            auth::require_auth,
        ));
    // kaijukit-only endpoints; ids are named `nid` so the content guard leaves them alone
    let kk = Router::new()
        .route("/users", post(users::create))
        .route("/notifications", get(notifications::list))
        .route(
            "/notifications/unread-count",
            get(notifications::unread_count),
        )
        .route(
            "/notifications/read-all",
            post(notifications::mark_all_read),
        )
        .route("/notifications/{nid}/read", post(notifications::mark_read))
        .route_layer(middleware::from_fn_with_state(state.clone(), authz::guard))
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            auth::require_auth,
        ));
    Router::new()
        .route("/healthz", get(|| async { "ok" }))
        .route("/readyz", get(ready))
        .nest("/wiki/api/v2", v2)
        .nest("/wiki/rest/api", v1)
        .nest("/wiki/rest/kk/v1", kk)
        .nest("/wiki/download", download)
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
