use axum::Json;

use crate::models::HealthResponse;

pub async fn health_check() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok",
        service: "letsorder-backend",
    })
}
