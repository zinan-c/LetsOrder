use axum::{
    extract::{
        Query, State,
        ws::{Message, WebSocket, WebSocketUpgrade},
    },
    response::Response,
};
use serde::Deserialize;

use crate::{
    errors::{AppError, AppResult},
    routes::AppState,
    services::auth_service,
};

#[derive(Debug, Deserialize)]
pub struct WsQuery {
    ticket: Option<String>,
}

pub async fn websocket(
    State(state): State<AppState>,
    Query(query): Query<WsQuery>,
    ws: WebSocketUpgrade,
) -> AppResult<Response> {
    let ticket = query
        .ticket
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or(AppError::Unauthorized)?;

    let user = auth_service::consume_websocket_ticket(&state.pool, ticket).await?;

    Ok(ws.on_upgrade(move |socket| handle_socket(socket, state, user.id, user.role)))
}

async fn handle_socket(mut socket: WebSocket, state: AppState, user_id: uuid::Uuid, role: String) {
    let mut receiver = state.realtime_tx.subscribe();

    loop {
        let event = match receiver.recv().await {
            Ok(event) => event,
            Err(tokio::sync::broadcast::error::RecvError::Lagged(skipped)) => {
                tracing::warn!(skipped, "websocket receiver lagged; continuing");
                continue;
            }
            Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
        };

        if let Some(gathering_id) = event.gathering_id
            && role != "admin"
        {
            let visible: Option<(String,)> = sqlx::query_as(
                "SELECT id FROM participants WHERE gathering_id = ? AND user_id = ? LIMIT 1",
            )
            .bind(gathering_id.to_string())
            .bind(user_id.to_string())
            .fetch_optional(&state.pool)
            .await
            .ok()
            .flatten();
            if visible.is_none() {
                continue;
            }
        }
        let Ok(message) = serde_json::to_string(&event) else {
            continue;
        };

        if socket.send(Message::Text(message.into())).await.is_err() {
            break;
        }
    }
}
