use super::*;
use crate::{
    error::ApiError,
    validation::{EditContact, NewContact},
};
use axum::{
    extract::{Path, State},
    http::StatusCode,
};
use tracing::{debug, info, instrument};
use validator::Validate;

pub fn create_router(state: ApiState) -> Router {
    Router::new()
        .route("/contacts", get(list_contacts).post(add_contact))
        .route("/contacts/:id", get(get_contact_by_id).patch(edit_contact).delete(delete_contact))
        .with_state(state)
}

#[instrument(
    level = "debug",
    skip(state),
    fields(contact_id = %id),
    err,
    ret,
)]
async fn get_contact_by_id(
    Path(id): Path<Uuid>,
    State(state): State<ApiState>
) -> Result<impl IntoResponse, ApiError> {
    info!(contact_id=%id, "getting contact");

    let contact = state.service.get_contact(id).await.map_err(|_| ApiError::NotFound)?;
    Ok((StatusCode::OK, Json(contact)))
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
        
        let contacts = state.service.list_contacts().await?;
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

    let email = payload.email.unwrap_or_default();
    let tag = payload.tag.unwrap_or_default();

    let new_contact = 
        state.service.add_contact(
            Contact::new(payload.name, payload.phone, email, tag)
        ).await?;

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

    let updated_contact = state.service.edit_contact(
        id, payload.name,
        payload.phone, payload.email,
        payload.tag
    ).await?;
    info!(new_data = ?updated_contact, "editing complete");
    Ok((StatusCode::OK, Json(updated_contact)))
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

    let deleted_contact = state.service.delete_contact(id).await?;

    info!(?deleted_contact, "delete complete");
    Ok((StatusCode::OK, Json(deleted_contact)))
}
