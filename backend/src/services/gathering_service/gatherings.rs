use chrono::Utc;
use uuid::Uuid;

use crate::{
    db::DbPool,
    errors::{AppError, AppResult},
    models::{
        CreateGatheringRequest, CreateGatheringResponse, Gathering, GatheringListItem, Participant,
        UpdateGatheringRequest,
    },
};

use super::common::{
    ensure_actor_can_manage, get_gathering_by_id, get_participant_by_id, insert_activity_log,
    sync_expired_gathering, unique_invite_code,
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

    if payload.host_name.trim() == "suite-admin" {
        return Err(AppError::Validation(
            "这是系统管理员账号名称，请使用系统管理员账号及密码登陆".to_string(),
        ));
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

pub async fn list_active_gatherings(pool: &DbPool) -> AppResult<Vec<GatheringListItem>> {
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
        WHERE g.status = 'active'
          AND g.is_locked = 0
          AND g.expires_at > ?
        GROUP BY g.id
        ORDER BY g.expires_at ASC, g.created_at DESC
        "#,
    )
    .bind(Utc::now())
    .fetch_all(pool)
    .await?;

    Ok(rows)
}

pub async fn list_gatherings_for_user(
    pool: &DbPool,
    user_id: Uuid,
) -> AppResult<Vec<GatheringListItem>> {
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
          AND EXISTS (
              SELECT 1
              FROM participants current_participant
              WHERE current_participant.gathering_id = g.id
                AND current_participant.user_id = ?
          )
        GROUP BY g.id
        ORDER BY g.created_at DESC
        "#,
    )
    .bind(user_id)
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

pub async fn list_participants(pool: &DbPool, gathering_id: Uuid) -> AppResult<Vec<Participant>> {
    get_gathering_by_id(pool, gathering_id).await?;

    let participants = sqlx::query_as::<_, Participant>(
        r#"
        SELECT p.id, p.gathering_id, p.user_id, p.display_name, p.role, p.last_menu_activity_at,
               p.joined_at, p.created_at, p.updated_at
        FROM participants p
        LEFT JOIN users u ON u.id = p.user_id
        WHERE p.gathering_id = ?
          AND p.display_name != 'suite-admin'
          AND COALESCE(u.role, '') != 'admin'
        ORDER BY COALESCE(last_menu_activity_at, joined_at) DESC
        "#,
    )
    .bind(gathering_id)
    .fetch_all(pool)
    .await?;

    Ok(participants)
}
