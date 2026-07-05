pub mod auth;
pub mod gatherings;
pub mod health;
pub mod realtime;

use std::path::Path;

use axum::{Router, routing::get};
use serde::Serialize;
use tokio::sync::broadcast;
use tower_http::services::ServeDir;
use uuid::Uuid;

use crate::db::DbPool;

#[derive(Clone)]
pub struct AppState {
    pub pool: DbPool,
    pub realtime_tx: broadcast::Sender<RealtimeEvent>,
}

#[derive(Clone, Debug, Serialize)]
pub struct RealtimeEvent {
    pub event: String,
    pub gathering_id: Option<Uuid>,
}

pub fn router(pool: DbPool, realtime_tx: broadcast::Sender<RealtimeEvent>) -> Router {
    let state = AppState { pool, realtime_tx };

    Router::new()
        .route("/health", get(health::health_check))
        .route("/api/ws", get(realtime::websocket))
        .nest("/api/auth", auth::router())
        .nest("/api/gatherings", gatherings::router())
        .nest("/api/menu-items", gatherings::menu_item_router())
        .nest("/api/photos", gatherings::photo_router())
        .nest_service("/resources", ServeDir::new(resource_dir()))
        .with_state(state)
}

fn resource_dir() -> String {
    std::env::var("LETSORDER_RESOURCE_DIR").unwrap_or_else(|_| {
        if Path::new("backend/resources").exists() {
            "backend/resources".to_string()
        } else {
            "resources".to_string()
        }
    })
}
