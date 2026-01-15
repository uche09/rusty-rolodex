# Rusty Rolodex — Usage Guide

This document describes how to use the `rolodex` CLI after Week 3 implementation.

## Installation
Make sure you have Rust installed on you device. [Install rust from here](https://www.rust-lang.org/tools/install).  

Clone the repo

```bash
git clone https://github.com/uche09/rusty-rolodex.git
```

Build project executable binary, run
```bash
cargo build --release
```

After installation you should have a rolodex binary in your PATH.

## Data persistence

Contacts are stored in a JSON file named contacts.json in the current working directory. When the tool starts, it will load existing contacts; when it exits or makes changes, it will write the updated JSON file.

If an older plain-text store exists (from Week 2), the program will attempt to migrate data at startup.

## CLI commands

With clap-based command structure, the following commands and options are supported:

`rolodex add`

### Add a new contact.

```text
Usage: rolodex add --name <NAME> --phone <PHONE> --email <EMAIL>

```

**Options:**

- --name <NAME> — contact name, must be non-empty, alphabetic (or spaces)

- --phone <PHONE> — phone number, must match regex ^\+?\d{10,15}$ (or whatever pattern you chose)

- --email <EMAIL> — must match regex pattern for a valid email

On success, prints something like:
```text
Contact added successfully
```

On invalid input (phone/email/name formats), prints a friendly error and exits with a non-zero status.


### rolodex list

List all contacts.

```text
Usage: rolodex list [--sort <FIELD>] [--tag <CATEGORY>]
```

**Options:**

--sort <FIELD> — one of `name` or `email`; default is no specific sort (in insertion order)

Output might look like:

```text
1. Alice        +234123456789   ailce@gmail.com
2. Dr Sam       08111111111     info@samclinic.ng
3. james        +2348881454872  ja.mes@gmail.com
4. Jerry        09422138746     jex@gmail.com

```
If no contacts exist, prints a message like:
```text
No contact yet
```


### rolodex delete

Delete a contact by name.
```txt
Usage: rolodex delete --name <NAME> --phone <PHONE NUMBER>
```

**Options:**
- --name <NAME> — Stored name on contact you want to delete
- --phone <PHONE NUMBER> — Optional phone number incase multiple contact have same name.


If name matches multiple contact, phone was not provided, it prints:
```text
Found multiple contacts with this name: Jerry, please provide number. See help
```

If a contact with exactly that name exists, or phone was provided deletes it and prints:
```txt
Contact deleted successfully
```

If no such contact is found, prints:
```text
Contact Not found
```

### rolodex edit
```text
Usage: rolodex edit --name <NAME> --phone <PHONE> [--new_name <NEW_NAME>] [--new_phone <NEW_PHONE>] [--new_email <NEW_EMAIL>] [--new_tag <NEW_TAG>]
```

**Options:**
- --name <NAME> — current contact name
- --phone <PHONE> — current phone number
- --new_name <NEW_NAME> — optional new name
- --new_phone <NEW_PHONE> — optional new phone number
- --new_email <NEW_EMAIL> — optional new email address
- --new_tag <NEW_TAG> — optional new tag


On success, prints something like:
```text
Contact updated successfully
```

If no such contact is found, prints:
```text
Contact Not found
```

### rolodex search
Search for contacts.
```text
Usage: rolodex search [--by <KEY>] [--name <NAME>] [--domain <DOMAIN>]
```

**Options:**
- --by <KEY> — search mode: N for name, D for email domain
- --name <NAME> — name to search for (when --by N)
- --domain <DOMAIN> — email domain to search for (when --by D)

Output lists matching contacts, similar to list command.

If no matches, prints:
```text
No matching contacts found
```


### rolodex import
Import contacts from a CSV file.
```text
Usage: rolodex import [--src <FILE>]
```
**Options:**
- --src <FILE> — path to the source CSV file; if not provided, defaults to `"./import_export/contacts.csv"`.

On success, prints:
```text
Contacts imported successfully
```

On error (file not found, invalid format), prints an error message.


### rolodex export
Export contacts to a CSV file.
```text
Usage: rolodex export [--des <FILE>]
```
**Options:**
--des <FILE> — path to the destination CSV file; if not provided, defaults to `"./import_export/exported.csv"`.

On success, prints something like:
```text
Contacts exported successfully
```

On error, prints an error message.


### rolodex help
Automatic help via clap:
```bash
rolodex --help
rolodex add --help
rolodex list --help
rolodex delete --help
```
Each subcommand shows the available flags, required inputs, and a short description.


## Validation and Error Handling

Input values (--name, --phone, --email) are validated using regex. Invalid values result in a clear error message.

JSON persistence is done via serde_json. On startup, the tool attempts to load contacts.json. If it does not exist, it starts with an empty list of contacts.

Text persistence is done via `helper::serialize_contacts()` and `helper::deserialize_contacts_from_txt_buffer()` function I implemented.

If a migration from a prior plain-text store is needed, the tool will attempt to read the old format and convert it to JSON. If the old file is corrupt/unreadable, the tool warns the user and proceeds with an empty list (or aborts, depending on your chosen behavior).

All I/O errors, parse errors, and validation errors are wrapped in custom error types (Week 2) and handled gracefully at the user-facing boundary: no unwrap() or panics in normal operation.