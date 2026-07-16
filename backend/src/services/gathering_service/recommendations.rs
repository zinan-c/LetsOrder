use crate::{db::DbPool, errors::AppResult, models::DishRecommendation};

const DEFAULT_RECOMMENDATION_LIMIT: i64 = 8;
const MAX_RECOMMENDATION_LIMIT: i64 = 24;

pub async fn list_dish_recommendations(
    pool: &DbPool,
    chef_name: &str,
    limit: Option<i64>,
) -> AppResult<Vec<DishRecommendation>> {
    let chef_name = chef_name.trim();
    let limit = limit
        .unwrap_or(DEFAULT_RECOMMENDATION_LIMIT)
        .clamp(1, MAX_RECOMMENDATION_LIMIT);

    let recommendations = sqlx::query_as::<_, DishRecommendation>(
        r#"
        WITH candidate_items AS (
            SELECT
                m.id,
                LOWER(TRIM(m.name)) AS dish_key,
                m.name,
                m.category,
                m.quantity,
                m.unit,
                m.reference_url,
                m.note,
                m.created_at,
                m.updated_at
            FROM menu_items m
            WHERE m.owner_name = ?
              AND m.status IN ('prepared', 'done')
              AND TRIM(m.name) != ''
        ),
        rating_summary AS (
            SELECT
                c.dish_key,
                AVG(r.rating) AS average_rating,
                COUNT(r.id) AS rating_count,
                MAX(c.updated_at) AS last_made_at
            FROM candidate_items c
            LEFT JOIN menu_item_ratings r ON r.menu_item_id = c.id
            GROUP BY c.dish_key
        ),
        latest_items AS (
            SELECT
                c.*,
                ROW_NUMBER() OVER (
                    PARTITION BY c.dish_key
                    ORDER BY c.updated_at DESC, c.created_at DESC
                ) AS row_number
            FROM candidate_items c
        )
        SELECT
            l.dish_key,
            l.name,
            l.category,
            l.quantity,
            l.unit,
            l.reference_url,
            l.note,
            s.average_rating,
            s.rating_count,
            s.last_made_at
        FROM latest_items l
        JOIN rating_summary s ON s.dish_key = l.dish_key
        WHERE l.row_number = 1
        ORDER BY
            CASE WHEN s.average_rating IS NULL THEN 1 ELSE 0 END ASC,
            s.average_rating DESC,
            s.rating_count DESC,
            s.last_made_at DESC
        LIMIT ?
        "#,
    )
    .bind(chef_name)
    .bind(limit)
    .fetch_all(pool)
    .await?;

    Ok(recommendations)
}
