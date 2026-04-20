use super::*;
use crate::{
    error::ApiError,
    validation::{EditContact, NewContact},
};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::patch,
};
use libs::domain::manager;
use tracing::{instrument, debug, info};
use validator::Validate;

pub fn create_router(state: ApiState) -> Router {
    Router::new()
        .route("/contacts", get(list_contacts).post(add_contact))
        .route("/contacts/:id", patch(edit_contact).delete(delete_contact))
        .with_state(state)
}

#[instrument(
    level = "debug",
    skip(state),
    fields(contacts = tracing::field::Empty),
    err,
)]
#[axum::debug_handler]
async fn list_contacts(State(state): State<ApiState>) -> Result<impl IntoResponse, ApiError> {
    let contact_list: Vec<Contact> = {
        info!("generating contact list");
        debug!("acquring read Lock on manager state");
        let manager = state.manager.read().await;
        debug!("read Lock acquired");
        let contacts = manager.mem.values().cloned().collect();
        debug!("read Lock Released");
        contacts
    };
    tracing::Span::current().record("contact", contact_list.len());

    let res_body = Json(contact_list);
    info!("generated contact list");
    Ok((StatusCode::OK, res_body))
}

#[instrument(
    level = "debug",
    skip(state),
    fields(contact_name = %payload.name),
    err,
    ret,
)]
#[axum::debug_handler]
async fn add_contact(
    State(state): State<ApiState>,
    Json(payload): Json<NewContact>,
) -> Result<impl IntoResponse, ApiError> {
    info!("creating new contact");
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
        debug!("acquiring Write Lock on manager state");
        let mut manager = state.manager.write().await;
        debug!("acquired write Lock on manager state");

        debug!("synchronizing local data from storage");
        let mut base = manager.mem.clone();

        // This function synchronizes latest data from its own storage incase other process (e.g cli)
        // has updated the storage
        manager
            .sync_from_contacts_map(
                &mut base,
                manager.storage.load().await?,
                manager::SyncPolicy::LastWriteWinsPolicy(manager::LastWriteWinsPolicy),
            )
            .await?;
        debug!("synchronization complete");

        if new_contact.already_exist(&manager.contact_list()) {
            info!(name=%new_contact.name, phone=%new_contact.phone, "rejected: contact already exists");
            return Err(ApiError::BadRequest("Contact already exist".to_string()));
        }
        manager.add_contact(new_contact.clone());
        manager.save().await?;
        debug!("write Lock Released");
    }

    info!(user = ?new_contact, "created new contact");
    let body = Json(new_contact);
    Ok((StatusCode::CREATED, body))
}

#[instrument(
    level = "debug",
    skip(state),
    fields(contact_id = %id),
    err,
    ret,
)]
async fn edit_contact(
    Path(id): Path<Uuid>,
    State(state): State<ApiState>,
    Json(payload): Json<EditContact>,
) -> Result<impl IntoResponse, ApiError> {
    info!("editing a contact");
    payload.validate()?;

    let updated_contact = Json({
        debug!("acquiring Write Lock on manager state");
        let mut manager = state.manager.write().await;
        debug!("acquired write Lock on manager state");

        debug!("synchronizing local data from storage");
        let mut base = manager.mem.clone();

        // This function synchronizes latest data from its own storage incase other process (e.g cli)
        // has updated the storage
        manager
            .sync_from_contacts_map(
                &mut base,
                manager.storage.load().await?,
                manager::SyncPolicy::LastWriteWinsPolicy(manager::LastWriteWinsPolicy),
            )
            .await?;
        debug!("synchronization complete");

        let target = manager.mem.get(&id);
        info!(initial_data = ?target, "editing target contact");

        manager
            .edit_contact(&id, payload.name, payload.phone, payload.email, payload.tag)
            .map_err(|_| {
                info!(contact_id=%id, "contact not found");
                ApiError::NotFound
            })?;

        manager.save().await?;
        let contact = manager.mem.get(&id).unwrap().clone();

        debug!("write Lock Released");
        contact
    });
    info!(new_data = ?updated_contact, "editing complete");
    Ok((StatusCode::OK, updated_contact))
}

#[instrument(
    level = "debug",
    skip(state),
    fields(contact_id = %id),
    err,
    ret,
)]
async fn delete_contact(
    Path(id): Path<Uuid>,
    State(state): State<ApiState>,
) -> Result<impl IntoResponse, ApiError> {
    info!("deleting a contact");

    let deleted_contact = Json({
        debug!("acquiring Write Lock on manager state");
        let mut manager = state.manager.write().await;
        debug!("acquired write Lock on manager state");

        debug!("synchronizing local data from storage");
        // This function synchronizes latest data from its own storage incase other process (e.g cli)
        // has updated the storage
        manager
            .sync_from_contacts_map(
                &mut manager.mem.clone(),
                manager.storage.load().await?,
                manager::SyncPolicy::LastWriteWinsPolicy(manager::LastWriteWinsPolicy),
            )
            .await?;
        debug!("synchronization complete");

        let target_contact = manager.mem.get(&id).ok_or_else(|| {
            info!(contact_id=%id, "contact not found");
            ApiError::NotFound
        })?.clone();
        
        info!(target_contact = ?target_contact, "deleting target contact");
        manager.delete_contact(&id).ok();
        manager.save().await?;

        debug!("write Lock Released");
        target_contact
    });

    info!(?deleted_contact, "delete complete");
    Ok((StatusCode::OK, deleted_contact))
}
