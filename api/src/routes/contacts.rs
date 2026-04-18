use super::*;
use crate::{
    error::ApiError,
    validation::{EditContact, NewContact},
};
use axum::{
    extract::{Path, State},
    http::StatusCode, routing::patch,
};
use libs::domain::manager;
use validator::Validate;

pub fn create_router(state: ApiState) -> Router {
    Router::new()
        .route("/contacts", get(list_contacts).post(add_contact))
        .route("/contacts/:id", patch(edit_contact).delete(delete_contact))
        .with_state(state)
}

#[axum::debug_handler]
async fn list_contacts(State(state): State<ApiState>) -> Result<impl IntoResponse, ApiError> {
    let contact_list: Vec<Contact> = {
        let manager = state.manager.blocking_read();
        manager.mem.values().cloned().collect()
    };

    let res_body = Json(contact_list);
    Ok((StatusCode::OK, res_body))
}

#[axum::debug_handler]
async fn add_contact(
    State(state): State<ApiState>,
    Json(payload): Json<NewContact>,
) -> Result<impl IntoResponse, ApiError> {
    payload.validate()?;

    let mut email = String::new();
    let mut tag = String::new();

    if let Some(req_email) = payload.email {
        email = req_email;
    }
    if let Some(req_tag) = payload.tag {
        tag = req_tag;
    }

    let new_contact = Contact::new(payload.name, payload.phone, email, tag);

    {
        let mut manager = state.manager.blocking_write();
        let base = manager.mem.clone();
        sync_updates_from_storage_data(
            base,
            &mut manager,
            manager::SyncPolicy::LastWriteWinsPolicy(manager::LastWriteWinsPolicy),
        )
        .await?;

        if new_contact.already_exist(&manager.contact_list()) {
            return Err(ApiError::BadRequest("Contact already exist".to_string()));
        }
        manager.add_contact(new_contact.clone());
        manager.save().await?
    }

    let body = Json(new_contact);
    Ok((StatusCode::CREATED, body))
}

async fn edit_contact(
    Path(id): Path<Uuid>,
    State(state): State<ApiState>,
    Json(payload): Json<EditContact>,
) -> Result<impl IntoResponse, ApiError> {
    payload.validate()?;

    let updated_contact = Json({
        let mut manager = state.manager.blocking_write();
        let base = manager.mem.clone();

        sync_updates_from_storage_data(
            base,
            &mut manager,
            manager::SyncPolicy::LastWriteWinsPolicy(manager::LastWriteWinsPolicy),
        )
        .await?;

        manager.edit_contact(
            &id, payload.name, payload.phone, 
            payload.email, payload.tag
        ).map_err(|_| ApiError::NotFound)?;

        manager.save().await?;
        manager.mem.get(&id).unwrap().clone()
    });

    Ok((StatusCode::OK, updated_contact))
}

async fn delete_contact(
    Path(id): Path<Uuid>,
    State(state): State<ApiState>,
) -> Result<impl IntoResponse, ApiError> {
    let deleted_contact = Json({
        let mut manager = state.manager.blocking_write();
        sync_updates_from_storage_data(manager.mem.clone(), 
            &mut manager, manager::SyncPolicy::LastWriteWinsPolicy(manager::LastWriteWinsPolicy)
        )
        .await?;

        let target_contact = manager.mem.get(&id).ok_or(ApiError::NotFound)?.clone();
        manager.delete_contact(&id).ok();
        manager.save().await?;

        target_contact
    });

    Ok((StatusCode::OK, deleted_contact))
}
