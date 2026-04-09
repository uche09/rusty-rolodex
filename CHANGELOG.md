# Changelog

<!--
Added: For new features.

Changed: For changes in existing functionality.

Fixed: For bug fixes.

Removed: For deprecated or removed features.
 -->

## v0.8-week-8 (25-02-2026)

### Added

- `lock.rs` module for storage access cordination.

### Changes

- `file` module now uses asynchronous file operation via `tokio::fs`.
- Used async **non-blocking client** from the `reqwest` crate for http request in `storage::remote`.
- Writing into storage file (.json and .txt) now acquires an **exclusive lock** from the `lock::FileLock` struct via the `fs2` crate to aid coordinate and linearize storage access.
- Reading from storage file (.json and .txt) now acquires a **shared lock** from the `lock::FileLock` struct via the `fs2` crate to aid coordination and concurrent storage read access.

### Removed

- `Sync` command. Synchronization implicitly during import via the `Import` command.
