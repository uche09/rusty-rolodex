# Rusty Rolodex

[![Rust Version](https://img.shields.io/badge/Rust-1.78+-orange?style=flat-square&logo=rust)](https://www.rust-lang.org/)
[![Push and Pull Request Test](https://github.com/uche09/rusty-rolodex/actions/workflows/ci.yml/badge.svg)](https://github.com/uche09/rusty-rolodex/actions/workflows/ci.yml)

## Project Overview

**Rusty Rolodex** is a production-grade contact management system written in Rust, demonstrating backend engineering fundamentals through a carefully architected, thoroughly tested, and performance-optimized codebase. It showcases progression from Rust fundamentals (ownership, error handling, traits) to advanced patterns (scoped concurrency, interior mutability, data synchronization strategies).

Key achievement: **Validated at scale**, benchmarked with 100k contacts, sub-270ms JSON serialization, and zero unsafe code.

---

## System Architecture

**TL;DR:** Workspace with two member crates: `libs` (domain logic + storage) and `cli` (command-line interface). Clean three-tier architecture (CLI -> Domain -> Storage) with pluggable backends (JSON, CSV, TXT, HTTP) via trait-based abstraction.

```
┌────────────────────────── Workspace ─────────────────────────┐
│                                                              │
│    ┌─────────────────────────────────┐  ┌──────────────────┐ │
│    │    libs/ (Library Crate)        │  │  cli/ (CLI Crate)│ │
│    │                                 │  │                  │ │
│    │  ┌────────────────────────────┐ │  │  ┌─────────────┐ │ │
│    │  │  Storage Layer             │ │  │  │CLI Interface│ │ │
│    │  │  (JSON/CSV/TXT/HTTP)       │ │  │  │(clap, cmds) │ │ │
│    │  └────────────────────────────┘ │  │  └──────┬──────┘ │ │
│    │           ▲                     │  │         │        │ │
│    │           │                     │  │         │        │ │
│    │  ┌────────┴───────────────────┐ │  │  ┌──────▼──────┐ │ │
│    │  │  Domain Layer              │ │  │  │ContactMgr   │ │ │
│    │  │  (Manager, validation,     │ ◄──┼──┤(exported)   │ │ │
│    │  │   sync, indexing)          │ │  │  └─────────────┘ │ │
│    │  └────────────────────────────┘ │  │                  │ │
│    │                                 │  │                  │ │
│    └─────────────────────────────────┘  └──────────────────┘ │
│                                                              │
└──────────────────────────────────────────────────────────────┘
```

**Design Philosophy:** Workspace separation enables:
- **libs** crate: Core business logic (domain models, storage backends, sync policy) as a reusable library
- **cli** crate: Command-line interface with `clap` binding to managers/stores from **libs**
- Strategy pattern for pluggable storage backends enables runtime flexibility without coupling CLI to storage implementation. All backends satisfy the `ContactStore` trait, allowing seamless format migration and extensibility.

---

## Core Technical Features

<details>
<summary><strong>TL;DR:</strong> Multi-backend storage (JSON/CSV/TXT/HTTP), indexed O(1) search, Last-Write-Wins sync, type-safe validation, clap CLI.</summary>

### 1. **Multi-Backend Storage System**

Trait-based abstraction supporting four storage backends:

| Backend | Implementation | Use Case |
|---------|---|---|
| **JSON** | In-memory `HashMap<Uuid, Contact>` + file persistence | Default; fast structured queries, serialization straightforward |
| **TXT** | Custom deserialization via state machine | Lightweight human-readable format, slower for reads |
| **CSV** | CSV reader/writer via `csv` crate with serde | Import/export workflows, third-party tool compatibility |
| **Remote** | HTTP API via `reqwest` blocking client | Cloud sync, distributed workflows via jsonstorage.net |

**Key Implementation Details:**
- `ContactStore` trait (in `libs/src/storage/mod.rs`) with three core methods: `load()`, `save()`, `get_medium()`
- Format migration: auto-detects old `Vec` format and migrates to new `HashMap` structure (libs/src/storage/file.rs)
- Remote backend uses `RefCell<Option<String>>` for interior mutability (libs/src/storage/remote.rs)
- Fallback logic: Remote storage attempts PUT first, falls back to POST on initial upload, then uses resource ID for updates

### 2. **Advanced Search & Indexing**

Two-tier indexing strategy achieving O(1) lookup performance (see `libs/src/domain/indexing.rs`):

**Name Index:** `HashMap<String, HashSet<Uuid>>`
- Splits contact names by whitespace; each word indexed separately
- Enables partial name search: query "John" returns "John Doe"
- Pre-allocated capacity (2× contact count) to avoid repeated allocations

**Email Domain Index:** Similar structure for domain-based queries

**Fuzzy Search:** Levenshtein distance matching with configurable threshold (0.4 minimum similarity)
- Returns top 10 results sorted by match score
- Distances converted to `i32` (×1000) for HashSet compatibility (floats lack Eq+Hash)

**Multi-threaded Index Building:** Work distributed across 1-5 threads based on data size (see [Thread Work Distribution](#thread-work-distribution-heuristic) below).

### 3. **Sync & Conflict Resolution**

**Last-Write-Wins (LWW) Policy:** Custom conflict resolution strategy in `libs/src/domain/manager.rs`
- Field-level merge: resolved per field (name, phone, email, tag) based on timestamp
- Soft delete handling: deleted flag propagates to remote storage
- Prevents conflicts from missing `created_at` verification by comparing `updated_at` timestamps

**Merge Algorithm:**
```rust
// Pseudocode: field-level resolution
if local.deleted -> mark remote.deleted
if local.updated_at >= remote.updated_at -> use local
else -> use remote
```

**Soft-Delete with Recovery Window:**
- Deleted contacts soft-deleted (flag set) rather than hard-deleted
- `purge_soft_deleted_older_than(days)` removes old deletions (default: 1 day)
- Prevents accidental data loss during sync sessions; allows recovery period

### 4. **Validation at Boundaries**

Type-safe validation enforced on all user inputs:

**Contact Validation Regex Patterns:**
- **Name:** `^[A-Za-z][A-Za-z\s'-\.]*\w*$` (50 char max), letters, spaces, hyphens, apostrophes
- **Phone:** `^\+?\d{10,15}$` (international format), country code optional, 10-15 digits
- **Email:** `^[^@\s]+@[^@\s]+\.[^@\s]+$` (254 char max, optional field)

**Custom Phone Matching:** `phone_number_matches()` function compares last 7 digits after stripping country codes or leading zeros, enabling **duplicate detection across format variations** (e.g., "08861473537" matches "+2348861473537").

**Type-Safe Enums:** `SortKey`, `SearchKey`, `ImportExportOption` prevent invalid command combinations at compile time.

### 5. **CLI Architecture**

Built with `clap` derive macros for compile-time argument validation:

- `#[derive(Parser)]` on `Cli` struct (auto help/version generation)
- `#[derive(Subcommand)]` for command variants (Add, List, Edit, Delete, Search, Import, Export)
- `#[arg(long, env = "...", default_value_t = ...)]` binds environment variables and defaults
- `ValueEnum` for type-safe sort keys and search modes
- Environment variable configuration: `STORAGE_TYPE`, `STORAGE_PATH`, `API_URL`

</details>

---

## Backend Engineering Proof Points

<details>
<summary><strong>TL;DR:</strong> 6 integration test suites, benchmarked at 100k scale, zero unsafe code, GitHub Actions CI, soft deletes, backward compatibility.</summary>

### Testing Strategy

**Integration Tests (6 suites, in `cli/tests/`):**
- [add.rs](./cli/tests/add.rs) : Happy path, duplicate detection, validation error handling
- [list.rs](./cli/tests/list.rs) : Sorting (name, email, created date, updated date), tag filtering
- [delete.rs](./cli/tests/delete.rs) : Soft delete behavior, identifier matching, recovery
- [edit_search.rs](./cli/tests/edit_search.rs) : Field updates, partial name matching, case sensitivity
- [import_export.rs](./cli/tests/import_export.rs) : CSV round-trip, format migration, data integrity
- [sync.rs](./cli/tests/sync.rs) : Last-write-wins conflict resolution, field-level merging, remote integration

**Unit Tests:** Embedded in libs modules (contact validation, error display, index building, sync logic)

**Test Framework:** `assert_cmd` + `predicates` for end-to-end CLI testing; direct `ContactManager` instantiation for unit tests

**Coverage:** Validation failures, duplicate scenarios, multi-contact updates, conflict edges cases

### Performance & Scalability

**Criterion Benchmarks:** Measured across 1k, 5k, 10k, 20k, 50k, 100k contacts

| Operation | 1k Contacts | 100k Contacts | Scaling | Optimization |
|-----------|---|---|---|---|
| **Add** | 0.24ms | 54ms | O(n) | Index updates dominate; pre-allocated capacity |
| **Search (name)** | 0.40ms | 30ms | Superlinear | Index lookup + fuzzy scoring |
| **List + Filter** | 0.16ms | 56ms | O(n) | Linear iteration required |
| **Save (JSON)** | 2.06ms | 270ms | O(n) | Serialization bottleneck; largest cost |
| **Index Build** | 0.21ms | 48ms | O(n) | Multi-threaded with adaptive worker count |

**Performance Analysis:** See [perf-notes.md](./libs/docs/perf-notes.md) for detailed methodology and findings.

**Thread Work Distribution Heuristic** (see [manager.rs#L637](./libs/src/domain/manager.rs#L637)):
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
Scales threading only when data size justifies context-switch cost. Prevents under-utilization on small datasets and over-subscription on large ones.

**Lock Contention Reduction:**
- Local accumulation: Each thread builds local `HashMap<String, HashSet<Uuid>>`, then merges once (vs. locking on each insert)
- Arc cloning: Reduces pointer dereferencing in closure captures
- Result type conversion: `i32` distance storage avoids float-based HashSet incompatibility

### CI/CD & Code Quality

**GitHub Actions Pipeline** ([.github/workflows/ci.yml](./.github/workflows/ci.yml)):
```yaml
on: [push, pull_request to main]
jobs:
  - cargo clippy      # Linting warnings as hard errors
  - cargo fmt         # Code formatting enforcement
  - cargo test        # Serial test execution (prevents race conditions)
  - cargo build       # Release build validation
```

**Code Quality Signals:**
- **Zero unsafe code**: All concurrency via safe primitives (thread::scope, Arc, Mutex)
- **Comprehensive error handling**: 10 error variants in `AppError` enum with Display impl; no panics in normal code paths
- **Error type conversion**: `impl From<T> for AppError` for all std library error types enables `?` operator chaining
- **Clippy integration**: Enforces idiomatic Rust patterns
- **Serial test execution**: Prevents timing-dependent test failures

### Production-Ready Patterns

**Soft Deletes:** Marked with `deleted` flag; hard purge on configurable schedule (default: 1 day)
- Prevents accidental data loss
- Enables recovery until purge window expires
- Reduces HashMap shrinking cost during active sessions

**Backward Compatibility:** Auto-migration from old `Vec` format to new `HashMap` structure via [file.rs](./libs/src/storage/file.rs#L100)
- Detects format version on deserialization
- Transforms old contacts to new UUID-based structure
- Zero data loss; seamless upgrade path

**Rollback on Failure:** `import_contacts_from_storage()` calls `save()?` to restore previous state on error
- Ensures data consistency on failed imports
- Prevents partial updates from corrupting state

**Poison Error Handling:** Catches thread panics in critical sections
```rust
impl<T> From<PoisonError<T>> for AppError {
    fn from(err: PoisonError<T>) -> Self {
        AppError::Poison(err.to_string())
    }
}
```
Allows safe recovery from mutex poisoning without unwrap.

</details>

---

## Advanced Rust Concepts Demonstrated

<details>
<summary><strong>TL;DR:</strong> Ownership/borrowing (zero-copy), traits (4 storage backends), error handling (?-chains), concurrency (scoped threads, Arc, Mutex), interior mutability, design patterns.</summary>

### 1. **Ownership & Borrowing**

**Zero-Copy Search Results:** `Vec<&Contact>` returns references to manager's HashMap
```rust
pub fn search(&self, ...) -> Result<Vec<&Contact>, AppError>
```
Eliminates clone overhead while maintaining lifetime safety.

**Borrowing for Iteration:** `for contact in &contact_list[start..end]` avoids unnecessary moves

**Move Semantics in Storage:** `Box<dyn ContactStore>` enables runtime polymorphism while maintaining ownership guarantees

### 2. **Traits & Generics**

**ContactStore Trait** (see [storage/mod.rs](./libs/src/storage/mod.rs)):
```rust
pub trait ContactStore {
    fn load(&self) -> Result<HashMap<Uuid, Contact>, AppError>;
    fn save(&self, contacts: &HashMap<Uuid, Contact>) -> Result<(), AppError>;
    fn get_medium(&self) -> &str;
}
```
Four implementations (JSON, CSV, TXT, Remote) without coupling CLI to storage detail.

**Generic Error Conversion:**
```rust
impl<T> From<PoisonError<T>> for AppError { ... }
```
Allows any type wrapped in PoisonError to convert automatically.

### 3. **Error Handling**

**Result-Based Control Flow:** Pervasive use of `Result<T, AppError>` throughout
- 14 `impl From<T> for AppError` conversions enable automatic error type conversion
- `?` operator chains simplify error propagation without bloat
- Zero panics in normal code paths; all errors surface as Results

**Comprehensive AppError Enum** (see [errors.rs](./libs/src/errors.rs)):
```rust
pub enum AppError {
    CsvError(csv::Error),
    DateTime(chrono::ParseError),
    FailedRequest(reqwest::Error),
    Io(std::io::Error),
    JsonPerser(serde_json::Error),
    NotFound(String),
    Poison(String),
    RegexError(regex::Error),
    Synchronization(String),
    Validation(String),
}
```
Display trait provides user-friendly messages vs. internal error details.

### 4. **Concurrency**

**Scoped Threads (Rust 1.63+)** (see [manager.rs#L427](./libs/src/domain/manager.rs#L427)):
```rust
std::thread::scope(|s| {
    for chunk in work_chunks {
        s.spawn(move || { /* build local index */ });
    }
});
```
Guarantees thread lifetime matches scope; no detached thread complexity.

**Arc for Reference Counting:** Cloned across thread closures for shared immutable state
```rust
let shared_data = Arc::new(manager_data);
s.spawn(move || { let local = Arc::clone(&shared_data); ... });
```

**Mutex + Poison Handling:** Thread-safe mutable state with panic recovery
```rust
Arc<Mutex<HashMap<..>>>.lock()? -> Result catches poisoned locks
```

**Zero Unsafe Code:** All concurrency via safe primitives; no raw pointers, no manual memory management

### 5. **Interior Mutability**

**RefCell for Mutable State Without `&mut`** (see [storage/remote.rs](./libs/src/storage/remote.rs)):
```rust
pub struct RemoteStorage {
    active_url: RefCell<Option<String>>,
    // ...
}
```
Allows mutable `active_url` field updates in methods taking `&self` (vs. `&mut self`), solving borrow checker constraints for stateful HTTP clients.

### 6. **Design Patterns**

**Strategy Pattern:** Storage backends implement `ContactStore` trait, enabling runtime selection
```rust
let storage: Box<dyn ContactStore> = match config.storage_type {
    StorageType::Json => Box::new(JsonStorage::new(...)),
    StorageType::Remote => Box::new(RemoteStorage::new(...)),
    // ...
};
```

**Template Method:** `LastWriteWinsPolicy` struct encapsulates merge algorithm
```rust
impl LastWriteWinsPolicy {
    fn conflict_resolution(&self, local: &Contact, remote: &Contact) -> SyncDecision { ... }
}
```
Allows strategy swapping without modifying manager code.

**Factory Pattern:** `parse_storage_type_env_config()` creates storage instances from environment config

### 7. **Serde Customization** (see `libs/src/domain/contact.rs`):

**Custom Deserialization with Version Migration:**
```rust
#[serde(deserialize_with = "deserialize_deleted_field")]
pub deleted: bool,
```
Handles format changes gracefully; old contacts gain default `deleted: false`.

**Default Values with UUID Generation:**
```rust
#[serde(default = "Uuid::new_v4")]
pub id: Uuid,
```
Ensures backward compatibility when new fields are added.

**Derive Macros:** `#[derive(Serialize, Deserialize)]` combined with attribute customization reduces boilerplate while enabling sophisticated serialization logic.

</details>

---

## Current Features

- ✅ Add/edit/delete contacts with name, phone, email, tags
- ✅ View and search contacts (by name or email domain)
- ✅ Sort contacts by name, email, creation date, or update date
- ✅ Tag-based filtering
- ✅ Import/export from CSV with format migration
- ✅ Multi-backend storage (JSON, CSV, TXT, HTTP API)
- ✅ Conflict-free synchronization with last-write-wins policy
- ✅ Soft deletes with configurable retention window
- ✅ Fuzzy search with Levenshtein distance
- ✅ Comprehensive validation (regex + type safety)
- ✅ Production-grade testing (6 integration suites, unit tests)
- ✅ Performance-optimized (indexed search, adaptive threading)


## What I Learned

- **Rust Fundamentals:** Ownership, borrowing, traits, error handling, concurrency (scoped threads, Arc, Mutex), collections, pattern matching
- **Production Engineering:** Testing (6 integration suites), performance optimization, benchmarking, soft deletes, backward compatibility, zero unsafe code
- **Design Patterns:** Strategy (storage backends), Template Method (sync policy), Factory (storage init), Interior Mutability (RefCell)

---

## Module Organization

**Workspace Structure:** Two member crates with clean separation of concerns:

### `libs/` Crate
Core business logic exposed as a reusable library:
- `src/domain/` — Contact entity, ContactManager (CRUD, search, indexing, sync), validation
- `src/storage/` — ContactStore trait + four implementations (JSON, CSV, TXT, Remote HTTP)
- `src/errors.rs` — Unified error handling (AppError enum with From conversions)
- `src/prelude.rs` — Public API exports for downstream crates
- `benches/` — Criterion benchmarks across 1k–100k contact scales
- `docs/` — Architecture deep-dives, usage guides, performance analysis

### `cli/` Crate
Command-line interface consuming the `libs` crate:
- `src/main.rs` — Entry point, clap argument parsing, command routing
- `src/cli_component/` — Command implementations calling into libs managers
- `examples/` — Shell script examples demonstrating CLI workflows
- `tests/` — Integration test suites (add.rs, delete.rs, edit_search.rs, import_export.rs, list.rs, sync.rs)

**Three-Tier Architecture** (implemented within libs crate):
1. **Storage Layer** (storage/) — Trait-based backends for JSON, CSV, TXT, HTTP
2. **Domain Layer** (domain/) — ContactManager, Contact entity, validation, indexing, sync logic
3. **CLI Layer** (cli/) — clap-based argument parsing and command dispatch

---

## Example Usage

```bash
# Add a contact
add --name Jerry --phone 08861473537
# Contact added successfully

add --name Alice --phone +234123456789 --email alice@gmail.com --tag friends
# Contact added successfully

# List all contacts (default: insertion order)
list
  1. Jerry                08861473537     
  2. Alice               +234123456789   alice@gmail.com

# Sort by name
list --sort name
  1. Alice               +234123456789   alice@gmail.com
  2. Jerry               08861473537     

# Filter by tag
list --tag friends
  1. Alice               +234123456789   alice@gmail.com

# Search by name
search --by N --name Alice
  1. Alice               +234123456789   alice@gmail.com

# Search by email domain
search --by D --domain gmail.com
  1. Alice               +234123456789   alice@gmail.com

# Edit a contact
edit --name Alice --new_email alice.updated@example.com
# Contact updated successfully

# Delete a contact
delete --name Jerry
# Contact deleted successfully

# Import from CSV
import --src contacts.csv
# Contacts imported successfully

# Export to CSV
export --des backup.csv
# Contacts exported successfully

# View help
cargo run -- --help
cargo run -- add --help
```

See [USAGE.md](./libs/docs/USAGE.md) for detailed command documentation. Also check [WALKTHROUGH.md](./libs/docs/WALKTHROUGH.md) for architecture deep-dive and [perf-notes.md](./libs/docs/perf-notes.md) for benchmarking analysis. Example shell scripts are provided in [cli/examples/](./cli/examples/).

---

## How to Run

### Prerequisites
- Rust 1.78+ ([install here](https://www.rust-lang.org/tools/install))
- Cargo (comes with Rust)

The project is organized as a **Cargo workspace** with two member crates:
- `libs/` — Core business logic library (domain, storage, sync, indexing)
- `cli/` — Command-line interface consuming libs

### Build & Run

```bash
# Clone the repository
git clone https://github.com/uche09/rusty-rolodex.git
cd rusty-rolodex

# Build and run the CLI (from workspace root, runs cli/ crate)
cargo run -- add --name "John Doe" --phone +1234567890

# Build cli release binary specifically
cargo build --release -p cli

# Run tests 
cargo test                              # All tests (cli integration + libs unit/integration)
cargo test -p libs                      # Libs tests only
cargo test -p cli                       # CLI integration tests only
cargo test --lib -p libs                # Libs unit tests only

# Run benchmarks (Criterion - libs only)
cargo bench -p libs

# Use environment variables to configure storage (applies to CLI)
export STORAGE_TYPE=json  # Options: json, csv, txt, remote
export STORAGE_PATH=./data/contacts.json
cargo run -- list
```

### Configuration

Set environment variables to customize storage:

```bash
STORAGE_TYPE    # json (default) | csv | txt | remote
STORAGE_PATH    # File path for json/csv/txt (default: ./data/contacts.json)
API_URL         # Base URL for remote storage (default: https://jsonblob.com/api/jsonblob)
```

---

## Performance Notes

**Time Complexity:** Add O(n), Search O(1)+scoring, List O(n log n), Save O(n). **Space Complexity:** Indexes use ~3× original data size.

See [perf-notes.md](./libs/docs/perf-notes.md) for benchmark analysis at 1k–100k scales.

**Synchronization:** Last-Write-Wins policy with field-level merge and soft deletes (recovery window: 1 day). See [manager.rs#L120](./libs/src/domain/manager.rs#L120).

**Custom Backends:** To add a custom storage backend, implement the `ContactStore` trait (from `libs` crate) with `load()`, `save()`, `get_medium()`. See [Remote storage implementation](./libs/src/storage/remote.rs) for HTTP example (~150 lines). Then register in CLI via [cli component](./cli/src/cli_component/).

---

## Testing & Code Quality

**Test Coverage:**
- **libs** crate: Unit tests embedded in each module (contact validation, error handling, sync logic, storage backends)
- **cli** crate: 6 integration test suites covering happy paths, edge cases, error scenarios via `assert_cmd` + `predicates`
  - add.rs — Duplicate detection, validation error handling
  - delete.rs — Soft delete behavior, identifier matching
  - edit_search.rs — Field updates, partial name matching, fuzzy search
  - import_export.rs — CSV round-trip, format migration
  - list.rs — Sorting (name, email, dates), tag filtering
  - sync.rs — Last-write-wins conflict resolution, remote integration

**Code Quality Tools:**
- `cargo clippy` enforces idiomatic Rust patterns
- `cargo fmt` ensures consistent formatting
- GitHub Actions CI runs on every push/PR (clippy, fmt, test, release build)

**Running Tests:**
```bash
cargo test                                # Run all tests (workspace)
cargo test -p libs                        # Libs unit + integration tests
cargo test -p cli                         # CLI integration tests only
cargo test --lib -p libs                  # Libs unit tests only
cargo test -p cli --test add              # Specific integration test
cargo test -- --nocapture                 # Print output during tests
```

---

**If you find this project valuable, please consider giving it a star ⭐**, it motivates continued development and demonstrates the project's impact to potential collaborators and employers.

**Last Updated:** March 2026 | **Rust Version:** 1.78+