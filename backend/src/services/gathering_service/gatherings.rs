use chrono::Utc;
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::{
    db::DbPool,
    errors::{AppError, AppResult},
    models::{
        CreateGatheringRequest, CreateGatheringResponse, Gathering, GatheringListItem,
        GatheringListItemRow, GatheringRow, Participant, ParticipantRow, UpdateGatheringRequest,
        User,
    },
};

use super::common::{
    ensure_user_can_manage, get_gathering_by_id, get_participant_by_id, insert_activity_log,
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
    .bind(gathering_id.to_string())
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
    .bind(host_id.to_string())
    .bind(gathering_id.to_string())
    .bind(payload.host_name.trim())
    .bind(hash_host_claim_token(&access_token))
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

pub async fn claim_host(
    pool: &DbPool,
    gathering_id: Uuid,
    user: &User,
    claim_token: &str,
) -> AppResult<Participant> {
    if user.role == "admin" || claim_token.trim().is_empty() {
        return Err(AppError::Forbidden);
    }

    let mut transaction = pool.begin().await?;
    let host_row: Option<(String,)> = sqlx::query_as(
        r#"
        SELECT id
        FROM participants
        WHERE gathering_id = ?
          AND role = 'host'
          AND user_id IS NULL
          AND access_token_hash = ?
        "#,
    )
    .bind(gathering_id.to_string())
    .bind(hash_host_claim_token(claim_token))
    .fetch_optional(&mut *transaction)
    .await?;
    let Some((host_id,)) = host_row else {
        return Err(AppError::Conflict(serde_json::json!({
            "error": "host claim token is invalid or already used"
        })));
    };

    let existing_participant: Option<(String,)> =
        sqlx::query_as("SELECT id FROM participants WHERE gathering_id = ? AND user_id = ?")
            .bind(gathering_id.to_string())
            .bind(user.id.to_string())
            .fetch_optional(&mut *transaction)
            .await?;

    if let Some((participant_id,)) = existing_participant {
        sqlx::query(
            "UPDATE participants SET role = 'participant', access_token_hash = '', updated_at = ? WHERE id = ?",
        )
            .bind(Utc::now())
            .bind(&host_id)
            .execute(&mut *transaction)
            .await?;
        sqlx::query(
            "UPDATE participants SET role = 'host', display_name = ?, updated_at = ? WHERE id = ?",
        )
        .bind(&user.display_name)
        .bind(Utc::now())
        .bind(&participant_id)
        .execute(&mut *transaction)
        .await?;
        transaction.commit().await?;
        return get_participant_by_id(
            pool,
            Uuid::parse_str(&participant_id).map_err(|error| {
                AppError::Validation(format!("invalid participant id: {error}"))
            })?,
        )
        .await;
    }

    let result = sqlx::query(
        r#"
        UPDATE participants
        SET user_id = ?, display_name = ?, updated_at = ?
        WHERE gathering_id = ?
          AND role = 'host'
          AND user_id IS NULL
          AND access_token_hash = ?
        "#,
    )
    .bind(user.id.to_string())
    .bind(&user.display_name)
    .bind(Utc::now())
    .bind(gathering_id.to_string())
    .bind(hash_host_claim_token(claim_token))
    .execute(&mut *transaction)
    .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::Conflict(serde_json::json!({
            "error": "host claim token is invalid or already used"
        })));
    }

    transaction.commit().await?;

    list_participants(pool, gathering_id)
        .await?
        .into_iter()
        .find(|participant| participant.user_id == Some(user.id))
        .ok_or(AppError::NotFound)
}

fn hash_host_claim_token(token: &str) -> String {
    let digest = Sha256::digest(token.as_bytes());
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}

pub async fn get_gathering_by_invite_code(
    pool: &DbPool,
    invite_code: &str,
) -> AppResult<Gathering> {
    let row = sqlx::query_as::<_, GatheringRow>(
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

    sync_expired_gathering(pool, row.try_into()?).await
}

pub async fn list_gatherings(pool: &DbPool) -> AppResult<Vec<GatheringListItem>> {
    let rows = sqlx::query_as::<_, GatheringListItemRow>(
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

    rows.into_iter().map(TryInto::try_into).collect()
}

pub async fn list_active_gatherings(pool: &DbPool) -> AppResult<Vec<GatheringListItem>> {
    let rows = sqlx::query_as::<_, GatheringListItemRow>(
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

    rows.into_iter().map(TryInto::try_into).collect()
}

pub async fn list_gatherings_for_user(
    pool: &DbPool,
    user_id: Uuid,
) -> AppResult<Vec<GatheringListItem>> {
    let rows = sqlx::query_as::<_, GatheringListItemRow>(
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
    .bind(user_id.to_string())
    .fetch_all(pool)
    .await?;

    rows.into_iter().map(TryInto::try_into).collect()
}

pub async fn archive_gathering(
    pool: &DbPool,
    gathering_id: Uuid,
    actor: &User,
) -> AppResult<Gathering> {
    ensure_user_can_manage(pool, gathering_id, actor).await?;

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
    .bind(gathering_id.to_string())
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
    actor: &User,
) -> AppResult<Gathering> {
    ensure_user_can_manage(pool, gathering_id, actor).await?;
    let current = get_gathering_by_id(pool, gathering_id).await?;
    if current.status == "archived" {
        return Err(AppError::Conflict(serde_json::json!({
            "error": "archived gatherings cannot be reopened"
        })));
    }

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
        WHERE id = ? AND status != 'archived'
        "#,
    )
    .bind(payload.expires_at)
    .bind(should_lock)
    .bind(should_lock)
    .bind(should_lock)
    .bind(now)
    .bind(now)
    .bind(gathering_id.to_string())
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

    let rows = sqlx::query_as::<_, ParticipantRow>(
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
    .bind(gathering_id.to_string())
    .fetch_all(pool)
    .await?;

    rows.into_iter().map(TryInto::try_into).collect()
}
