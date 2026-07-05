use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct Gathering {
    pub id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub invite_code: String,
    pub status: String,
    pub is_locked: bool,
    pub starts_at: Option<DateTime<Utc>>,
    pub expires_at: DateTime<Utc>,
    pub locked_at: Option<DateTime<Utc>>,
    pub archived_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct GatheringListItem {
    pub id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub invite_code: String,
    pub status: String,
    pub is_locked: bool,
    pub expires_at: DateTime<Utc>,
    pub item_count: i64,
    pub prepared_count: i64,
    pub participant_count: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateGatheringRequest {
    pub title: String,
    pub description: Option<String>,
    pub host_name: String,
    pub starts_at: Option<DateTime<Utc>>,
    pub expires_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct CreateGatheringResponse {
    pub gathering: Gathering,
    pub host: Participant,
    pub access_token: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateGatheringRequest {
    pub expires_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct Participant {
    pub id: Uuid,
    pub gathering_id: Uuid,
    pub user_id: Option<Uuid>,
    pub display_name: String,
    pub role: String,
    pub last_menu_activity_at: Option<DateTime<Utc>>,
    pub joined_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct JoinGatheringRequest {}

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct MenuItem {
    pub id: Uuid,
    pub gathering_id: Uuid,
    pub created_by: Uuid,
    pub updated_by: Option<Uuid>,
    pub name: String,
    pub category: Option<String>,
    pub quantity: i64,
    pub unit: Option<String>,
    pub owner_name: Option<String>,
    pub reference_url: Option<String>,
    pub note: Option<String>,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateMenuItemRequest {
    pub created_by: Uuid,
    pub name: String,
    pub category: Option<String>,
    pub quantity: Option<i64>,
    pub unit: Option<String>,
    pub owner_name: Option<String>,
    pub reference_url: Option<String>,
    pub note: Option<String>,
    pub status: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateMenuItemRequest {
    pub updated_by: Uuid,
    pub name: Option<String>,
    pub category: Option<String>,
    pub quantity: Option<i64>,
    pub unit: Option<String>,
    pub owner_name: Option<String>,
    pub reference_url: Option<String>,
    pub note: Option<String>,
    pub status: Option<String>,
}

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct ActivityLog {
    pub id: Uuid,
    pub gathering_id: Uuid,
    pub actor_id: Option<Uuid>,
    pub actor_name: Option<String>,
    pub action: String,
    pub target_type: String,
    pub target_id: Option<Uuid>,
    pub detail: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct Photo {
    pub id: Uuid,
    pub gathering_id: Uuid,
    pub uploaded_by: Uuid,
    pub file_url: String,
    pub thumbnail_url: Option<String>,
    pub caption: Option<String>,
    pub taken_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct UpdatePhotoRequest {
    pub caption: String,
}
