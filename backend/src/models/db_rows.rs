use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::errors::{AppError, AppResult};

use super::{
    ActivityLog, Gathering, GatheringListItem, MenuItem, MenuItemRatingSummary, Participant, Photo,
    User,
};

#[derive(Debug, sqlx::FromRow)]
pub struct UserRow {
    pub id: String,
    pub username: String,
    pub display_name: String,
    pub role: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl TryFrom<UserRow> for User {
    type Error = AppError;

    fn try_from(row: UserRow) -> AppResult<Self> {
        Ok(Self {
            id: parse_uuid(&row.id)?,
            username: row.username,
            display_name: row.display_name,
            role: row.role,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }
}

#[derive(Debug, sqlx::FromRow)]
pub struct GatheringRow {
    pub id: String,
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

impl TryFrom<GatheringRow> for Gathering {
    type Error = AppError;

    fn try_from(row: GatheringRow) -> AppResult<Self> {
        Ok(Self {
            id: parse_uuid(&row.id)?,
            title: row.title,
            description: row.description,
            invite_code: row.invite_code,
            status: row.status,
            is_locked: row.is_locked,
            starts_at: row.starts_at,
            expires_at: row.expires_at,
            locked_at: row.locked_at,
            archived_at: row.archived_at,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }
}

#[derive(Debug, sqlx::FromRow)]
pub struct GatheringListItemRow {
    pub id: String,
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

impl TryFrom<GatheringListItemRow> for GatheringListItem {
    type Error = AppError;

    fn try_from(row: GatheringListItemRow) -> AppResult<Self> {
        Ok(Self {
            id: parse_uuid(&row.id)?,
            title: row.title,
            description: row.description,
            invite_code: row.invite_code,
            status: row.status,
            is_locked: row.is_locked,
            expires_at: row.expires_at,
            item_count: row.item_count,
            prepared_count: row.prepared_count,
            participant_count: row.participant_count,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }
}

#[derive(Debug, sqlx::FromRow)]
pub struct ParticipantRow {
    pub id: String,
    pub gathering_id: String,
    pub user_id: Option<String>,
    pub display_name: String,
    pub role: String,
    pub last_menu_activity_at: Option<DateTime<Utc>>,
    pub joined_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl TryFrom<ParticipantRow> for Participant {
    type Error = AppError;

    fn try_from(row: ParticipantRow) -> AppResult<Self> {
        Ok(Self {
            id: parse_uuid(&row.id)?,
            gathering_id: parse_uuid(&row.gathering_id)?,
            user_id: row.user_id.as_deref().map(parse_uuid).transpose()?,
            display_name: row.display_name,
            role: row.role,
            last_menu_activity_at: row.last_menu_activity_at,
            joined_at: row.joined_at,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }
}

#[derive(Debug, sqlx::FromRow)]
pub struct MenuItemRow {
    pub id: String,
    pub gathering_id: String,
    pub created_by: String,
    pub updated_by: Option<String>,
    pub name: String,
    pub category: Option<String>,
    pub quantity: i64,
    pub unit: Option<String>,
    pub owner_name: Option<String>,
    pub reference_url: Option<String>,
    pub note: Option<String>,
    pub status: String,
    pub revision: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl TryFrom<MenuItemRow> for MenuItem {
    type Error = AppError;

    fn try_from(row: MenuItemRow) -> AppResult<Self> {
        Ok(Self {
            id: parse_uuid(&row.id)?,
            gathering_id: parse_uuid(&row.gathering_id)?,
            created_by: parse_uuid(&row.created_by)?,
            updated_by: row.updated_by.as_deref().map(parse_uuid).transpose()?,
            name: row.name,
            category: row.category,
            quantity: row.quantity,
            unit: row.unit,
            owner_name: row.owner_name,
            reference_url: row.reference_url,
            note: row.note,
            status: row.status,
            revision: row.revision,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }
}

#[derive(Debug, sqlx::FromRow)]
pub struct MenuItemRatingSummaryRow {
    pub menu_item_id: String,
    pub average_rating: Option<f64>,
    pub rating_count: i64,
    pub my_rating: Option<i64>,
}

impl TryFrom<MenuItemRatingSummaryRow> for MenuItemRatingSummary {
    type Error = AppError;

    fn try_from(row: MenuItemRatingSummaryRow) -> AppResult<Self> {
        Ok(Self {
            menu_item_id: parse_uuid(&row.menu_item_id)?,
            average_rating: row.average_rating,
            rating_count: row.rating_count,
            my_rating: row.my_rating,
        })
    }
}

#[derive(Debug, sqlx::FromRow)]
pub struct ActivityLogRow {
    pub id: String,
    pub gathering_id: String,
    pub actor_id: Option<String>,
    pub actor_name: Option<String>,
    pub action: String,
    pub target_type: String,
    pub target_id: Option<String>,
    pub detail: Option<String>,
    pub created_at: DateTime<Utc>,
}

impl TryFrom<ActivityLogRow> for ActivityLog {
    type Error = AppError;

    fn try_from(row: ActivityLogRow) -> AppResult<Self> {
        Ok(Self {
            id: parse_uuid(&row.id)?,
            gathering_id: parse_uuid(&row.gathering_id)?,
            actor_id: row.actor_id.as_deref().map(parse_uuid).transpose()?,
            actor_name: row.actor_name,
            action: row.action,
            target_type: row.target_type,
            target_id: row.target_id.as_deref().map(parse_uuid).transpose()?,
            detail: row.detail,
            created_at: row.created_at,
        })
    }
}

#[derive(Debug, sqlx::FromRow)]
pub struct PhotoRow {
    pub id: String,
    pub gathering_id: String,
    pub uploaded_by: String,
    pub file_url: String,
    pub thumbnail_url: Option<String>,
    pub caption: Option<String>,
    pub taken_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl TryFrom<PhotoRow> for Photo {
    type Error = AppError;

    fn try_from(row: PhotoRow) -> AppResult<Self> {
        Ok(Self {
            id: parse_uuid(&row.id)?,
            gathering_id: parse_uuid(&row.gathering_id)?,
            uploaded_by: parse_uuid(&row.uploaded_by)?,
            file_url: row.file_url,
            thumbnail_url: row.thumbnail_url,
            caption: row.caption,
            taken_at: row.taken_at,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }
}

fn parse_uuid(value: &str) -> AppResult<Uuid> {
    Uuid::parse_str(value).map_err(|error| AppError::Validation(format!("invalid uuid: {error}")))
}
