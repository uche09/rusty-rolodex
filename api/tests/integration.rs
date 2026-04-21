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
