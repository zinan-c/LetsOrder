use sqlx::{Sqlite, Transaction};
use uuid::Uuid;

use crate::{
    db::DbPool,
    errors::{AppError, AppResult},
    models::{CreateMenuItemRequest, MenuItem, MenuItemRow, UpdateMenuItemRequest},
};

use super::common::{
    ensure_gathering_editable, ensure_participant_in_gathering, get_gathering_by_id,
    get_menu_item_by_id, insert_activity_log_tx, parse_uuid, touch_participant_menu_activity_tx,
};

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

struct FieldChangeLog {
    action: &'static str,
    field: &'static str,
    before: serde_json::Value,
    after: serde_json::Value,
}

pub async fn list_menu_items(pool: &DbPool, gathering_id: Uuid) -> AppResult<Vec<MenuItem>> {
    get_gathering_by_id(pool, gathering_id).await?;

    let rows = sqlx::query_as::<_, MenuItemRow>(
        r#"
        SELECT id, gathering_id, created_by, updated_by, name, category, quantity,
               unit, owner_name, reference_url, note, status, revision, created_at, updated_at
        FROM menu_items
        WHERE gathering_id = ?
        ORDER BY created_at ASC
        "#,
    )
    .bind(gathering_id.to_string())
    .fetch_all(pool)
    .await?;

    rows.into_iter().map(TryInto::try_into).collect()
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

    let now = chrono::Utc::now();
    let mut transaction = pool.begin().await?;
    let menu_item_id = Uuid::new_v4();

    sqlx::query(
        r#"
        INSERT INTO menu_items (
            id, gathering_id, created_by, name, category, quantity, unit,
            owner_name, reference_url, note, status, revision, created_at, updated_at
        )
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, 1, ?, ?)
        "#,
    )
    .bind(menu_item_id.to_string())
    .bind(gathering_id.to_string())
    .bind(payload.created_by.to_string())
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
    .execute(&mut *transaction)
    .await?;

    insert_activity_log_tx(
        &mut transaction,
        gathering_id,
        Some(payload.created_by),
        "menu_item_created",
        "menu_item",
        Some(menu_item_id),
        None,
    )
    .await?;
    touch_participant_menu_activity_tx(&mut transaction, payload.created_by).await?;

    transaction.commit().await?;

    get_menu_item_by_id(pool, menu_item_id).await
}

pub async fn update_menu_item(
    pool: &DbPool,
    menu_item_id: Uuid,
    payload: UpdateMenuItemRequest,
) -> AppResult<MenuItem> {
    let current = get_menu_item_by_id(pool, menu_item_id).await?;
    let gathering_id = current.gathering_id;
    ensure_gathering_editable(pool, gathering_id).await?;
    ensure_participant_in_gathering(pool, gathering_id, payload.updated_by).await?;

    let expected_revision = payload
        .expected_revision
        .ok_or_else(|| AppError::Validation("expected_revision is required".to_string()))?;

    if expected_revision != current.revision {
        return Err(AppError::Conflict(serde_json::json!({
            "error": "menu item was updated by someone else",
            "latest_menu_item": current,
            "submitted": payload
        })));
    }

    let submitted_payload = payload.clone();
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

    let now = chrono::Utc::now();
    let mut transaction = pool.begin().await?;

    let update_result = sqlx::query(
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
            revision = revision + 1,
            updated_at = ?
        WHERE id = ?
          AND revision = ?
        "#,
    )
    .bind(payload.updated_by.to_string())
    .bind(name.trim())
    .bind(category.as_deref())
    .bind(quantity)
    .bind(unit.as_deref())
    .bind(owner_name.as_deref())
    .bind(reference_url.as_deref())
    .bind(note.as_deref())
    .bind(&status)
    .bind(now)
    .bind(menu_item_id.to_string())
    .bind(expected_revision)
    .execute(&mut *transaction)
    .await?;

    if update_result.rows_affected() == 0 {
        let latest = get_menu_item_by_id(pool, menu_item_id).await?;

        return Err(AppError::Conflict(serde_json::json!({
            "error": "menu item was updated by someone else",
            "latest_menu_item": latest,
            "submitted": submitted_payload
        })));
    }

    insert_menu_item_change_logs_tx(
        &mut transaction,
        gathering_id,
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
    touch_participant_menu_activity_tx(&mut transaction, payload.updated_by).await?;

    transaction.commit().await?;

    get_menu_item_by_id(pool, menu_item_id).await
}

pub async fn menu_item_gathering_id(pool: &DbPool, menu_item_id: Uuid) -> AppResult<Uuid> {
    let row: Option<(String,)> = sqlx::query_as(
        r#"
        SELECT gathering_id
        FROM menu_items
        WHERE id = ?
        "#,
    )
    .bind(menu_item_id.to_string())
    .fetch_optional(pool)
    .await?;

    row.map(|(gathering_id,)| parse_uuid(&gathering_id))
        .transpose()?
        .ok_or(AppError::NotFound)
}

async fn insert_menu_item_change_logs_tx(
    transaction: &mut Transaction<'_, Sqlite>,
    gathering_id: Uuid,
    actor_id: Uuid,
    menu_item_id: Uuid,
    before: MenuItem,
    after: MenuItemChangeAfter,
) -> AppResult<()> {
    if before.name != after.name {
        insert_field_change_log(
            transaction,
            gathering_id,
            actor_id,
            menu_item_id,
            FieldChangeLog {
                action: "menu_item_name_changed",
                field: "name",
                before: serde_json::json!(before.name),
                after: serde_json::json!(after.name),
            },
        )
        .await?;
    }

    if before.category != after.category {
        insert_field_change_log(
            transaction,
            gathering_id,
            actor_id,
            menu_item_id,
            FieldChangeLog {
                action: "menu_item_category_changed",
                field: "category",
                before: serde_json::json!(before.category),
                after: serde_json::json!(after.category),
            },
        )
        .await?;
    }

    if before.quantity != after.quantity {
        insert_field_change_log(
            transaction,
            gathering_id,
            actor_id,
            menu_item_id,
            FieldChangeLog {
                action: "menu_item_quantity_changed",
                field: "quantity",
                before: serde_json::json!(before.quantity),
                after: serde_json::json!(after.quantity),
            },
        )
        .await?;
    }

    if before.unit != after.unit {
        insert_field_change_log(
            transaction,
            gathering_id,
            actor_id,
            menu_item_id,
            FieldChangeLog {
                action: "menu_item_unit_changed",
                field: "unit",
                before: serde_json::json!(before.unit),
                after: serde_json::json!(after.unit),
            },
        )
        .await?;
    }

    if before.owner_name != after.owner_name {
        insert_field_change_log(
            transaction,
            gathering_id,
            actor_id,
            menu_item_id,
            FieldChangeLog {
                action: "menu_item_owner_changed",
                field: "owner_name",
                before: serde_json::json!(before.owner_name),
                after: serde_json::json!(after.owner_name),
            },
        )
        .await?;
    }

    if before.reference_url != after.reference_url {
        insert_field_change_log(
            transaction,
            gathering_id,
            actor_id,
            menu_item_id,
            FieldChangeLog {
                action: "menu_item_reference_url_changed",
                field: "reference_url",
                before: serde_json::json!(before.reference_url),
                after: serde_json::json!(after.reference_url),
            },
        )
        .await?;
    }

    if before.note != after.note {
        insert_field_change_log(
            transaction,
            gathering_id,
            actor_id,
            menu_item_id,
            FieldChangeLog {
                action: "menu_item_note_changed",
                field: "note",
                before: serde_json::json!(before.note),
                after: serde_json::json!(after.note),
            },
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
            transaction,
            gathering_id,
            actor_id,
            menu_item_id,
            FieldChangeLog {
                action,
                field: "status",
                before: serde_json::json!(before.status),
                after: serde_json::json!(after.status),
            },
        )
        .await?;
    }

    Ok(())
}

async fn insert_field_change_log(
    transaction: &mut Transaction<'_, Sqlite>,
    gathering_id: Uuid,
    actor_id: Uuid,
    menu_item_id: Uuid,
    change: FieldChangeLog,
) -> AppResult<()> {
    insert_activity_log_tx(
        transaction,
        gathering_id,
        Some(actor_id),
        change.action,
        "menu_item",
        Some(menu_item_id),
        Some(
            serde_json::json!({
                "field": change.field,
                "before": change.before,
                "after": change.after,
            })
            .to_string(),
        ),
    )
    .await
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
        "planned" | "prepared" | "done" | "cancelled" => Ok(()),
        _ => Err(AppError::Validation(
            "status must be planned, prepared, done, or cancelled".to_string(),
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
