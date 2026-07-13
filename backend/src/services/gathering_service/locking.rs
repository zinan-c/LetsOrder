use chrono::Utc;
use uuid::Uuid;

use crate::{db::DbPool, errors::AppResult, models::Gathering};

use super::common::{ensure_actor_can_manage, get_gathering_by_id, insert_activity_log};

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
    .bind(gathering_id.to_string())
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

pub async fn lock_expired_gatherings(pool: &DbPool, limit: i64) -> AppResult<Vec<Gathering>> {
    let now = Utc::now();
    let candidates = sqlx::query_as::<_, (String,)>(
        r#"
        SELECT id
        FROM gatherings
        WHERE status = 'active'
          AND is_locked = 0
          AND expires_at <= ?
        ORDER BY expires_at ASC
        LIMIT ?
        "#,
    )
    .bind(now)
    .bind(limit)
    .fetch_all(pool)
    .await?;

    let mut locked_gatherings = Vec::new();

    for (gathering_id,) in candidates {
        let gathering_id = super::common::parse_uuid(&gathering_id)?;
        let result = sqlx::query(
            r#"
            UPDATE gatherings
            SET status = 'locked',
                is_locked = 1,
                locked_at = COALESCE(locked_at, ?),
                updated_at = ?
            WHERE id = ?
              AND status = 'active'
              AND is_locked = 0
            "#,
        )
        .bind(now)
        .bind(now)
        .bind(gathering_id.to_string())
        .execute(pool)
        .await?;

        if result.rows_affected() > 0 {
            insert_activity_log(
                pool,
                gathering_id,
                None,
                "gathering_auto_locked",
                "gathering",
                Some(gathering_id),
                Some(format!("auto locked at {now}")),
            )
            .await?;
            locked_gatherings.push(get_gathering_by_id(pool, gathering_id).await?);
        }
    }

    Ok(locked_gatherings)
}
