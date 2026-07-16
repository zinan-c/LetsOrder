use chrono::Utc;
use uuid::Uuid;

use crate::{
    db::DbPool,
    errors::{AppError, AppResult},
    models::{MenuItemRatingSummary, MenuItemRatingSummaryRow, RateMenuItemRequest},
};

use super::common::{ensure_participant_in_gathering, get_gathering_by_id, insert_activity_log};

pub async fn list_menu_ratings(
    pool: &DbPool,
    gathering_id: Uuid,
    participant_id: Option<Uuid>,
) -> AppResult<Vec<MenuItemRatingSummary>> {
    get_gathering_by_id(pool, gathering_id).await?;

    let rows = sqlx::query_as::<_, MenuItemRatingSummaryRow>(
        r#"
        SELECT
            m.id AS menu_item_id,
            AVG(r.rating) AS average_rating,
            COUNT(r.id) AS rating_count,
            MAX(CASE WHEN r.participant_id = ? THEN r.rating ELSE NULL END) AS my_rating
        FROM menu_items m
        LEFT JOIN menu_item_ratings r ON r.menu_item_id = m.id
        WHERE m.gathering_id = ?
        GROUP BY m.id
        ORDER BY m.created_at ASC
        "#,
    )
    .bind(participant_id.map(|id| id.to_string()))
    .bind(gathering_id.to_string())
    .fetch_all(pool)
    .await?;

    rows.into_iter().map(TryInto::try_into).collect()
}

pub async fn rate_menu_item(
    pool: &DbPool,
    menu_item_id: Uuid,
    participant_id: Uuid,
    payload: RateMenuItemRequest,
) -> AppResult<MenuItemRatingSummary> {
    validate_rating(payload.rating)?;

    let (gathering_id,) = sqlx::query_as::<_, (String,)>(
        r#"
        SELECT gathering_id
        FROM menu_items
        WHERE id = ?
        "#,
    )
    .bind(menu_item_id.to_string())
    .fetch_optional(pool)
    .await?
    .ok_or(AppError::NotFound)?;
    let gathering_id = Uuid::parse_str(&gathering_id)
        .map_err(|error| AppError::Validation(format!("invalid uuid: {error}")))?;
    let gathering = get_gathering_by_id(pool, gathering_id).await?;
    if !gathering.is_locked {
        return Err(AppError::Forbidden);
    }

    ensure_participant_in_gathering(pool, gathering_id, participant_id).await?;

    let now = Utc::now();
    sqlx::query(
        r#"
        INSERT INTO menu_item_ratings (
            id, menu_item_id, participant_id, rating, created_at, updated_at
        )
        VALUES (?, ?, ?, ?, ?, ?)
        ON CONFLICT(menu_item_id, participant_id) DO UPDATE SET
            rating = excluded.rating,
            updated_at = excluded.updated_at
        "#,
    )
    .bind(Uuid::new_v4().to_string())
    .bind(menu_item_id.to_string())
    .bind(participant_id.to_string())
    .bind(payload.rating)
    .bind(now)
    .bind(now)
    .execute(pool)
    .await?;

    insert_activity_log(
        pool,
        gathering_id,
        Some(participant_id),
        "menu_item_rated",
        "menu_item",
        Some(menu_item_id),
        Some(serde_json::json!({ "rating": payload.rating }).to_string()),
    )
    .await?;

    let mut summaries = list_menu_ratings(pool, gathering_id, Some(participant_id)).await?;
    summaries
        .drain(..)
        .find(|summary| summary.menu_item_id == menu_item_id)
        .ok_or(AppError::NotFound)
}

fn validate_rating(rating: i64) -> AppResult<()> {
    if (1..=5).contains(&rating) {
        Ok(())
    } else {
        Err(AppError::Validation(
            "rating must be between 1 and 5".to_string(),
        ))
    }
}
