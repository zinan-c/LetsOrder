use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::Participant;

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub display_name: String,
    pub role: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    pub display_name: String,
    pub gathering_id: Option<Uuid>,
    pub invite_code: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateAccountRequest {
    pub display_name: Option<String>,
    pub password: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateMemberRequest {
    pub display_name: Option<String>,
    pub password: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub user: User,
    pub token: String,
    pub generated_password: Option<String>,
    pub participant: Option<Participant>,
}
