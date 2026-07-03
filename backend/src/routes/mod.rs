pub mod gatherings;
pub mod health;

use axum::{Router, routing::get};

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
        .with_state(state)
}
