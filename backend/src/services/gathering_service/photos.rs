use std::path::Path;

use axum::extract::Multipart;
use chrono::Utc;
use image::{GenericImageView, ImageReader};
use tokio::io::AsyncWriteExt;
use uuid::Uuid;

use crate::{
    db::DbPool,
    errors::{AppError, AppResult},
    models::{Photo, PhotoRow, User},
};

use super::common::{
    ensure_participant_in_gathering, ensure_user_is_admin, get_gathering_by_id, get_photo_by_id,
    insert_activity_log, insert_activity_log_tx, resource_dir,
};

const MAX_PHOTO_BYTES: usize = 8 * 1024 * 1024;

pub async fn list_photos(pool: &DbPool, gathering_id: Uuid) -> AppResult<Vec<Photo>> {
    let gathering = get_gathering_by_id(pool, gathering_id).await?;
    if !gathering.is_locked {
        return Err(AppError::Forbidden);
    }

    let rows = sqlx::query_as::<_, PhotoRow>(
        r#"
        SELECT id, gathering_id, uploaded_by, file_url, thumbnail_url, caption,
               taken_at, created_at, updated_at
        FROM photos
        WHERE gathering_id = ?
        ORDER BY created_at DESC
        "#,
    )
    .bind(gathering_id.to_string())
    .fetch_all(pool)
    .await?;

    rows.into_iter().map(TryInto::try_into).collect()
}

pub async fn upload_photo(
    pool: &DbPool,
    gathering_id: Uuid,
    uploaded_by: Uuid,
    mut multipart: Multipart,
) -> AppResult<Photo> {
    let gathering = get_gathering_by_id(pool, gathering_id).await?;
    if !gathering.is_locked {
        return Err(AppError::Forbidden);
    }
    ensure_participant_in_gathering(pool, gathering_id, uploaded_by).await?;
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
    if file_bytes.len() > MAX_PHOTO_BYTES {
        return Err(AppError::Validation("photo is too large".to_string()));
    }

    let image = ImageReader::new(std::io::Cursor::new(&file_bytes))
        .with_guessed_format()
        .map_err(|_| AppError::Validation("unsupported or invalid image".to_string()))?
        .decode()
        .map_err(|_| AppError::Validation("unsupported or invalid image".to_string()))?;
    let (width, height) = image.dimensions();
    if width == 0 || height == 0 || width > 12_000 || height > 12_000 {
        return Err(AppError::Validation(
            "image dimensions are not supported".to_string(),
        ));
    }

    let requested_extension = file_name
        .as_deref()
        .and_then(|name| Path::new(name).extension())
        .and_then(|extension| extension.to_str())
        .map(str::to_lowercase);
    let detected_extension = detect_image_extension(&file_bytes)
        .ok_or_else(|| AppError::Validation("unsupported or invalid image".to_string()))?;
    let extension = match requested_extension.as_deref() {
        Some("jpg" | "jpeg") if detected_extension == "jpg" => requested_extension.unwrap(),
        Some(extension) if extension == detected_extension => extension.to_string(),
        Some(_) => {
            return Err(AppError::Validation(
                "file extension does not match image content".to_string(),
            ));
        }
        None => detected_extension.to_string(),
    };
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
    if let Err(error) = file.write_all(&file_bytes).await {
        let _ = tokio::fs::remove_file(&file_path).await;
        return Err(AppError::Validation(format!(
            "could not write upload file: {error}"
        )));
    }

    let file_url = format!("/resources/uploads/{stored_file_name}");
    let caption = caption
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("Image");

    let mut transaction = pool.begin().await?;
    sqlx::query(
        r#"
        INSERT INTO photos (
            id, gathering_id, uploaded_by, file_url, thumbnail_url, caption,
            created_at, updated_at
        )
        VALUES (?, ?, ?, ?, NULL, ?, ?, ?)
        "#,
    )
    .bind(photo_id.to_string())
    .bind(gathering_id.to_string())
    .bind(uploaded_by.to_string())
    .bind(&file_url)
    .bind(caption)
    .bind(now)
    .bind(now)
    .execute(&mut *transaction)
    .await?;

    insert_activity_log_tx(
        &mut transaction,
        gathering_id,
        Some(uploaded_by),
        "photo_uploaded",
        "photo",
        Some(photo_id),
        Some(serde_json::json!({ "file_url": file_url }).to_string()),
    )
    .await?;

    if let Err(error) = transaction.commit().await {
        let _ = tokio::fs::remove_file(&file_path).await;
        return Err(error.into());
    }

    get_photo_by_id(pool, photo_id).await
}

fn detect_image_extension(bytes: &[u8]) -> Option<&'static str> {
    if bytes.starts_with(b"\x89PNG\r\n\x1a\n") {
        return Some("png");
    }

    if bytes.starts_with(b"\xff\xd8\xff") && bytes.ends_with(b"\xff\xd9") {
        return Some("jpg");
    }

    if bytes.len() >= 12 && bytes.starts_with(b"RIFF") && &bytes[8..12] == b"WEBP" {
        return Some("webp");
    }

    if bytes.starts_with(b"GIF87a") || bytes.starts_with(b"GIF89a") {
        return Some("gif");
    }

    None
}

pub async fn update_photo_caption(
    pool: &DbPool,
    photo_id: Uuid,
    caption: String,
    actor: &User,
) -> AppResult<Photo> {
    ensure_user_is_admin(actor)?;
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
    .bind(photo_id.to_string())
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

pub async fn delete_photo(pool: &DbPool, photo_id: Uuid, actor: &User) -> AppResult<Photo> {
    ensure_user_is_admin(actor)?;
    let photo = get_photo_by_id(pool, photo_id).await?;

    sqlx::query(
        r#"
        DELETE FROM photos
        WHERE id = ?
        "#,
    )
    .bind(photo_id.to_string())
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
