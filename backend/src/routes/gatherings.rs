use axum::{
    Json, Router,
    extract::{Multipart, Path, State},
    http::HeaderMap,
    routing::{get, patch, post},
};
use uuid::Uuid;

use crate::{
    errors::{AppError, AppResult},
    models::{
        CreateGatheringRequest, CreateMenuItemRequest, JoinGatheringRequest,
        UpdateGatheringRequest, UpdatePhotoRequest,
    },
    routes::{AppState, RealtimeEvent},
    services::{auth_service, gathering_service},
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
            "/{gathering_id}/photos",
            get(list_photos).post(upload_photo),
        )
        .route(
            "/{gathering_id}/menu-items",
            get(list_menu_items).post(create_menu_item),
        )
}

async fn list_gatherings(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> AppResult<Json<serde_json::Value>> {
    let user = require_user(&state, &headers).await?;
    let gatherings = if user.role == "admin" {
        gathering_service::list_gatherings(&state.pool).await?
    } else {
        gathering_service::list_gatherings_for_user(&state.pool, user.id).await?
    };
    Ok(Json(serde_json::json!({ "gatherings": gatherings })))
}

async fn create_gathering(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<CreateGatheringRequest>,
) -> AppResult<Json<serde_json::Value>> {
    ensure_admin(&state, &headers).await?;
    let gathering = gathering_service::create_gathering(&state.pool, payload).await?;
    notify_refresh(&state, Some(gathering.gathering.id));
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
    headers: HeaderMap,
) -> AppResult<Json<serde_json::Value>> {
    let gathering = gathering_service::archive_gathering(
        &state.pool,
        gathering_id,
        actor_name(&state, &headers).await,
    )
    .await?;
    notify_refresh(&state, Some(gathering.id));
    Ok(Json(serde_json::json!({ "gathering": gathering })))
}

async fn update_gathering(
    State(state): State<AppState>,
    Path(gathering_id): Path<Uuid>,
    headers: HeaderMap,
    Json(payload): Json<UpdateGatheringRequest>,
) -> AppResult<Json<serde_json::Value>> {
    let gathering = gathering_service::update_gathering_deadline(
        &state.pool,
        gathering_id,
        payload,
        actor_name(&state, &headers).await,
    )
    .await?;
    notify_refresh(&state, Some(gathering.id));
    Ok(Json(serde_json::json!({ "gathering": gathering })))
}

async fn lock_gathering(
    State(state): State<AppState>,
    Path(gathering_id): Path<Uuid>,
    headers: HeaderMap,
) -> AppResult<Json<serde_json::Value>> {
    let gathering = gathering_service::lock_gathering(
        &state.pool,
        gathering_id,
        actor_name(&state, &headers).await,
    )
    .await?;
    notify_refresh(&state, Some(gathering.id));
    Ok(Json(serde_json::json!({ "gathering": gathering })))
}

async fn join_gathering(
    State(state): State<AppState>,
    Path(gathering_id): Path<Uuid>,
    headers: HeaderMap,
    Json(payload): Json<JoinGatheringRequest>,
) -> AppResult<Json<serde_json::Value>> {
    let _ = payload;
    let user = require_user(&state, &headers).await?;
    let participant =
        auth_service::ensure_participant_for_user(&state.pool, gathering_id, user.id).await?;
    notify_refresh(&state, Some(gathering_id));
    Ok(Json(serde_json::json!({
        "participant": participant,
        "access_token": ""
    })))
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

async fn list_photos(
    State(state): State<AppState>,
    Path(gathering_id): Path<Uuid>,
) -> AppResult<Json<serde_json::Value>> {
    let photos = gathering_service::list_photos(&state.pool, gathering_id).await?;
    Ok(Json(serde_json::json!({ "photos": photos })))
}

async fn upload_photo(
    State(state): State<AppState>,
    Path(gathering_id): Path<Uuid>,
    headers: HeaderMap,
    multipart: Multipart,
) -> AppResult<Json<serde_json::Value>> {
    let photo = gathering_service::upload_photo(
        &state.pool,
        gathering_id,
        actor_name(&state, &headers).await,
        multipart,
    )
    .await?;
    notify_refresh(&state, Some(photo.gathering_id));
    Ok(Json(serde_json::json!({ "photo": photo })))
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
    headers: HeaderMap,
    Json(mut payload): Json<CreateMenuItemRequest>,
) -> AppResult<Json<serde_json::Value>> {
    let user = require_user(&state, &headers).await?;
    let participant =
        auth_service::ensure_participant_for_user(&state.pool, gathering_id, user.id).await?;
    payload.created_by = participant.id;
    let menu_item = gathering_service::create_menu_item(&state.pool, gathering_id, payload).await?;
    notify_refresh(&state, Some(menu_item.gathering_id));
    Ok(Json(serde_json::json!({ "menu_item": menu_item })))
}

pub fn menu_item_router() -> Router<AppState> {
    Router::new().route("/{menu_item_id}", patch(update_menu_item))
}

pub fn photo_router() -> Router<AppState> {
    Router::new().route("/{photo_id}", patch(update_photo).delete(delete_photo))
}

async fn update_menu_item(
    State(state): State<AppState>,
    Path(menu_item_id): Path<Uuid>,
    headers: HeaderMap,
    Json(mut payload): Json<crate::models::UpdateMenuItemRequest>,
) -> AppResult<Json<serde_json::Value>> {
    let user = require_user(&state, &headers).await?;
    let gathering_id = gathering_service::menu_item_gathering_id(&state.pool, menu_item_id).await?;
    let participant =
        auth_service::ensure_participant_for_user(&state.pool, gathering_id, user.id).await?;
    payload.updated_by = participant.id;
    let menu_item = gathering_service::update_menu_item(&state.pool, menu_item_id, payload).await?;
    notify_refresh(&state, Some(menu_item.gathering_id));
    Ok(Json(serde_json::json!({ "menu_item": menu_item })))
}

async fn update_photo(
    State(state): State<AppState>,
    Path(photo_id): Path<Uuid>,
    headers: HeaderMap,
    Json(payload): Json<UpdatePhotoRequest>,
) -> AppResult<Json<serde_json::Value>> {
    let photo = gathering_service::update_photo_caption(
        &state.pool,
        photo_id,
        payload.caption,
        actor_name(&state, &headers).await,
    )
    .await?;
    notify_refresh(&state, Some(photo.gathering_id));
    Ok(Json(serde_json::json!({ "photo": photo })))
}

async fn delete_photo(
    State(state): State<AppState>,
    Path(photo_id): Path<Uuid>,
    headers: HeaderMap,
) -> AppResult<Json<serde_json::Value>> {
    let photo =
        gathering_service::delete_photo(&state.pool, photo_id, actor_name(&state, &headers).await)
            .await?;
    notify_refresh(&state, Some(photo.gathering_id));
    Ok(Json(serde_json::json!({ "photo": photo })))
}

async fn ensure_admin(state: &AppState, headers: &HeaderMap) -> AppResult<()> {
    let user = require_user(state, headers).await?;

    if user.role == "admin" {
        Ok(())
    } else {
        Err(AppError::Forbidden)
    }
}

async fn require_user(state: &AppState, headers: &HeaderMap) -> AppResult<crate::models::User> {
    let Some(token) = optional_bearer_token(headers) else {
        return Err(AppError::Forbidden);
    };

    auth_service::user_from_token(&state.pool, token).await
}

async fn actor_name(state: &AppState, headers: &HeaderMap) -> Option<String> {
    let token = headers
        .get("authorization")
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "))
        .map(str::trim)
        .filter(|value| !value.is_empty())?;

    auth_service::user_from_token(&state.pool, token)
        .await
        .ok()
        .map(|user| {
            if user.role == "admin" {
                "suite-admin".to_string()
            } else {
                user.display_name
            }
        })
}

fn optional_bearer_token(headers: &HeaderMap) -> Option<&str> {
    headers
        .get("authorization")
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "))
        .map(str::trim)
        .filter(|value| !value.is_empty())
}

fn notify_refresh(state: &AppState, gathering_id: Option<Uuid>) {
    let _ = state.realtime_tx.send(RealtimeEvent {
        event: "refresh".to_string(),
        gathering_id,
    });
}
