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
    token: Option<String>,
}

pub async fn websocket(
    State(state): State<AppState>,
    Query(query): Query<WsQuery>,
    ws: WebSocketUpgrade,
) -> AppResult<Response> {
    let token = query
        .token
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or(AppError::Unauthorized)?;

    auth_service::me(&state.pool, token).await?;

    Ok(ws.on_upgrade(move |socket| handle_socket(socket, state)))
}

async fn handle_socket(mut socket: WebSocket, state: AppState) {
    let mut receiver = state.realtime_tx.subscribe();

    while let Ok(event) = receiver.recv().await {
        let Ok(message) = serde_json::to_string(&event) else {
            continue;
        };

        if socket.send(Message::Text(message.into())).await.is_err() {
            break;
        }
    }
}
