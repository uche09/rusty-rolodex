# Rusty Rolodex CLI

A command-line contact management tool built on the Rusty Rolodex library. Add, search, organize, and sync contacts across multiple storage formats (JSON, CSV, TXT, and remote HTTP).

## Installation

### Prerequisites

- Rust 1.78+ ([Install from rustlang.org](https://www.rust-lang.org/tools/install))
- Git

### Build from Source

```bash
git clone https://github.com/uche09/rusty-rolodex.git
cd rusty-rolodex
cargo build --release
```

The executable will be at `target/release/rolodex`.

### Add to PATH (Optional)

```bash
# Copy to a directory in your PATH
cp target/release/rolodex ~/.local/bin/

# Or use a symlink
ln -s $(pwd)/target/release/rolodex ~/.local/bin/rolodex
```

Then test:

```bash
rolodex --help
rolodex --version
```

---

## Quick Start

### 1. Add a Contact

```bash
rolodex add --name "Alice Johnson" --phone "+234123456789" --email "alice@example.com"
```

**Output:** `Contact added successfully`

### 2. List All Contacts

```bash
rolodex list
```

**Output:**

```
1. Alice Johnson     +234123456789   alice@example.com
2. Dr Sam           08111111111     info@samclinic.ng
3. James Patterson  +2348881454872  james@example.com
```

### 3. Search for a Contact

```bash
# Search by name
rolodex search --by N --name "Alice"

# Search by email domain
rolodex search --by D --domain "gmail.com"
```

### 4. Edit a Contact

```bash
rolodex edit --name "Alice Johnson" --phone "+234123456789" --new-email "alice.new@example.com"
```

### 5. Delete a Contact

```bash
rolodex delete --name "Alice Johnson" --phone "+234123456789"
```

---

## Commands Reference

### `add` - Add a New Contact

```bash
rolodex add --name <NAME> --phone <PHONE> --email <EMAIL> [--tag <TAG>]
```

**Arguments:**

- `--name <NAME>` (required)
  - Contact's full name
  - Format: Letters, spaces, hyphens, apostrophes (50 chars max)
  - Example: `"Alice Johnson"`, `"Dr. Sam O'Brien"`

- `--phone <PHONE>` (required)
  - Phone number in international format
  - Format: Optional `+` prefix, 10-15 digits
  - Examples: `"+234123456789"`, `"08861473537"`
  - Duplicate detection works across format variations (with/without country code)

- `--email <EMAIL>` (optional)
  - Contact's email address
  - Format: Standard email format (max 254 chars)
  - Example: `"alice@example.com"`

- `--tag <TAG>` (optional)
  - Category or label for contact
  - Example: `"work"`, `"family"`, `"client"`

**Examples:**

```bash
# Minimal (phone only)
rolodex add --name "Alice" --phone "+234123456789"

# Complete contact
rolodex add --name "Alice Johnson" --phone "+234123456789" --email "alice@example.com" --tag "work"

# With special characters
rolodex add --name "Dr. Mary O'Brien" --phone "+1-555-123-4567" --email "mary@hospital.org"
```

**Error Handling:**

- Invalid name format → "Name validation failed"
- Invalid phone format → "Phone validation failed"
- Invalid email format → "Email validation failed"
- Duplicate contact detected → "Duplicate contact found"

---

### `list` - Display All Contacts

```bash
rolodex list [--sort <FIELD>] [--tag <CATEGORY>]
```

**Options:**

- `--sort <FIELD>` (optional)
  - Sort order for results
  - Values: `name`, `email`, `created`, `updated`
  - Default: Insertion order (no sort)

- `--tag <CATEGORY>` (optional)
  - Filter by tag
  - Shows only contacts with matching tag
  - Example: `--tag work`

**Examples:**

```bash
# All contacts (insertion order)
rolodex list

# Sorted by name
rolodex list --sort name

# Sorted by email
rolodex list --sort email

# Work contacts only
rolodex list --tag work

# Work contacts sorted by name
rolodex list --sort name --tag work
```

**Output Format:**

```
ID. Name              Phone            Email
1.  Alice Johnson     +234123456789    alice@example.com
2.  Dr Sam           08111111111      info@samclinic.ng
```

---

### `search` - Find Contacts

```bash
rolodex search [--by <KEY>] [--name <NAME>] [--domain <DOMAIN>]
```

**Options:**

- `--by <KEY>` (optional)
  - Search type: `N` for name, `D` for domain
  - Default: Name search

- `--name <NAME>` (when using `--by N`)
  - Performs fuzzy matching with Levenshtein distance
  - Returns partial matches (example: "John" matches "John Doe")
  - Scored by match quality

- `--domain <DOMAIN>` (when using `--by D`)
  - Searches email domains
  - Example: `--domain "gmail.com"` finds all gmail addresses

**Examples:**

```bash
# Search by name (fuzzy)
rolodex search --by N --name "Alice"

# Search with typo tolerance
rolodex search --by N --name "Alcie"  # Still finds "Alice"

# Search by email domain
rolodex search --by D --domain "example.com"

# Default is name search
rolodex search --name "John"
```

**Output:**

```
Found 2 matching contacts:
1. Alice Johnson     +234123456789    alice@example.com
2. Alice Smith       +2348901234567   alice.smith@example.com
```

---

### `edit` - Update a Contact

```bash
rolodex edit --name <NAME> --phone <PHONE> [--new-name <NEW_NAME>] [--new-phone <NEW_PHONE>] [--new-email <NEW_EMAIL>] [--new-tag <NEW_TAG>]
```

**Arguments:**

- `--name <NAME>` (required)
  - Current contact name (identifier)

- `--phone <PHONE>` (required)
  - Current phone (with name, ensures unique identification)

- `--new-name <NEW_NAME>` (optional)
  - New name (if changing)

- `--new-phone <NEW_PHONE>` (optional)
  - New phone (if changing)

- `--new-email <NEW_EMAIL>` (optional)
  - New email (if changing)

- `--new-tag <NEW_TAG>` (optional)
  - New tag (if changing)

**Examples:**

```bash
# Change email only
rolodex edit --name "Alice Johnson" --phone "+234123456789" --new-email "alice.new@example.com"

# Change name and phone
rolodex edit --name "Alice Johnson" --phone "+234123456789" --new-name "Alice Smith" --new-phone "+234198765432"

# Change all fields
rolodex edit --name "Alice Johnson" --phone "+234123456789" --new-name "Dr. Alice Smith" --new-phone "+234198765432" --new-email "dr.alice@hospital.org" --new-tag "medical"
```

**Validation:** Same rules as `add` command apply to new values.

---

### `delete` - Remove a Contact

```bash
rolodex delete --name <NAME> [--phone <PHONE>]
```

**Arguments:**

- `--name <NAME>` (required)
  - Contact name to delete

- `--phone <PHONE>` (optional, recommended)
  - If multiple contacts share the same name, disambiguate with phone
  - Prevents accidental deletion

**Examples:**

```bash
# Delete with name only (if unique)
rolodex delete --name "Alice Johnson"

# Delete with name + phone (safe, recommended)
rolodex delete --name "Alice Johnson" --phone "+234123456789"
```

**Behavior:**

- Soft delete: Contact marked as deleted but recoverable for 1 day
- Permanently purged after 1 day
- Use with care!

**Output:**

```
Contact deleted successfully
```

---

### `import` - Import Contacts from CSV

```bash
rolodex import [--src <FILE>]
```

**Options:**

- `--src <FILE>` (optional)
  - Path to CSV file with contacts
  - Default: `./import_export/contacts.csv`
  - Format: `name,phone,email,tag` (header required)

**CSV Format:**

```csv
name,phone,email,tag
Alice Johnson,+234123456789,alice@example.com,work
Dr Sam,08111111111,info@samclinic.ng,medical
James Patterson,+2348881454872,james@example.com,friend
```

**Examples:**

```bash
# From default location
rolodex import

# From custom file
rolodex import --src "~/my_contacts.csv"

# Import and handle conflicts (Last-Write-Wins)
rolodex import --src "backup.csv"  # Merges with existing contacts
```

**Conflict Resolution:** Uses Last-Write-Wins (LWW) for field-level merging:

- If imported contact has newer timestamp, it overwrites local
- If local contact has newer timestamp, local version is kept
- Soft-deleted contacts are recovered if non-deleted version is imported

**Output:**

```
Contacts imported successfully
Total imported: 42 contacts
Auto-migrated: None
```

---

### `export` - Export Contacts to CSV

```bash
rolodex export [--des <FILE>]
```

**Options:**

- `--des <FILE>` (optional)
  - Path to destination CSV file
  - Default: `./import_export/exported.csv`

**Examples:**

```bash
# Export to default location
rolodex export

# Export to custom file
rolodex export --des "~/backup.csv"

# Full workflow: export for backup
rolodex export --des "backup_$(date +%Y-%m-%d).csv"
```

**Output Format:**

```csv
id,name,phone,email,tag,created_at,updated_at,deleted
550e8400-e29b-41d4-a716-446655440000,Alice Johnson,+234123456789,alice@example.com,work,2024-01-15T10:30:00Z,2024-01-15T14:22:00Z,false
```

**Output:**

```
Contacts exported successfully
Exported: 42 contacts
```

---

### `help` - Get Command Help

```bash
rolodex --help                   # General help
rolodex <COMMAND> --help         # Command-specific help
```

**Examples:**

```bash
rolodex --help                   # Show all commands
rolodex add --help              # Show add options
rolodex list --help             # Show list options
rolodex search --help           # Show search options
```

---

## Data Storage

### Default Behavior

Contacts are stored in a **JSON file** in your current working directory:

- **File:** `contacts.json` (in current directory)
- **Format:** Structured JSON with UUIDs and timestamps
- **Persistence:** Automatically saved after each command

### First Run

When you run any command for the first time:

1. The tool looks for `contacts.json` in the current directory
2. If not found, it creates an empty store
3. All subsequent operations persist to this file

### Changing Storage Type

Use environment variables to switch backends:

```bash
# JSON (default)
export STORAGE_TYPE=json
export STORAGE_PATH=contacts.json

# CSV
export STORAGE_TYPE=csv
export STORAGE_PATH=contacts.csv

# TXT
export STORAGE_TYPE=txt
export STORAGE_PATH=contacts.txt

# Remote (HTTP)
export STORAGE_TYPE=remote
export API_URL=https://api.jsonstorage.net/v1/json/YOUR_USER_ID
export API_KEY=your_jsonstorage_api_key
```

Then run commands normally:

```bash
rolodex list  # Uses configured storage
```

### Backup & Migration

```bash
# Backup to CSV
rolodex export --des "backup_$(date +%Y-%m-%d).csv"

# Migrate to a different format
export STORAGE_TYPE=csv
rolodex import --src "old_contacts.csv"

# Check what's in the store
rolodex list
```

---

## Configuration

### Environment Variables

Create a `.env` file in your working directory or set environment variables:

```env
# Storage configuration
STORAGE_TYPE=json                                    # json, csv, txt, remote
STORAGE_PATH=contacts.json                          # Only for json/csv/txt
API_URL=https://api.jsonstorage.net/v1/json/...     # For remote storage
API_KEY=your_jsonstorage_api_key                    # For remote storage

# Behavior
SOFT_DELETE_PURGE_DAYS=1                           # Purge deleted contacts after N days
FuzzyMatchThreshold=0.4                            # Levenshtein similarity (0-1)
```

### Example `.env`

```env
# Store locally in JSON
STORAGE_TYPE=json
STORAGE_PATH=~/.local/share/rolodex/contacts.json

# Or use CSV for import/export workflows
STORAGE_TYPE=csv
STORAGE_PATH=./contacts.csv

# Or sync with cloud
STORAGE_TYPE=remote
API_URL=https://api.jsonstorage.net/v1/json/abc123def456
API_KEY=your_jsonstorage_key
```

Load the `.env` automatically by placing it in your working directory:

```bash
# .env file in current directory
cd ~/my_rolodex
echo "STORAGE_TYPE=json" > .env
rolodex list  # Reads .env automatically
```

---

## Error Messages

| Error                     | Meaning                     | Solution                                      |
| ------------------------- | --------------------------- | --------------------------------------------- |
| `Contact not found`       | Name/phone doesn't exist    | Check spelling with `list` or `search`        |
| `Duplicate contact found` | Name + phone already exists | Use `edit` instead of `add`                   |
| `Name validation failed`  | Invalid name format         | Names must be alphabetic; avoid numbers       |
| `Phone validation failed` | Invalid phone format        | Use `+` prefix for country code; 10-15 digits |
| `Email validation failed` | Invalid email format        | Standard email format required                |
| `Failed to load storage`  | Storage file not readable   | Check file permissions and path               |
| `Failed to save storage`  | Can't write to storage      | Check disk space and permissions              |
| `File not found`          | Import file doesn't exist   | Check path in `--src` option                  |

---

## Examples & Workflows

### Workflow 1: Building Your First Contact List

```bash
# Create contacts
rolodex add --name "Alice Johnson" --phone "+234123456789" --email "alice@example.com" --tag "work"
rolodex add --name "Bob Smith" --phone "+234198765432" --email "bob@example.com" --tag "work"
rolodex add --name "Carol Davis" --phone "+234102938475" --email "carol@example.com" --tag "personal"

# Verify
rolodex list

# Organize by tag
rolodex list --sort name --tag work
rolodex list --sort name --tag personal
```

### Workflow 2: Search & Update

```bash
# Find a contact
rolodex search --by N --name "Alice"

# Update it
rolodex edit --name "Alice Johnson" --phone "+234123456789" --new-email "alice.new@example.com"

# Verify
rolodex search --by N --name "Alice"
```

### Workflow 3: Backup & Restore

```bash
# Export to CSV for backup
rolodex export --des "backup_2024-01-15.csv"

# Move to another machine
scp "backup_2024-01-15.csv" user@remote-machine:~/

# Import on new machine
cd ~/
rolodex import --src "backup_2024-01-15.csv"

# Verify all contacts restored
rolodex list
```

### Workflow 4: Sync with Remote Storage

```bash
# Set up remote storage (get API key from jsonstorage.net)
export API_URL=https://api.jsonstorage.net/v1/json/YOUR_USER_ID
export API_KEY=your_api_key
export STORAGE_TYPE=remote

# All operations now sync to cloud
rolodex add --name "Dave" --phone "+234105555555"
rolodex list

# On another machine, access same data
export API_URL=https://api.jsonstorage.net/v1/json/YOUR_USER_ID
rolodex list  # See Dave + all other contacts
```

---

## Troubleshooting

### Contacts Lost After Import?

The import uses **Last-Write-Wins** conflict resolution. If a contact in the import file has an older timestamp:

```bash
# Export everything first (as backup)
rolodex export --des "backup_before_import.csv"

# Then import carefully
rolodex import --src "suspicious_file.csv"

# Check what was actually imported
rolodex list
```

### Storage File Corrupted?

```bash
# Export to CSV (safest format)
rolodex export --des "exported_backup.csv"

# Change to CSV storage type
export STORAGE_TYPE=csv
export STORAGE_PATH=contacts.csv

# Re-import from backup
rolodex import --src "exported_backup.csv"
```

### Permission Denied on Storage File?

```bash
# Check permissions
ls -la contacts.json

# Fix (make writable for current user)
chmod 644 contacts.json

# Or move elsewhere
mv contacts.json ~/.local/share/rolodex/contacts.json
export STORAGE_PATH=~/.local/share/rolodex/contacts.json
```

### Help with a Specific Command?

```bash
# Get built-in help
rolodex <COMMAND> --help

# Examples:
rolodex add --help
rolodex search --help
rolodex import --help
```

---

## Further Reading

- **Library Documentation:** See [../libs/README.md][libs-readme] for integration guide
- **Workspace Architecture:** See [../README.md][readme-root]
- **Performance Notes:** See [../libs/docs/perf-notes.md][perf-notes]
- **Change History:** See [../CHANGELOG.md][changelog]

<!-- Link Aliases - Update paths here if documentation structure changes -->

[readme-root]: ../README.md
[libs-readme]: ../libs/README.md
[api-readme]: ../api/README.md
[changelog]: ../CHANGELOG.md
[perf-notes]: ../libs/docs/perf-notes.md
[walkthrough]: ../libs/docs/WALKTHROUGH.md
