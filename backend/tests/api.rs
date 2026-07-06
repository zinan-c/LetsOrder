use axum::{
    Router,
    body::{Body, to_bytes},
    http::{Method, Request, StatusCode, header},
};
use letsorder_backend::{db, routes, services::gathering_service};
use serde_json::{Value, json};
use tokio::sync::broadcast;
use tower::ServiceExt;
use uuid::Uuid;

async fn test_app() -> (Router, db::DbPool) {
    let database_path = format!("/tmp/letsorder-test-{}.db", Uuid::new_v4());
    let database_url = format!("sqlite://{database_path}?mode=rwc");
    let pool = db::connect(&database_url)
        .await
        .expect("test database should connect");
    let (realtime_tx, _) = broadcast::channel(16);
    let app = routes::router(pool.clone(), realtime_tx);

    (app, pool)
}

async fn request_json(
    app: &Router,
    method: Method,
    path: &str,
    token: Option<&str>,
    payload: Value,
) -> (StatusCode, Value) {
    let mut builder = Request::builder()
        .method(method)
        .uri(path)
        .header(header::CONTENT_TYPE, "application/json");

    if let Some(token) = token {
        builder = builder.header(header::AUTHORIZATION, format!("Bearer {token}"));
    }

    let response = app
        .clone()
        .oneshot(
            builder
                .body(Body::from(payload.to_string()))
                .expect("request should build"),
        )
        .await
        .expect("request should complete");
    let status = response.status();

    if status == StatusCode::NO_CONTENT {
        return (status, Value::Null);
    }

    let bytes = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body should read");
    let body = serde_json::from_slice(&bytes).unwrap_or(Value::Null);

    (status, body)
}

async fn request_empty(
    app: &Router,
    method: Method,
    path: &str,
    token: Option<&str>,
) -> (StatusCode, Value) {
    let mut builder = Request::builder().method(method).uri(path);

    if let Some(token) = token {
        builder = builder.header(header::AUTHORIZATION, format!("Bearer {token}"));
    }

    let response = app
        .clone()
        .oneshot(builder.body(Body::empty()).expect("request should build"))
        .await
        .expect("request should complete");
    let status = response.status();

    if status == StatusCode::NO_CONTENT {
        return (status, Value::Null);
    }

    let bytes = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body should read");
    let body = serde_json::from_slice(&bytes).unwrap_or(Value::Null);

    (status, body)
}

async fn login_admin(app: &Router) -> String {
    let (status, body) = request_json(
        app,
        Method::POST,
        "/api/auth/login",
        None,
        json!({
            "username": "suite-admin",
            "password": "Admin_1234"
        }),
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["user"]["role"], "admin");
    body["token"]
        .as_str()
        .expect("token should exist")
        .to_string()
}

async fn create_gathering(app: &Router, admin_token: &str, title: &str, expires_at: &str) -> Value {
    let (status, body) = request_json(
        app,
        Method::POST,
        "/api/gatherings",
        Some(admin_token),
        json!({
            "title": title,
            "description": "Integration test gathering",
            "host_name": "suite-admin",
            "expires_at": expires_at
        }),
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    body["gathering"].clone()
}

async fn register_user(app: &Router, display_name: &str, gathering_id: &str) -> Value {
    let (status, body) = request_json(
        app,
        Method::POST,
        "/api/auth/register",
        None,
        json!({
            "display_name": display_name,
            "gathering_id": gathering_id
        }),
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    assert!(
        body["generated_password"]
            .as_str()
            .expect("generated password should exist")
            .starts_with(display_name)
    );
    body
}

#[tokio::test]
async fn auth_gathering_menu_activity_and_permissions_flow() {
    let (app, _) = test_app().await;
    let admin_token = login_admin(&app).await;
    let gathering =
        create_gathering(&app, &admin_token, "周中小聚1.0", "2099-07-06T12:00:00Z").await;
    let gathering_id = gathering["id"].as_str().expect("gathering id");
    let invite_code = gathering["invite_code"].as_str().expect("invite code");

    assert_eq!(invite_code.len(), 8);
    assert!(
        invite_code
            .chars()
            .all(|character| character.is_ascii_hexdigit())
    );

    let user_response = register_user(&app, "Nico", gathering_id).await;
    let user_token = user_response["token"].as_str().expect("user token");
    let participant_id = user_response["participant"]["id"]
        .as_str()
        .expect("participant id");

    let (join_status, join_body) = request_json(
        &app,
        Method::POST,
        &format!("/api/gatherings/{gathering_id}/participants"),
        Some(user_token),
        json!({}),
    )
    .await;
    assert_eq!(join_status, StatusCode::OK);
    assert_eq!(join_body["participant"]["id"], participant_id);

    let share_text = "懒人冰滴咖啡 http://xhslink.com/o/3shicEMk8TL 存好这段口令";
    let (create_item_status, create_item_body) = request_json(
        &app,
        Method::POST,
        &format!("/api/gatherings/{gathering_id}/menu-items"),
        Some(user_token),
        json!({
            "created_by": participant_id,
            "name": "Cold brew",
            "category": "Drink",
            "quantity": 1,
            "unit": "cups",
            "owner_name": "Nico",
            "reference_url": share_text,
            "status": "planned"
        }),
    )
    .await;
    assert_eq!(create_item_status, StatusCode::OK);
    assert_eq!(
        create_item_body["menu_item"]["reference_url"],
        "http://xhslink.com/o/3shicEMk8TL"
    );

    let menu_item_id = create_item_body["menu_item"]["id"]
        .as_str()
        .expect("menu item id");
    let (update_item_status, update_item_body) = request_json(
        &app,
        Method::PATCH,
        &format!("/api/menu-items/{menu_item_id}"),
        Some(user_token),
        json!({
            "updated_by": participant_id,
            "quantity": 3,
            "status": "done"
        }),
    )
    .await;
    assert_eq!(update_item_status, StatusCode::OK);
    assert_eq!(update_item_body["menu_item"]["quantity"], 3);
    assert_eq!(update_item_body["menu_item"]["status"], "done");

    let (activity_status, activity_body) = request_empty(
        &app,
        Method::GET,
        &format!("/api/gatherings/{gathering_id}/activity-logs"),
        Some(user_token),
    )
    .await;
    assert_eq!(activity_status, StatusCode::OK);
    let actions = activity_body["activity_logs"]
        .as_array()
        .expect("activity logs should be an array")
        .iter()
        .map(|log| log["action"].as_str().unwrap_or_default())
        .collect::<Vec<_>>();
    assert!(actions.contains(&"menu_item_created"));
    assert!(actions.contains(&"menu_item_quantity_changed"));
    assert!(actions.contains(&"menu_item_status_changed"));

    let (forbidden_status, _) = request_empty(
        &app,
        Method::POST,
        &format!("/api/gatherings/{gathering_id}/lock"),
        Some(user_token),
    )
    .await;
    assert_eq!(forbidden_status, StatusCode::FORBIDDEN);

    let (lock_status, lock_body) = request_empty(
        &app,
        Method::POST,
        &format!("/api/gatherings/{gathering_id}/lock"),
        Some(&admin_token),
    )
    .await;
    assert_eq!(lock_status, StatusCode::OK);
    assert_eq!(lock_body["gathering"]["is_locked"], true);
}

#[tokio::test]
async fn photo_admin_controls_and_token_lifecycle_flow() {
    let (app, pool) = test_app().await;
    let admin_token = login_admin(&app).await;
    let gathering =
        create_gathering(&app, &admin_token, "Photo test", "2099-07-06T12:00:00Z").await;
    let gathering_id = gathering["id"].as_str().expect("gathering id");
    let user_response = register_user(&app, "Mia", gathering_id).await;
    let user_token = user_response["token"].as_str().expect("user token");

    let boundary = "letsorder-test-boundary";
    let multipart_body = format!(
        "--{boundary}\r\n\
Content-Disposition: form-data; name=\"file\"; filename=\"memory.png\"\r\n\
Content-Type: image/png\r\n\r\n\
fake-image-bytes\r\n\
--{boundary}--\r\n"
    );
    let upload_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri(format!("/api/gatherings/{gathering_id}/photos"))
                .header(header::AUTHORIZATION, format!("Bearer {user_token}"))
                .header(
                    header::CONTENT_TYPE,
                    format!("multipart/form-data; boundary={boundary}"),
                )
                .body(Body::from(multipart_body))
                .expect("upload request should build"),
        )
        .await
        .expect("upload request should complete");
    assert_eq!(upload_response.status(), StatusCode::OK);
    let upload_body: Value = serde_json::from_slice(
        &to_bytes(upload_response.into_body(), usize::MAX)
            .await
            .expect("upload body should read"),
    )
    .expect("upload response should be json");
    assert_eq!(upload_body["photo"]["caption"], "Image");
    let photo_id = upload_body["photo"]["id"].as_str().expect("photo id");

    let (forbidden_update_status, _) = request_json(
        &app,
        Method::PATCH,
        &format!("/api/photos/{photo_id}"),
        Some(user_token),
        json!({ "caption": "Dinner table" }),
    )
    .await;
    assert_eq!(forbidden_update_status, StatusCode::FORBIDDEN);

    let (update_status, update_body) = request_json(
        &app,
        Method::PATCH,
        &format!("/api/photos/{photo_id}"),
        Some(&admin_token),
        json!({ "caption": "Dinner table" }),
    )
    .await;
    assert_eq!(update_status, StatusCode::OK);
    assert_eq!(update_body["photo"]["caption"], "Dinner table");

    let (delete_status, _) = request_empty(
        &app,
        Method::DELETE,
        &format!("/api/photos/{photo_id}"),
        Some(&admin_token),
    )
    .await;
    assert_eq!(delete_status, StatusCode::OK);

    sqlx::query("UPDATE auth_sessions SET expires_at = ? WHERE token = ?")
        .bind("2000-01-01T00:00:00Z")
        .bind(user_token)
        .execute(&pool)
        .await
        .expect("session should update");
    let (expired_status, _) =
        request_empty(&app, Method::GET, "/api/auth/me", Some(user_token)).await;
    assert_eq!(expired_status, StatusCode::UNAUTHORIZED);

    let second_admin_token = login_admin(&app).await;
    let (logout_status, _) = request_empty(
        &app,
        Method::POST,
        "/api/auth/logout",
        Some(&second_admin_token),
    )
    .await;
    assert_eq!(logout_status, StatusCode::NO_CONTENT);
    let (revoked_status, _) =
        request_empty(&app, Method::GET, "/api/auth/me", Some(&second_admin_token)).await;
    assert_eq!(revoked_status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn scheduled_locking_only_locks_oldest_unlocked_gatherings() {
    let (app, pool) = test_app().await;
    let admin_token = login_admin(&app).await;

    for index in 0..12 {
        let expires_at = format!("2020-01-{:02}T00:00:00Z", index + 1);
        create_gathering(
            &app,
            &admin_token,
            &format!("Expired gathering {index}"),
            &expires_at,
        )
        .await;
    }

    let locked = gathering_service::lock_expired_gatherings(&pool, 10)
        .await
        .expect("expired gatherings should lock");
    assert_eq!(locked.len(), 10);

    let locked_count: (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM gatherings WHERE is_locked = 1")
            .fetch_one(&pool)
            .await
            .expect("locked count should query");
    let unlocked_count: (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM gatherings WHERE is_locked = 0")
            .fetch_one(&pool)
            .await
            .expect("unlocked count should query");

    assert_eq!(locked_count.0, 10);
    assert_eq!(unlocked_count.0, 2);
}
