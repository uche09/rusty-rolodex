# Rusty Rolodex Library

The `libs` crate provides the core business logic and storage abstraction for contact management. It's designed as a reusable library that can be integrated into any application (CLI, API, GUI, etc.).

## Quick Start

Add to your `Cargo.toml`:

```toml
[dependencies]
libs = { path = "../libs" }
```

Basic usage:

```rust
use libs::domain::manager::ContactManager;
use libs::storage::file::FileContactStore;

// Create a manager with JSON file storage
let storage = FileContactStore::new("contacts.json".to_string());
let mut manager = ContactManager::new(Box::new(storage))?;

// Add a contact
manager.add_contact("Alice Johnson", "+234123456789", "alice@example.com", None)?;

// Search by name
let results = manager.search_name("Alice")?;
for contact in results {
    println!("{}", contact);
}
```

---

## Core Modules

### Domain Layer (`src/domain/`)

**ContactManager** - High-level orchestration for all contact operations:

- `add_contact()` - Create new contact with validation
- `list_contacts()` - Retrieve all contacts with sorting/filtering
- `search_name()` / `search_email_domain()` - Indexed search with fuzzy matching
- `edit_contact()` - Update contact fields with validation
- `delete_contact()` - Soft delete with recovery window
- `import_contacts()` / `export_contacts()` - Data migration with conflict resolution

**Contact Model** - Type-safe representation:

```rust
pub struct Contact {
    pub id: Uuid,
    pub name: String,
    pub phone: String,
    pub email: Option<String>,
    pub tag: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted: bool,
}
```

**Validation** - Boundary enforcement via regex patterns:

- **Name:** Alphanumeric with spaces, hyphens, apostrophes (50 char max)
- **Phone:** International format `+?[0-9]{10,15}` with duplicate detection
- **Email:** Standard format (254 char max, optional field)

See [validation.rs](./src/domain/validation.rs) for implementation.

**Indexing** - Two-tier O(1) search:

- **Name Index:** `HashMap<String, HashSet<Uuid>>` splitting contact names by whitespace
- **Email Domain Index:** Similar structure for domain-based queries
- **Fuzzy Matching:** Levenshtein distance with configurable threshold

See [indexing.rs](./src/domain/indexing.rs) for implementation.

**Sync & Conflict Resolution** - Last-Write-Wins (LWW):

- Field-level resolution based on timestamps (vs. record-level)
- Soft-delete propagation to remote backends
- Configurable purge window for deleted contacts

See [manager.rs (sync_contacts method)](./src/domain/manager.rs) for implementation.

### Storage Layer (`src/storage/`)

**ContactStore Trait** - Abstraction for pluggable backends:

```rust
pub trait ContactStore {
    fn load(&self) -> Result<HashMap<Uuid, Contact>, AppError>;
    fn save(&self, contacts: &HashMap<Uuid, Contact>) -> Result<(), AppError>;
    fn get_medium(&self) -> &str;
}
```

**Available Backends:**

| Backend    | File                | Format          | Use Case                              |
| ---------- | ------------------- | --------------- | ------------------------------------- |
| **JSON**   | `storage/file.rs`   | Structured JSON | Default; fast queries, human-readable |
| **CSV**    | `storage/csv.rs`    | CSV rows        | Import/export, third-party tools      |
| **TXT**    | `storage/text.rs`   | Custom text     | Lightweight, human-readable fallback  |
| **Remote** | `storage/remote.rs` | HTTP + JSON     | Cloud sync via jsonstorage.net        |

**File Storage Features:**

- Automatic format detection and migration
- Backward compatibility with older `Vec`-based formats
- Atomic writes with file locking via `fs2` crate
- Concurrent read access with exclusive write locks

**Remote Storage Features:**

- HTTP-based sync with jsonstorage.net API
- Automatic fallback: PUT (update) → POST (create) → GET (retrieve)
- Interior mutability for resource ID tracking
- Configurable API endpoints and keys via `.env`

See [storage/mod.rs](./src/storage/mod.rs) for the trait and backend implementations.

### Error Handling (`src/errors.rs`)

Comprehensive `AppError` enum with type-safe conversions:

```rust
pub enum AppError {
    CsvError(csv::Error),
    DateTime(chrono::ParseError),
    FailedRequest(reqwest::Error),
    Io(std::io::Error),
    JsonParser(serde_json::Error),
    NotFound(String),
    Poison(String),          // Mutex poisoning
    RegexError(regex::Error),
    Synchronization(String),
    Validation(String),
}
```

All error types implement `Display` for user-friendly messages and `From` for seamless `?` operator chaining.

---

## Advanced Features

### Performance Optimization

The library is benchmarked at scale (1k–100k contacts) with linear O(n) operations:

| Operation          | 100k Contacts | Optimization                              |
| ------------------ | ------------- | ----------------------------------------- |
| Add                | 54ms          | Pre-allocated capacity, indexed updates   |
| Search             | 30ms          | Two-tier indexing with fuzzy scoring      |
| List + Filter      | 56ms          | Linear iteration (unavoidable)            |
| JSON Serialization | 270ms         | Serde + JSON bottleneck                   |
| Index Building     | 48ms          | Multi-threaded with adaptive worker count |

**Adaptive Threading Heuristic:**

```rust
fn determine_num_of_workers_thread_for_a_work_size(work_length: usize) -> usize {
    match work_length {
        0..=100 => 1,      // Single thread (no scheduler overhead)
        101..=200 => 2,    // Breakpoint analysis
        201..=500 => 3,
        501..=1000 => 4,
        _ => 5,            // Cap at 5 threads
    }
}
```

Prevents under-utilization on small datasets and over-subscription on large ones.

See [docs/perf-notes.md](./docs/perf-notes.md) for detailed methodology and insights.

### Concurrency Safety

- **Scoped Threads:** Safe parallel work with no 'static requirement
- **Arc + Mutex:** No unsafe code; all synchronization via standard library
- **Local Accumulation:** Thread-local HashMaps merged once (reduces contention)
- **Poison Handling:** Safe recovery from mutex poisoning via `From<PoisonError>`

### Data Integrity

**Soft Deletes:**

- Contacts marked `deleted: true` rather than hard-removed
- Configurable purge window (default: 1 day)
- Recovery possible within window

**Backward Compatibility:**

- Auto-migration from old `Vec` format to new `HashMap<Uuid, Contact>` structure
- Zero data loss on upgrade

**Rollback on Import Failure:**

- Failed import restores previous state via automatic re-save
- No partial updates left behind

---

## Testing

The library includes comprehensive unit tests embedded in each module:

```bash
cargo test --lib
```

**Coverage:**

- Contact validation (boundary conditions, regex patterns)
- Index building and fuzzy search accuracy
- Sync conflict resolution (field-level merges)
- Storage backend round-trip (format migrations)
- Error type conversions

---

## Integration Guide for Other Crates

### 1. Create a ContactManager

```rust
use libs::domain::manager::ContactManager;
use libs::storage::file::FileContactStore;

let storage = Box::new(FileContactStore::new("contacts.json".to_string()));
let manager = ContactManager::new(storage)?;
```

### 2. Choose Your Storage Backend

```rust
// JSON
let json_store = Box::new(FileContactStore::new("data.json".to_string()));

// CSV
let csv_store = Box::new(CsvContactStore::new("data.csv".to_string()));

// TXT
let txt_store = Box::new(TextContactStore::new("data.txt".to_string()));

// Remote (HTTP)
let remote_store = Box::new(RemoteContactStore::new(
    Some("https://api.jsonstorage.net/v1/json/YOUR_USER_ID".to_string())
));

let manager = ContactManager::new(storage)?;
```

### 3. Use the Manager API

```rust
// Add
manager.add_contact("Alice", "+234123456789", "alice@example.com", None)?;

// Search
let results = manager.search_name("Alice")?;

// List with sorting
let contacts = manager.list_contacts(Some(SortKey::Name), None)?;

// Edit
manager.edit_contact("Alice", "+234123456789", None, Some("+234198765432"), None, None)?;

// Delete (soft)
manager.delete_contact("Alice")?;

// Sync with remote
manager.sync_contacts(&remote_manager)?;
```

### 4. Error Handling

```rust
use libs::errors::AppError;

match manager.add_contact(...) {
    Ok(_) => println!("Success!"),
    Err(AppError::Validation(msg)) => eprintln!("Input error: {}", msg),
    Err(AppError::NotFound(msg)) => eprintln!("Not found: {}", msg),
    Err(e) => eprintln!("Error: {}", e),
}
```

---

## Environment Configuration

Optional `.env` file settings:

```env
# Storage
STORAGE_TYPE=json              # json, csv, txt, remote
STORAGE_PATH=contacts.json
API_URL=https://api.jsonstorage.net/v1/json/YOUR_USER_ID
API_KEY=your_jsonstorage_api_key

# Sync behavior
SOFT_DELETE_PURGE_DAYS=1       # Days before permanent deletion
FUZZY_MATCH_THRESHOLD=0.4      # Levenshtein distance ratio (0-1)
```

---

## Further Reading

## Further Reading

- **Workspace Architecture:** See [../README.md][readme-root]
- **Change History:** See [../CHANGELOG.md][changelog]
- **Performance Analysis:** See [docs/perf-notes.md][perf-notes] for benchmarks
- **Design Walkthrough:** See [docs/WALKTHROUGH.md][walkthrough] for detailed implementation notes

<!-- Link Aliases - Update paths here if documentation structure changes -->

[readme-root]: ../README.md
[changelog]: ../CHANGELOG.md
[perf-notes]: ./docs/perf-notes.md
[walkthrough]: ./docs/WALKTHROUGH.md
[cli-readme]: ../cli/README.md
[api-readme]: ../api/README.md
