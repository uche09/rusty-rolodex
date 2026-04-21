# Rusty Rolodex REST API

A lightweight REST API for contact management built with [Axum](https://github.com/tokio-rs/axum), running on Tokio's async runtime. This API exposes the Rusty Rolodex library's functionality via HTTP endpoints.

## Quick Start

### Prerequisites

- Rust 1.78+
- Port 3000 available (configurable)

### Run the Server

```bash
# From workspace root
cargo run --release -p api

# Or with custom port
API_PORT=8080 cargo run -p api
```

**Expected Output:**

```
API server starting on port 3000...
[INFO] Server is running
```

Test the server:

```bash
curl http://localhost:3000/health
```

---

## API Endpoints

### Health Check

```http
GET /health
```

**Description:** Verify the API server is running and healthy.

**Response:** `200 OK`

```json
{
  "status": "ok",
  "message": "Server is running"
}
```

**Example:**

```bash
curl -X GET http://localhost:3000/health
```

---

### List Contacts

```http
GET /contacts
```

**Description:** Retrieve all contacts (excludes soft-deleted contacts).

**Response:** `200 OK`

```json
[
  {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "name": "John Doe",
    "phone": "5551234567",
    "email": "john@example.com",
    "tag": "friend",
    "deleted": false
  },
  {
    "id": "550e8400-e29b-41d4-a716-446655440001",
    "name": "Jane Smith",
    "phone": "5559876543",
    "email": "jane@example.com",
    "tag": "work",
    "deleted": false
  }
]
```

**Example:**

```bash
curl -X GET http://localhost:3000/contacts
```

---

### Get Contact

```http
GET /contacts/:id
```

**Description:** Retrieve a specific contact by ID. Returns 404 if contact is not found or has been soft-deleted.

**Response:** `200 OK`

```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "name": "John Doe",
  "phone": "5551234567",
  "email": "john@example.com",
  "tag": "friend",
  "deleted": false
}
```

**Example:**

```bash
curl -X GET http://localhost:3000/contacts/550e8400-e29b-41d4-a716-446655440000
```

---

### Add Contact

```http
POST /contacts
Content-Type: application/json
```

**Description:** Create a new contact.

**Request Body:**

```json
{
  "name": "Alice Johnson",
  "phone": "5555551234",
  "email": "alice@example.com",
  "tag": "colleague"
}
```

**Response:** `201 Created`

```json
{
  "id": "550e8400-e29b-41d4-a716-446655440002",
  "name": "Alice Johnson",
  "phone": "5555551234",
  "email": "alice@example.com",
  "tag": "colleague",
  "deleted": false
}
```

**Example:**

```bash
curl -X POST http://localhost:3000/contacts \
  -H "Content-Type: application/json" \
  -d '{"name":"Alice Johnson","phone":"5555551234","email":"alice@example.com","tag":"colleague"}'
```

---

### Edit Contact

```http
PATCH /contacts/:id
Content-Type: application/json
```

**Description:** Update an existing contact by ID.

**Request Body:**

```json
{
  "name": "Alice J. Johnson",
  "phone": "5555551234",
  "email": "alice.johnson@example.com",
  "tag": "senior colleague"
}
```

**Response:** `200 OK`

```json
{
  "id": "550e8400-e29b-41d4-a716-446655440002",
  "name": "Alice J. Johnson",
  "phone": "5555551234",
  "email": "alice.johnson@example.com",
  "tag": "senior colleague",
  "deleted": false
}
```

**Example:**

```bash
curl -X PATCH http://localhost:3000/contacts/550e8400-e29b-41d4-a716-446655440002 \
  -H "Content-Type: application/json" \
  -d '{"name":"Alice J. Johnson","phone":"5555551234","email":"alice.johnson@example.com","tag":"senior colleague"}'
```

---

### Delete Contact

```http
DELETE /contacts/:id
```

**Description:** Remove a contact by ID.

**Response:** `200 OK`

```json
{
  "id": "550e8400-e29b-41d4-a716-446655440002",
  "name": "Alice J. Johnson",
  "phone": "5555551234",
  "email": "alice.johnson@example.com",
  "tag": "senior colleague",
  "deleted": true
}
```

**Example:**

```bash
curl -X DELETE http://localhost:3000/contacts/550e8400-e29b-41d4-a716-446655440002
```

**Note:** Contacts are soft-deleted (marked with `deleted: true`). Soft-deleted contacts are excluded from `GET /contacts` and `GET /contacts/:id` responses but are retained in storage.

---

## Architecture

### Tech Stack

| Component           | Library                                         | Purpose                                 |
| ------------------- | ----------------------------------------------- | --------------------------------------- |
| **Web Framework**   | [Axum](https://github.com/tokio-rs/axum)        | Lightweight async HTTP server           |
| **Async Runtime**   | [Tokio](https://tokio.rs/)                      | Async task execution and concurrency    |
| **Serialization**   | [Serde + serde_json](https://serde.rs/)         | JSON request/response handling          |
| **Validation**      | [validator](https://github.com/Keats/validator) | Request payload validation              |
| **Error Handling**  | Custom `ApiError` enum                          | Uniform error responses                 |
| **Contact Logic**   | `libs::ContactManager`                          | Business logic for contact operations   |
| **Synchronization** | `Arc<RwLock<>>`                                 | Thread-safe concurrent state management |

### Project Structure

```
api/
├── Cargo.toml                # Package configuration
├── src/
│   ├── main.rs              # Server startup and Tokio runtime setup
│   ├── lib.rs               # Module exports
│   ├── config.rs            # Configuration from environment
│   ├── state.rs             # ApiState with shared resources
│   ├── error.rs             # ApiError enum and error handling
│   ├── service.rs           # ContactService business logic layer (unit tests)
│   ├── validation.rs        # Request payload validation schemas
│   └── routes/
│       ├── mod.rs           # Router setup and health check
│       └── contacts.rs      # Contact HTTP endpoints (handlers)
├── tests/                   # Integration tests
└── README.md                # This file
```

### State Management

The API uses **ApiState** for managing shared application state across concurrent requests:

```rust
#[derive(Clone)]
pub struct ApiState {
    pub config: Arc<Config>,
    pub service: Arc<ContactService>,
}
```

**Key Design Decisions:**

- **ContactService Layer**: Abstracts business logic from HTTP handlers, enabling:
  - Unit testability of core logic
  - Consistent validation and filtering (e.g., soft-delete filtering)
  - Reusable logic pattern for API and CLI
- **Arc<RwLock<ContactManager>>**: Inside `ContactService`, provides thread-safe concurrent access:
  - Multiple readers for `GET /contacts`
  - Exclusive writer for mutations (`POST`, `PATCH`, `DELETE`)
- **Async Integration**: `ContactManager` is async-first, properly handling file I/O and storage operations

### Request Flow

1. **Incoming Request** → Axum router matches endpoint
2. **State Extraction** → Handler receives `State(state): State<ApiState>`
3. **Validation** → Request payload is validated against schema
4. **Service Call** → Handler delegates to `ContactService` method (business logic layer)
5. **Lock Acquisition** → Service acquires `blocking_read()` or `blocking_write()` on manager
6. **Sync from Storage** → Before mutations, sync latest data from persistent storage
7. **Operation Execution** → Perform contact CRUD operations
8. **Filtering** → Service filters soft-deleted contacts from results
9. **Persistence** → `manager.save().await?` persists changes to storage
10. **Response** → Handler returns result with appropriate HTTP status code

### Sync Strategy

When mutations occur (add, edit, delete), the `ContactService` implements **LastWriteWins** synchronization before modifying data:

```rust
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
```

This ensures changes from other processes (e.g., CLI) are reflected before operations, preventing data loss in multi-process scenarios.

### Soft-Delete Behavior

Contacts are **soft-deleted** (marked with `deleted: true`) rather than permanently removed:

- `ContactService::list_contacts()` filters out soft-deleted contacts
- `ContactService::get_contact()` filters out soft-deleted contacts (returns `NotFound`)
- Underlying storage retains soft-deleted records for audit trails
- Allows re-adding contacts with identical details after deletion

---

## Configuration

### Environment Variables

Set these before starting the server:

| Variable       | Default           | Purpose                                         |
| -------------- | ----------------- | ----------------------------------------------- |
| `API_PORT`     | `3000`            | HTTP server port                                |
| `STORAGE_TYPE` | `json`            | Storage backend (json, csv, txt, remote)        |
| `STORAGE_PATH` | `./contacts.json` | File path for json/csv/txt backends             |
| `API_URL`      | —                 | Remote API URL (for remote storage)             |
| `API_KEY`      | —                 | Remote API key (for remote storage)             |
| `RUST_LOG`     | `my_api=debug`    | Logging level (trace, debug, info, warn, error) |

### Example `.env`

```env
API_PORT=3000
STORAGE_TYPE=json
STORAGE_PATH=./contacts.json
RUST_LOG=debug
```

Load it:

```bash
# Load from .env and run
dotenv::dotenv().ok();
cargo run -p api
```

---

## Error Handling

The API implements a unified error response format via the `ApiError` enum:

```rust
#[derive(Debug)]
pub enum ApiError {
    BadRequest(String),
    NotFound,
    InvalidInput(String),
    InternalError(String),
    ConfigError,
}
```

### Error Response Format

All errors return JSON with appropriate HTTP status codes:

**400 Bad Request:**

```json
{
  "error": "Contact already exist"
}
```

**404 Not Found:**

```json
{
  "error": "Contact not found"
}
```

**422 Unprocessable Entity:**

```json
{
  "error": "Validation failed: email format is invalid"
}
```

**500 Internal Server Error:**

```json
{
  "error": "Internal server error"
}
```

---

## Development

### Running Tests

```bash
# Run API tests only
cargo test -p api

# Run all tests (including libs and cli)
cargo test --workspace
```

Tests cover endpoint functionality and error cases. Add new tests to `tests/` directory.

### Code Quality

The project uses standard Rust tooling:

```bash
# Format check
cargo fmt --check -p api

# Lint with clippy
cargo clippy -p api -- -D warnings

# Both (as in CI)
cargo fmt -p api && cargo clippy -p api -- -D warnings
```

---

## Adding New Endpoints

### 1. Define Request/Response Validation (if needed)

In `src/validation.rs`:

```rust
use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct NewCustomData {
    #[validate(length(min = 1))]
    pub field: String,
}
```

### 2. Create Handler Function

In `src/routes/contacts.rs` or new route module:

```rust
#[axum::debug_handler]
async fn my_handler(
    State(state): State<ApiState>,
    Json(payload): Json<MyRequest>,
) -> Result<impl IntoResponse, ApiError> {
    payload.validate()?;

    // Lock manager for read or write
    let mut manager = state.manager.blocking_write();

    // Perform operation
    // manager.some_operation()?;

    Ok((StatusCode::OK, Json(response_data)))
}
```

### 3. Register Route

In `src/routes/mod.rs` or `src/routes/contacts.rs`:

```rust
pub fn create_router(state: ApiState) -> Router {
    Router::new()
        .route("/contacts", get(list_contacts).post(add_contact))
        .route("/contacts/:id", patch(edit_contact).delete(delete_contact))
        .route("/my-endpoint", post(my_handler))  // New endpoint
        .with_state(state)
}
```

### 4. Test

```bash
curl -X POST http://localhost:3000/my-endpoint \
  -H "Content-Type: application/json" \
  -d '{"field":"value"}'
```

---

## Concurrency & Thread Safety

### RwLock Pattern

The API uses `blocking_read()` / `blocking_write()` pattern from Tokio:

**For Read Operations (GET):**

```rust
let manager = state.manager.blocking_read();
let contacts = manager.mem.values().cloned().collect();
```

- Multiple handlers can acquire read locks simultaneously
- Non-blocking for other readers

**For Write Operations (POST, PATCH, DELETE):**

```rust
let mut manager = state.manager.blocking_write();
manager.add_contact(contact);
manager.save().await?;
```

- Exclusive lock; other operations wait
- Ensures consistency before/after mutations

### Important Notes

- ⚠️ `blocking_*()` is used instead of `.read()`/`.write().await` because handlers must remain `async fn` for Axum compatibility
- All async operations (file I/O, storage) happen within the lock scope
- The sync mechanism ensures multi-process consistency (API + CLI can safely coexist)

---

## Deployment

### Local Development

```bash
# Development mode with hot-reload support
cargo run -p api

# Listening on http://localhost:3000
```

For watch-mode development (requires `cargo-watch`):

```bash
cargo install cargo-watch
cargo watch -x 'run -p api'
```

### Production Release Build

```bash
# Optimized binary for deployment
cargo build --release -p api

# Binary location: target/release/api
./target/release/api
```

**Performance Notes:**

- Release builds are 10-100x faster than debug builds
- Async/await has zero runtime overhead in release mode
- RwLock contention is minimal for typical contact management workloads

---

## Troubleshooting

| Issue                                     | Solution                                                                            |
| ----------------------------------------- | ----------------------------------------------------------------------------------- |
| `Error: couldn't bind to port 3000`       | Change port: `API_PORT=8080 cargo run -p api`                                       |
| `{error: "Internal server error"}`        | Check logs: `RUST_LOG=debug cargo run -p api`                                       |
| `Connection refused`                      | Ensure server is running; check with `curl http://localhost:3000/health`            |
| `JSON parsing failed`                     | Validate request format; send proper `Content-Type: application/json`               |
| `Contact already exist`                   | The contact (name, phone, email) already exists; try with different values          |
| `Contact not found`                       | The UUID doesn't match any existing contact or has been soft-deleted; verify the ID |
| `Validation failed: phone format invalid` | Phone must be 10-15 digits, optionally starting with `+`; example: `5551234567`     |
| `Validation failed: email format invalid` | Email must be RFC 5322 compliant; example: `user@example.com`                       |
| `Validation failed: name too long`        | Name must be 1-50 characters; shorten the name                                      |
| `GET requests slow with many contacts`    | This is expected for large datasets; consider pagination (future feature)           |
| Lock timeout errors                       | Rare; indicates handler deadlock. Check logs and file an issue.                     |

### Enable Debug Logging

```bash
RUST_LOG=debug cargo run -p api
```

Output includes:

- Request routing
- Storage I/O operations
- Lock acquisition/release timing
- Error stack traces

---

## API Usage Examples

### Complete Workflow

```bash
# 1. Check server health
curl http://localhost:3000/health

# 2. Add a contact
CONTACT_RESPONSE=$(curl -X POST http://localhost:3000/contacts \
  -H "Content-Type: application/json" \
  -d '{
    "name": "Bob Wilson",
    "phone": "5559876543",
    "email": "bob@example.com",
    "tag": "friend"
  }')

CONTACT_ID=$(echo $CONTACT_RESPONSE | jq -r '.id')
echo "Created contact: $CONTACT_ID"

# 3. Get the contact by ID
curl http://localhost:3000/contacts/$CONTACT_ID

# 4. List all contacts
curl http://localhost:3000/contacts

# 5. Update the contact
curl -X PATCH http://localhost:3000/contacts/$CONTACT_ID \
  -H "Content-Type: application/json" \
  -d '{
    "name": "Robert Wilson",
    "phone": "5559876543",
    "email": "robert@example.com",
    "tag": "friend"
  }'

# 6. Delete the contact
curl -X DELETE http://localhost:3000/contacts/$CONTACT_ID

# 7. Verify contact is gone (returns 404)
curl http://localhost:3000/contacts/$CONTACT_ID
```

---

## Integration with CLI & Libs

The API, CLI, and libraries are designed to work together:

- **API** → Exposes contact management via HTTP (this package)
- **CLI** → Command-line interface for contact management (`../cli/`)
- **Libs** → Core business logic and storage (`../libs/`)

They share the same:

- `ContactManager` for business logic
- Storage backend (JSON/CSV/etc.)
- Sync policies for multi-process consistency

This allows:

- ✅ Using API and CLI simultaneously on shared contacts
- ✅ Data consistency across processes
- ✅ Easy switching between interfaces

---

## Architecture Decisions

### Why RwLock Instead of Mutex?

- **RwLock**: Allows multiple concurrent readers; only blocks on writes
- **Mutex**: Would serialize all requests, even pure reads
- **Trade-off**: Slight performance overhead on writes for significant read throughput gains

### Why blocking\_\* Instead of .await?

- Axum handlers must be `async fn`
- `RwLock::read().await` / `.write().await` would require complex pinning
- `blocking_*()` abstracts this complexity and is safe here (lock scope is short)

### Why Sync Before Every Mutation?

Since API and CLI can both modify the contact store:

- Without sync: API might overwrite CLI changes
- With sync: Last operation always wins (consistent behavior)
- Cost: Single file read per mutation (negligible for typical workloads)

---

## Further Reading

- **Library Documentation:** See [../libs/README.md][libs-readme]
- **CLI Documentation:** See [../cli/README.md][cli-readme]
- **Workspace Architecture:** See [../README.md][readme-root]
- **Axum Docs:** [tokio-rs.github.io/axum](https://tokio-rs.github.io/axum/)
- **Tokio Docs:** [tokio.rs](https://tokio.rs/)

<!-- Link Aliases - Update paths here if documentation structure changes -->

[readme-root]: ../README.md
[cli-readme]: ../cli/README.md
[libs-readme]: ../libs/README.md
[changelog]: ../CHANGELOG.md
