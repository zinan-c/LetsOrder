use chrono::Utc;
use uuid::Uuid;

use crate::{
    db::DbPool,
    errors::{AppError, AppResult},
    models::{CreateGatheringRequest, Gathering, GatheringStatus},
};

pub async fn create_gathering(
    _pool: &DbPool,
    payload: CreateGatheringRequest,
) -> AppResult<Gathering> {
    if payload.title.trim().is_empty() {
        return Err(AppError::Validation("title is required".to_string()));
    }

    if payload.host_name.trim().is_empty() {
        return Err(AppError::Validation("host_name is required".to_string()));
    }

    let now = Utc::now();

    Ok(Gathering {
        id: Uuid::new_v4(),
        title: payload.title,
        description: payload.description,
        invite_code: Uuid::new_v4().simple().to_string()[..10].to_string(),
        status: GatheringStatus::Active,
        starts_at: payload.starts_at,
        expires_at: payload.expires_at,
        locked_at: None,
        archived_at: None,
        created_at: now,
        updated_at: now,
    })
}
