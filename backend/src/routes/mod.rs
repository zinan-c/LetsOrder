pub mod gatherings;
pub mod health;

use std::path::Path;

use axum::{Router, routing::get};
use tower_http::services::ServeDir;

use crate::db::DbPool;

#[derive(Clone)]
pub struct AppState {
    pub pool: DbPool,
}

pub fn router(pool: DbPool) -> Router {
    let state = AppState { pool };

    Router::new()
        .route("/health", get(health::health_check))
        .nest("/api/gatherings", gatherings::router())
        .nest("/api/menu-items", gatherings::menu_item_router())
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
