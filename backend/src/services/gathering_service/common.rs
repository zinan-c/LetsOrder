use std::path::Path;

use chrono::Utc;
use uuid::Uuid;

use crate::{
    db::DbPool,
    errors::{AppError, AppResult},
    models::{
        Gathering, GatheringRow, MenuItem, MenuItemRow, Participant, ParticipantRow, Photo,
        PhotoRow, User,
    },
};

pub(super) async fn get_gathering_by_id(pool: &DbPool, gathering_id: Uuid) -> AppResult<Gathering> {
    let row = sqlx::query_as::<_, GatheringRow>(
        r#"
        SELECT id, title, description, invite_code, status, starts_at, expires_at,
               is_locked, locked_at, archived_at, created_at, updated_at
        FROM gatherings
        WHERE id = ?
        "#,
    )
    .bind(gathering_id.to_string())
    .fetch_optional(pool)
    .await?
    .ok_or(AppError::NotFound)?;

    row.try_into()
}

pub(super) async fn get_photo_by_id(pool: &DbPool, photo_id: Uuid) -> AppResult<Photo> {
    let row = sqlx::query_as::<_, PhotoRow>(
        r#"
        SELECT id, gathering_id, uploaded_by, file_url, thumbnail_url, caption,
               taken_at, created_at, updated_at
        FROM photos
        WHERE id = ?
        "#,
    )
    .bind(photo_id.to_string())
    .fetch_optional(pool)
    .await?
    .ok_or(AppError::NotFound)?;

    row.try_into()
}

pub(super) async fn get_participant_by_id(
    pool: &DbPool,
    participant_id: Uuid,
) -> AppResult<Participant> {
    let row = sqlx::query_as::<_, ParticipantRow>(
        r#"
        SELECT id, gathering_id, user_id, display_name, role, last_menu_activity_at,
               joined_at, created_at, updated_at
        FROM participants
        WHERE id = ?
        "#,
    )
    .bind(participant_id.to_string())
    .fetch_optional(pool)
    .await?
    .ok_or(AppError::NotFound)?;

    row.try_into()
}

pub(super) async fn get_menu_item_by_id(pool: &DbPool, menu_item_id: Uuid) -> AppResult<MenuItem> {
    let row = sqlx::query_as::<_, MenuItemRow>(
        r#"
        SELECT id, gathering_id, created_by, updated_by, name, category, quantity,
               unit, owner_name, reference_url, note, status, revision, created_at, updated_at
        FROM menu_items
        WHERE id = ?
        "#,
    )
    .bind(menu_item_id.to_string())
    .fetch_optional(pool)
    .await?
    .ok_or(AppError::NotFound)?;

    row.try_into()
}

pub(super) async fn ensure_participant_in_gathering(
    pool: &DbPool,
    gathering_id: Uuid,
    participant_id: Uuid,
) -> AppResult<()> {
    let exists: (i64,) = sqlx::query_as(
        r#"
        SELECT COUNT(*)
        FROM participants
        WHERE id = ? AND gathering_id = ?
        "#,
    )
    .bind(participant_id.to_string())
    .bind(gathering_id.to_string())
    .fetch_one(pool)
    .await?;

    if exists.0 == 0 {
        return Err(AppError::Forbidden);
    }

    Ok(())
}

pub(super) async fn ensure_gathering_editable(pool: &DbPool, gathering_id: Uuid) -> AppResult<()> {
    let gathering =
        sync_expired_gathering(pool, get_gathering_by_id(pool, gathering_id).await?).await?;

    if gathering.status != "active" || gathering.is_locked {
        return Err(AppError::Forbidden);
    }

    Ok(())
}

pub(super) async fn ensure_user_can_manage(
    pool: &DbPool,
    gathering_id: Uuid,
    user: &User,
) -> AppResult<()> {
    if user.role == "admin" {
        return Ok(());
    }

    let role: Option<(String,)> = sqlx::query_as(
        r#"
        SELECT role
        FROM participants
        WHERE gathering_id = ? AND user_id = ?
        "#,
    )
    .bind(gathering_id.to_string())
    .bind(user.id.to_string())
    .fetch_optional(pool)
    .await?;

    match role {
        Some((role,)) if role == "host" => Ok(()),
        _ => Err(AppError::Forbidden),
    }
}

pub(super) fn ensure_user_is_admin(user: &User) -> AppResult<()> {
    if user.role == "admin" {
        Ok(())
    } else {
        Err(AppError::Forbidden)
    }
}

pub(super) async fn sync_expired_gathering(
    pool: &DbPool,
    gathering: Gathering,
) -> AppResult<Gathering> {
    if gathering.status == "active" && gathering.expires_at <= Utc::now() {
        let now = Utc::now();

        sqlx::query(
            r#"
            UPDATE gatherings
            SET status = 'locked', is_locked = 1, locked_at = ?, updated_at = ?
            WHERE id = ? AND status = 'active'
            "#,
        )
        .bind(now)
        .bind(now)
        .bind(gathering.id.to_string())
        .execute(pool)
        .await?;

        return get_gathering_by_id(pool, gathering.id).await;
    }

    Ok(gathering)
}

pub(super) async fn insert_activity_log(
    pool: &DbPool,
    gathering_id: Uuid,
    actor_id: Option<Uuid>,
    action: &str,
    target_type: &str,
    target_id: Option<Uuid>,
    detail: Option<String>,
) -> AppResult<()> {
    sqlx::query(
        r#"
        INSERT INTO activity_logs (
            id, gathering_id, actor_id, action, target_type, target_id, detail, created_at
        )
        VALUES (?, ?, ?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(Uuid::new_v4().to_string())
    .bind(gathering_id.to_string())
    .bind(actor_id.map(|id| id.to_string()))
    .bind(action)
    .bind(target_type)
    .bind(target_id.map(|id| id.to_string()))
    .bind(detail)
    .bind(Utc::now())
    .execute(pool)
    .await?;

    Ok(())
}

pub(super) async fn touch_participant_menu_activity(
    pool: &DbPool,
    participant_id: Uuid,
) -> AppResult<()> {
    sqlx::query(
        r#"
        UPDATE participants
        SET last_menu_activity_at = ?, updated_at = ?
        WHERE id = ?
        "#,
    )
    .bind(Utc::now())
    .bind(Utc::now())
    .bind(participant_id.to_string())
    .execute(pool)
    .await?;

    Ok(())
}

pub(super) fn resource_dir() -> String {
    std::env::var("LETSORDER_RESOURCE_DIR").unwrap_or_else(|_| {
        if Path::new("backend/resources").exists() {
            "backend/resources".to_string()
        } else {
            "resources".to_string()
        }
    })
}

pub(super) async fn unique_invite_code(pool: &DbPool, title: &str) -> AppResult<String> {
    let base = slugify_title(title);
    let mut candidate = base.clone();
    let mut suffix = 2;

    loop {
        let exists: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(*)
            FROM gatherings
            WHERE invite_code = ?
            "#,
        )
        .bind(&candidate)
        .fetch_one(pool)
        .await?;

        if exists.0 == 0 {
            return Ok(candidate);
        }

        candidate = format!("{base}-{suffix}");
        suffix += 1;
    }
}

fn slugify_title(title: &str) -> String {
    if !title.is_ascii() {
        return Uuid::new_v4().simple().to_string()[..8].to_string();
    }

    let mut slug = String::new();
    let mut last_was_dash = false;

    for character in title.trim().to_lowercase().chars() {
        if character.is_ascii_alphanumeric() {
            slug.push(character);
            last_was_dash = false;
        } else if !last_was_dash && !slug.is_empty() {
            slug.push('-');
            last_was_dash = true;
        }
    }

    while slug.ends_with('-') {
        slug.pop();
    }

    if slug.is_empty() {
        Uuid::new_v4().simple().to_string()[..8].to_string()
    } else {
        slug
    }
}

pub(super) fn parse_uuid(value: &str) -> AppResult<Uuid> {
    Uuid::parse_str(value).map_err(|error| AppError::Validation(format!("invalid uuid: {error}")))
}
