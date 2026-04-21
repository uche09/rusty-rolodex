use api::{config::Config, routes, service::ContactService, state::ApiState};
use axum::http::StatusCode;
use axum_test::TestServer;
use libs::{
    domain::manager::Index,
    prelude::{ContactManager, JsonStorage},
};
use serde_json::{Value, json};
use std::{collections::HashMap, sync::Arc};
use tempfile::{TempDir, tempdir};
use tokio::sync::RwLock;

async fn setup() -> (TestServer, TempDir) {
    let config = Config::from_env().unwrap();
    let (manager, dir) = create_mock_manager();
    let manager = Arc::new(RwLock::new(manager));
    let state = ApiState {
        config: Arc::new(config),
        service: Arc::new(ContactService::new(manager)),
    };
    let router = routes::create_router(state);
    (TestServer::new(router).unwrap(), dir)
}

fn create_mock_manager() -> (ContactManager, TempDir) {
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
        index: Index {
            name: HashMap::new(),
            domain: HashMap::new(),
        },
    };
    let index = Index::new(&manager).unwrap();
    manager.index = index;
    (manager, dir)
}

#[tokio::test]
async fn test_health_check() {
    let (server, _dir) = setup().await;
    let response = server.get("/health").await;
    assert_eq!(response.status_code(), 200);
}

#[tokio::test]
async fn test_list_contacts() {
    let (server, _dir) = setup().await;
    let response = server.get("/contacts").await;
    assert_eq!(response.status_code(), 200);
}

#[tokio::test]
async fn test_add_contact_valid() {
    let (server, _dir) = setup().await;
    let response = server
        .post("/contacts")
        .json(&json!({
            "name": "Alice Smith",
            "phone": "5551234567"
        }))
        .await;
    assert_eq!(response.status_code(), 201);
}

#[tokio::test]
async fn test_add_contact_missing_name() {
    let (server, _dir) = setup().await;
    let response = server
        .post("/contacts")
        .json(&json!({"phone": "5551234567"}))
        .await;
    assert_eq!(response.status_code(), 422);
}

#[tokio::test]
async fn test_add_contact_invalid_phone() {
    let (server, _dir) = setup().await;
    let response = server
        .post("/contacts")
        .json(&json!({
            "name": "John Doe",
            "phone": "123"
        }))
        .await;
    // assert_eq!(response.status_code(), 422);
    response.assert_status(StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn test_get_contact() {
    let (server, _dir) = setup().await;
    let create = server
        .post("/contacts")
        .json(&json!({
            "name": "John Doe",
            "phone": "5551234567",
            "email": "john@example.com",
            "tag": "friend"
        }))
        .await;
    let body = create.json::<Value>();
    let id = body["id"].as_str().unwrap().to_string();

    let response = server.get(&format!("/contacts/{}", id)).await;
    assert_eq!(response.status_code(), 200);
    let contact = response.json::<Value>();
    assert_eq!(contact["id"], id);
    assert_eq!(contact["name"], "John Doe");
    assert_eq!(contact["phone"], "5551234567");
    assert_eq!(contact["email"], "john@example.com");
    assert_eq!(contact["tag"], "friend");
    assert_eq!(contact["deleted"], false);
}

#[tokio::test]
async fn test_get_contact_not_found() {
    let (server, _dir) = setup().await;
    let response = server
        .get("/contacts/00000000-0000-0000-0000-000000000000")
        .await;
    assert_eq!(response.status_code(), 404);
}

#[tokio::test]
async fn test_edit_contact() {
    let (server, _dir) = setup().await;
    let create = server
        .post("/contacts")
        .json(&json!({
            "name": "John Doe",
            "phone": "5551234567"
        }))
        .await;
    let body = create.json::<Value>();
    let id = body["id"].as_str().unwrap().to_string();

    let response = server
        .patch(&format!("/contacts/{}", id))
        .json(&json!({
            "name": "Jane Doe"
        }))
        .await;
    assert_eq!(response.status_code(), 200);
}

#[tokio::test]
async fn test_delete_contact() {
    let (server, _dir) = setup().await;
    let create = server
        .post("/contacts")
        .json(&json!({
            "name": "John Doe",
            "phone": "5551234567"
        }))
        .await;
    let body = create.json::<Value>();
    let id = body["id"].as_str().unwrap().to_string();

    let response = server.delete(&format!("/contacts/{}", id)).await;
    assert_eq!(response.status_code(), 200);
}

#[tokio::test]
async fn test_delete_not_found() {
    let (server, _dir) = setup().await;
    let response = server
        .delete("/contacts/00000000-0000-0000-0000-000000000000")
        .await;
    assert_eq!(response.status_code(), 404);
}

#[tokio::test]
async fn test_crud_workflow() {
    let (server, _dir) = setup().await;

    let create = server
        .post("/contacts")
        .json(&json!({
            "name": "Alice",
            "phone": "5551111111"
        }))
        .await;
    assert_eq!(create.status_code(), 201);
    let body = create.json::<Value>();
    let id = body["id"].as_str().unwrap().to_string();

    let list = server.get("/contacts").await;
    assert_eq!(list.status_code(), 200);

    let edit = server
        .patch(&format!("/contacts/{}", id))
        .json(&json!({"name": "Alice Smith"}))
        .await;
    assert_eq!(edit.status_code(), 200);

    let delete = server.delete(&format!("/contacts/{}", id)).await;
    assert_eq!(delete.status_code(), 200);
}

#[tokio::test]
async fn test_duplicate_contact() {
    let (server, _dir) = setup().await;
    let payload = json!({"name": "John Doe", "phone": "5551234567"});

    let resp1 = server.post("/contacts").json(&payload).await;
    assert_eq!(resp1.status_code(), 201);

    let resp2 = server.post("/contacts").json(&payload).await;
    assert_eq!(resp2.status_code(), 400);
}

#[tokio::test]
async fn test_add_contact_name_too_long() {
    let (server, _dir) = setup().await;
    let long_name = "A".repeat(51);
    let response = server
        .post("/contacts")
        .json(&json!({
            "name": long_name,
            "phone": "5551234567"
        }))
        .await;
    assert_eq!(response.status_code(), 422);
}

#[tokio::test]
async fn test_add_contact_with_email() {
    let (server, _dir) = setup().await;
    let response = server
        .post("/contacts")
        .json(&json!({
            "name": "Bob Smith",
            "phone": "5559876543",
            "email": "bob@example.com"
        }))
        .await;
    assert_eq!(response.status_code(), 201);
    let body = response.json::<Value>();
    assert_eq!(body["email"], "bob@example.com");
}

#[tokio::test]
async fn test_add_contact_invalid_email() {
    let (server, _dir) = setup().await;
    let response = server
        .post("/contacts")
        .json(&json!({
            "name": "Bob Smith",
            "phone": "5559876543",
            "email": "not-an-email"
        }))
        .await;
    assert_eq!(response.status_code(), 422);
}

#[tokio::test]
async fn test_add_contact_with_tag() {
    let (server, _dir) = setup().await;
    let response = server
        .post("/contacts")
        .json(&json!({
            "name": "Charlie Brown",
            "phone": "5554443333",
            "tag": "colleague"
        }))
        .await;
    assert_eq!(response.status_code(), 201);
    let body = response.json::<Value>();
    assert_eq!(body["tag"], "colleague");
}

#[tokio::test]
async fn test_add_contact_phone_boundary_min() {
    let (server, _dir) = setup().await;
    let response = server
        .post("/contacts")
        .json(&json!({
            "name": "Min Phone",
            "phone": "1234567890"
        }))
        .await;

    assert_eq!(response.status_code(), 201);
}

#[tokio::test]
async fn test_add_contact_phone_with_plus() {
    let (server, _dir) = setup().await;
    let response = server
        .post("/contacts")
        .json(&json!({
            "name": "International",
            "phone": "+441234567890"
        }))
        .await;
    assert_eq!(response.status_code(), 201);
}

#[tokio::test]
async fn test_edit_non_existent_contact() {
    let (server, _dir) = setup().await;
    let fake_uuid = "00000000-0000-0000-0000-000000000000";
    let response = server
        .patch(&format!("/contacts/{}", fake_uuid))
        .json(&json!({"name": "Updated"}))
        .await;
    assert_eq!(response.status_code(), 404);
}

#[tokio::test]
async fn test_edit_contact_invalid_uuid() {
    let (server, _dir) = setup().await;
    let response = server
        .patch("/contacts/not-a-uuid")
        .json(&json!({"name": "Updated"}))
        .await;
    assert_eq!(response.status_code(), 400);
}

#[tokio::test]
async fn test_edit_partial_update() {
    let (server, _dir) = setup().await;
    let create = server
        .post("/contacts")
        .json(&json!({
            "name": "Original Name",
            "phone": "5551111111",
            "email": "orig@example.com"
        }))
        .await;
    let body = create.json::<Value>();
    let id = body["id"].as_str().unwrap().to_string();

    let edit = server
        .patch(&format!("/contacts/{}", id))
        .json(&json!({"email": "updated@example.com"}))
        .await;
    assert_eq!(edit.status_code(), 200);
    let edited = edit.json::<Value>();
    assert_eq!(edited["name"], "Original Name");
    assert_eq!(edited["email"], "updated@example.com");
}

#[tokio::test]
async fn test_add_contact_missing_phone() {
    let (server, _dir) = setup().await;
    let response = server
        .post("/contacts")
        .json(&json!({"name": "Alice Smith"}))
        .await;
    assert_eq!(response.status_code(), 422);
}

#[tokio::test]
async fn test_delete_invalid_uuid() {
    let (server, _dir) = setup().await;
    let response = server.delete("/contacts/invalid-uuid").await;
    assert_eq!(response.status_code(), 400);
}

// SOFT-DELETE EDGE CASE TESTS

#[tokio::test]
async fn test_deleted_contact_not_in_list() {
    let (server, _dir) = setup().await;

    // Add a contact
    let create = server
        .post("/contacts")
        .json(&json!({
            "name": "To Delete",
            "phone": "5559999999"
        }))
        .await;
    assert_eq!(create.status_code(), 201);
    let body = create.json::<Value>();
    let id = body["id"].as_str().unwrap().to_string();

    // Confirm contact is in list
    let list = server.get("/contacts").await;
    assert_eq!(list.status_code(), 200);
    let contacts = list.json::<Vec<Value>>();
    assert!(contacts.iter().any(|c| c["id"].as_str().unwrap() == id));

    // Delete the contact
    let delete = server.delete(&format!("/contacts/{}", id)).await;
    assert_eq!(delete.status_code(), 200);

    // Confirm contact is NOT in list after deletion
    let list = server.get("/contacts").await;
    assert_eq!(list.status_code(), 200);
    let contacts = list.json::<Vec<Value>>();
    assert!(!contacts.iter().any(|c| c["id"].as_str().unwrap() == id));
}

#[tokio::test]
async fn test_get_deleted_contact_returns_not_found() {
    let (server, _dir) = setup().await;

    let create = server
        .post("/contacts")
        .json(&json!({
            "name": "To Delete",
            "phone": "5554444444"
        }))
        .await;
    let body = create.json::<Value>();
    let id = body["id"].as_str().unwrap().to_string();

    // Delete the contact
    let delete = server.delete(&format!("/contacts/{}", id)).await;
    assert_eq!(delete.status_code(), 200);

    // Try to get the deleted contact - should return 404
    let get = server.get(&format!("/contacts/{}", id)).await;
    assert_eq!(get.status_code(), 404);
}

#[tokio::test]
async fn test_edit_deleted_contact_returns_not_found() {
    let (server, _dir) = setup().await;

    let create = server
        .post("/contacts")
        .json(&json!({
            "name": "Original",
            "phone": "5558888888"
        }))
        .await;
    let body = create.json::<Value>();
    let id = body["id"].as_str().unwrap().to_string();

    // Delete the contact
    let delete = server.delete(&format!("/contacts/{}", id)).await;
    assert_eq!(delete.status_code(), 200);

    // Try to edit the deleted contact - should return 404
    let edit = server
        .patch(&format!("/contacts/{}", id))
        .json(&json!({"name": "Updated"}))
        .await;
    assert_eq!(edit.status_code(), 404);
}

#[tokio::test]
async fn test_delete_then_read_contact() {
    let (server, _dir) = setup().await;

    let payload = json!({"name": "Reusable", "phone": "5557777777"});

    // Add contact
    let create1 = server.post("/contacts").json(&payload).await;
    assert_eq!(create1.status_code(), 201);
    let body = create1.json::<Value>();
    let id1 = body["id"].as_str().unwrap().to_string();

    // Delete it
    let delete = server.delete(&format!("/contacts/{}", id1)).await;
    assert_eq!(delete.status_code(), 200);

    // Add it again - should succeed because first one is soft-deleted
    let create2 = server.post("/contacts").json(&payload).await;
    assert_eq!(create2.status_code(), 201);
    let body = create2.json::<Value>();
    let id2 = body["id"].as_str().unwrap().to_string();

    // IDs should be different
    assert_ne!(id1, id2);

    // List should only contain the new contact
    let list = server.get("/contacts").await;
    let contacts = list.json::<Vec<Value>>();
    assert_eq!(contacts.len(), 1);
    assert_eq!(contacts[0]["id"].as_str().unwrap(), id2);
}

#[tokio::test]
async fn test_list_with_multiple_deleted_contacts() {
    let (server, _dir) = setup().await;

    // Add 5 contacts
    let mut ids = Vec::new();
    for i in 1..=5 {
        let create = server
            .post("/contacts")
            .json(&json!({
                "name": format!("Contact {}", i),
                "phone": format!("555000000{}", i)
            }))
            .await;
        let body = create.json::<Value>();
        let id = body["id"].as_str().unwrap().to_string();
        ids.push(id);
    }

    // Verify all 5 are in list
    let list = server.get("/contacts").await;
    let contacts = list.json::<Vec<Value>>();
    assert_eq!(contacts.len(), 5);

    // Delete contacts at index 1 and 3 (2nd and 4th contacts)
    let delete1 = server.delete(&format!("/contacts/{}", ids[1])).await;
    assert_eq!(delete1.status_code(), 200);

    let delete2 = server.delete(&format!("/contacts/{}", ids[3])).await;
    assert_eq!(delete2.status_code(), 200);

    // Verify list now has only 3 contacts
    let list = server.get("/contacts").await;
    let contacts = list.json::<Vec<Value>>();
    assert_eq!(contacts.len(), 3);

    // Verify correct contacts are present
    let response_ids: Vec<String> = contacts
        .iter()
        .map(|c| c["id"].as_str().unwrap().to_string())
        .collect();

    assert!(response_ids.contains(&ids[0]));
    assert!(!response_ids.contains(&ids[1]));
    assert!(response_ids.contains(&ids[2]));
    assert!(!response_ids.contains(&ids[3]));
    assert!(response_ids.contains(&ids[4]));
}

#[tokio::test]
async fn test_crud_workflow_with_delete_and_list() {
    let (server, _dir) = setup().await;

    // Create 3 contacts
    let create1 = server
        .post("/contacts")
        .json(&json!({"name": "Alice", "phone": "5551111111"}))
        .await;
    let id1 = create1.json::<Value>()["id"].as_str().unwrap().to_string();

    let create2 = server
        .post("/contacts")
        .json(&json!({"name": "Bob", "phone": "5552222222"}))
        .await;
    let id2 = create2.json::<Value>()["id"].as_str().unwrap().to_string();

    let create3 = server
        .post("/contacts")
        .json(&json!({"name": "Charlie", "phone": "5553333333"}))
        .await;
    let id3 = create3.json::<Value>()["id"].as_str().unwrap().to_string();

    // List should have 3
    assert_eq!(server.get("/contacts").await.json::<Vec<Value>>().len(), 3);

    // Edit one
    let edit = server
        .patch(&format!("/contacts/{}", id1))
        .json(&json!({"name": "Alice Updated"}))
        .await;
    assert_eq!(edit.status_code(), 200);

    // Delete one
    let delete = server.delete(&format!("/contacts/{}", id2)).await;
    assert_eq!(delete.status_code(), 200);

    // List should have 2
    let list = server.get("/contacts").await.json::<Vec<Value>>();
    assert_eq!(list.len(), 2);
    assert!(list.iter().any(|c| c["id"].as_str().unwrap() == id1));
    assert!(!list.iter().any(|c| c["id"].as_str().unwrap() == id2));
    assert!(list.iter().any(|c| c["id"].as_str().unwrap() == id3));

    // Verify the edited contact has updated name
    assert_eq!(
        list.iter()
            .find(|c| c["id"].as_str().unwrap() == id1)
            .unwrap()["name"],
        "Alice Updated"
    );
}

#[tokio::test]
async fn test_delete_and_get_returns_404() {
    let (server, _dir) = setup().await;

    // Create and immediately delete
    let create = server
        .post("/contacts")
        .json(&json!({"name": "Temporary", "phone": "5556666666"}))
        .await;
    let id = create.json::<Value>()["id"].as_str().unwrap().to_string();

    server.delete(&format!("/contacts/{}", id)).await;

    // Get should return 404
    let get = server.get(&format!("/contacts/{}", id)).await;
    assert_eq!(get.status_code(), 404);
}

#[tokio::test]
async fn test_empty_list_after_deleting_only_contact() {
    let (server, _dir) = setup().await;

    // Create only one contact
    let create = server
        .post("/contacts")
        .json(&json!({"name": "Only One", "phone": "5557777777"}))
        .await;
    let id = create.json::<Value>()["id"].as_str().unwrap().to_string();

    // List should have 1
    assert_eq!(server.get("/contacts").await.json::<Vec<Value>>().len(), 1);

    // Delete it
    server.delete(&format!("/contacts/{}", id)).await;

    // List should be empty
    assert_eq!(server.get("/contacts").await.json::<Vec<Value>>().len(), 0);
}
