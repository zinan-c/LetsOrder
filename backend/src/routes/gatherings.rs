use axum::{
    Json, Router,
    extract::{Path, State},
    routing::{get, patch, post},
};
use uuid::Uuid;

use crate::{
    errors::AppResult,
    models::{
        CreateGatheringRequest, CreateMenuItemRequest, JoinGatheringRequest, UpdateGatheringRequest,
    },
    routes::AppState,
    services::gathering_service,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_gatherings).post(create_gathering))
        .route(
            "/{identifier}",
            get(get_gathering)
                .patch(update_gathering)
                .delete(delete_gathering),
        )
        .route("/{gathering_id}/lock", post(lock_gathering))
        .route("/{gathering_id}/activity-logs", get(list_activity_logs))
        .route(
            "/{gathering_id}/participants",
            post(join_gathering).get(list_participants),
        )
        .route(
            "/{gathering_id}/menu-items",
            get(list_menu_items).post(create_menu_item),
        )
}

async fn list_gatherings(State(state): State<AppState>) -> AppResult<Json<serde_json::Value>> {
    let gatherings = gathering_service::list_gatherings(&state.pool).await?;
    Ok(Json(serde_json::json!({ "gatherings": gatherings })))
}

async fn create_gathering(
    State(state): State<AppState>,
    Json(payload): Json<CreateGatheringRequest>,
) -> AppResult<Json<serde_json::Value>> {
    let gathering = gathering_service::create_gathering(&state.pool, payload).await?;
    Ok(Json(serde_json::json!(gathering)))
}

async fn get_gathering(
    State(state): State<AppState>,
    Path(invite_code): Path<String>,
) -> AppResult<Json<serde_json::Value>> {
    let gathering =
        gathering_service::get_gathering_by_invite_code(&state.pool, &invite_code).await?;
    Ok(Json(serde_json::json!({ "gathering": gathering })))
}

async fn delete_gathering(
    State(state): State<AppState>,
    Path(gathering_id): Path<Uuid>,
) -> AppResult<Json<serde_json::Value>> {
    let gathering = gathering_service::archive_gathering(&state.pool, gathering_id).await?;
    Ok(Json(serde_json::json!({ "gathering": gathering })))
}

async fn update_gathering(
    State(state): State<AppState>,
    Path(gathering_id): Path<Uuid>,
    Json(payload): Json<UpdateGatheringRequest>,
) -> AppResult<Json<serde_json::Value>> {
    let gathering =
        gathering_service::update_gathering_deadline(&state.pool, gathering_id, payload).await?;
    Ok(Json(serde_json::json!({ "gathering": gathering })))
}

async fn lock_gathering(
    State(state): State<AppState>,
    Path(gathering_id): Path<Uuid>,
) -> AppResult<Json<serde_json::Value>> {
    let gathering = gathering_service::lock_gathering(&state.pool, gathering_id).await?;
    Ok(Json(serde_json::json!({ "gathering": gathering })))
}

async fn join_gathering(
    State(state): State<AppState>,
    Path(gathering_id): Path<Uuid>,
    Json(payload): Json<JoinGatheringRequest>,
) -> AppResult<Json<serde_json::Value>> {
    let participant = gathering_service::join_gathering(&state.pool, gathering_id, payload).await?;
    Ok(Json(serde_json::json!(participant)))
}

async fn list_participants(
    State(state): State<AppState>,
    Path(gathering_id): Path<Uuid>,
) -> AppResult<Json<serde_json::Value>> {
    let participants = gathering_service::list_participants(&state.pool, gathering_id).await?;
    Ok(Json(serde_json::json!({ "participants": participants })))
}

async fn list_activity_logs(
    State(state): State<AppState>,
    Path(gathering_id): Path<Uuid>,
) -> AppResult<Json<serde_json::Value>> {
    let activity_logs = gathering_service::list_activity_logs(&state.pool, gathering_id).await?;
    Ok(Json(serde_json::json!({ "activity_logs": activity_logs })))
}

async fn list_menu_items(
    State(state): State<AppState>,
    Path(gathering_id): Path<Uuid>,
) -> AppResult<Json<serde_json::Value>> {
    let menu_items = gathering_service::list_menu_items(&state.pool, gathering_id).await?;
    Ok(Json(serde_json::json!({ "menu_items": menu_items })))
}

async fn create_menu_item(
    State(state): State<AppState>,
    Path(gathering_id): Path<Uuid>,
    Json(payload): Json<CreateMenuItemRequest>,
) -> AppResult<Json<serde_json::Value>> {
    let menu_item = gathering_service::create_menu_item(&state.pool, gathering_id, payload).await?;
    Ok(Json(serde_json::json!({ "menu_item": menu_item })))
}

pub fn menu_item_router() -> Router<AppState> {
    Router::new().route("/{menu_item_id}", patch(update_menu_item))
}

async fn update_menu_item(
    State(state): State<AppState>,
    Path(menu_item_id): Path<Uuid>,
    Json(payload): Json<crate::models::UpdateMenuItemRequest>,
) -> AppResult<Json<serde_json::Value>> {
    let menu_item = gathering_service::update_menu_item(&state.pool, menu_item_id, payload).await?;
    Ok(Json(serde_json::json!({ "menu_item": menu_item })))
}
