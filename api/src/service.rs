use libs::{
    domain::manager::{LastWriteWinsPolicy, SyncPolicy},
    prelude::{Contact, ContactManager, uuid::Uuid},
};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info};

use crate::error::ApiError;

pub struct ContactService {
    manager: Arc<RwLock<ContactManager>>,
}

impl ContactService {
    pub fn new(manager: Arc<RwLock<ContactManager>>) -> Self {
        Self { manager }
    }

    pub async fn list_contacts(&self) -> Result<Vec<Contact>, ApiError> {
        debug!("acquring read Lock on manager state");
        let manager = self.manager.read().await;
        debug!("read Lock acquired");

        Ok(manager
            .mem
            .values()
            .filter(|c| !c.deleted)
            .cloned()
            .collect())
    }

    pub async fn get_contact(&self, id: Uuid) -> Result<Contact, ApiError> {
        debug!("acquiring Read Lock on manager state");
        let manager = self.manager.read().await;
        debug!("acquired Read Lock on manager state");

        manager.mem.get(&id).filter(|c| !c.deleted).cloned().ok_or(ApiError::NotFound)
    }

    pub async fn add_contact(&self, new_contact: Contact) -> Result<Contact, ApiError> {
        debug!("acquiring Write Lock on manager state");
        let mut manager = self.manager.write().await;
        debug!("acquired write Lock on manager state");

        debug!("synchronizing local data from storage");
        // This function synchronizes latest data from its own storage incase other process (e.g cli)
        // has updated the storage
        self.sync(&mut manager).await?;
        debug!("synchronization complete");

        if new_contact.already_exist(&manager.contact_list()) {
            info!(name=%new_contact.name, phone=%new_contact.phone, "rejected: contact already exists");
            return Err(ApiError::BadRequest("Contact already exist".to_string()));
        }
        manager.add_contact(new_contact.clone());
        manager.save().await?;
        debug!("write Lock Released");
        Ok(new_contact)
    }

    pub async fn edit_contact(
        &self,
        id: Uuid,
        name: Option<String>,
        phone: Option<String>,
        email: Option<String>,
        tag: Option<String>,
    ) -> Result<Contact, ApiError> {
        debug!("acquiring Write Lock on manager state");
        let mut manager = self.manager.write().await;
        debug!("acquired write Lock on manager state");

        debug!("synchronizing local data from storage");
        // This function synchronizes latest data from its own storage incase other process (e.g cli)
        // has updated the storage
        self.sync(&mut manager).await?;
        debug!("synchronization complete");

        let target = manager.mem.get(&id);
        info!(initial_data = ?target, "editing target contact");

        manager
            .edit_contact(&id, name, phone, email, tag)
            .map_err(|_| {
                info!(contact_id=%id, "contact not found");
                ApiError::NotFound
            })?;

        manager.save().await?;
        let contact = manager.mem.get(&id).ok_or(ApiError::NotFound)?.clone();

        debug!("write Lock Released");
        Ok(contact)
    }

    pub async fn delete_contact(&self, id: Uuid) -> Result<Contact, ApiError> {
        debug!("acquiring Write Lock on manager state");
        let mut manager = self.manager.write().await;
        debug!("acquired write Lock on manager state");

        debug!("synchronizing local data from storage");
        // This function synchronizes latest data from its own storage incase other process (e.g cli)
        // has updated the storage
        self.sync(&mut manager).await?;
        debug!("synchronization complete");

        let target_contact = manager
            .delete_contact(&id)
            .map_err(|_| ApiError::NotFound)?;
        manager.save().await?;

        debug!("write Lock Released");
        Ok(target_contact)
    }

    pub async fn sync(&self, manager: &mut ContactManager) -> Result<(), ApiError> {
        let mut base = manager.mem.clone();
        manager
            .sync_from_contacts_map(
                &mut base,
                manager.storage.load().await?,
                SyncPolicy::LastWriteWinsPolicy(LastWriteWinsPolicy),
            )
            .await?;
        Ok(())
    }
}
