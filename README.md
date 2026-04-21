# Rusty Rolodex

[![Rust Version](https://img.shields.io/badge/Rust-1.78+-orange?style=flat-square&logo=rust)](https://www.rust-lang.org/)
[![Push and Pull Request Test](https://github.com/uche09/rusty-rolodex/actions/workflows/ci.yml/badge.svg?event=pull_request)](https://github.com/uche09/rusty-rolodex/actions/workflows/ci.yml)

## Project Overview

**Rusty Rolodex** is a production-grade contact management system written in Rust, demonstrating backend engineering fundamentals through a carefully architected, thoroughly tested, and performance-optimized codebase. It showcases progression from Rust fundamentals (ownership, error handling, traits) to advanced patterns (scoped concurrency, interior mutability, data synchronization strategies).

Key achievement: **Validated at scale**, benchmarked with 100k contacts, sub-270ms JSON serialization, and zero unsafe code.

---

## Table of Contents

- **[Quick Start](#quick-start)** — Run the project in 5 minutes
- **[Documentation Structure](#documentation-structure)** — Where to find what
- **[System Architecture](#system-architecture)** — Workspace design
- **[Core Technical Features](#core-technical-features)** — Storage, indexing, sync, validation
- **[Backend Engineering Proof Points](#backend-engineering-proof-points)** — Testing, performance, quality
- **[Advanced Rust Concepts](#advanced-rust-concepts-demonstrated)** — Ownership, traits, concurrency
- **[Module Organization](#module-organization)** — Crate structure
- **[Testing & Code Quality](#testing--code-quality)** — How we ensure reliability
- **[How to Run](#how-to-run)** — Build and test locally

---

## Quick Start

### For CLI Users

```bash
git clone https://github.com/uche09/rusty-rolodex.git
cd rusty-rolodex
cargo run -p cli -- add --name "Alice" --phone "+234123456789"
cargo run -p cli -- list
```

👉 **[Full CLI Guide →][cli-readme]**

### For API Users

```bash
cargo run -p api  # Starts server on http://localhost:3000
curl http://localhost:3000/health
```

👉 **[Full API Guide →][api-readme]**

### For Library Integration

```toml
[dependencies]
libs = { path = "path/to/libs" }
```

👉 **[Library Integration Guide →][libs-readme]**

---

## Documentation Structure

This workspace uses a **three-level documentation strategy:**

| Level         | Location                                                | For                                              | Links to                                         |
| ------------- | ------------------------------------------------------- | ------------------------------------------------ | ------------------------------------------------ |
| **Workspace** | Root [`README.md`](README.md)                           | Project overview, architecture, design decisions | Architecture diagrams, testing approach          |
| **Package**   | `libs/`, `cli/`, `api/` [`README.md`](./libs/README.md) | Package-specific usage and integration           | Core concepts, API reference, configuration      |
| **Details**   | `docs/` subdirectories                                  | Deep-dive analysis and examples                  | Performance benchmarks, changelogs, walkthroughs |

**Start here based on your role:**

- 👤 **Using the CLI?** → [cli/README.md][cli-readme] for commands, examples, configuration
- 🔧 **Building an API or app?** → [libs/README.md][libs-readme] for library API, storage backends, integration
- 🚀 **Running the API server?** → [api/README.md][api-readme] for endpoints, deployment, configuration
- 🏗️ **Understanding the architecture?** → Continue reading this document

---

## System Architecture

**TL;DR:** Workspace with three member crates: `libs` (domain logic + storage), `cli` (command-line interface), and `api` (REST HTTP server). Clean layered architecture with trait-based pluggable backends (JSON, CSV, TXT, HTTP) and service layer abstracting business logic.

```
┌──────────────────────────── Workspace ────────────────────────────┐
│                                                                   │
│  ┌──────────────────────┐  ┌──────────────────┐  ┌─────────────┐  │
│  │  libs/ (Library)     │  │  cli/ (CLI)      │  │  api/ (REST)│  │
│  │                      │  │                  │  │             │  │
│  │ ┌────────────────┐   │  │  ┌──────────────┐│  │ ┌────────┐  │  │
│  │ │Storage Layer   │   │  │  │CLI Interface ││  │ │HTTP    │  │  │
│  │ │(JSON/CSV/TXT/  │   │  │  │(clap, cmds)  ││  │ │Handlers│  │  │
│  │ │HTTP)           │   │  │  └────────┬─────┘│  │ └───┬────┘  │  │
│  │ └──────▲─────────┘   │  │           │      │  │     │       │  │
│  │        │             │  │      ┌────▼────┐ │  │  ┌──▼──────┐│  │
│  │ ┌──────┴───────┐     │  │      │Service  │ │  │  │Service  ││  │
│  │ │Domain Layer  │     │  │      │(Manager)│ │  │  │Layer    ││  │
│  │ │(Manager,     │◄────┼──┼──────│(Arc<    │ │  │  │(business││  │
│  │ │validation,   │     │  │      │ RwLock>)│ │  │  │logic)   ││  │
│  │ │sync,         │     │  │      └─────────┘ │  │  └───│─────┘│  │
│  │ │indexing)     │     │  │                  │  │      │      │  │
│  │ └─────▲────────┘     │  │                  │  │      │      │  │
│  └───────│──────────────┘  └──────────────────┘  └──────│──────┘  │
│          │                                              │         │
│          │                                              │         │
│          │                                              │         │
│          └────── Shared ContactManager◄─────────────────┘         │
│                  (via Arc<RwLock<>>)                              │
│                                                                   │
└───────────────────────────────────────────────────────────────────┘
```

**Design Philosophy:** Workspace separation enables:

- **libs** crate: Core business logic (domain models, storage backends, sync policy) as a reusable library
- **cli** crate: Command-line interface with `clap` binding to managers/stores from **libs**
- **api** crate: REST HTTP server with `Axum` and a **Service Layer** (ContactService) that abstracts business logic from HTTP handlers
- **Strategy pattern** for pluggable storage backends enables runtime flexibility without coupling interfaces to storage implementation. All backends satisfy the `ContactStore` trait, allowing seamless format migration and extensibility.
- **Service Layer Pattern** separates HTTP concerns (routing, status codes, validation) from business logic (sync, filtering, operations). ContactService provides unit-testable core logic reusable by CLI and API.

---

## Core Technical Features

<details>
<summary><strong>TL;DR:</strong> Multi-backend storage (JSON/CSV/TXT/HTTP), indexed O(1) search, Last-Write-Wins sync, type-safe validation, clap CLI.</summary>

### 1. **Multi-Backend Storage System**

Trait-based abstraction supporting four storage backends:

| Backend    | Implementation                                        | Use Case                                                        |
| ---------- | ----------------------------------------------------- | --------------------------------------------------------------- |
| **JSON**   | In-memory `HashMap<Uuid, Contact>` + file persistence | Default; fast structured queries, serialization straightforward |
| **TXT**    | Custom deserialization via state machine              | Lightweight human-readable format, slower for reads             |
| **CSV**    | CSV reader/writer via `csv` crate with serde          | Import/export workflows, third-party tool compatibility         |
| **Remote** | HTTP API via `reqwest` blocking client                | Cloud sync, distributed workflows via jsonstorage.net           |

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

- [add.rs][test-add] : Happy path, duplicate detection, validation error handling
- [list.rs][test-list] : Sorting (name, email, created date, updated date), tag filtering
- [delete.rs][test-delete] : Soft delete behavior, identifier matching, recovery
- [edit_search.rs][test-edit-search] : Field updates, partial name matching, case sensitivity
- [import_export.rs][test-import-export] : CSV round-trip, format migration, data integrity
- [sync.rs][test-sync] : Last-write-wins conflict resolution, field-level merging, remote integration

**Unit Tests:** Embedded in libs modules (contact validation, error display, index building, sync logic)

**Test Framework:** `assert_cmd` + `predicates` for end-to-end CLI testing; direct `ContactManager` instantiation for unit tests

**Coverage:** Validation failures, duplicate scenarios, multi-contact updates, conflict edges cases

### Performance & Scalability

**Criterion Benchmarks:** Measured across 1k, 5k, 10k, 20k, 50k, 100k contacts

| Operation         | 1k Contacts | 100k Contacts | Scaling     | Optimization                                   |
| ----------------- | ----------- | ------------- | ----------- | ---------------------------------------------- |
| **Add**           | 0.24ms      | 54ms          | O(n)        | Index updates dominate; pre-allocated capacity |
| **Search (name)** | 0.40ms      | 30ms          | Superlinear | Index lookup + fuzzy scoring                   |
| **List + Filter** | 0.16ms      | 56ms          | O(n)        | Linear iteration required                      |
| **Save (JSON)**   | 2.06ms      | 270ms         | O(n)        | Serialization bottleneck; largest cost         |
| **Index Build**   | 0.21ms      | 48ms          | O(n)        | Multi-threaded with adaptive worker count      |

**Performance Analysis:** See [libs/docs/perf-notes.md][perf-notes] for detailed methodology and findings.

**Thread Work Distribution Heuristic** (see [manager.rs#L637][manager-rs]):

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

**GitHub Actions Pipeline** ([.github/workflows/ci.yml][ci-yml]):

```yaml
on: [push, pull_request to main]
jobs:
  - cargo clippy # Linting warnings as hard errors
  - cargo fmt # Code formatting enforcement
  - cargo test # Serial test execution (prevents race conditions)
  - cargo build # Release build validation
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

**Backward Compatibility:** Auto-migration from old `Vec` format to new `HashMap` structure via [file.rs][storage-file]

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

**ContactStore Trait** (see [storage/mod.rs][storage-mod]):

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

**Comprehensive AppError Enum** (see [errors.rs][errors-rs]):

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

**Scoped Threads (Rust 1.63+)** (see [manager.rs#L427][manager-rs-427]):

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

**RefCell for Mutable State Without `&mut`** (see [storage/remote.rs][storage-remote]):

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

**👉 See [cli/README.md](./cli/README.md) for command reference and examples.**

---

## Module Organization

**Workspace Structure:** Three member crates with clean separation of concerns:

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

### `api/` Crate

REST HTTP server with service layer abstraction:

- `src/main.rs` — Axum server startup, route setup, configuration
- `src/service.rs` — **ContactService** layer (business logic, testing via unit tests in `#[cfg(test)]`)
- `src/routes/contacts.rs` — HTTP handlers (GET, POST, PATCH, DELETE contact endpoints)
- `src/config.rs` — Environment configuration
- `src/state.rs` — ApiState with Arc<ContactService>
- `src/validation.rs` — Request payload validation schemas
- `src/error.rs` — HTTP error responses
- `tests/integration.rs` — Integration tests covering HTTP endpoints and soft-delete edge cases

**Four-Tier Architecture** (across workspace):

1. **HTTP Layer** (api/routes) — Axum handlers, request routing, status codes
2. **Service Layer** (api/service) — ContactService business logic, filtering, sync (unit tested)
3. **Domain Layer** (libs/domain) — ContactManager, Contact entity, validation, indexing
4. **Storage Layer** (libs/storage) — Trait-based backends for JSON, CSV, TXT, HTTP

---

## How to Run

### Prerequisites

- Rust 1.78+ ([install here](https://www.rust-lang.org/tools/install))
- Cargo (comes with Rust)

### Build & Run

```bash
# Clone the repository
git clone https://github.com/uche09/rusty-rolodex.git
cd rusty-rolodex

# Run the CLI
cargo run -p cli -- add --name "John Doe" --phone +1234567890
cargo run -p cli -- list

# Run the API
cargo run -p api  # Starts on http://localhost:3000

# Run all tests
cargo test

# Run benchmarks (Criterion)
cargo bench -p libs
```

**For detailed instructions:**

- **CLI users:** [cli/README.md][cli-readme-install]
- **API users:** [api/README.md][api-readme-quickstart]
- **Library integration:** [libs/README.md][libs-readme-quickstart]

---

## Module Organization

**Workspace Structure:** Three member crates with clean separation of concerns:

### `libs/` Crate

Core business logic exposed as a reusable library:

- `src/domain/` — Contact entity, ContactManager (CRUD, search, indexing, sync), validation
- `src/storage/` — ContactStore trait + four implementations (JSON, CSV, TXT, Remote HTTP)
- `src/errors.rs` — Unified error handling (AppError enum with From conversions)
- `src/prelude.rs` — Public API exports for downstream crates
- `benches/` — Criterion benchmarks across 1k–100k contact scales
- `docs/` — Architecture deep-dives, usage guides, performance analysis

👉 **[Full library guide →][libs-readme]**

### `cli/` Crate

Command-line interface consuming the `libs` crate:

- `src/main.rs` — Entry point, clap argument parsing, command routing
- `src/cli_component/` — Command implementations calling into libs managers
- `examples/` — Shell script examples demonstrating CLI workflows
- `tests/` — Integration test suites (add, delete, edit_search, import_export, list, sync)

👉 **[Full CLI guide →][cli-readme]**

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

## Further Reading

| Topic               | Location                                | Purpose                                           |
| ------------------- | --------------------------------------- | ------------------------------------------------- |
| **CLI Usage**       | [cli/README.md][cli-readme]             | Commands, examples, troubleshooting               |
| **Library API**     | [libs/README.md][libs-readme]           | Integration guide, core concepts, modules         |
| **REST API**        | [api/README.md][api-readme]             | Endpoints, service layer, deployment, config      |
| **Changelog**       | [CHANGELOG.md][changelog]               | Release history, version notes                    |
| **Performance**     | [libs/docs/perf-notes.md][perf-notes]   | Benchmarks, analysis, optimization                |
| **Walkthrough**     | [libs/docs/WALKTHROUGH.md][walkthrough] | Implementation deep-dives                         |
| **Service Pattern** | [api/src/service.rs][api-service]       | ContactService layer, unit tests, filtering logic |

---

**If you find this project valuable, please consider giving it a star ⭐**, it motivates continued development and demonstrates the project's impact to potential collaborators and employers.

<!-- Link Aliases - Update paths here if documentation structure changes -->

[readme-root]: ./README.md
[cli-readme]: ./cli/README.md
[cli-readme-install]: ./cli/README.md#installation
[api-readme]: ./api/README.md
[api-readme-quickstart]: ./api/README.md#quick-start
[libs-readme]: ./libs/README.md
[libs-readme-quickstart]: ./libs/README.md#quick-start
[changelog]: ./CHANGELOG.md
[perf-notes]: ./libs/docs/perf-notes.md
[walkthrough]: ./libs/docs/WALKTHROUGH.md
[ci-yml]: ./.github/workflows/ci.yml
[test-add]: ./cli/tests/add.rs
[test-list]: ./cli/tests/list.rs
[test-delete]: ./cli/tests/delete.rs
[test-edit-search]: ./cli/tests/edit_search.rs
[test-import-export]: ./cli/tests/import_export.rs
[test-sync]: ./cli/tests/sync.rs
[manager-rs]: ./libs/src/domain/manager.rs#L637
[manager-rs-427]: ./libs/src/domain/manager.rs#L427
[storage-mod]: ./libs/src/storage/mod.rs
[storage-file]: ./libs/src/storage/file.rs#L100
[storage-remote]: ./libs/src/storage/remote.rs
[errors-rs]: ./libs/src/errors.rs
[api-service]: ./api/src/service.rs
