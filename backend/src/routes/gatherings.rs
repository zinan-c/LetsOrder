use axum::{Json, Router, extract::State, routing::post};

use crate::{
    errors::AppResult, models::CreateGatheringRequest, routes::AppState,
    services::gathering_service,
};

pub fn router() -> Router<AppState> {
    Router::new().route("/", post(create_gathering))
}

async fn create_gathering(
    State(state): State<AppState>,
    Json(payload): Json<CreateGatheringRequest>,
) -> AppResult<Json<serde_json::Value>> {
    let gathering = gathering_service::create_gathering(&state.pool, payload).await?;
    Ok(Json(serde_json::json!({ "gathering": gathering })))
}
