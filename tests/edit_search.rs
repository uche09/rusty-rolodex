use assert_cmd::Command;
use predicates::str::contains;
use std::{fs, path::Path};

#[test]
fn edit_search() -> Result<(), Box<dyn std::error::Error>> {
    // Use json storage for the test run to avoid touching txt files
    let storage_env = ("STORAGE_CHOICE", "json");

    // Add a contact
    Command::cargo_bin(env!("CARGO_PKG_NAME"))?
        .env(storage_env.0, storage_env.1) // Set storage choice env
        .arg("add")
        .arg("--name")
        .arg("Alice Dept. Computer Science")
        .arg("--phone")
        .arg("08031234567")
        .arg("--email")
        .arg("alice@example.com")
        .arg("--tag")
        .arg("school")
        .assert()
        .success()
        .stdout(contains("Contact added successfully"));

    // Search by a portion of the name (should find the contact)
    Command::cargo_bin(env!("CARGO_PKG_NAME"))?
        .env(storage_env.0, storage_env.1)
        .arg("search")
        .arg("--name")
        .arg("computer science dpt")
        .assert()
        .success()
        .stdout(contains("Alice Dept. Computer Science"));

    // Edit the contact (change phone)
    Command::cargo_bin(env!("CARGO_PKG_NAME"))?
        .env(storage_env.0, storage_env.1)
        .arg("edit")
        .arg("--name")
        .arg("Alice Dept. Computer Science")
        .arg("--phone")
        .arg("+2348031234567")
        .arg("--new-phone")
        .arg("09123456789")
        .assert()
        .success()
        .stdout(contains("Contact updated successfully"));


    // Delete the contact by name and updated phone
    Command::cargo_bin(env!("CARGO_PKG_NAME"))?
        .env(storage_env.0, storage_env.1)
        .arg("delete")
        .arg("--name")
        .arg("Alice Dept. Computer Science")
        .arg("--phone")
        .arg("+2349123456789")
        .assert()
        .success()
        .stdout(contains("Contact deleted successfully"));

    // Cleanup: remove storage file created in .instance (json)
    let json_path = Path::new("./.instance/contacts.json");
    if json_path.exists() {
        let _ = fs::remove_file(json_path);
    }

    Ok(())
}