use std::env;

use kk_server::{app, bootstrap_admin, migrate, with_web, AppState, LocalBlobs, DEV_TOKEN};
use sqlx::postgres::PgPoolOptions;

fn var(name: &str, default: &str) -> String {
    env::var(name).unwrap_or_else(|_| default.into())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .json()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .init();

    let db = PgPoolOptions::new()
        .max_connections(10)
        .connect(&var(
            "KK_DATABASE_URL",
            "postgres://kk:kk@localhost:5432/kk",
        ))
        .await?;
    if env::args().any(|a| a == "--migrate") {
        migrate(&db).await?;
    }
    bootstrap_admin(
        &db,
        &var("KK_ADMIN_EMAIL", "admin@kaijukit.local"),
        &var("KK_ADMIN_TOKEN", DEV_TOKEN),
    )
    .await?;

    let state = AppState {
        db,
        base_url: var("KK_BASE_URL", "http://localhost:8080")
            .trim_end_matches('/')
            .into(),
        blobs: std::sync::Arc::new(LocalBlobs::new(var("KK_BLOB_DIR", "./blobs"))?),
        max_upload: var("KK_MAX_UPLOAD_BYTES", "26214400").parse()?,
    };
    let listener = tokio::net::TcpListener::bind(var("KK_BIND", "0.0.0.0:8080")).await?;
    tracing::info!(addr = %listener.local_addr()?, "listening");
    let router = match env::var("KK_WEB_DIR") {
        Ok(dir) => with_web(app(state), dir.into()),
        Err(_) => app(state),
    };
    axum::serve(listener, router)
        .with_graceful_shutdown(async {
            let _ = tokio::signal::ctrl_c().await;
        })
        .await?;
    Ok(())
}
