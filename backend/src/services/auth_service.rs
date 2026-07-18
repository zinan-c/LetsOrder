use argon2::{
    Argon2, PasswordHash, PasswordHasher, PasswordVerifier,
    password_hash::{SaltString, rand_core::OsRng},
};
use chrono::{Duration, Utc};
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::{
    db::DbPool,
    errors::{AppError, AppResult},
    models::{
        AuthResponse, LoginRequest, Participant, ParticipantRow, RegisterRequest,
        UpdateAccountRequest, UpdateMemberRequest, User, UserRow,
    },
};

const PASSWORD_SALT: &str = "letsorder-auth-v1";
const SYSTEM_ADMIN_ID: &str = "00000000-0000-0000-0000-000000000001";
const SYSTEM_ADMIN_USERNAME: &str = "suite-admin";
const SYSTEM_ADMIN_PASSWORD: &str = "Admin_1234";
const SESSION_TTL_HOURS: i64 = 48;
const HASH_SCHEME_SHA256: &str = "sha256";

pub async fn login(pool: &DbPool, payload: LoginRequest) -> AppResult<AuthResponse> {
    let username = payload.username.trim();
    if username.is_empty() || payload.password.is_empty() {
        return Err(AppError::Validation(
            "username and password are required".to_string(),
        ));
    }

    if username == SYSTEM_ADMIN_USERNAME && payload.password == system_admin_password() {
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

    let row: Option<(String, String)> = sqlx::query_as(
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
    let user_id = parse_uuid(&user_id)?;

    if !verify_password(&payload.password, &password_hash) {
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
    validate_display_name(display_name)?;

    let username = unique_username(pool, display_name).await?;
    let generated_password = random_password();
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
    .bind(user_id.to_string())
    .bind(&username)
    .bind(display_name)
    .bind(hash_password(&generated_password))
    .bind(now)
    .bind(now)
    .execute(pool)
    .await?;

    let gathering_id = if let Some(gathering_id) = payload.gathering_id {
        Some(gathering_id)
    } else if let Some(invite_code) = payload.invite_code.as_deref() {
        Some(gathering_id_by_invite_code(pool, invite_code).await?)
    } else {
        None
    };

    let participant = if let Some(gathering_id) = gathering_id {
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

pub async fn create_websocket_ticket(pool: &DbPool, token: &str) -> AppResult<String> {
    let user = user_from_token(pool, token).await?;
    let ticket = Uuid::new_v4().to_string();
    sqlx::query("INSERT INTO websocket_tickets (ticket, user_id, expires_at) VALUES (?, ?, ?)")
        .bind(&ticket)
        .bind(user.id.to_string())
        .bind(Utc::now() + Duration::minutes(1))
        .execute(pool)
        .await?;
    Ok(ticket)
}

pub async fn consume_websocket_ticket(pool: &DbPool, ticket: &str) -> AppResult<User> {
    let row: Option<(String, chrono::DateTime<Utc>)> =
        sqlx::query_as("SELECT user_id, expires_at FROM websocket_tickets WHERE ticket = ?")
            .bind(ticket.trim())
            .fetch_optional(pool)
            .await?;
    sqlx::query("DELETE FROM websocket_tickets WHERE ticket = ?")
        .bind(ticket.trim())
        .execute(pool)
        .await?;
    let Some((user_id, expires_at)) = row else {
        return Err(AppError::Unauthorized);
    };
    if expires_at <= Utc::now() {
        return Err(AppError::Unauthorized);
    }
    get_user_by_id(pool, parse_uuid(&user_id)?).await
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
    validate_display_name(display_name)?;
    let password_hash = payload
        .password
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(hash_password);
    let password_changed = password_hash.is_some();
    if let Some(password) = payload.password.as_deref()
        && password.trim().len() < 8
    {
        return Err(AppError::Validation(
            "password must be at least 8 characters".to_string(),
        ));
    }
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
    .bind(user.id.to_string())
    .execute(pool)
    .await?;

    if password_changed {
        sqlx::query("DELETE FROM auth_sessions WHERE user_id = ? AND token != ?")
            .bind(user.id.to_string())
            .bind(token)
            .execute(pool)
            .await?;
    }

    sqlx::query(
        r#"
        UPDATE participants
        SET display_name = ?, updated_at = ?
        WHERE user_id = ?
        "#,
    )
    .bind(display_name)
    .bind(now)
    .bind(user.id.to_string())
    .execute(pool)
    .await?;

    get_user_by_id(pool, user.id).await
}

pub async fn list_members(pool: &DbPool, token: &str) -> AppResult<Vec<User>> {
    ensure_admin_token(pool, token).await?;

    let rows = sqlx::query_as::<_, UserRow>(
        r#"
        SELECT id, username, display_name, role, created_at, updated_at
        FROM users
        ORDER BY
            CASE WHEN role = 'admin' THEN 0 ELSE 1 END,
            display_name COLLATE NOCASE ASC,
            created_at DESC
        "#,
    )
    .fetch_all(pool)
    .await?;

    rows.into_iter().map(TryInto::try_into).collect()
}

pub async fn update_member(
    pool: &DbPool,
    token: &str,
    user_id: Uuid,
    payload: UpdateMemberRequest,
) -> AppResult<User> {
    ensure_admin_token(pool, token).await?;
    let target = get_user_by_id(pool, user_id).await?;

    if target.username == SYSTEM_ADMIN_USERNAME {
        return Ok(target);
    }

    let display_name = payload
        .display_name
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or(target.display_name.as_str());
    validate_display_name(display_name)?;
    let password_hash = payload
        .password
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(hash_password);
    let password_changed = password_hash.is_some();
    if let Some(password) = payload.password.as_deref()
        && password.trim().len() < 8
    {
        return Err(AppError::Validation(
            "password must be at least 8 characters".to_string(),
        ));
    }
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
    .bind(user_id.to_string())
    .execute(pool)
    .await?;

    if password_changed {
        sqlx::query("DELETE FROM auth_sessions WHERE user_id = ?")
            .bind(user_id.to_string())
            .execute(pool)
            .await?;
    }

    sqlx::query(
        r#"
        UPDATE participants
        SET display_name = ?, updated_at = ?
        WHERE user_id = ?
        "#,
    )
    .bind(display_name)
    .bind(now)
    .bind(user_id.to_string())
    .execute(pool)
    .await?;

    get_user_by_id(pool, user_id).await
}

pub async fn user_from_token(pool: &DbPool, token: &str) -> AppResult<User> {
    let token = token.trim();
    if token.is_empty() {
        return Err(AppError::Forbidden);
    }

    let session: Option<(String, Option<chrono::DateTime<Utc>>)> = sqlx::query_as(
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
    let user_id = parse_uuid(&user_id)?;

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
    if user.role == "admin" {
        return Err(AppError::Forbidden);
    }

    if let Some((participant_id,)) = sqlx::query_as::<_, (String,)>(
        r#"
        SELECT id
        FROM participants
        WHERE gathering_id = ? AND user_id = ?
        "#,
    )
    .bind(gathering_id.to_string())
    .bind(user_id.to_string())
    .fetch_optional(pool)
    .await?
    {
        return get_participant_by_id(pool, parse_uuid(&participant_id)?).await;
    }

    let participant_id = Uuid::new_v4();
    let now = Utc::now();
    let result = sqlx::query(
        r#"
        INSERT OR IGNORE INTO participants (
            id, gathering_id, user_id, display_name, role, access_token_hash, joined_at, created_at, updated_at
        )
        VALUES (?, ?, ?, ?, 'participant', ?, ?, ?, ?)
        "#,
    )
    .bind(participant_id.to_string())
    .bind(gathering_id.to_string())
    .bind(user_id.to_string())
    .bind(&user.display_name)
    .bind(Uuid::new_v4().to_string())
    .bind(now)
    .bind(now)
    .bind(now)
    .execute(pool)
    .await?;

    if result.rows_affected() == 0 {
        let existing = sqlx::query_as::<_, (String,)>(
            "SELECT id FROM participants WHERE gathering_id = ? AND user_id = ?",
        )
        .bind(gathering_id.to_string())
        .bind(user_id.to_string())
        .fetch_one(pool)
        .await?;
        return get_participant_by_id(pool, parse_uuid(&existing.0)?).await;
    }

    sqlx::query(
        r#"
        INSERT INTO activity_logs (
            id, gathering_id, actor_id, action, target_type, target_id, detail, created_at
        )
        VALUES (?, ?, ?, 'participant_joined', 'participant', ?, NULL, ?)
        "#,
    )
    .bind(Uuid::new_v4().to_string())
    .bind(gathering_id.to_string())
    .bind(participant_id.to_string())
    .bind(participant_id.to_string())
    .bind(now)
    .execute(pool)
    .await?;

    get_participant_by_id(pool, participant_id).await
}

pub async fn participant_for_user(
    pool: &DbPool,
    gathering_id: Uuid,
    user_id: Uuid,
) -> AppResult<Option<Participant>> {
    let participant_id = sqlx::query_as::<_, (String,)>(
        r#"
        SELECT id
        FROM participants
        WHERE gathering_id = ? AND user_id = ?
        "#,
    )
    .bind(gathering_id.to_string())
    .bind(user_id.to_string())
    .fetch_optional(pool)
    .await?
    .map(|(participant_id,)| participant_id);

    match participant_id {
        Some(participant_id) => get_participant_by_id(pool, parse_uuid(&participant_id)?)
            .await
            .map(Some),
        None => Ok(None),
    }
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
    .bind(user_id.to_string())
    .bind(now)
    .bind(expires_at)
    .execute(pool)
    .await?;

    Ok(token)
}

async fn ensure_admin_token(pool: &DbPool, token: &str) -> AppResult<User> {
    let user = user_from_token(pool, token).await?;

    if user.role == "admin" {
        Ok(user)
    } else {
        Err(AppError::Forbidden)
    }
}

async fn gathering_id_by_invite_code(pool: &DbPool, invite_code: &str) -> AppResult<Uuid> {
    let invite_code = invite_code.trim();
    if invite_code.is_empty() {
        return Err(AppError::Validation("invite_code is required".to_string()));
    }

    sqlx::query_as::<_, (String,)>(
        r#"
        SELECT id
        FROM gatherings
        WHERE invite_code = ?
        "#,
    )
    .bind(invite_code)
    .fetch_optional(pool)
    .await?
    .map(|(gathering_id,)| parse_uuid(&gathering_id))
    .transpose()?
    .ok_or(AppError::NotFound)
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
    .bind(user_id.to_string())
    .bind(SYSTEM_ADMIN_USERNAME)
    .bind(SYSTEM_ADMIN_USERNAME)
    .bind(hash_password(&system_admin_password()))
    .bind(now)
    .bind(now)
    .execute(pool)
    .await?;

    Ok(user_id)
}

async fn get_user_by_id(pool: &DbPool, user_id: Uuid) -> AppResult<User> {
    let row = sqlx::query_as::<_, UserRow>(
        r#"
        SELECT id, username, display_name, role, created_at, updated_at
        FROM users
        WHERE id = ?
        "#,
    )
    .bind(user_id.to_string())
    .fetch_optional(pool)
    .await?
    .ok_or(AppError::NotFound)?;

    row.try_into()
}

async fn get_participant_by_id(pool: &DbPool, participant_id: Uuid) -> AppResult<Participant> {
    let row = sqlx::query_as::<_, ParticipantRow>(
        r#"
        SELECT id, gathering_id, user_id, display_name, role, last_menu_activity_at,
               joined_at, created_at, updated_at
        FROM participants
        WHERE id = ?
        "#,
    )
    .bind(participant_id.to_string())
    .fetch_optional(pool)
    .await?
    .ok_or(AppError::NotFound)?;

    row.try_into()
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
        format!("user-{}", &Uuid::new_v4().simple().to_string()[..8])
    } else {
        slug
    }
}

fn random_password() -> String {
    Uuid::new_v4().simple().to_string()[..12].to_string()
}

fn hash_password(password: &str) -> String {
    let salt = SaltString::generate(&mut OsRng);
    Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .expect("argon2 password hashing should succeed")
        .to_string()
}

fn verify_password(password: &str, stored_hash: &str) -> bool {
    if stored_hash.starts_with("$argon2") {
        return PasswordHash::new(stored_hash).ok().is_some_and(|parsed| {
            Argon2::default()
                .verify_password(password.as_bytes(), &parsed)
                .is_ok()
        });
    }

    if let Some((scheme, rest)) = stored_hash.split_once('$')
        && scheme == HASH_SCHEME_SHA256
    {
        let Some((salt, _digest)) = rest.split_once('$') else {
            return false;
        };

        return salted_sha256_password(password, salt) == stored_hash;
    }

    stored_hash == legacy_hash_password(password)
}

fn salted_sha256_password(password: &str, salt: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(salt.as_bytes());
    hasher.update(b":");
    hasher.update(password.as_bytes());
    let digest = hasher.finalize();

    format!("{HASH_SCHEME_SHA256}${salt}${}", encode_hex(&digest))
}

fn legacy_hash_password(password: &str) -> String {
    let input = format!("{PASSWORD_SALT}:{password}");
    let mut hash = 0xcbf29ce484222325u64;

    for byte in input.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }

    format!("{hash:016x}")
}

fn encode_hex(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut output = String::with_capacity(bytes.len() * 2);

    for byte in bytes {
        output.push(HEX[(byte >> 4) as usize] as char);
        output.push(HEX[(byte & 0x0f) as usize] as char);
    }

    output
}

fn system_admin_password() -> String {
    std::env::var("LETSORDER_ADMIN_PASSWORD").unwrap_or_else(|_| SYSTEM_ADMIN_PASSWORD.to_string())
}

fn validate_display_name(display_name: &str) -> AppResult<()> {
    if display_name
        .trim()
        .eq_ignore_ascii_case(SYSTEM_ADMIN_USERNAME)
    {
        return Err(AppError::Validation(
            "这是系统管理员账号名称，请使用系统管理员账号及密码登陆".to_string(),
        ));
    }

    Ok(())
}

fn parse_uuid(value: &str) -> AppResult<Uuid> {
    Uuid::parse_str(value).map_err(|error| AppError::Validation(format!("invalid uuid: {error}")))
}
