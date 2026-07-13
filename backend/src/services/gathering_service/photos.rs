use std::path::Path;

use axum::extract::Multipart;
use chrono::Utc;
use tokio::io::AsyncWriteExt;
use uuid::Uuid;

use crate::{
    db::DbPool,
    errors::{AppError, AppResult},
    models::Photo,
};

use super::common::{
    ensure_actor_is_admin, get_gathering_by_id, get_or_create_participant_by_name, get_photo_by_id,
    insert_activity_log, resource_dir,
};

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
    let caption = caption
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("Image");

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
    .bind(caption)
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

pub async fn update_photo_caption(
    pool: &DbPool,
    photo_id: Uuid,
    caption: String,
    actor_name: Option<String>,
) -> AppResult<Photo> {
    ensure_actor_is_admin(actor_name.as_deref())?;
    let current = get_photo_by_id(pool, photo_id).await?;
    let now = Utc::now();
    let caption = caption.trim();
    let caption = if caption.is_empty() { "Image" } else { caption };

    sqlx::query(
        r#"
        UPDATE photos
        SET caption = ?, updated_at = ?
        WHERE id = ?
        "#,
    )
    .bind(caption)
    .bind(now)
    .bind(photo_id)
    .execute(pool)
    .await?;

    insert_activity_log(
        pool,
        current.gathering_id,
        None,
        "photo_caption_updated",
        "photo",
        Some(photo_id),
        Some(format!(
            "caption: {} -> {}",
            current.caption.unwrap_or_else(|| "Image".to_string()),
            caption
        )),
    )
    .await?;

    get_photo_by_id(pool, photo_id).await
}

pub async fn delete_photo(
    pool: &DbPool,
    photo_id: Uuid,
    actor_name: Option<String>,
) -> AppResult<Photo> {
    ensure_actor_is_admin(actor_name.as_deref())?;
    let photo = get_photo_by_id(pool, photo_id).await?;

    sqlx::query(
        r#"
        DELETE FROM photos
        WHERE id = ?
        "#,
    )
    .bind(photo_id)
    .execute(pool)
    .await?;

    if let Some(relative_path) = photo.file_url.strip_prefix("/resources/") {
        let file_path = Path::new(&resource_dir()).join(relative_path);
        let _ = tokio::fs::remove_file(file_path).await;
    }

    insert_activity_log(
        pool,
        photo.gathering_id,
        None,
        "photo_deleted",
        "photo",
        Some(photo_id),
        photo
            .caption
            .clone()
            .map(|caption| format!("caption: {caption}")),
    )
    .await?;

    Ok(photo)
}
