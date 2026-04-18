mod contacts;

pub use crate::state::ApiState;
pub use axum::{Json, Router, response::IntoResponse, routing::get};
use libs::prelude::{Contact, uuid::Uuid};
pub use serde_json::json;

pub fn create_router(state: ApiState) -> Router {
    Router::new()
        .route("/health", get(health_check))
        .merge(contacts::create_router(state))
}

pub async fn health_check() -> impl IntoResponse {
    Json(json!({
        "status": "ok",
        "message": "Server is running",
    }))
}
