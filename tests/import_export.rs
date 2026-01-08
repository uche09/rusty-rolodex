use assert_cmd::Command;
use predicates::str::contains;
use tempfile::tempdir;
use std::{fs, path::Path};


fn listing_format(i: i32, name: &str, phone: &str, email: &str, tag: &str) -> String {
    format!("{i:>3}. {name:<20} {phone:15} {email:^30} {tag:<15}")
}

#[test]
fn export_import() -> Result<(), Box<dyn std::error::Error>> {
    // Use json storage for the test run to avoid touching txt files
    let storage_env = ("STORAGE_CHOICE", "json");

    // Add a contact
    Command::cargo_bin(env!("CARGO_PKG_NAME"))?
        .env(storage_env.0, storage_env.1) // Set storage choice env
        .arg("add")
        .arg("--name")
        .arg("Alice")
        .arg("--phone")
        .arg("08031234567")
        .arg("--email")
        .arg("alice@example.com")
        .assert()
        .success()
        .stdout(contains("Contact added successfully"));

    // Export to a temporary CSV file
    let dir = tempdir()?;
    let out_path = dir.path().join("out.csv");
    let out_path_str = out_path.to_string_lossy().to_string();

    Command::cargo_bin(env!("CARGO_PKG_NAME"))?
        .env(storage_env.0, storage_env.1)
        .arg("export")
        .arg("--des")
        .arg(&out_path_str)
        .assert()
        .success()
        .stdout(contains("Successfully exported"));

    // Ensure the exported file exists and has content
    assert!(Path::new(&out_path_str).exists());
    let exported = fs::read_to_string(&out_path_str)?;
    assert!(exported.contains("Alice"));

    // Delete the contact by name 
    Command::cargo_bin(env!("CARGO_PKG_NAME"))?
        .env(storage_env.0, storage_env.1)
        .arg("delete")
        .arg("--name")
        .arg("Alice")
        .assert()
        .success()
        .stdout(contains("Contact deleted successfully"));

    // Import from the exported CSV (importing back should succeed)
    Command::cargo_bin(env!("CARGO_PKG_NAME"))?
        .env(storage_env.0, storage_env.1)
        .arg("import")
        .arg("--src")
        .arg(&out_path_str)
        .assert()
        .success()
        .stdout(contains("Successfully imported"));


    Command::cargo_bin(env!("CARGO_PKG_NAME"))?
        .env(storage_env.0, storage_env.1)
        .arg("list")
        .assert()
        .success()
        .stdout(contains(listing_format(
            1,
            "Alice",
            "08031234567",
            "alice@example.com",
            "",
        )));

    // Cleanup: remove storage file created in .instance (json)
    let json_path = Path::new("./.instance/contacts.json");
    if json_path.exists() {
        let _ = fs::remove_file(json_path);
    }

    Ok(())
}