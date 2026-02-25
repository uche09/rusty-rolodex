# Changelog

<!--
Added: For new features.

Changed: For changes in existing functionality.

Fixed: For bug fixes.

Removed: For deprecated or removed features.
 -->


## v0.8-week-8 (25-02-2026)

### Added
- `storage::remote` module to organize remote storage struct.
- `remote::RemoteStorage` struct implements `ContactStore` trait as a storage medium.
- `AppError::FailedRequest` error in error.rs to handle remote http request failures.
- Helper functions in `helper.rs` to interface reading/seting persistent env data.
- `command::ImportExportOption` enum in `command.rs` to distinguish between file or remote http operation.



### Changes
- The `storage` module now has `remote` module (`remote.rs`) to handle remote storage, and `stores.rs` now renamed to `file.rs` to handle all file storage.
- Modified `Import` and `Export` command to use `command::ImportExportOption` as a command option for either **file import/export** or **remote http import/export**.
- Refactored `ContactManager.import_contacts_from_csv()` to `ContactManager.import_contacts_from_storage()` which now **imports and synchronizes contacts** from both remote storage and selected file storage.
- Refactored `ContactManager.export_contacts_to_csv()` to `ContactManager.export_contacts_to_storage()` which now exports contacts to both remote storage and selected file storage.
- Replaced all initial env interaction with new env interfacing helper functions.


### Removed
- `Sync` command. Synchronization implicitly during import via the `Import` command.