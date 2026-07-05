use std::path::Path;

use axum::extract::Multipart;
use chrono::Utc;
use tokio::io::AsyncWriteExt;
use uuid::Uuid;

use crate::{
    db::DbPool,
    errors::{AppError, AppResult},
    models::{
        CreateGatheringRequest, CreateGatheringResponse, CreateMenuItemRequest, Gathering,
        GatheringListItem, JoinGatheringRequest, JoinGatheringResponse, MenuItem, Participant,
        Photo, UpdateGatheringRequest, UpdateMenuItemRequest,
    },
};

pub async fn create_gathering(
    pool: &DbPool,
    payload: CreateGatheringRequest,
) -> AppResult<CreateGatheringResponse> {
    if payload.title.trim().is_empty() {
        return Err(AppError::Validation("title is required".to_string()));
    }

    if payload.host_name.trim().is_empty() {
        return Err(AppError::Validation("host_name is required".to_string()));
    }

    let now = Utc::now();
    let gathering_id = Uuid::new_v4();
    let host_id = Uuid::new_v4();
    let invite_code = unique_invite_code(pool, &payload.title).await?;
    let access_token = Uuid::new_v4().to_string();

    sqlx::query(
        r#"
        INSERT INTO gatherings (
            id, title, description, invite_code, status, is_locked, starts_at, expires_at, created_at, updated_at
        )
        VALUES (?, ?, ?, ?, 'active', 0, ?, ?, ?, ?)
        "#,
    )
    .bind(gathering_id)
    .bind(payload.title.trim())
    .bind(payload.description.as_deref().map(str::trim))
    .bind(&invite_code)
    .bind(payload.starts_at)
    .bind(payload.expires_at)
    .bind(now)
    .bind(now)
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        INSERT INTO participants (
            id, gathering_id, display_name, role, access_token_hash, joined_at, created_at, updated_at
        )
        VALUES (?, ?, ?, 'host', ?, ?, ?, ?)
        "#,
    )
    .bind(host_id)
    .bind(gathering_id)
    .bind(payload.host_name.trim())
    .bind(&access_token)
    .bind(now)
    .bind(now)
    .bind(now)
    .execute(pool)
    .await?;

    insert_activity_log(
        pool,
        gathering_id,
        Some(host_id),
        "gathering_created",
        "gathering",
        Some(gathering_id),
        None,
    )
    .await?;

    let gathering = get_gathering_by_id(pool, gathering_id).await?;
    let host = get_participant_by_id(pool, host_id).await?;

    Ok(CreateGatheringResponse {
        gathering,
        host,
        access_token,
    })
}

pub async fn get_gathering_by_invite_code(
    pool: &DbPool,
    invite_code: &str,
) -> AppResult<Gathering> {
    let gathering = sqlx::query_as::<_, Gathering>(
        r#"
        SELECT id, title, description, invite_code, status, starts_at, expires_at,
               is_locked, locked_at, archived_at, created_at, updated_at
        FROM gatherings
        WHERE invite_code = ?
        "#,
    )
    .bind(invite_code)
    .fetch_optional(pool)
    .await?
    .ok_or(AppError::NotFound)?;

    sync_expired_gathering(pool, gathering).await
}

pub async fn list_gatherings(pool: &DbPool) -> AppResult<Vec<GatheringListItem>> {
    let rows = sqlx::query_as::<_, GatheringListItem>(
        r#"
        SELECT
            g.id,
            g.title,
            g.description,
            g.invite_code,
            g.status,
            g.is_locked,
            g.expires_at,
            COUNT(DISTINCT m.id) AS item_count,
            COUNT(DISTINCT CASE WHEN m.status = 'prepared' THEN m.id END) AS prepared_count,
            COUNT(DISTINCT p.id) AS participant_count,
            g.created_at,
            g.updated_at
        FROM gatherings g
        LEFT JOIN menu_items m ON m.gathering_id = g.id
        LEFT JOIN participants p ON p.gathering_id = g.id
        WHERE g.status != 'archived'
        GROUP BY g.id
        ORDER BY g.created_at DESC
        "#,
    )
    .fetch_all(pool)
    .await?;

    Ok(rows)
}

pub async fn archive_gathering(
    pool: &DbPool,
    gathering_id: Uuid,
    actor_name: Option<String>,
) -> AppResult<Gathering> {
    ensure_actor_can_manage(pool, gathering_id, actor_name.as_deref()).await?;

    let now = Utc::now();

    let result = sqlx::query(
        r#"
        UPDATE gatherings
        SET status = 'archived',
            archived_at = COALESCE(archived_at, ?),
            updated_at = ?
        WHERE id = ?
        "#,
    )
    .bind(now)
    .bind(now)
    .bind(gathering_id)
    .execute(pool)
    .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound);
    }

    insert_activity_log(
        pool,
        gathering_id,
        None,
        "gathering_archived",
        "gathering",
        Some(gathering_id),
        None,
    )
    .await?;

    get_gathering_by_id(pool, gathering_id).await
}

pub async fn update_gathering_deadline(
    pool: &DbPool,
    gathering_id: Uuid,
    payload: UpdateGatheringRequest,
    actor_name: Option<String>,
) -> AppResult<Gathering> {
    ensure_actor_can_manage(pool, gathering_id, actor_name.as_deref()).await?;
    let current = get_gathering_by_id(pool, gathering_id).await?;

    let now = Utc::now();
    let should_lock = payload.expires_at <= now;

    sqlx::query(
        r#"
        UPDATE gatherings
        SET expires_at = ?,
            status = CASE WHEN ? THEN 'locked' ELSE 'active' END,
            is_locked = CASE WHEN ? THEN 1 ELSE 0 END,
            locked_at = CASE WHEN ? THEN COALESCE(locked_at, ?) ELSE NULL END,
            updated_at = ?
        WHERE id = ?
        "#,
    )
    .bind(payload.expires_at)
    .bind(should_lock)
    .bind(should_lock)
    .bind(should_lock)
    .bind(now)
    .bind(now)
    .bind(gathering_id)
    .execute(pool)
    .await?;

    if current.is_locked && !should_lock {
        insert_activity_log(
            pool,
            gathering_id,
            None,
            "menu_reopened",
            "gathering",
            Some(gathering_id),
            Some(
                serde_json::json!({
                    "before": {
                        "expires_at": current.expires_at,
                        "status": current.status,
                        "is_locked": current.is_locked,
                    },
                    "after": {
                        "expires_at": payload.expires_at,
                        "status": "active",
                        "is_locked": false,
                    }
                })
                .to_string(),
            ),
        )
        .await?;
    }

    insert_activity_log(
        pool,
        gathering_id,
        None,
        "gathering_deadline_updated",
        "gathering",
        Some(gathering_id),
        Some(
            serde_json::json!({
                "before": {
                    "expires_at": current.expires_at,
                    "status": current.status,
                    "is_locked": current.is_locked,
                },
                "after": {
                    "expires_at": payload.expires_at,
                    "status": if should_lock { "locked" } else { "active" },
                    "is_locked": should_lock,
                }
            })
            .to_string(),
        ),
    )
    .await?;

    get_gathering_by_id(pool, gathering_id).await
}

pub async fn join_gathering(
    pool: &DbPool,
    gathering_id: Uuid,
    payload: JoinGatheringRequest,
) -> AppResult<JoinGatheringResponse> {
    if payload.display_name.trim().is_empty() {
        return Err(AppError::Validation("display_name is required".to_string()));
    }

    get_gathering_by_id(pool, gathering_id).await?;

    let now = Utc::now();
    let display_name = payload.display_name.trim();
    let existing: Option<(Uuid,)> = sqlx::query_as(
        r#"
        SELECT id
        FROM participants
        WHERE gathering_id = ? AND display_name = ?
        "#,
    )
    .bind(gathering_id)
    .bind(display_name)
    .fetch_optional(pool)
    .await?;

    if let Some((participant_id,)) = existing {
        let access_token = Uuid::new_v4().to_string();

        sqlx::query(
            r#"
            UPDATE participants
            SET access_token_hash = ?, updated_at = ?
            WHERE id = ?
            "#,
        )
        .bind(&access_token)
        .bind(now)
        .bind(participant_id)
        .execute(pool)
        .await?;

        return Ok(JoinGatheringResponse {
            participant: get_participant_by_id(pool, participant_id).await?,
            access_token,
        });
    }

    let participant_id = Uuid::new_v4();
    let access_token = Uuid::new_v4().to_string();

    sqlx::query(
        r#"
        INSERT INTO participants (
            id, gathering_id, display_name, role, access_token_hash, joined_at, created_at, updated_at
        )
        VALUES (?, ?, ?, 'participant', ?, ?, ?, ?)
        "#,
    )
    .bind(participant_id)
    .bind(gathering_id)
    .bind(payload.display_name.trim())
    .bind(&access_token)
    .bind(now)
    .bind(now)
    .bind(now)
    .execute(pool)
    .await?;

    insert_activity_log(
        pool,
        gathering_id,
        Some(participant_id),
        "participant_joined",
        "participant",
        Some(participant_id),
        None,
    )
    .await?;

    Ok(JoinGatheringResponse {
        participant: get_participant_by_id(pool, participant_id).await?,
        access_token,
    })
}

pub async fn list_participants(pool: &DbPool, gathering_id: Uuid) -> AppResult<Vec<Participant>> {
    get_gathering_by_id(pool, gathering_id).await?;

    let participants = sqlx::query_as::<_, Participant>(
        r#"
        SELECT id, gathering_id, display_name, role, last_menu_activity_at,
               joined_at, created_at, updated_at
        FROM participants
        WHERE gathering_id = ?
        ORDER BY COALESCE(last_menu_activity_at, joined_at) DESC
        "#,
    )
    .bind(gathering_id)
    .fetch_all(pool)
    .await?;

    Ok(participants)
}

pub async fn list_menu_items(pool: &DbPool, gathering_id: Uuid) -> AppResult<Vec<MenuItem>> {
    get_gathering_by_id(pool, gathering_id).await?;

    let items = sqlx::query_as::<_, MenuItem>(
        r#"
        SELECT id, gathering_id, created_by, updated_by, name, category, quantity,
               unit, owner_name, reference_url, note, status, created_at, updated_at
        FROM menu_items
        WHERE gathering_id = ?
        ORDER BY created_at ASC
        "#,
    )
    .bind(gathering_id)
    .fetch_all(pool)
    .await?;

    Ok(items)
}

pub async fn create_menu_item(
    pool: &DbPool,
    gathering_id: Uuid,
    payload: CreateMenuItemRequest,
) -> AppResult<MenuItem> {
    ensure_gathering_editable(pool, gathering_id).await?;
    ensure_participant_in_gathering(pool, gathering_id, payload.created_by).await?;
    validate_menu_item_name(&payload.name)?;

    let quantity = payload.quantity.unwrap_or(1);
    validate_quantity(quantity)?;

    let status = payload.status.unwrap_or_else(|| "planned".to_string());
    validate_menu_status(&status)?;

    let now = Utc::now();
    let menu_item_id = Uuid::new_v4();

    sqlx::query(
        r#"
        INSERT INTO menu_items (
            id, gathering_id, created_by, name, category, quantity, unit,
            owner_name, reference_url, note, status, created_at, updated_at
        )
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(menu_item_id)
    .bind(gathering_id)
    .bind(payload.created_by)
    .bind(payload.name.trim())
    .bind(payload.category.as_deref().map(str::trim))
    .bind(quantity)
    .bind(payload.unit.as_deref().map(str::trim))
    .bind(payload.owner_name.as_deref().map(str::trim))
    .bind(normalize_reference_url(payload.reference_url.as_deref()).as_deref())
    .bind(payload.note.as_deref().map(str::trim))
    .bind(status)
    .bind(now)
    .bind(now)
    .execute(pool)
    .await?;

    insert_activity_log(
        pool,
        gathering_id,
        Some(payload.created_by),
        "menu_item_created",
        "menu_item",
        Some(menu_item_id),
        None,
    )
    .await?;
    touch_participant_menu_activity(pool, payload.created_by).await?;

    get_menu_item_by_id(pool, menu_item_id).await
}

pub async fn update_menu_item(
    pool: &DbPool,
    menu_item_id: Uuid,
    payload: UpdateMenuItemRequest,
) -> AppResult<MenuItem> {
    let current = get_menu_item_by_id(pool, menu_item_id).await?;
    ensure_gathering_editable(pool, current.gathering_id).await?;
    ensure_participant_in_gathering(pool, current.gathering_id, payload.updated_by).await?;

    let before = current.clone();

    let name = payload.name.unwrap_or_else(|| current.name.clone());
    validate_menu_item_name(&name)?;

    let quantity = payload.quantity.unwrap_or(current.quantity);
    validate_quantity(quantity)?;

    let status = payload.status.unwrap_or_else(|| current.status.clone());
    validate_menu_status(&status)?;
    let category = payload
        .category
        .or_else(|| current.category.clone())
        .map(|value| value.trim().to_string());
    let unit = payload
        .unit
        .or_else(|| current.unit.clone())
        .map(|value| value.trim().to_string());
    let owner_name = payload
        .owner_name
        .or_else(|| current.owner_name.clone())
        .map(|value| value.trim().to_string());
    let reference_url = payload
        .reference_url
        .map(|value| normalize_reference_url(Some(&value)))
        .unwrap_or_else(|| current.reference_url.clone());
    let note = payload
        .note
        .or_else(|| current.note.clone())
        .map(|value| value.trim().to_string());

    let now = Utc::now();

    sqlx::query(
        r#"
        UPDATE menu_items
        SET updated_by = ?,
            name = ?,
            category = ?,
            quantity = ?,
            unit = ?,
            owner_name = ?,
            reference_url = ?,
            note = ?,
            status = ?,
            updated_at = ?
        WHERE id = ?
        "#,
    )
    .bind(payload.updated_by)
    .bind(name.trim())
    .bind(category.as_deref())
    .bind(quantity)
    .bind(unit.as_deref())
    .bind(owner_name.as_deref())
    .bind(reference_url.as_deref())
    .bind(note.as_deref())
    .bind(&status)
    .bind(now)
    .bind(menu_item_id)
    .execute(pool)
    .await?;

    insert_menu_item_change_logs(
        pool,
        current.gathering_id,
        payload.updated_by,
        menu_item_id,
        before,
        MenuItemChangeAfter {
            name: name.trim().to_string(),
            category,
            quantity,
            unit,
            owner_name,
            reference_url,
            note,
            status,
        },
    )
    .await?;
    touch_participant_menu_activity(pool, payload.updated_by).await?;

    get_menu_item_by_id(pool, menu_item_id).await
}

pub async fn lock_gathering(
    pool: &DbPool,
    gathering_id: Uuid,
    actor_name: Option<String>,
) -> AppResult<Gathering> {
    ensure_actor_can_manage(pool, gathering_id, actor_name.as_deref()).await?;
    get_gathering_by_id(pool, gathering_id).await?;

    let now = Utc::now();

    sqlx::query(
        r#"
        UPDATE gatherings
        SET status = 'locked',
            is_locked = 1,
            expires_at = ?,
            locked_at = ?,
            updated_at = ?
        WHERE id = ?
        "#,
    )
    .bind(now)
    .bind(now)
    .bind(now)
    .bind(gathering_id)
    .execute(pool)
    .await?;

    insert_activity_log(
        pool,
        gathering_id,
        None,
        "gathering_locked",
        "gathering",
        Some(gathering_id),
        None,
    )
    .await?;

    get_gathering_by_id(pool, gathering_id).await
}

pub async fn list_activity_logs(
    pool: &DbPool,
    gathering_id: Uuid,
) -> AppResult<Vec<crate::models::ActivityLog>> {
    get_gathering_by_id(pool, gathering_id).await?;

    let logs = sqlx::query_as::<_, crate::models::ActivityLog>(
        r#"
        SELECT
            a.id,
            a.gathering_id,
            a.actor_id,
            p.display_name AS actor_name,
            a.action,
            a.target_type,
            a.target_id,
            a.detail,
            a.created_at
        FROM activity_logs a
        LEFT JOIN participants p ON p.id = a.actor_id
        WHERE a.gathering_id = ?
        ORDER BY a.created_at DESC
        "#,
    )
    .bind(gathering_id)
    .fetch_all(pool)
    .await?;

    Ok(logs)
}

pub async fn list_photos(pool: &DbPool, gathering_id: Uuid) -> AppResult<Vec<Photo>> {
    get_gathering_by_id(pool, gathering_id).await?;

    let photos = sqlx::query_as::<_, Photo>(
        r#"
        SELECT id, gathering_id, uploaded_by, file_url, thumbnail_url, caption,
               taken_at, created_at, updated_at
        FROM photos
        WHERE gathering_id = ?
        ORDER BY created_at DESC
        "#,
    )
    .bind(gathering_id)
    .fetch_all(pool)
    .await?;

    Ok(photos)
}

pub async fn upload_photo(
    pool: &DbPool,
    gathering_id: Uuid,
    actor_name: Option<String>,
    mut multipart: Multipart,
) -> AppResult<Photo> {
    get_gathering_by_id(pool, gathering_id).await?;

    let actor_name = actor_name
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or(AppError::Forbidden)?;
    let uploaded_by = get_or_create_participant_by_name(pool, gathering_id, actor_name).await?;
    let mut caption: Option<String> = None;
    let mut file_name: Option<String> = None;
    let mut file_bytes: Option<Vec<u8>> = None;

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|error| AppError::Validation(format!("invalid multipart payload: {error}")))?
    {
        let field_name = field.name().unwrap_or_default().to_string();

        match field_name.as_str() {
            "caption" => {
                caption = Some(field.text().await.map_err(|error| {
                    AppError::Validation(format!("invalid caption field: {error}"))
                })?);
            }
            "file" => {
                file_name = field.file_name().map(ToOwned::to_owned);
                file_bytes = Some(
                    field
                        .bytes()
                        .await
                        .map_err(|error| {
                            AppError::Validation(format!("invalid file field: {error}"))
                        })?
                        .to_vec(),
                );
            }
            _ => {}
        }
    }

    let file_bytes = file_bytes.ok_or_else(|| AppError::Validation("file is required".into()))?;
    let extension = file_name
        .as_deref()
        .and_then(|name| Path::new(name).extension())
        .and_then(|extension| extension.to_str())
        .map(str::to_lowercase)
        .filter(|extension| matches!(extension.as_str(), "jpg" | "jpeg" | "png" | "webp" | "gif"))
        .unwrap_or_else(|| "jpg".to_string());
    let now = Utc::now();
    let photo_id = Uuid::new_v4();
    let stored_file_name = format!("{}.{}", photo_id.simple(), extension);
    let resource_dir = resource_dir();
    let upload_dir = Path::new(&resource_dir).join("uploads");
    tokio::fs::create_dir_all(&upload_dir)
        .await
        .map_err(|error| AppError::Validation(format!("could not create upload dir: {error}")))?;

    let file_path = upload_dir.join(&stored_file_name);
    let mut file = tokio::fs::File::create(&file_path)
        .await
        .map_err(|error| AppError::Validation(format!("could not create upload file: {error}")))?;
    file.write_all(&file_bytes)
        .await
        .map_err(|error| AppError::Validation(format!("could not write upload file: {error}")))?;

    let file_url = format!("/resources/uploads/{stored_file_name}");

    sqlx::query(
        r#"
        INSERT INTO photos (
            id, gathering_id, uploaded_by, file_url, thumbnail_url, caption,
            created_at, updated_at
        )
        VALUES (?, ?, ?, ?, NULL, ?, ?, ?)
        "#,
    )
    .bind(photo_id)
    .bind(gathering_id)
    .bind(uploaded_by)
    .bind(&file_url)
    .bind(caption.as_deref().map(str::trim))
    .bind(now)
    .bind(now)
    .execute(pool)
    .await?;

    insert_activity_log(
        pool,
        gathering_id,
        Some(uploaded_by),
        "photo_uploaded",
        "photo",
        Some(photo_id),
        Some(serde_json::json!({ "file_url": file_url }).to_string()),
    )
    .await?;

    get_photo_by_id(pool, photo_id).await
}

async fn get_gathering_by_id(pool: &DbPool, gathering_id: Uuid) -> AppResult<Gathering> {
    sqlx::query_as::<_, Gathering>(
        r#"
        SELECT id, title, description, invite_code, status, starts_at, expires_at,
               is_locked, locked_at, archived_at, created_at, updated_at
        FROM gatherings
        WHERE id = ?
        "#,
    )
    .bind(gathering_id)
    .fetch_optional(pool)
    .await?
    .ok_or(AppError::NotFound)
}

async fn get_photo_by_id(pool: &DbPool, photo_id: Uuid) -> AppResult<Photo> {
    sqlx::query_as::<_, Photo>(
        r#"
        SELECT id, gathering_id, uploaded_by, file_url, thumbnail_url, caption,
               taken_at, created_at, updated_at
        FROM photos
        WHERE id = ?
        "#,
    )
    .bind(photo_id)
    .fetch_optional(pool)
    .await?
    .ok_or(AppError::NotFound)
}

async fn get_participant_by_id(pool: &DbPool, participant_id: Uuid) -> AppResult<Participant> {
    sqlx::query_as::<_, Participant>(
        r#"
        SELECT id, gathering_id, display_name, role, last_menu_activity_at,
               joined_at, created_at, updated_at
        FROM participants
        WHERE id = ?
        "#,
    )
    .bind(participant_id)
    .fetch_optional(pool)
    .await?
    .ok_or(AppError::NotFound)
}

async fn get_or_create_participant_by_name(
    pool: &DbPool,
    gathering_id: Uuid,
    display_name: &str,
) -> AppResult<Uuid> {
    if let Some((participant_id,)) = sqlx::query_as::<_, (Uuid,)>(
        r#"
        SELECT id
        FROM participants
        WHERE gathering_id = ? AND display_name = ?
        "#,
    )
    .bind(gathering_id)
    .bind(display_name)
    .fetch_optional(pool)
    .await?
    {
        return Ok(participant_id);
    }

    let now = Utc::now();
    let participant_id = Uuid::new_v4();
    let access_token = Uuid::new_v4().to_string();

    sqlx::query(
        r#"
        INSERT INTO participants (
            id, gathering_id, display_name, role, access_token_hash, joined_at, created_at, updated_at
        )
        VALUES (?, ?, ?, 'participant', ?, ?, ?, ?)
        "#,
    )
    .bind(participant_id)
    .bind(gathering_id)
    .bind(display_name)
    .bind(&access_token)
    .bind(now)
    .bind(now)
    .bind(now)
    .execute(pool)
    .await?;

    insert_activity_log(
        pool,
        gathering_id,
        Some(participant_id),
        "participant_joined",
        "participant",
        Some(participant_id),
        None,
    )
    .await?;

    Ok(participant_id)
}

async fn get_menu_item_by_id(pool: &DbPool, menu_item_id: Uuid) -> AppResult<MenuItem> {
    sqlx::query_as::<_, MenuItem>(
        r#"
        SELECT id, gathering_id, created_by, updated_by, name, category, quantity,
               unit, owner_name, reference_url, note, status, created_at, updated_at
        FROM menu_items
        WHERE id = ?
        "#,
    )
    .bind(menu_item_id)
    .fetch_optional(pool)
    .await?
    .ok_or(AppError::NotFound)
}

async fn ensure_participant_in_gathering(
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
    .bind(participant_id)
    .bind(gathering_id)
    .fetch_one(pool)
    .await?;

    if exists.0 == 0 {
        return Err(AppError::Forbidden);
    }

    Ok(())
}

async fn ensure_gathering_editable(pool: &DbPool, gathering_id: Uuid) -> AppResult<()> {
    let gathering =
        sync_expired_gathering(pool, get_gathering_by_id(pool, gathering_id).await?).await?;

    if gathering.status != "active" || gathering.is_locked {
        return Err(AppError::Forbidden);
    }

    Ok(())
}

async fn ensure_actor_can_manage(
    pool: &DbPool,
    gathering_id: Uuid,
    actor_name: Option<&str>,
) -> AppResult<()> {
    let actor_name = actor_name.ok_or(AppError::Forbidden)?;

    if actor_name == "admin" {
        return Ok(());
    }

    let role: Option<(String,)> = sqlx::query_as(
        r#"
        SELECT role
        FROM participants
        WHERE gathering_id = ? AND display_name = ?
        "#,
    )
    .bind(gathering_id)
    .bind(actor_name)
    .fetch_optional(pool)
    .await?;

    match role {
        Some((role,)) if role == "host" => Ok(()),
        _ => Err(AppError::Forbidden),
    }
}

async fn sync_expired_gathering(pool: &DbPool, gathering: Gathering) -> AppResult<Gathering> {
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
        .bind(gathering.id)
        .execute(pool)
        .await?;

        return get_gathering_by_id(pool, gathering.id).await;
    }

    Ok(gathering)
}

async fn insert_activity_log(
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
    .bind(Uuid::new_v4())
    .bind(gathering_id)
    .bind(actor_id)
    .bind(action)
    .bind(target_type)
    .bind(target_id)
    .bind(detail)
    .bind(Utc::now())
    .execute(pool)
    .await?;

    Ok(())
}

fn resource_dir() -> String {
    std::env::var("LETSORDER_RESOURCE_DIR").unwrap_or_else(|_| {
        if Path::new("backend/resources").exists() {
            "backend/resources".to_string()
        } else {
            "resources".to_string()
        }
    })
}

struct MenuItemChangeAfter {
    name: String,
    category: Option<String>,
    quantity: i64,
    unit: Option<String>,
    owner_name: Option<String>,
    reference_url: Option<String>,
    note: Option<String>,
    status: String,
}

async fn insert_menu_item_change_logs(
    pool: &DbPool,
    gathering_id: Uuid,
    actor_id: Uuid,
    menu_item_id: Uuid,
    before: MenuItem,
    after: MenuItemChangeAfter,
) -> AppResult<()> {
    if before.name != after.name {
        insert_field_change_log(
            pool,
            gathering_id,
            actor_id,
            menu_item_id,
            "menu_item_name_changed",
            "name",
            serde_json::json!(before.name),
            serde_json::json!(after.name),
        )
        .await?;
    }

    if before.category != after.category {
        insert_field_change_log(
            pool,
            gathering_id,
            actor_id,
            menu_item_id,
            "menu_item_category_changed",
            "category",
            serde_json::json!(before.category),
            serde_json::json!(after.category),
        )
        .await?;
    }

    if before.quantity != after.quantity {
        insert_field_change_log(
            pool,
            gathering_id,
            actor_id,
            menu_item_id,
            "menu_item_quantity_changed",
            "quantity",
            serde_json::json!(before.quantity),
            serde_json::json!(after.quantity),
        )
        .await?;
    }

    if before.unit != after.unit {
        insert_field_change_log(
            pool,
            gathering_id,
            actor_id,
            menu_item_id,
            "menu_item_unit_changed",
            "unit",
            serde_json::json!(before.unit),
            serde_json::json!(after.unit),
        )
        .await?;
    }

    if before.owner_name != after.owner_name {
        insert_field_change_log(
            pool,
            gathering_id,
            actor_id,
            menu_item_id,
            "menu_item_owner_changed",
            "owner_name",
            serde_json::json!(before.owner_name),
            serde_json::json!(after.owner_name),
        )
        .await?;
    }

    if before.reference_url != after.reference_url {
        insert_field_change_log(
            pool,
            gathering_id,
            actor_id,
            menu_item_id,
            "menu_item_reference_url_changed",
            "reference_url",
            serde_json::json!(before.reference_url),
            serde_json::json!(after.reference_url),
        )
        .await?;
    }

    if before.note != after.note {
        insert_field_change_log(
            pool,
            gathering_id,
            actor_id,
            menu_item_id,
            "menu_item_note_changed",
            "note",
            serde_json::json!(before.note),
            serde_json::json!(after.note),
        )
        .await?;
    }

    if before.status != after.status {
        let action = if after.status == "cancelled" {
            "menu_item_cancelled"
        } else {
            "menu_item_status_changed"
        };

        insert_field_change_log(
            pool,
            gathering_id,
            actor_id,
            menu_item_id,
            action,
            "status",
            serde_json::json!(before.status),
            serde_json::json!(after.status),
        )
        .await?;
    }

    Ok(())
}

async fn insert_field_change_log(
    pool: &DbPool,
    gathering_id: Uuid,
    actor_id: Uuid,
    menu_item_id: Uuid,
    action: &str,
    field: &str,
    before: serde_json::Value,
    after: serde_json::Value,
) -> AppResult<()> {
    insert_activity_log(
        pool,
        gathering_id,
        Some(actor_id),
        action,
        "menu_item",
        Some(menu_item_id),
        Some(
            serde_json::json!({
                "field": field,
                "before": before,
                "after": after,
            })
            .to_string(),
        ),
    )
    .await
}

async fn touch_participant_menu_activity(pool: &DbPool, participant_id: Uuid) -> AppResult<()> {
    sqlx::query(
        r#"
        UPDATE participants
        SET last_menu_activity_at = ?, updated_at = ?
        WHERE id = ?
        "#,
    )
    .bind(Utc::now())
    .bind(Utc::now())
    .bind(participant_id)
    .execute(pool)
    .await?;

    Ok(())
}

fn validate_menu_item_name(name: &str) -> AppResult<()> {
    if name.trim().is_empty() {
        return Err(AppError::Validation("name is required".to_string()));
    }

    Ok(())
}

fn validate_quantity(quantity: i64) -> AppResult<()> {
    if quantity <= 0 {
        return Err(AppError::Validation(
            "quantity must be greater than 0".to_string(),
        ));
    }

    Ok(())
}

fn validate_menu_status(status: &str) -> AppResult<()> {
    match status {
        "planned" | "prepared" | "cancelled" => Ok(()),
        _ => Err(AppError::Validation(
            "status must be planned, prepared, or cancelled".to_string(),
        )),
    }
}

fn normalize_reference_url(value: Option<&str>) -> Option<String> {
    let value = value?.trim();
    if value.is_empty() {
        return None;
    }

    extract_first_url(value).or_else(|| Some(value.to_string()))
}

fn extract_first_url(value: &str) -> Option<String> {
    value.split_whitespace().find_map(|token| {
        let trimmed = token.trim_matches(|character: char| {
            matches!(
                character,
                '，' | ',' | '。' | '.' | '！' | '!' | '？' | '?' | '）' | ')' | '】' | ']'
            )
        });

        if trimmed.starts_with("http://") || trimmed.starts_with("https://") {
            Some(trimmed.to_string())
        } else {
            None
        }
    })
}

async fn unique_invite_code(pool: &DbPool, title: &str) -> AppResult<String> {
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
