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

        manager
            .mem
            .get(&id)
            .filter(|c| !c.deleted)
            .cloned()
            .ok_or(ApiError::NotFound)
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

#[cfg(test)]
mod tests {
    use super::*;
    use libs::prelude::{ContactManager, JsonStorage};
    use std::collections::HashMap;
    use tempfile::tempdir;

    fn create_test_manager() -> (ContactService, tempfile::TempDir) {
        let dir = tempdir().unwrap();
        let path = dir.path().join("contacts.json");
        let path = path.to_string_lossy();
        let storage = Box::new(JsonStorage {
            medium: "json".to_string(),
            path: path.to_string(),
        });
        let mut manager = ContactManager {
            mem: HashMap::new(),
            storage,
            index: libs::domain::manager::Index {
                name: HashMap::new(),
                domain: HashMap::new(),
            },
        };
        let index = libs::domain::manager::Index::new(&manager).unwrap();
        manager.index = index;

        let manager = Arc::new(RwLock::new(manager));
        let service = ContactService::new(manager);
        (service, dir)
    }

    // BASIC FUNCTIONALITY TESTS

    #[tokio::test]
    async fn test_list_contacts_empty() {
        let (service, _dir) = create_test_manager();
        let contacts = service.list_contacts().await.unwrap();
        assert_eq!(contacts.len(), 0);
    }

    #[tokio::test]
    async fn test_add_contact_success() {
        let (service, _dir) = create_test_manager();

        let contact = Contact::new(
            "John Doe".to_string(),
            "5551234567".to_string(),
            "john@example.com".to_string(),
            "colleague".to_string(),
        );

        let result = service.add_contact(contact.clone()).await;
        assert!(result.is_ok());

        let added = result.unwrap();
        assert_eq!(added.name, "John Doe");
        assert_eq!(added.phone, "5551234567");
        assert_eq!(added.email, "john@example.com");
        assert_eq!(added.tag, "colleague");
        assert!(!added.deleted);
    }

    #[tokio::test]
    async fn test_list_contacts_after_add() {
        let (service, _dir) = create_test_manager();

        let contact = Contact::new(
            "Alice".to_string(),
            "5551111111".to_string(),
            String::new(),
            String::new(),
        );

        service.add_contact(contact).await.unwrap();

        let contacts = service.list_contacts().await.unwrap();
        assert_eq!(contacts.len(), 1);
        assert_eq!(contacts[0].name, "Alice");
    }

    #[tokio::test]
    async fn test_add_duplicate_contact() {
        let (service, _dir) = create_test_manager();

        let contact = Contact::new(
            "John Doe".to_string(),
            "5551234567".to_string(),
            String::new(),
            String::new(),
        );

        let result1 = service.add_contact(contact.clone()).await;
        assert!(result1.is_ok());

        let result2 = service.add_contact(contact).await;
        assert!(result2.is_err());

        if let Err(ApiError::BadRequest(msg)) = result2 {
            assert!(msg.contains("already exist"));
        } else {
            panic!("Expected BadRequest error");
        }
    }

    #[tokio::test]
    async fn test_get_contact_success() {
        let (service, _dir) = create_test_manager();

        let contact = Contact::new(
            "Bob".to_string(),
            "5552222222".to_string(),
            String::new(),
            String::new(),
        );

        let added = service.add_contact(contact).await.unwrap();
        let retrieved = service.get_contact(added.id).await.unwrap();

        assert_eq!(retrieved.id, added.id);
        assert_eq!(retrieved.name, "Bob");
    }

    #[tokio::test]
    async fn test_get_contact_not_found() {
        let (service, _dir) = create_test_manager();

        let fake_id = Uuid::new_v4();
        let result = service.get_contact(fake_id).await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ApiError::NotFound));
    }

    #[tokio::test]
    async fn test_edit_contact_success() {
        let (service, _dir) = create_test_manager();

        let contact = Contact::new(
            "Original Name".to_string(),
            "5551234567".to_string(),
            "original@example.com".to_string(),
            "old".to_string(),
        );

        let added = service.add_contact(contact).await.unwrap();

        let updated = service
            .edit_contact(
                added.id,
                Some("Updated Name".to_string()),
                None,
                Some("updated@example.com".to_string()),
                None,
            )
            .await
            .unwrap();

        assert_eq!(updated.name, "Updated Name");
        assert_eq!(updated.phone, "5551234567");
        assert_eq!(updated.email, "updated@example.com");
        assert_eq!(updated.tag, "old");
    }

    #[tokio::test]
    async fn test_edit_contact_partial_update() {
        let (service, _dir) = create_test_manager();

        let contact = Contact::new(
            "Name".to_string(),
            "5551234567".to_string(),
            "email@example.com".to_string(),
            "tag".to_string(),
        );

        let added = service.add_contact(contact).await.unwrap();

        // Only update email
        let updated = service
            .edit_contact(
                added.id,
                None,
                None,
                Some("newemail@example.com".to_string()),
                None,
            )
            .await
            .unwrap();

        assert_eq!(updated.name, "Name");
        assert_eq!(updated.phone, "5551234567");
        assert_eq!(updated.email, "newemail@example.com");
        assert_eq!(updated.tag, "tag");
    }

    #[tokio::test]
    async fn test_edit_contact_not_found() {
        let (service, _dir) = create_test_manager();

        let fake_id = Uuid::new_v4();
        let result = service
            .edit_contact(fake_id, Some("Name".to_string()), None, None, None)
            .await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ApiError::NotFound));
    }

    #[tokio::test]
    async fn test_delete_contact_success() {
        let (service, _dir) = create_test_manager();

        let contact = Contact::new(
            "To Delete".to_string(),
            "5559999999".to_string(),
            String::new(),
            String::new(),
        );

        let added = service.add_contact(contact).await.unwrap();
        let deleted = service.delete_contact(added.id).await.unwrap();

        assert_eq!(deleted.id, added.id);
        assert!(deleted.deleted);
    }

    #[tokio::test]
    async fn test_delete_contact_not_found() {
        let (service, _dir) = create_test_manager();

        let fake_id = Uuid::new_v4();
        let result = service.delete_contact(fake_id).await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ApiError::NotFound));
    }

    // SOFT-DELETE EDGE CASES

    #[tokio::test]
    async fn test_deleted_contact_not_in_list() {
        let (service, _dir) = create_test_manager();

        let contact = Contact::new(
            "To Remove".to_string(),
            "5553333333".to_string(),
            String::new(),
            String::new(),
        );

        let added = service.add_contact(contact).await.unwrap();
        let id = added.id;

        // Confirm contact is in list
        let list = service.list_contacts().await.unwrap();
        assert_eq!(list.len(), 1);
        assert!(list.iter().any(|c| c.id == id));

        // Delete the contact
        service.delete_contact(id).await.unwrap();

        // Confirm contact is NOT in list after deletion
        let list = service.list_contacts().await.unwrap();
        assert_eq!(list.len(), 0);
        assert!(!list.iter().any(|c| c.id == id));
    }

    #[tokio::test]
    async fn test_get_deleted_contact_returns_not_found() {
        let (service, _dir) = create_test_manager();

        let contact = Contact::new(
            "Soft Deleted".to_string(),
            "5554444444".to_string(),
            String::new(),
            String::new(),
        );

        let added = service.add_contact(contact).await.unwrap();
        let id = added.id;

        // Delete the contact
        service.delete_contact(id).await.unwrap();

        // Get the deleted contact - should return NotFound (filters out deleted)
        let result = service.get_contact(id).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ApiError::NotFound));
    }

    #[tokio::test]
    async fn test_edit_deleted_contact_fails() {
        let (service, _dir) = create_test_manager();

        let contact = Contact::new(
            "Will Delete".to_string(),
            "5555555555".to_string(),
            String::new(),
            String::new(),
        );

        let added = service.add_contact(contact).await.unwrap();
        let id = added.id;

        // Delete the contact
        service.delete_contact(id).await.unwrap();

        // Try to edit deleted contact - should fail
        let result = service
            .edit_contact(id, Some("New Name".to_string()), None, None, None)
            .await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ApiError::NotFound));
    }

    #[tokio::test]
    async fn test_multiple_contacts_with_deleted() {
        let (service, _dir) = create_test_manager();

        let contact1 = Contact::new(
            "Contact 1".to_string(),
            "5551111111".to_string(),
            String::new(),
            String::new(),
        );
        let contact2 = Contact::new(
            "Contact 2".to_string(),
            "5552222222".to_string(),
            String::new(),
            String::new(),
        );
        let contact3 = Contact::new(
            "Contact 3".to_string(),
            "5553333333".to_string(),
            String::new(),
            String::new(),
        );

        let c1 = service.add_contact(contact1).await.unwrap();
        let c2 = service.add_contact(contact2).await.unwrap();
        let c3 = service.add_contact(contact3).await.unwrap();

        // Confirm all 3 are in list
        let list = service.list_contacts().await.unwrap();
        assert_eq!(list.len(), 3);

        // Delete the middle one
        service.delete_contact(c2.id).await.unwrap();

        // Confirm only 2 remain in list
        let list = service.list_contacts().await.unwrap();
        assert_eq!(list.len(), 2);
        assert!(list.iter().any(|c| c.id == c1.id));
        assert!(!list.iter().any(|c| c.id == c2.id));
        assert!(list.iter().any(|c| c.id == c3.id));
    }

    #[tokio::test]
    async fn test_can_add_contact_with_same_details_after_delete() {
        let (service, _dir) = create_test_manager();

        let contact = Contact::new(
            "Reusable".to_string(),
            "5556666666".to_string(),
            String::new(),
            String::new(),
        );

        // Add contact
        let added1 = service.add_contact(contact.clone()).await.unwrap();

        // Delete it
        service.delete_contact(added1.id).await.unwrap();

        // Create a new contact with same details (this will have a different UUID)
        let contact2 = Contact::new(
            "Reusable".to_string(),
            "5556666666".to_string(),
            String::new(),
            String::new(),
        );

        // Should be able to add it again because the first one is soft-deleted
        let result = service.add_contact(contact2).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_list_filters_only_non_deleted() {
        let (service, _dir) = create_test_manager();

        // Add 5 contacts
        let mut contact_ids = Vec::new();
        for i in 1..=5 {
            let contact = Contact::new(
                format!("Contact {}", i),
                format!("555000000{}", i),
                String::new(),
                String::new(),
            );
            let added = service.add_contact(contact).await.unwrap();
            contact_ids.push((added.id, format!("Contact {}", i)));
        }

        let all_contacts = service.list_contacts().await.unwrap();
        assert_eq!(all_contacts.len(), 5);

        // Delete contacts at index 1 and 3 (Contact 2 and Contact 4)
        service.delete_contact(contact_ids[1].0).await.unwrap();
        service.delete_contact(contact_ids[3].0).await.unwrap();

        // List should only have 3
        let remaining = service.list_contacts().await.unwrap();
        assert_eq!(remaining.len(), 3);

        // Verify the correct ones are there
        let names: Vec<String> = remaining.iter().map(|c| c.name.clone()).collect();
        assert!(names.contains(&"Contact 1".to_string()));
        assert!(!names.contains(&"Contact 2".to_string()));
        assert!(names.contains(&"Contact 3".to_string()));
        assert!(!names.contains(&"Contact 4".to_string()));
        assert!(names.contains(&"Contact 5".to_string()));
    }
}
