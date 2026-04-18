mod contacts;

pub use crate::state::ApiState;
pub use axum::{Json, Router, response::IntoResponse, routing::get};
use libs::{
    domain::manager::SyncPolicy,
    errors::AppError,
    prelude::{Contact, ContactManager, uuid::Uuid},
};
pub use serde_json::json;
use std::collections::HashMap;
use tokio::sync::RwLockWriteGuard;

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

async fn sync_updates_from_storage_data<'a>(
    base: HashMap<Uuid, Contact>,
    manager: &mut RwLockWriteGuard<'a, ContactManager>,
    policy: SyncPolicy,
) -> Result<(), AppError> {
    let mut base = base;
    let rt = tokio::runtime::Runtime::new().map_err(|e| AppError::Io(e))?;
    rt.block_on(manager.sync_from_contacts_map(&mut base, manager.storage.load().await?, policy))
}
