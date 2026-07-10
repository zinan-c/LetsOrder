use axum::{
    Json, Router,
    extract::{Path, State},
    http::HeaderMap,
    http::StatusCode,
    routing::{get, patch, post},
};
use uuid::Uuid;

use crate::{
    errors::{AppError, AppResult},
    models::{LoginRequest, RegisterRequest, UpdateAccountRequest, UpdateMemberRequest},
    routes::AppState,
    services::auth_service,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/login", post(login))
        .route("/register", post(register))
        .route("/me", get(me))
        .route("/logout", post(logout))
        .route("/account", patch(update_account))
        .route("/members", get(list_members))
        .route("/members/{user_id}", patch(update_member))
}

async fn login(
    State(state): State<AppState>,
    Json(payload): Json<LoginRequest>,
) -> AppResult<Json<serde_json::Value>> {
    let response = auth_service::login(&state.pool, payload).await?;
    Ok(Json(serde_json::json!(response)))
}

async fn register(
    State(state): State<AppState>,
    Json(payload): Json<RegisterRequest>,
) -> AppResult<Json<serde_json::Value>> {
    let response = auth_service::register(&state.pool, payload).await?;
    Ok(Json(serde_json::json!(response)))
}

async fn me(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> AppResult<Json<serde_json::Value>> {
    let user = auth_service::me(&state.pool, bearer_token(&headers)?).await?;
    Ok(Json(serde_json::json!({ "user": user })))
}

async fn update_account(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<UpdateAccountRequest>,
) -> AppResult<Json<serde_json::Value>> {
    let user = auth_service::update_account(&state.pool, bearer_token(&headers)?, payload).await?;
    Ok(Json(serde_json::json!({ "user": user })))
}

async fn list_members(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> AppResult<Json<serde_json::Value>> {
    let members = auth_service::list_members(&state.pool, bearer_token(&headers)?).await?;
    Ok(Json(serde_json::json!({ "members": members })))
}

async fn update_member(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(user_id): Path<Uuid>,
    Json(payload): Json<UpdateMemberRequest>,
) -> AppResult<Json<serde_json::Value>> {
    let member =
        auth_service::update_member(&state.pool, bearer_token(&headers)?, user_id, payload).await?;
    Ok(Json(serde_json::json!({ "member": member })))
}

async fn logout(State(state): State<AppState>, headers: HeaderMap) -> AppResult<StatusCode> {
    auth_service::logout(&state.pool, bearer_token(&headers)?).await?;
    Ok(StatusCode::NO_CONTENT)
}

fn bearer_token(headers: &HeaderMap) -> AppResult<&str> {
    headers
        .get("authorization")
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "))
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or(AppError::Forbidden)
}
