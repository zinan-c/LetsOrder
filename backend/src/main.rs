mod config;
mod db;
mod errors;
mod models;
mod routes;
mod services;

use std::net::SocketAddr;

use anyhow::Context;
use tokio::{net::TcpListener, sync::broadcast, time};
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
    let (realtime_tx, _) = broadcast::channel(128);
    spawn_expired_gathering_lock_job(pool.clone(), realtime_tx.clone());

    let app = routes::router(pool, realtime_tx)
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

fn spawn_expired_gathering_lock_job(
    pool: db::DbPool,
    realtime_tx: broadcast::Sender<routes::RealtimeEvent>,
) {
    tokio::spawn(async move {
        let mut interval = time::interval(time::Duration::from_secs(600));

        loop {
            interval.tick().await;
            match services::gathering_service::lock_expired_gatherings(&pool, 10).await {
                Ok(locked_gatherings) if !locked_gatherings.is_empty() => {
                    for gathering in &locked_gatherings {
                        let _ = realtime_tx.send(routes::RealtimeEvent {
                            event: "refresh".to_string(),
                            gathering_id: Some(gathering.id),
                        });
                    }
                    tracing::info!(
                        count = locked_gatherings.len(),
                        "auto locked expired gatherings"
                    );
                }
                Ok(_) => {}
                Err(error) => {
                    tracing::warn!(error = %error, "failed to auto lock expired gatherings");
                }
            }
        }
    });
}
