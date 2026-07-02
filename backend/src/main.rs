mod config;
mod db;
mod errors;
mod models;
mod routes;
mod services;

use std::net::SocketAddr;

use anyhow::Context;
use tokio::net::TcpListener;
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "letsorder_backend=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let config = config::Config::from_env();
    let pool = db::connect(&config.database_url).await?;

    let app = routes::router(pool)
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http());

    let addr = SocketAddr::from(([0, 0, 0, 0], config.port));
    let listener = TcpListener::bind(addr)
        .await
        .with_context(|| format!("failed to bind server to {addr}"))?;

    tracing::info!("LetsOrder backend listening on http://{addr}");
    axum::serve(listener, app).await.context("server failed")?;

    Ok(())
}
