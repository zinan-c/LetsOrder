pub mod auth;
pub mod chefs;
pub mod gatherings;
pub mod health;
pub mod realtime;

use std::{
    collections::HashMap,
    path::Path,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

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
    pub login_limiter: LoginRateLimiter,
}

#[derive(Clone, Default)]
pub struct LoginRateLimiter {
    attempts: Arc<Mutex<HashMap<String, LoginAttempt>>>,
}

struct LoginAttempt {
    failures: u32,
    blocked_until: Option<Instant>,
    last_attempt: Instant,
}

impl LoginRateLimiter {
    const MAX_FAILURES: u32 = 5;
    const BLOCK_DURATION: Duration = Duration::from_secs(60);

    pub fn is_blocked(&self, key: &str) -> bool {
        let Ok(mut attempts) = self.attempts.lock() else {
            return false;
        };
        let now = Instant::now();
        attempts.retain(|_, attempt| attempt.last_attempt + Self::BLOCK_DURATION > now);
        attempts
            .get(key)
            .and_then(|attempt| attempt.blocked_until)
            .is_some_and(|until| until > now)
    }

    pub fn record_failure(&self, key: &str) {
        let Ok(mut attempts) = self.attempts.lock() else {
            return;
        };
        let attempt = attempts.entry(key.to_string()).or_insert(LoginAttempt {
            failures: 0,
            blocked_until: None,
            last_attempt: Instant::now(),
        });
        attempt.failures += 1;
        attempt.last_attempt = Instant::now();
        if attempt.failures >= Self::MAX_FAILURES {
            attempt.blocked_until = Some(Instant::now() + Self::BLOCK_DURATION);
        }
    }

    pub fn clear(&self, key: &str) {
        if let Ok(mut attempts) = self.attempts.lock() {
            attempts.remove(key);
        }
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct RealtimeEvent {
    pub event: String,
    pub gathering_id: Option<Uuid>,
}

pub fn router(pool: DbPool, realtime_tx: broadcast::Sender<RealtimeEvent>) -> Router {
    let state = AppState {
        pool,
        realtime_tx,
        login_limiter: LoginRateLimiter::default(),
    };

    Router::new()
        .route("/health", get(health::health_check))
        .route("/api/ws", get(realtime::websocket))
        .route(
            "/resources/uploads/{filename}",
            get(gatherings::serve_photo_resource),
        )
        .nest("/api/auth", auth::router())
        .nest("/api/chefs", chefs::router())
        .nest("/api/gatherings", gatherings::router())
        .nest("/api/menu-items", gatherings::menu_item_router())
        .nest("/api/photos", gatherings::photo_router())
        .nest_service(
            "/resources/mock",
            ServeDir::new(Path::new(&resource_dir()).join("mock")),
        )
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
