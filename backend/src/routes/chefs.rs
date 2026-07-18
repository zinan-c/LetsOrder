use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::HeaderMap,
    routing::get,
};
use serde::Deserialize;

use crate::{
    errors::{AppError, AppResult},
    routes::AppState,
    services::{auth_service, gathering_service},
};

#[derive(Debug, Deserialize)]
struct DishRecommendationQuery {
    limit: Option<i64>,
}

pub fn router() -> Router<AppState> {
    Router::new().route(
        "/{chef_name}/dish-recommendations",
        get(list_dish_recommendations),
    )
}

async fn list_dish_recommendations(
    State(state): State<AppState>,
    Path(chef_name): Path<String>,
    Query(query): Query<DishRecommendationQuery>,
    headers: HeaderMap,
) -> AppResult<Json<serde_json::Value>> {
    let user = require_user(&state, &headers).await?;

    let recommendations =
        gathering_service::list_dish_recommendations(&state.pool, &chef_name, query.limit, &user)
            .await?;

    Ok(Json(
        serde_json::json!({ "recommendations": recommendations }),
    ))
}

async fn require_user(state: &AppState, headers: &HeaderMap) -> AppResult<crate::models::User> {
    let Some(token) = optional_bearer_token(headers) else {
        return Err(AppError::Forbidden);
    };

    auth_service::user_from_token(&state.pool, token).await
}

fn optional_bearer_token(headers: &HeaderMap) -> Option<&str> {
    headers
        .get("authorization")
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "))
        .map(str::trim)
        .filter(|value| !value.is_empty())
}
