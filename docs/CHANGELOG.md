# Changelog

## v0.1-week-1 (22-08-2025)

<!--
Added: For new features.

Changed: For changes in existing functionality.

Fixed: For bug fixes.

Removed: For deprecated or removed features.
 -->

### Added
- Initialized project setup and file structure
- Create modules for Seperation of Concerns (SOC) including src/cli.rs, src/domain.rs, src/store.rs, src/validation.rs whith content as follows:
    1. **src/cli.rs:**
    This module hold construct and functions related to input, output and user interactions such as:
        - `show_menu()`
        - `confirm_action()`
        - `display_contact()`
        - `get_input()`
        - `get_input_to_lower()`
        - `get_command()`
    
    2. **src/domain.rs:**
    This module holds the main logic of the project. As a "contact book", the domain.rs currently contains:
        - `Contact` struct
            - `name` field
            - `phone` field
            - `email` field
        - `Command` enum
            - `::AddContact` variant
            - `::ListContacts` variant
            - `::DeleteContact` variant
            - `::Exit` variant
        - `ContactSore` struct
            - A `store: Store` field
            - A constructor `new()`
            - `add_contact()` method
            - `contact_list()` method
            - `delete_contact()` method
            - `get_index_by_name()` method
    3. **src/store.rs:**
    This module handles the storage how and where the contacts are stored. It contains:
        - `FILE_PATH` const
        - `Store` struct that has:
            - `mem: Vec<Contact>` field
            - `file: File` field
            - a constructor `new()`
        - `create_file_parent()` function.
    4. **src/validation.rs:** 
    This module handles all user input validation. It contains:
        - `validate_name()`
        - `validate_number()`
        - `validate_email()`
        - `contact_exist()`
- Documentation


## v0.1-week-2 (2-09-2025)

### Added
- Allow memory persistency by storing contact in .txt file.
- GitHub workflow to test branch on PR.
- Custom error messages with `enum AppError` for a unified custom Error handling.
- Tests features.
- Generic function to reduce duplicate input logic.


## v0.1-week-3 (_-09-2025)

### Added
- Regex validation
- Allows storage choice (mem, text, json) via .env var. Defaults to mum if no .env value.
- `struct JsonStore` to parse and store contact in .json file.
- Migration `load_migrated_contact()` loads contacts both from legacy storage and current storage and stores them on current storage. Handles duplicate contacts when loading data.
- Add Command Line Argument Parser (CLAP) crate for neat and organized cli interaction.
- Implemented `enum ValidationReq{}` to parse validation requirement in cases of validation failure.
- Wrote test to test new functionality.

### Changes
- implemented `phone_number_matches()` to match phone numbers with or without country code eg +234123456789 and 0123456789 will match.

### Fixed
- **Partial Delete** in cases where storage choice changes from txt to json or vice versa, all contacts are read (migrated) from initial storage and saved in the current storage choice but the storage file is preserved. When a contact that was migrated is deleted from the current storage choice, a copy of it is migrated back from initial storage when app restarts.  
This fix ensures all copies are deleted.