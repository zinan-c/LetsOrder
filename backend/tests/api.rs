use axum::{
    Router,
    body::{Body, to_bytes},
    http::{Method, Request, StatusCode, header},
};
use letsorder_backend::{
    db, routes,
    services::{auth_service, gathering_service},
};
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
            "host_name": "Test Host",
            "expires_at": expires_at
        }),
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    let mut gathering = body["gathering"].clone();
    gathering["access_token"] = body["access_token"].clone();
    gathering
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
            .len()
            >= 12
    );
    body
}

async fn register_user_without_gathering(app: &Router, display_name: &str) -> Value {
    let (status, body) = request_json(
        app,
        Method::POST,
        "/api/auth/register",
        None,
        json!({
            "display_name": display_name
        }),
    )
    .await;

    assert_eq!(status, StatusCode::OK);
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

    let (anonymous_active_status, _) =
        request_empty(&app, Method::GET, "/api/gatherings/active", None).await;
    assert_eq!(anonymous_active_status, StatusCode::FORBIDDEN);

    let (admin_active_status, admin_active_body) = request_empty(
        &app,
        Method::GET,
        "/api/gatherings/active",
        Some(&admin_token),
    )
    .await;
    assert_eq!(admin_active_status, StatusCode::OK);
    assert_eq!(admin_active_body["gatherings"][0]["id"], gathering_id);

    let (reserved_host_status, reserved_host_body) = request_json(
        &app,
        Method::POST,
        "/api/gatherings",
        Some(&admin_token),
        json!({
            "title": "Reserved host name test",
            "description": "Should be rejected",
            "host_name": "suite-admin",
            "expires_at": "2099-07-06T12:00:00Z"
        }),
    )
    .await;
    assert_eq!(reserved_host_status, StatusCode::BAD_REQUEST);
    assert!(
        reserved_host_body["error"]
            .as_str()
            .expect("validation error should exist")
            .contains("系统管理员账号名称")
    );

    let (admin_join_status, admin_join_body) = request_json(
        &app,
        Method::POST,
        &format!("/api/gatherings/invite/{invite_code}/participants"),
        Some(&admin_token),
        json!({}),
    )
    .await;
    assert_eq!(admin_join_status, StatusCode::OK);
    assert!(admin_join_body["participant"].is_null());

    let (admin_participants_status, admin_participants_body) = request_empty(
        &app,
        Method::GET,
        &format!("/api/gatherings/{gathering_id}/participants"),
        Some(&admin_token),
    )
    .await;
    assert_eq!(admin_participants_status, StatusCode::OK);
    assert!(
        admin_participants_body["participants"]
            .as_array()
            .expect("participants should be an array")
            .iter()
            .all(|participant| participant["display_name"] != "suite-admin")
    );

    let (admin_activity_status, admin_activity_body) = request_empty(
        &app,
        Method::GET,
        &format!("/api/gatherings/{gathering_id}/activity-logs"),
        Some(&admin_token),
    )
    .await;
    assert_eq!(admin_activity_status, StatusCode::OK);
    assert!(
        admin_activity_body["activity_logs"]
            .as_array()
            .expect("activity logs should be an array")
            .iter()
            .all(|log| {
                log["action"] != "participant_joined" || log["actor_name"] != "suite-admin"
            })
    );

    let (anonymous_detail_status, _) = request_empty(
        &app,
        Method::GET,
        &format!("/api/gatherings/{invite_code}"),
        None,
    )
    .await;
    assert_eq!(anonymous_detail_status, StatusCode::FORBIDDEN);

    let unjoined_response = register_user_without_gathering(&app, "Alex").await;
    let unjoined_token = unjoined_response["token"].as_str().expect("unjoined token");
    let (unjoined_menu_status, _) = request_empty(
        &app,
        Method::GET,
        &format!("/api/gatherings/{gathering_id}/menu-items"),
        Some(unjoined_token),
    )
    .await;
    assert_eq!(unjoined_menu_status, StatusCode::FORBIDDEN);

    let (invite_join_status, invite_join_body) = request_json(
        &app,
        Method::POST,
        &format!("/api/gatherings/invite/{invite_code}/participants"),
        Some(unjoined_token),
        json!({}),
    )
    .await;
    assert_eq!(invite_join_status, StatusCode::OK);
    assert_eq!(invite_join_body["gathering"]["id"], gathering_id);

    let (joined_menu_status, _) = request_empty(
        &app,
        Method::GET,
        &format!("/api/gatherings/{gathering_id}/menu-items"),
        Some(unjoined_token),
    )
    .await;
    assert_eq!(joined_menu_status, StatusCode::OK);

    let user_response = register_user(&app, "Nico", gathering_id).await;
    let mut user_token = user_response["token"].as_str().expect("user token");
    let user_id = user_response["user"]["id"].as_str().expect("user id");
    let username = user_response["user"]["username"]
        .as_str()
        .expect("username");
    let participant_id = user_response["participant"]["id"]
        .as_str()
        .expect("participant id");

    let (forbidden_members_status, _) =
        request_empty(&app, Method::GET, "/api/auth/members", Some(user_token)).await;
    assert_eq!(forbidden_members_status, StatusCode::FORBIDDEN);

    let (reserved_name_status, reserved_name_body) = request_json(
        &app,
        Method::PATCH,
        "/api/auth/account",
        Some(user_token),
        json!({
            "display_name": "suite-admin"
        }),
    )
    .await;
    assert_eq!(reserved_name_status, StatusCode::BAD_REQUEST);
    assert!(
        reserved_name_body["error"]
            .as_str()
            .expect("reserved name error should exist")
            .contains("系统管理员账号名称")
    );

    let (members_status, members_body) =
        request_empty(&app, Method::GET, "/api/auth/members", Some(&admin_token)).await;
    assert_eq!(members_status, StatusCode::OK);
    assert!(
        members_body["members"]
            .as_array()
            .is_some_and(|members| { members.iter().any(|member| member["id"] == user_id) })
    );

    let (update_member_status, update_member_body) = request_json(
        &app,
        Method::PATCH,
        &format!("/api/auth/members/{user_id}"),
        Some(&admin_token),
        json!({
            "display_name": "Nico Chef",
            "password": "Nico_456"
        }),
    )
    .await;
    assert_eq!(update_member_status, StatusCode::OK);
    assert_eq!(update_member_body["member"]["display_name"], "Nico Chef");

    let (member_login_status, member_login_body) = request_json(
        &app,
        Method::POST,
        "/api/auth/login",
        None,
        json!({
            "username": username,
            "password": "Nico_456"
        }),
    )
    .await;
    assert_eq!(member_login_status, StatusCode::OK);
    assert_eq!(member_login_body["user"]["display_name"], "Nico Chef");
    user_token = member_login_body["token"].as_str().expect("member token");

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
            "owner_name": "Nico Chef",
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
    assert_eq!(create_item_body["menu_item"]["revision"], 1);

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
            "status": "done",
            "expected_revision": 1
        }),
    )
    .await;
    assert_eq!(update_item_status, StatusCode::OK);
    assert_eq!(update_item_body["menu_item"]["quantity"], 3);
    assert_eq!(update_item_body["menu_item"]["status"], "done");
    assert_eq!(update_item_body["menu_item"]["revision"], 2);

    let (create_second_item_status, create_second_item_body) = request_json(
        &app,
        Method::POST,
        &format!("/api/gatherings/{gathering_id}/menu-items"),
        Some(user_token),
        json!({
            "created_by": participant_id,
            "name": "Iced tea",
            "category": "Drink",
            "quantity": 2,
            "unit": "cups",
            "owner_name": "Nico Chef",
            "status": "done"
        }),
    )
    .await;
    assert_eq!(create_second_item_status, StatusCode::OK);
    let second_menu_item_id = create_second_item_body["menu_item"]["id"]
        .as_str()
        .expect("second menu item id");

    let (stale_update_status, stale_update_body) = request_json(
        &app,
        Method::PATCH,
        &format!("/api/menu-items/{menu_item_id}"),
        Some(user_token),
        json!({
            "updated_by": participant_id,
            "quantity": 5,
            "expected_revision": 1
        }),
    )
    .await;
    assert_eq!(stale_update_status, StatusCode::CONFLICT);
    assert_eq!(stale_update_body["latest_menu_item"]["revision"], 2);
    assert_eq!(stale_update_body["submitted"]["quantity"], 5);

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

    let (early_rate_status, _) = request_json(
        &app,
        Method::POST,
        &format!("/api/menu-items/{menu_item_id}/rating"),
        Some(user_token),
        json!({ "rating": 5 }),
    )
    .await;
    assert_eq!(early_rate_status, StatusCode::FORBIDDEN);

    let (lock_status, lock_body) = request_empty(
        &app,
        Method::POST,
        &format!("/api/gatherings/{gathering_id}/lock"),
        Some(&admin_token),
    )
    .await;
    assert_eq!(lock_status, StatusCode::OK);
    assert_eq!(lock_body["gathering"]["is_locked"], true);

    let (ratings_status, ratings_body) = request_empty(
        &app,
        Method::GET,
        &format!("/api/gatherings/{gathering_id}/menu-ratings"),
        Some(user_token),
    )
    .await;
    assert_eq!(ratings_status, StatusCode::OK);
    assert_eq!(ratings_body["ratings"][0]["menu_item_id"], menu_item_id);
    assert_eq!(ratings_body["ratings"][0]["rating_count"], 0);
    assert!(ratings_body["ratings"][0]["average_rating"].is_null());

    let (rate_status, rate_body) = request_json(
        &app,
        Method::POST,
        &format!("/api/menu-items/{menu_item_id}/rating"),
        Some(user_token),
        json!({ "rating": 5 }),
    )
    .await;
    assert_eq!(rate_status, StatusCode::OK);
    assert_eq!(rate_body["rating"]["menu_item_id"], menu_item_id);
    assert_eq!(rate_body["rating"]["rating_count"], 1);
    assert_eq!(rate_body["rating"]["my_rating"], 5);
    assert_eq!(rate_body["rating"]["average_rating"], 5.0);

    let (second_rate_status, second_rate_body) = request_json(
        &app,
        Method::POST,
        &format!("/api/menu-items/{second_menu_item_id}/rating"),
        Some(user_token),
        json!({ "rating": 3 }),
    )
    .await;
    assert_eq!(second_rate_status, StatusCode::OK);
    assert_eq!(second_rate_body["rating"]["average_rating"], 3.0);

    let (empty_recommendation_status, empty_recommendation_body) = request_empty(
        &app,
        Method::GET,
        "/api/chefs/Test%20Host/dish-recommendations",
        Some(user_token),
    )
    .await;
    assert_eq!(empty_recommendation_status, StatusCode::OK);
    assert_eq!(
        empty_recommendation_body["recommendations"]
            .as_array()
            .expect("recommendations should be an array")
            .len(),
        0
    );

    let (recommendation_status, recommendation_body) = request_empty(
        &app,
        Method::GET,
        "/api/chefs/Nico%20Chef/dish-recommendations",
        Some(user_token),
    )
    .await;
    assert_eq!(recommendation_status, StatusCode::OK);
    assert_eq!(
        recommendation_body["recommendations"][0]["name"],
        "Cold brew"
    );
    assert_eq!(
        recommendation_body["recommendations"][0]["average_rating"],
        5.0
    );
    assert_eq!(
        recommendation_body["recommendations"][1]["name"],
        "Iced tea"
    );
    assert_eq!(
        recommendation_body["recommendations"][1]["average_rating"],
        3.0
    );

    let (invalid_rate_status, _) = request_json(
        &app,
        Method::POST,
        &format!("/api/menu-items/{menu_item_id}/rating"),
        Some(user_token),
        json!({ "rating": 6 }),
    )
    .await;
    assert_eq!(invalid_rate_status, StatusCode::BAD_REQUEST);

    let (rated_summary_status, rated_summary_body) = request_empty(
        &app,
        Method::GET,
        &format!("/api/gatherings/{gathering_id}/menu-ratings"),
        Some(user_token),
    )
    .await;
    assert_eq!(rated_summary_status, StatusCode::OK);
    assert_eq!(rated_summary_body["ratings"][0]["rating_count"], 1);
    assert_eq!(rated_summary_body["ratings"][0]["my_rating"], 5);
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

    let (lock_status, _) = request_empty(
        &app,
        Method::POST,
        &format!("/api/gatherings/{gathering_id}/lock"),
        Some(&admin_token),
    )
    .await;
    assert_eq!(lock_status, StatusCode::OK);

    let boundary = "letsorder-test-boundary";
    let invalid_multipart_body = format!(
        "--{boundary}\r\n\
Content-Disposition: form-data; name=\"file\"; filename=\"fake.png\"\r\n\
Content-Type: image/png\r\n\r\n\
fake-image-bytes\r\n\
--{boundary}--\r\n"
    );
    let invalid_upload_response = app
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
                .body(Body::from(invalid_multipart_body))
                .expect("invalid upload request should build"),
        )
        .await
        .expect("invalid upload request should complete");
    assert_eq!(invalid_upload_response.status(), StatusCode::BAD_REQUEST);

    let mut multipart_body = format!(
        "--{boundary}\r\n\
Content-Disposition: form-data; name=\"file\"; filename=\"memory.png\"\r\n\
Content-Type: image/png\r\n\r\n"
    )
    .into_bytes();
    multipart_body.extend_from_slice(&[
        0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a, 0x00, 0x00, 0x00, 0x0d, 0x49, 0x48, 0x44,
        0x52, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x04, 0x00, 0x00, 0x00, 0xb5,
        0x1c, 0x0c, 0x02, 0x00, 0x00, 0x00, 0x0b, 0x49, 0x44, 0x41, 0x54, 0x78, 0xda, 0x63, 0x64,
        0xf8, 0x0f, 0x00, 0x01, 0x05, 0x01, 0x01, 0x27, 0x18, 0xe3, 0x66, 0x00, 0x00, 0x00, 0x00,
        0x49, 0x45, 0x4e, 0x44, 0xae, 0x42, 0x60, 0x82,
    ]);
    multipart_body.extend_from_slice(format!("\r\n--{boundary}--\r\n").as_bytes());
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
    let photo_url = upload_body["photo"]["file_url"]
        .as_str()
        .expect("photo file url");
    let (photo_access_status, _) =
        request_empty(&app, Method::GET, photo_url, Some(user_token)).await;
    assert_eq!(photo_access_status, StatusCode::OK);
    let other_user = register_user_without_gathering(&app, "Private Photo Viewer").await;
    let (private_photo_status, _) =
        request_empty(&app, Method::GET, photo_url, other_user["token"].as_str()).await;
    assert_eq!(private_photo_status, StatusCode::FORBIDDEN);

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

#[tokio::test]
async fn host_claim_is_single_use_and_bound_to_authenticated_user() {
    let (app, _) = test_app().await;
    let admin_token = login_admin(&app).await;
    let gathering = create_gathering(
        &app,
        &admin_token,
        "Host claim test",
        "2099-07-06T12:00:00Z",
    )
    .await;
    let gathering_id = gathering["id"].as_str().expect("gathering id");
    let claim_token = gathering["access_token"].as_str().expect("claim token");

    let host_user = register_user(&app, "Host Person", gathering_id).await;
    let host_token = host_user["token"].as_str().expect("host token");
    let (claim_status, claim_body) = request_json(
        &app,
        Method::POST,
        &format!("/api/gatherings/{gathering_id}/host/claim"),
        Some(host_token),
        json!({ "claim_token": claim_token }),
    )
    .await;
    assert_eq!(claim_status, StatusCode::OK);
    assert_eq!(claim_body["participant"]["role"], "host");
    assert_eq!(
        claim_body["participant"]["user_id"],
        host_user["user"]["id"]
    );

    let (reused_status, _) = request_json(
        &app,
        Method::POST,
        &format!("/api/gatherings/{gathering_id}/host/claim"),
        Some(host_token),
        json!({ "claim_token": claim_token }),
    )
    .await;
    assert_eq!(reused_status, StatusCode::CONFLICT);

    let other_user = register_user(&app, "Host Person", gathering_id).await;
    let other_token = other_user["token"].as_str().expect("other token");
    let (other_claim_status, _) = request_json(
        &app,
        Method::POST,
        &format!("/api/gatherings/{gathering_id}/host/claim"),
        Some(other_token),
        json!({ "claim_token": claim_token }),
    )
    .await;
    assert_eq!(other_claim_status, StatusCode::CONFLICT);

    let (lock_status, _) = request_empty(
        &app,
        Method::POST,
        &format!("/api/gatherings/{gathering_id}/lock"),
        Some(host_token),
    )
    .await;
    assert_eq!(lock_status, StatusCode::OK);
}

#[tokio::test]
async fn login_attempts_are_rate_limited_per_username() {
    let (app, _) = test_app().await;
    for _ in 0..5 {
        let (status, _) = request_json(
            &app,
            Method::POST,
            "/api/auth/login",
            None,
            json!({ "username": "rate-limit-user", "password": "wrong" }),
        )
        .await;
        assert_eq!(status, StatusCode::FORBIDDEN);
    }

    let (blocked_status, _) = request_json(
        &app,
        Method::POST,
        "/api/auth/login",
        None,
        json!({ "username": "rate-limit-user", "password": "wrong" }),
    )
    .await;
    assert_eq!(blocked_status, StatusCode::TOO_MANY_REQUESTS);
}

#[tokio::test]
async fn websocket_ticket_is_consumed_once() {
    let (app, pool) = test_app().await;
    let admin_token = login_admin(&app).await;
    let ticket = auth_service::create_websocket_ticket(&pool, &admin_token)
        .await
        .expect("ticket should be created");

    let (first, second) = tokio::join!(
        auth_service::consume_websocket_ticket(&pool, &ticket),
        auth_service::consume_websocket_ticket(&pool, &ticket),
    );
    assert_eq!(first.is_ok() as u8 + second.is_ok() as u8, 1);
    assert!(
        auth_service::consume_websocket_ticket(&pool, &ticket)
            .await
            .is_err()
    );
}

#[tokio::test]
async fn archived_gathering_rejects_lock_and_deadline_mutations() {
    let (app, _) = test_app().await;
    let admin_token = login_admin(&app).await;
    let gathering = create_gathering(
        &app,
        &admin_token,
        "Archived mutation test",
        "2099-07-06T12:00:00Z",
    )
    .await;
    let gathering_id = gathering["id"].as_str().expect("gathering id");

    let (archive_status, _) = request_empty(
        &app,
        Method::DELETE,
        &format!("/api/gatherings/{gathering_id}"),
        Some(&admin_token),
    )
    .await;
    assert_eq!(archive_status, StatusCode::OK);

    let (lock_status, _) = request_empty(
        &app,
        Method::POST,
        &format!("/api/gatherings/{gathering_id}/lock"),
        Some(&admin_token),
    )
    .await;
    assert_eq!(lock_status, StatusCode::CONFLICT);

    let (deadline_status, _) = request_json(
        &app,
        Method::PATCH,
        &format!("/api/gatherings/{gathering_id}"),
        Some(&admin_token),
        json!({ "expires_at": "2099-07-07T12:00:00Z" }),
    )
    .await;
    assert_eq!(deadline_status, StatusCode::CONFLICT);
}

#[tokio::test]
async fn concurrent_join_requests_reuse_one_participant() {
    let (app, _) = test_app().await;
    let admin_token = login_admin(&app).await;
    let gathering = create_gathering(
        &app,
        &admin_token,
        "Concurrent join test",
        "2099-07-06T12:00:00Z",
    )
    .await;
    let gathering_id = gathering["id"].as_str().expect("gathering id");
    let user = register_user_without_gathering(&app, "Concurrent Guest").await;
    let user_token = user["token"].as_str().expect("user token");
    let participants_path = format!("/api/gatherings/{gathering_id}/participants");

    let (first, second) = tokio::join!(
        request_json(
            &app,
            Method::POST,
            &participants_path,
            Some(user_token),
            json!({}),
        ),
        request_json(
            &app,
            Method::POST,
            &participants_path,
            Some(user_token),
            json!({}),
        ),
    );
    assert_eq!(first.0, StatusCode::OK);
    assert_eq!(second.0, StatusCode::OK);
    assert_eq!(first.1["participant"]["id"], second.1["participant"]["id"]);
}

#[tokio::test]
async fn concurrent_registrations_allocate_distinct_usernames() {
    let (app, _) = test_app().await;
    let (first, second) = tokio::join!(
        request_json(
            &app,
            Method::POST,
            "/api/auth/register",
            None,
            json!({ "display_name": "Same Name" }),
        ),
        request_json(
            &app,
            Method::POST,
            "/api/auth/register",
            None,
            json!({ "display_name": "Same Name" }),
        ),
    );
    assert_eq!(first.0, StatusCode::OK);
    assert_eq!(second.0, StatusCode::OK);
    assert_ne!(first.1["user"]["username"], second.1["user"]["username"]);
}
