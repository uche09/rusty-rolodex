use chrono::{Duration, Utc};
use std::collections::HashMap;
use uuid::Uuid;

// Import the necessary types from rusty_rolodex
use rusty_rolodex::prelude::*;

/// Mock storage for testing synchronization scenarios
struct MockStorage {
    contacts: HashMap<Uuid, Contact>,
}

impl MockStorage {
    fn new(contacts: HashMap<Uuid, Contact>) -> Self {
        Self { contacts }
    }
}

impl ContactStore for MockStorage {
    fn load(&self) -> Result<HashMap<Uuid, Contact>, AppError> {
        Ok(self.contacts.clone())
    }

    fn save(&self, _contacts: &HashMap<Uuid, Contact>) -> Result<(), AppError> {
        // sync_from_storage() never calls save(), so this is a no-op for testing
        Ok(())
    }

    fn get_medium(&self) -> &str {
        "mock"
    }
}

// SCENARIO 1: Same contact edited in two places
//
// Laptop: changed phone number
// Phone: changed email
// Merge policy: field-by-field, last-write-wins

#[test]
fn sync_same_contact_different_fields_modified() -> Result<(), AppError> {
    // NOTE: With only contact-level updated_at, we cannot do field-by-field merging.
    // This test demonstrates contact-level last-write-wins:
    // whichever contact was updated last wins ALL fields.

    let contact_id = Uuid::new_v4();
    let base_time = Utc::now();

    let _original = Contact {
        id: contact_id,
        name: "John Doe".to_string(),
        phone: "1234567890".to_string(),
        email: "john@example.com".to_string(),
        tag: "work".to_string(),
        deleted: false,
        created_at: base_time,
        updated_at: base_time,
    };

    // Laptop modified phone at t1
    let laptop_time = base_time + Duration::seconds(10);
    let mut laptop_version = _original.clone();
    laptop_version.phone = "9876543210".to_string(); // Changed phone
    laptop_version.updated_at = laptop_time;

    // Phone modified email at t2 (LATER than laptop)
    let phone_time = base_time + Duration::seconds(20);
    let mut phone_version = _original.clone();
    phone_version.email = "john.doe@example.com".to_string(); // Changed email
    phone_version.updated_at = phone_time;

    // Local manager
    let mut local_manager = ContactManager::new()?;
    local_manager.add_contact(laptop_version);

    // Remote manager
    let mut remote_manager = ContactManager::new()?;
    remote_manager.add_contact(phone_version);

    // Sync: clone remote.mem into MockStorage
    let remote_storage = MockStorage::new(remote_manager.mem.clone());
    local_manager.sync_from_storage(Box::new(remote_storage))?;

    // Since remote has later timestamp (phone_time > laptop_time),
    // ALL remote fields win (including phone, which reverted)
    let synced = local_manager.mem.get(&contact_id).unwrap();
    assert_eq!(
        synced.phone, "1234567890",
        "Phone should be from remote (contact-level last-write-wins)"
    );
    assert_eq!(
        synced.email, "john.doe@example.com",
        "Email should be from remote (contact-level last-write-wins)"
    );
    assert_eq!(synced.updated_at, phone_time, "Timestamp should be latest");

    Ok(())
}

#[test]
fn sync_same_contact_same_field_last_write_wins() -> Result<(), AppError> {
    // Scenario: Both devices edited the phone field
    // Policy: Last-write-wins based on updated_at timestamp
    let contact_id = Uuid::new_v4();
    let base_time = Utc::now();

    let _original = Contact {
        id: contact_id,
        name: "Alice Smith".to_string(),
        phone: "5555555555".to_string(),
        email: "alice@example.com".to_string(),
        tag: "personal".to_string(),
        deleted: false,
        created_at: base_time,
        updated_at: base_time,
    };

    // Laptop edited phone (earlier)
    let laptop_time = base_time + Duration::seconds(5);
    let mut laptop_version = _original.clone();
    laptop_version.phone = "1111111111".to_string();
    laptop_version.updated_at = laptop_time;

    // Phone edited phone later (should win)
    let phone_time = base_time + Duration::seconds(15);
    let mut phone_version = _original.clone();
    phone_version.phone = "2222222222".to_string();
    phone_version.updated_at = phone_time;

    // Local manager
    let mut local_manager = ContactManager::new()?;
    local_manager.add_contact(laptop_version);

    // Remote manager
    let mut remote_manager = ContactManager::new()?;
    remote_manager.add_contact(phone_version);

    // Sync: clone remote.mem into MockStorage
    let remote_storage = MockStorage::new(remote_manager.mem.clone());
    local_manager.sync_from_storage(Box::new(remote_storage))?;

    let synced = local_manager.mem.get(&contact_id).unwrap();
    assert_eq!(
        synced.phone, "2222222222",
        "Phone should be from remote (last-write-wins with later timestamp)"
    );

    Ok(())
}

// SCENARIO 2: Remote file contains new contacts
//
// Offline additions must sync without creating duplicates

#[test]
fn sync_remote_contains_new_contacts() -> Result<(), AppError> {
    let mut local_manager = ContactManager::new()?;

    let contact1 = Contact::new(
        "Bob Johnson".to_string(),
        "5555555555".to_string(),
        "bob@example.com".to_string(),
        "work".to_string(),
    );
    let id1 = contact1.id;
    local_manager.add_contact(contact1.clone());

    // Remote manager
    let mut remote_manager = ContactManager::new()?;

    // Existing contact
    remote_manager.add_contact(contact1);

    // New contact 1
    let new_contact1 = Contact::new(
        "Carol White".to_string(),
        "6666666666".to_string(),
        "carol@example.com".to_string(),
        "personal".to_string(),
    );
    let id_new1 = new_contact1.id;
    remote_manager.add_contact(new_contact1);

    // New contact 2
    let new_contact2 = Contact::new(
        "David Green".to_string(),
        "7777777777".to_string(),
        "david@example.com".to_string(),
        "work".to_string(),
    );
    let id_new2 = new_contact2.id;
    remote_manager.add_contact(new_contact2);

    let remote_storage = MockStorage::new(remote_manager.mem.clone());

    // Before sync: local has 1 contact
    assert_eq!(local_manager.mem.len(), 1);

    local_manager.sync_from_storage(Box::new(remote_storage))?;

    // After sync: local should have 3 contacts
    assert_eq!(local_manager.mem.len(), 3);
    assert!(local_manager.mem.contains_key(&id1));
    assert!(local_manager.mem.contains_key(&id_new1));
    assert!(local_manager.mem.contains_key(&id_new2));

    Ok(())
}

#[test]
fn sync_offline_additions_no_duplicates() -> Result<(), AppError> {
    // Scenario: Both offline, both add same contact independently

    let same_contact_local = Contact::new(
        "Eve Taylor".to_string(),
        "8888888888".to_string(),
        "eve@example.com".to_string(),
        "friend".to_string(),
    );

    let same_contact_remote = Contact::new(
        "Eve Taylor".to_string(),
        "8888888888".to_string(),
        "eve@example.com".to_string(),
        "friend".to_string(),
    );

    let mut local_manager = ContactManager::new()?;
    local_manager.add_contact(same_contact_local);

    let mut remote_manager = ContactManager::new()?;
    remote_manager.add_contact(same_contact_remote);

    let remote_storage = MockStorage::new(remote_manager.mem.clone());

    local_manager.sync_from_storage(Box::new(remote_storage))?;

    // Should still have only 1 contact, not 2
    assert_eq!(
        local_manager.mem.len(),
        1,
        "Should not create duplicate after sync"
    );

    Ok(())
}

// SCENARIO 3: Local delete vs remote edit
//
// Deleted locally but edited remotely — delete wins

#[test]
fn sync_local_delete_remote_edit_delete_wins() -> Result<(), AppError> {
    let contact_id = Uuid::new_v4();
    let base_time = Utc::now();

    let _original = Contact {
        id: contact_id,
        name: "Frank Miller".to_string(),
        phone: "9999999999".to_string(),
        email: "frank@example.com".to_string(),
        tag: "work".to_string(),
        deleted: false,
        created_at: base_time,
        updated_at: base_time,
    };

    // Local: deleted
    let local_time = base_time + Duration::seconds(10);
    let mut local_deleted = _original.clone();
    local_deleted.deleted = true;
    local_deleted.updated_at = local_time;

    // Remote: edited (but not deleted)
    let remote_time = base_time + Duration::seconds(5); // Earlier than local deletion
    let mut remote_edited = _original.clone();
    remote_edited.phone = "0000000000".to_string(); // Changed
    remote_edited.email = "frank.miller@example.com".to_string(); // Changed
    remote_edited.tag = "personal".to_string(); // Changed
    remote_edited.updated_at = remote_time;

    let mut local_manager = ContactManager::new()?;
    local_manager.add_contact(local_deleted);

    let mut remote_manager = ContactManager::new()?;
    remote_manager.add_contact(remote_edited);

    let remote_storage = MockStorage::new(remote_manager.mem.clone());

    local_manager.sync_from_storage(Box::new(remote_storage))?;

    // Verify: Contact should remain deleted
    let contact = local_manager.mem.get(&contact_id).unwrap();
    assert!(
        contact.deleted,
        "Contact should remain deleted (local delete wins)"
    );

    Ok(())
}

#[test]
fn sync_remote_delete_local_edit_remote_delete_wins() -> Result<(), AppError> {
    let contact_id = Uuid::new_v4();
    let base_time = Utc::now();

    let _original = Contact {
        id: contact_id,
        name: "Grace Lee".to_string(),
        phone: "1010101010".to_string(),
        email: "grace@example.com".to_string(),
        tag: "personal".to_string(),
        deleted: false,
        created_at: base_time,
        updated_at: base_time,
    };

    // Local: edited (not deleted)
    let local_time = base_time + Duration::seconds(5);
    let mut local_edited = _original.clone();
    local_edited.email = "grace.lee@example.com".to_string(); // Changed
    local_edited.updated_at = local_time;

    // Remote: deleted
    let remote_time = base_time + Duration::seconds(10); // Later than local edit
    let mut remote_deleted = _original.clone();
    remote_deleted.deleted = true;
    remote_deleted.updated_at = remote_time;

    let mut local_manager = ContactManager::new()?;
    local_manager.add_contact(local_edited);

    let mut remote_manager = ContactManager::new()?;
    remote_manager.add_contact(remote_deleted);

    let remote_storage = MockStorage::new(remote_manager.mem.clone());

    local_manager.sync_from_storage(Box::new(remote_storage))?;

    let contact = local_manager.mem.get(&contact_id).unwrap();
    assert!(
        contact.deleted,
        "Contact should be deleted (remote delete, later timestamp wins)"
    );

    Ok(())
}

// SCENARIO 4: Import fails halfway
//
// This scenario tests rollback behavior on failure

#[test]
fn sync_state_unchanged_on_error() -> Result<(), AppError> {
    let contact_id = Uuid::new_v4();
    let base_time = Utc::now();

    let local_contact = Contact {
        id: contact_id,
        name: "Henry Brown".to_string(),
        phone: "1111111111".to_string(),
        email: "henry@example.com".to_string(),
        tag: "work".to_string(),
        deleted: false,
        created_at: base_time,
        updated_at: base_time,
    };

    let mut local_manager = ContactManager::new()?;
    local_manager.add_contact(local_contact.clone());

    // Create remote with contact that has mismatched created_at (will cause sync error)
    let conflicting_remote = Contact {
        id: contact_id,
        name: "Henry Brown".to_string(),
        phone: "1111111111".to_string(),
        email: "henry@example.com".to_string(),
        tag: "work".to_string(),
        deleted: false,
        created_at: base_time + Duration::seconds(100), // Different created_at = conflict
        updated_at: base_time,
    };

    let mut remote_manager = ContactManager::new()?;
    remote_manager.add_contact(conflicting_remote);

    let remote_storage = MockStorage::new(remote_manager.mem.clone());

    // Sync should fail
    let result = local_manager.sync_from_storage(Box::new(remote_storage));
    assert!(result.is_err(), "Sync should fail on timestamp conflict");

    // Verify local state unchanged (rollback semantics)
    let local_contact_after = local_manager.mem.get(&contact_id).unwrap();
    assert_eq!(
        local_contact_after.phone, "1111111111",
        "Local contact should be unchanged after sync error"
    );

    Ok(())
}

// SCENARIO 5: Conflicting or missing timestamps
//
// Clock drift or corruption - policy must produce deterministic results

#[test]
fn sync_same_timestamps_uses_local() -> Result<(), AppError> {
    let contact_id = Uuid::new_v4();
    let base_time = Utc::now();

    let local_contact = Contact {
        id: contact_id,
        name: "Iris Davis".to_string(),
        phone: "2222222222".to_string(),
        email: "iris@example.com".to_string(),
        tag: "personal".to_string(),
        deleted: false,
        created_at: base_time,
        updated_at: base_time,
    };

    let remote_contact = Contact {
        id: contact_id,
        name: "Iris Davis".to_string(),
        phone: "3333333333".to_string(), // Different phone
        email: "iris@example.com".to_string(),
        tag: "personal".to_string(),
        deleted: false,
        created_at: base_time,
        updated_at: base_time, // Same updated_at (clock drift)
    };

    let mut local_manager = ContactManager::new()?;
    local_manager.add_contact(local_contact);

    let mut remote_manager = ContactManager::new()?;
    remote_manager.add_contact(remote_contact);

    let remote_storage = MockStorage::new(remote_manager.mem.clone());

    local_manager.sync_from_storage(Box::new(remote_storage))?;

    let synced = local_manager.mem.get(&contact_id).unwrap();
    assert_eq!(
        synced.phone, "2222222222",
        "When timestamps equal, local should be kept (deterministic)"
    );

    Ok(())
}

#[test]
fn sync_created_at_mismatch_detected_as_conflict() -> Result<(), AppError> {
    let contact_id = Uuid::new_v4();
    let base_time = Utc::now();

    let local_contact = Contact {
        id: contact_id,
        name: "Jack Wilson".to_string(),
        phone: "4444444444".to_string(),
        email: "jack@example.com".to_string(),
        tag: "work".to_string(),
        deleted: false,
        created_at: base_time,
        updated_at: base_time,
    };

    // Same ID but different created_at - this indicates they're not the same contact
    let mut remote_contact = local_contact.clone();
    remote_contact.created_at = base_time + Duration::hours(1); // Different creation time

    let mut local_manager = ContactManager::new()?;
    local_manager.add_contact(local_contact);

    let mut remote_manager = ContactManager::new()?;
    remote_manager.add_contact(remote_contact);

    let remote_storage = MockStorage::new(remote_manager.mem.clone());

    let result = local_manager.sync_from_storage(Box::new(remote_storage));

    assert!(
        result.is_err(),
        "Sync should fail when created_at timestamps differ (indicates different contacts)"
    );

    match result {
        Err(AppError::Synchronization(_)) => {
            // Expected
        }
        _ => panic!("Expected Synchronization error"),
    }

    Ok(())
}

// SCENARIO 6: Duplicate entries from different devices
//
// Similar contacts with partial overlap

#[test]
fn sync_ignores_duplicate_by_name_and_phone() -> Result<(), AppError> {
    let mut local_manager = ContactManager::new()?;

    // Local has this contact
    let local_contact = Contact::new(
        "Kate Martinez".to_string(),
        "5555555555".to_string(),
        "kate@example.com".to_string(),
        "work".to_string(),
    );
    let local_id = local_contact.id;
    local_manager.add_contact(local_contact.clone());

    // Remote has same contact but with different ID and slightly different email
    let remote_contact = Contact::new(
        // Different ID
        "Kate Martinez".to_string(),
        "5555555555".to_string(), // Same name and phone = duplicate
        "kate.martinez@example.com".to_string(), // Different email
        "work".to_string(),
    );
    let remote_id = remote_contact.id;

    let mut remote_manager = ContactManager::new()?;
    remote_manager.add_contact(remote_contact);

    let remote_storage = MockStorage::new(remote_manager.mem.clone());

    local_manager.sync_from_storage(Box::new(remote_storage))?;

    // Should still have 1 contact (duplicate ignored), not 2
    assert_eq!(
        local_manager.mem.len(),
        1,
        "Duplicate contact should be ignored"
    );
    // And it should be the local one (ID preserved)
    assert!(local_manager.mem.contains_key(&local_id));
    assert!(!local_manager.mem.contains_key(&remote_id));

    Ok(())
}

#[test]
fn sync_duplicate_detection_requires_name_and_phone_match() -> Result<(), AppError> {
    let mut local_manager = ContactManager::new()?;

    let local_contact = Contact::new(
        "Leo Anderson".to_string(),
        "6666666666".to_string(),
        "leo@example.com".to_string(),
        "personal".to_string(),
    );
    let _local_id = local_contact.id;
    local_manager.add_contact(local_contact);

    // Remote: Same name, different phone - should NOT be treated as duplicate
    let remote_contact1 = Contact::new(
        "Leo Anderson".to_string(),
        "7777777777".to_string(), // Different phone
        "leo2@example.com".to_string(),
        "work".to_string(),
    );
    let _id1 = remote_contact1.id;

    // Remote: Same phone, different name - should NOT be treated as duplicate
    let remote_contact2 = Contact::new(
        "Leopold Anderson".to_string(), // Different name
        "6666666666".to_string(),       // Same phone
        "leopold@example.com".to_string(),
        "personal".to_string(),
    );
    let _id2 = remote_contact2.id;

    let mut remote_manager = ContactManager::new()?;
    remote_manager.add_contact(remote_contact1);
    remote_manager.add_contact(remote_contact2);

    let remote_storage = MockStorage::new(remote_manager.mem.clone());

    local_manager.sync_from_storage(Box::new(remote_storage))?;

    // Should have 3 contacts (original + 2 new ones, since they don't fully match)
    assert_eq!(
        local_manager.mem.len(),
        3,
        "Partial matches should not be treated as duplicates"
    );

    Ok(())
}

// EDGE CASES

#[test]
fn sync_empty_remote_no_changes() -> Result<(), AppError> {
    let mut local_manager = ContactManager::new()?;

    let contact = Contact::new(
        "Mia Roberts".to_string(),
        "8888888888".to_string(),
        "mia@example.com".to_string(),
        "friend".to_string(),
    );
    let contact_id = contact.id;
    local_manager.add_contact(contact);

    let empty_storage = MockStorage::new(HashMap::new());

    local_manager.sync_from_storage(Box::new(empty_storage))?;

    // Local contact should remain
    assert_eq!(local_manager.mem.len(), 1);
    assert!(local_manager.mem.contains_key(&contact_id));

    Ok(())
}

#[test]
fn sync_all_remote_contacts_deleted_locally() -> Result<(), AppError> {
    let contact_id = Uuid::new_v4();
    let base_time = Utc::now();

    let local_contact = Contact {
        id: contact_id,
        name: "Nina Clark".to_string(),
        phone: "9999999999".to_string(),
        email: "nina@example.com".to_string(),
        tag: "work".to_string(),
        deleted: true, // Already deleted
        created_at: base_time,
        updated_at: base_time,
    };

    let mut remote_contact = local_contact.clone();
    remote_contact.deleted = false; // Not deleted in remote;
    remote_contact.updated_at = base_time - Duration::seconds(10); // Older than local

    let mut local_manager = ContactManager::new()?;
    local_manager.add_contact(local_contact);

    let mut remote_manager = ContactManager::new()?;
    remote_manager.add_contact(remote_contact);

    let remote_storage = MockStorage::new(remote_manager.mem.clone());

    local_manager.sync_from_storage(Box::new(remote_storage))?;

    // Contact should remain deleted
    let contact = local_manager.mem.get(&contact_id).unwrap();
    assert!(contact.deleted);

    Ok(())
}

#[test]
fn sync_index_updated_after_merge() -> Result<(), AppError> {
    let contact_id = Uuid::new_v4();
    let base_time = Utc::now();

    let local_contact = Contact {
        id: contact_id,
        name: "Oscar Evans".to_string(),
        phone: "1010101010".to_string(),
        email: "oscar@example.com".to_string(),
        tag: "personal".to_string(),
        deleted: false,
        created_at: base_time,
        updated_at: base_time,
    };

    let mut remote_contact = local_contact.clone();
    remote_contact.name = "Oscar Jackson".to_string(); // Name changed
    remote_contact.email = "oscar@newdomain.com".to_string(); // Email changed
    remote_contact.updated_at = base_time + Duration::seconds(5);

    let mut local_manager = ContactManager::new()?;
    local_manager.add_contact(local_contact);

    let mut remote_manager = ContactManager::new()?;
    remote_manager.add_contact(remote_contact);

    let remote_storage = MockStorage::new(remote_manager.mem.clone());

    local_manager.sync_from_storage(Box::new(remote_storage))?;

    // Verify name index is updated (new names added)
    assert!(
        local_manager
            .index
            .name
            .get("oscar")
            .map(|ids| ids.contains(&contact_id))
            .unwrap_or(false),
        "Name index should contain 'oscar'"
    );

    // "evans" should no longer be in index
    assert!(
        !local_manager
            .index
            .name
            .get("evans")
            .map(|ids| ids.contains(&contact_id))
            .unwrap_or(false),
        "Name index should not contain old name 'evans'"
    );

    assert!(
        local_manager
            .index
            .name
            .get("jackson")
            .map(|ids| ids.contains(&contact_id))
            .unwrap_or(false),
        "Name index should contain new name 'jackson'"
    );

    // Verify email domain index is updated
    assert!(
        local_manager
            .index
            .domain
            .get("newdomain.com")
            .map(|ids| ids.contains(&contact_id))
            .unwrap_or(false),
        "Domain index should contain new domain"
    );

    Ok(())
}

#[test]
fn sync_multiple_contacts_mixed_operations() -> Result<(), AppError> {
    // Complex scenario: multiple contacts with add, update, delete, keep operations
    let base_time = Utc::now();

    // Contact 1: Exists in both, needs update
    let id1 = Uuid::new_v4();
    let local_1 = Contact {
        id: id1,
        name: "Alice".to_string(),
        phone: "1111111111".to_string(),
        email: "alice@old.com".to_string(),
        tag: "work".to_string(),
        deleted: false,
        created_at: base_time,
        updated_at: base_time,
    };

    let mut remote_1 = local_1.clone();
    remote_1.email = "alice@new.com".to_string();
    remote_1.updated_at = base_time + Duration::seconds(10);

    // Contact 2: Only in local, should remain
    let id2 = Uuid::new_v4();
    let local_2 = Contact {
        id: id2,
        name: "Bob".to_string(),
        phone: "2222222222".to_string(),
        email: "bob@example.com".to_string(),
        tag: "personal".to_string(),
        deleted: false,
        created_at: base_time,
        updated_at: base_time,
    };

    // Contact 3: Only in remote, should be added
    let id3 = Uuid::new_v4();
    let remote_3 = Contact {
        id: id3,
        name: "Charlie".to_string(),
        phone: "3333333333".to_string(),
        email: "charlie@example.com".to_string(),
        tag: "work".to_string(),
        deleted: false,
        created_at: base_time,
        updated_at: base_time,
    };

    // Contact 4: Deleted locally, older remote version
    let id4 = Uuid::new_v4();
    let local_4 = Contact {
        id: id4,
        name: "David".to_string(),
        phone: "4444444444".to_string(),
        email: "david@example.com".to_string(),
        tag: "work".to_string(),
        deleted: true,
        created_at: base_time,
        updated_at: base_time + Duration::seconds(20),
    };

    let mut remote_4 = local_4.clone();
    remote_4.deleted = false; // Not deleted remotely
    remote_4.updated_at = base_time + Duration::seconds(5); // Older than local delete

    let mut local_manager = ContactManager::new()?;
    local_manager.add_contact(local_1);
    local_manager.add_contact(local_2);
    local_manager.add_contact(local_4);

    let mut remote_manager = ContactManager::new()?;
    remote_manager.add_contact(remote_1);
    remote_manager.add_contact(remote_3);
    remote_manager.add_contact(remote_4);

    let remote_storage = MockStorage::new(remote_manager.mem.clone());

    local_manager.sync_from_storage(Box::new(remote_storage))?;

    // Verify results
    assert_eq!(local_manager.mem.len(), 4);

    // Contact 1: Updated with remote email
    assert_eq!(local_manager.mem.get(&id1).unwrap().email, "alice@new.com");

    // Contact 2: Unchanged
    assert_eq!(local_manager.mem.get(&id2).unwrap().name, "Bob");

    // Contact 3: Added
    assert_eq!(local_manager.mem.get(&id3).unwrap().name, "Charlie");

    // Contact 4: Remains deleted (local delete wins)
    assert!(local_manager.mem.get(&id4).unwrap().deleted);

    Ok(())
}
