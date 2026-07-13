use uuid::Uuid;

use crate::{
    db::DbPool,
    errors::AppResult,
    models::{ActivityLog, ActivityLogRow},
};

use super::common::get_gathering_by_id;

pub async fn list_activity_logs(pool: &DbPool, gathering_id: Uuid) -> AppResult<Vec<ActivityLog>> {
    get_gathering_by_id(pool, gathering_id).await?;

    let rows = sqlx::query_as::<_, ActivityLogRow>(
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
        LEFT JOIN users u ON u.id = p.user_id
        WHERE a.gathering_id = ?
          AND NOT (
              a.action = 'participant_joined'
              AND (COALESCE(u.role, '') = 'admin' OR p.display_name = 'suite-admin')
          )
        ORDER BY a.created_at DESC
        "#,
    )
    .bind(gathering_id.to_string())
    .fetch_all(pool)
    .await?;

    rows.into_iter().map(TryInto::try_into).collect()
}
