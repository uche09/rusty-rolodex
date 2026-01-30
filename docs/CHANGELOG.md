# Changelog

<!--
Added: For new features.

Changed: For changes in existing functionality.

Fixed: For bug fixes.

Removed: For deprecated or removed features.
 -->


## v0.7-week-7 (06-1-2026)

### Added
- `Contact` struct now has `deleted: bool` field to enable soft delete, which will be used in data synchronization.



### Changes
- `store` module has been renamed to `storage` module and `filestore.rs` now renamed to `stores.rs`.
- In `stores.rs` file, storage medium are now abstracted into their own struct (`stores::JsonStorage`, `stores::TxtStorage`, `stores::CsvStorage`) using the `ContactStore` trait to implement how data are read (`load()`) and stored (`save()`) on their respective storage medium.
- Former `Store<'_>` struct has been renamed to `ContactManager` and moved to `domain` module as it handles the core services of the application.
- `ContactManagaer` now has a `storage: Box<dyn ContactStore>` field that accepts any of the storage medium object that implements the ContactStore trait, accepts the object as a smart pointer which handles persistent storage.
- The implementation of `PartialEq` trait for `Contact` struct now implements an updated equality rule: Contact must have the **same ID** with the other, **OR** must have the **same name AND number** with the other to be seen as Equal (the same).

<!-- ### Fixed
-  -->


### Removed
- The "path" field has been removed from `ContactManager` struct (formerly `Store<'_>`) and no longer requires the lifetime for the storage file path. The file path and storage destination/source is now handled by the individual Storage struct in stores.rs.