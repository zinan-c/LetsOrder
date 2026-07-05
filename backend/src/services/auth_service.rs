use chrono::{Duration, Utc};
use uuid::Uuid;

use crate::{
    db::DbPool,
    errors::{AppError, AppResult},
    models::{
        AuthResponse, LoginRequest, Participant, RegisterRequest, UpdateAccountRequest, User,
    },
};

const PASSWORD_SALT: &str = "letsorder-auth-v1";
const SYSTEM_ADMIN_ID: &str = "00000000-0000-0000-0000-000000000001";
const SYSTEM_ADMIN_USERNAME: &str = "suite-admin";
const SYSTEM_ADMIN_PASSWORD: &str = "Admin_1234";
const SESSION_TTL_HOURS: i64 = 48;

pub async fn login(pool: &DbPool, payload: LoginRequest) -> AppResult<AuthResponse> {
    let username = payload.username.trim();
    if username.is_empty() || payload.password.is_empty() {
        return Err(AppError::Validation(
            "username and password are required".to_string(),
        ));
    }

    if username == SYSTEM_ADMIN_USERNAME && payload.password == SYSTEM_ADMIN_PASSWORD {
        let user_id = ensure_system_admin(pool).await?;
        let token = create_session(pool, user_id).await?;
        let user = get_user_by_id(pool, user_id).await?;

        return Ok(AuthResponse {
            user,
            token,
            generated_password: None,
            participant: None,
        });
    }

    let row: Option<(Uuid, String)> = sqlx::query_as(
        r#"
        SELECT id, password_hash
        FROM users
        WHERE username = ?
        "#,
    )
    .bind(username)
    .fetch_optional(pool)
    .await?;

    let Some((user_id, password_hash)) = row else {
        return Err(AppError::Forbidden);
    };

    if password_hash != hash_password(&payload.password) {
        return Err(AppError::Forbidden);
    }

    let token = create_session(pool, user_id).await?;
    let user = get_user_by_id(pool, user_id).await?;

    Ok(AuthResponse {
        user,
        token,
        generated_password: None,
        participant: None,
    })
}

pub async fn register(pool: &DbPool, payload: RegisterRequest) -> AppResult<AuthResponse> {
    let display_name = payload.display_name.trim();
    if display_name.is_empty() {
        return Err(AppError::Validation("display_name is required".to_string()));
    }

    let username = unique_username(pool, display_name).await?;
    let generated_password = format!("{display_name}{}", random_three_digits());
    let user_id = Uuid::new_v4();
    let now = Utc::now();

    sqlx::query(
        r#"
        INSERT INTO users (
            id, username, display_name, password_hash, role, created_at, updated_at
        )
        VALUES (?, ?, ?, ?, 'user', ?, ?)
        "#,
    )
    .bind(user_id)
    .bind(&username)
    .bind(display_name)
    .bind(hash_password(&generated_password))
    .bind(now)
    .bind(now)
    .execute(pool)
    .await?;

    let participant = if let Some(gathering_id) = payload.gathering_id {
        Some(ensure_participant_for_user(pool, gathering_id, user_id).await?)
    } else {
        None
    };
    let token = create_session(pool, user_id).await?;
    let user = get_user_by_id(pool, user_id).await?;

    Ok(AuthResponse {
        user,
        token,
        generated_password: Some(generated_password),
        participant,
    })
}

pub async fn me(pool: &DbPool, token: &str) -> AppResult<User> {
    let user = user_from_token(pool, token).await?;
    Ok(user)
}

pub async fn logout(pool: &DbPool, token: &str) -> AppResult<()> {
    sqlx::query(
        r#"
        DELETE FROM auth_sessions
        WHERE token = ?
        "#,
    )
    .bind(token.trim())
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn update_account(
    pool: &DbPool,
    token: &str,
    payload: UpdateAccountRequest,
) -> AppResult<User> {
    let user = user_from_token(pool, token).await?;
    if user.username == SYSTEM_ADMIN_USERNAME {
        return Ok(user);
    }

    let display_name = payload
        .display_name
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or(user.display_name.as_str());
    let password_hash = payload
        .password
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(hash_password);
    let now = Utc::now();

    sqlx::query(
        r#"
        UPDATE users
        SET display_name = ?,
            password_hash = COALESCE(?, password_hash),
            updated_at = ?
        WHERE id = ?
        "#,
    )
    .bind(display_name)
    .bind(password_hash)
    .bind(now)
    .bind(user.id)
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        UPDATE participants
        SET display_name = ?, updated_at = ?
        WHERE user_id = ?
        "#,
    )
    .bind(display_name)
    .bind(now)
    .bind(user.id)
    .execute(pool)
    .await?;

    get_user_by_id(pool, user.id).await
}

pub async fn user_from_token(pool: &DbPool, token: &str) -> AppResult<User> {
    let token = token.trim();
    if token.is_empty() {
        return Err(AppError::Forbidden);
    }

    let session: Option<(Uuid, Option<chrono::DateTime<Utc>>)> = sqlx::query_as(
        r#"
        SELECT user_id, expires_at
        FROM auth_sessions
        WHERE token = ?
        "#,
    )
    .bind(token)
    .fetch_optional(pool)
    .await?;

    let Some((user_id, expires_at)) = session else {
        return Err(AppError::Unauthorized);
    };

    if expires_at.is_some_and(|value| value <= Utc::now()) {
        logout(pool, token).await?;
        return Err(AppError::Unauthorized);
    }

    get_user_by_id(pool, user_id).await
}

pub async fn ensure_participant_for_user(
    pool: &DbPool,
    gathering_id: Uuid,
    user_id: Uuid,
) -> AppResult<Participant> {
    let user = get_user_by_id(pool, user_id).await?;

    if let Some((participant_id,)) = sqlx::query_as::<_, (Uuid,)>(
        r#"
        SELECT id
        FROM participants
        WHERE gathering_id = ? AND user_id = ?
        "#,
    )
    .bind(gathering_id)
    .bind(user_id)
    .fetch_optional(pool)
    .await?
    {
        return get_participant_by_id(pool, participant_id).await;
    }

    if let Some((participant_id,)) = sqlx::query_as::<_, (Uuid,)>(
        r#"
        SELECT id
        FROM participants
        WHERE gathering_id = ? AND display_name = ? AND user_id IS NULL
        "#,
    )
    .bind(gathering_id)
    .bind(&user.display_name)
    .fetch_optional(pool)
    .await?
    {
        sqlx::query(
            r#"
            UPDATE participants
            SET user_id = ?, updated_at = ?
            WHERE id = ?
            "#,
        )
        .bind(user_id)
        .bind(Utc::now())
        .bind(participant_id)
        .execute(pool)
        .await?;

        return get_participant_by_id(pool, participant_id).await;
    }

    let participant_id = Uuid::new_v4();
    let now = Utc::now();
    sqlx::query(
        r#"
        INSERT INTO participants (
            id, gathering_id, user_id, display_name, role, access_token_hash, joined_at, created_at, updated_at
        )
        VALUES (?, ?, ?, ?, 'participant', ?, ?, ?, ?)
        "#,
    )
    .bind(participant_id)
    .bind(gathering_id)
    .bind(user_id)
    .bind(&user.display_name)
    .bind(Uuid::new_v4().to_string())
    .bind(now)
    .bind(now)
    .bind(now)
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        INSERT INTO activity_logs (
            id, gathering_id, actor_id, action, target_type, target_id, detail, created_at
        )
        VALUES (?, ?, ?, 'participant_joined', 'participant', ?, NULL, ?)
        "#,
    )
    .bind(Uuid::new_v4())
    .bind(gathering_id)
    .bind(participant_id)
    .bind(participant_id)
    .bind(now)
    .execute(pool)
    .await?;

    get_participant_by_id(pool, participant_id).await
}

async fn create_session(pool: &DbPool, user_id: Uuid) -> AppResult<String> {
    let token = Uuid::new_v4().to_string();
    let now = Utc::now();
    let expires_at = now + Duration::hours(SESSION_TTL_HOURS);
    sqlx::query(
        r#"
        INSERT INTO auth_sessions (token, user_id, created_at, expires_at)
        VALUES (?, ?, ?, ?)
        "#,
    )
    .bind(&token)
    .bind(user_id)
    .bind(now)
    .bind(expires_at)
    .execute(pool)
    .await?;

    Ok(token)
}

async fn ensure_system_admin(pool: &DbPool) -> AppResult<Uuid> {
    let user_id = Uuid::parse_str(SYSTEM_ADMIN_ID)
        .map_err(|error| AppError::Validation(format!("invalid system admin id: {error}")))?;
    let now = Utc::now();

    sqlx::query(
        r#"
        INSERT INTO users (
            id, username, display_name, password_hash, role, created_at, updated_at
        )
        VALUES (?, ?, ?, ?, 'admin', ?, ?)
        ON CONFLICT(username) DO UPDATE SET
            id = excluded.id,
            display_name = excluded.display_name,
            password_hash = excluded.password_hash,
            role = excluded.role,
            updated_at = excluded.updated_at
        "#,
    )
    .bind(user_id)
    .bind(SYSTEM_ADMIN_USERNAME)
    .bind(SYSTEM_ADMIN_USERNAME)
    .bind(hash_password(SYSTEM_ADMIN_PASSWORD))
    .bind(now)
    .bind(now)
    .execute(pool)
    .await?;

    Ok(user_id)
}

async fn get_user_by_id(pool: &DbPool, user_id: Uuid) -> AppResult<User> {
    sqlx::query_as::<_, User>(
        r#"
        SELECT id, username, display_name, role, created_at, updated_at
        FROM users
        WHERE id = ?
        "#,
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await?
    .ok_or(AppError::NotFound)
}

async fn get_participant_by_id(pool: &DbPool, participant_id: Uuid) -> AppResult<Participant> {
    sqlx::query_as::<_, Participant>(
        r#"
        SELECT id, gathering_id, user_id, display_name, role, last_menu_activity_at,
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

async fn unique_username(pool: &DbPool, display_name: &str) -> AppResult<String> {
    let base = slugify_username(display_name);
    let mut candidate = base.clone();
    let mut suffix = 2;

    loop {
        let exists: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM users WHERE username = ?")
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

fn slugify_username(display_name: &str) -> String {
    let mut slug = String::new();
    let mut last_dash = false;

    for character in display_name.trim().to_lowercase().chars() {
        if character.is_ascii_alphanumeric() {
            slug.push(character);
            last_dash = false;
        } else if !last_dash && !slug.is_empty() {
            slug.push('-');
            last_dash = true;
        }
    }

    while slug.ends_with('-') {
        slug.pop();
    }

    if slug.is_empty() {
        format!(
            "user-{}",
            Uuid::new_v4().simple().to_string()[..8].to_string()
        )
    } else {
        slug
    }
}

fn random_three_digits() -> String {
    let value = (Uuid::new_v4().as_u128() % 900) + 100;
    value.to_string()
}

fn hash_password(password: &str) -> String {
    let input = format!("{PASSWORD_SALT}:{password}");
    let mut hash = 0xcbf29ce484222325u64;

    for byte in input.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }

    format!("{hash:016x}")
}
