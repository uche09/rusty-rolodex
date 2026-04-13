use axum::{Json, http::StatusCode, response::IntoResponse, };
use serde_json::json;
use thiserror::Error;
use validator::ValidationErrors;


#[derive(Debug, Error)]
pub enum ApiError {
    #[error("Not found")]
    NotFound,

    #[error("Bad request: {0}")]
    BadRequest(String),

    #[error("Internal error")]
    InternalError(#[from] anyhow::Error),

    #[error("Validation error")]
    Validation(#[from] ValidationErrors),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        let (status, error_message) = match self {
            ApiError::NotFound => (
                StatusCode::NOT_FOUND, "Data not found".to_string()
            ),
            ApiError::BadRequest(msg) => (
                StatusCode::BAD_REQUEST, msg
            ),
            ApiError::InternalError(e) => {
                tracing::error!("Internal error: {:?}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Internal server error".to_string()
                )
            },
            ApiError::Validation(e) => (
                StatusCode::UNPROCESSABLE_ENTITY,
                e.to_string()
            )
        };

        let body = Json(json!({
            "error": error_message,
        }));

        (status, body).into_response()
    }
}
