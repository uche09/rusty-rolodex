# Changelog

<!--
Added: For new features.

Changed: For changes in existing functionality.

Fixed: For bug fixes.

Removed: For deprecated or removed features.
 -->


## v0.7-week-7 (07-02-2026)

### Added
- `Contact` struct now has `deleted: bool` field to enable soft delete, which will be used in data synchronization.
- New `sync_from_storage(&mut self, base: HashMap<Uuid, Contact>, storage: Box<dyn ContactStore>, policy: SyncPolicy,) -> Result<(), AppError>` method added to `ContactManager` struct to synchronize contacts from another storage medium into the current storage medium.
- New function `get_medium(&self) -> &str` added to `ContactStore` trait to return the name of the storage medium (e.g., "JSON", "CSV", "TXT", etc.).
- Synchronization error `Synchronization(String)` for AppError Enum to handle errors during synchronization operation.
- Test cases for synchronization operation added to `tests/sync.rs` file.
- New `SyncPolicy` Enum added to define different synchronization policies (e.g., LastWriteWins, Merge, etc.) that can be used during synchronization operation.
- New `LastWriteWinsPolicy` struct added to implement the Last-Write-Wins synchronization policy and its underlying operations (methods), which will be used in the `sync_from_storage` method when the LastWriteWins policy is selected.
- New `SyncDecision` Enum added to represent the possible merge decisions during synchronization (e.g., LocalWins (keep local contact), RemoteWins (merge remote contact).




### Changes
- `store` module has been renamed to `storage` module and `filestore.rs` now renamed to `stores.rs`.
- In `stores.rs` file, storage medium are now abstracted into their own struct (`stores::JsonStorage`, `stores::TxtStorage`, `stores::CsvStorage`) using the `ContactStore` trait to implement how data are read (`load()`) and stored (`save()`) on their respective storage medium.
- Former `Store<'_>` struct has been renamed to `ContactManager` and moved to `domain` module as it handles the core services of the application.
- `ContactManagaer` now has a `storage: Box<dyn ContactStore>` field that accepts any of the storage medium object that implements the ContactStore trait, accepts the object as a smart pointer which handles persistent storage.
- The implementation of `PartialEq` trait for `Contact` struct now implements an updated equality rule: Contact must have the **same ID** with the other, **OR** must have the **same name AND number** with the other to be seen as Equal (the same).
- The implementation of `Hash` trait for `Contact` struct now only hashes the `name` and `phone` fields, and no longer includes the `id` field. This is to improve Hashing time by reducing the amount of data being hashed.
- Refined storage parser (`StoreChoice` renamed to `StorageMediums`) Enum.
- Implemented the `TryFrom` trait for `StorageMediums` Enum to convert from string input to the corresponding storage medium variant, which replaces the previous `from(str: &str)` method.

### Fixed
- Delete opereation now updates the updated_at timestamp of the contact being deleted.
- Data are now saved to persistent storage after synchronization operation.
- Fixed flawed equality comparison during synchronization that caused some updates to be skipped.


### Removed
- The "path" field has been removed from `ContactManager` struct (formerly `Store<'_>`) and no longer requires the lifetime for the storage file path. The file path and storage destination/source is now handled by the individual Storage struct in stores.rs.