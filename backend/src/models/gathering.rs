use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GatheringStatus {
    Draft,
    Active,
    Locked,
    Archived,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Gathering {
    pub id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub invite_code: String,
    pub status: GatheringStatus,
    pub starts_at: Option<DateTime<Utc>>,
    pub expires_at: DateTime<Utc>,
    pub locked_at: Option<DateTime<Utc>>,
    pub archived_at: Option<DateTime<Utc>>,
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
