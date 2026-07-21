use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("resource not found")]
    NotFound,
    #[error("authentication is required")]
    Unauthorized,
    #[error("request is not allowed")]
    Forbidden,
    #[error("too many login attempts; try again later")]
    RateLimited,
    #[error("validation failed: {0}")]
    Validation(String),
    #[error("conflict")]
    Conflict(serde_json::Value),
    #[error(transparent)]
    Database(#[from] sqlx::Error),
}

#[derive(Debug, Serialize)]
struct ErrorBody {
    error: String,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = match self {
            AppError::NotFound => StatusCode::NOT_FOUND,
            AppError::Unauthorized => StatusCode::UNAUTHORIZED,
            AppError::Forbidden => StatusCode::FORBIDDEN,
            AppError::RateLimited => StatusCode::TOO_MANY_REQUESTS,
            AppError::Validation(_) => StatusCode::BAD_REQUEST,
            AppError::Conflict(_) => StatusCode::CONFLICT,
            AppError::Database(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };

        if let AppError::Conflict(body) = self {
            return (status, Json(body)).into_response();
        }

        let body = Json(ErrorBody {
            error: self.to_string(),
        });

        (status, body).into_response()
    }
}

pub type AppResult<T> = Result<T, AppError>;
