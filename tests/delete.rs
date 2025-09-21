use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;

#[test]
fn deleting_contacts() {
    // Attempt to delete non existing contact
    Command::cargo_bin("rusty-rolodex")
        .unwrap()
        .args(&["delete", "--name", "Alice"])
        .assert()
        .success()
        .stderr(predicate::str::contains("Contact Not found"));

    // Add a contacts 1
    Command::cargo_bin("rusty-rolodex")
        .unwrap()
        .args(&[
            "add",
            "--name",
            "Patricia",
            "--phone",
            "08066809241",
            "--email",
            "lmartinez@bender-patterson.net",
            "--tag",
            "others",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("Contact added successfully"));

    // Add a contacts 2
    Command::cargo_bin("rusty-rolodex")
        .unwrap()
        .args(&[
            "add",
            "--name",
            "Diane",
            "--phone",
            "08064879199",
            "--email",
            "grahammatthew@gmail.com",
            "--tag",
            "school",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("Contact added successfully"));

    // Add a contacts 3
    Command::cargo_bin("rusty-rolodex")
        .unwrap()
        .args(&[
            "add",
            "--name",
            "John",
            "--phone",
            "08046516806",
            "--email",
            "wendy59@turner.com",
            "--tag",
            "friends",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("Contact added successfully"));

    // Add a contacts 4
    Command::cargo_bin("rusty-rolodex")
        .unwrap()
        .args(&[
            "add",
            "--name",
            "Wayne",
            "--phone",
            "08062866694",
            "--email",
            "jackie73@lopez.com",
            "--tag",
            "friends",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("Contact added successfully"));

    // Add a contacts 5
    Command::cargo_bin("rusty-rolodex")
        .unwrap()
        .args(&[
            "add",
            "--name",
            "Thomas",
            "--phone",
            "08019271836",
            "--email",
            "kdelacruz@yahoo.com",
            "--tag",
            "school",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("Contact added successfully"));

    // Add a contacts 6
    Command::cargo_bin("rusty-rolodex")
        .unwrap()
        .args(&[
            "add",
            "--name",
            "John",
            "--phone",
            "08031234567",
            "--email",
            "alice@example.com",
            "--tag",
            "work",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("Contact added successfully"));

    // LISTING ADDED CONTACT
    let normal_list_output = Command::cargo_bin("rusty-rolodex")
        .unwrap()
        .args(&["list"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let normal_list_str = String::from_utf8_lossy(&normal_list_output);
    let normal_list: Vec<_> = normal_list_str.lines().collect();

    // First outputed line "Current storage choice is: json"
    // Contact listing starts from second line
    // Total lines = total_contact + 1
    assert!(normal_list.len() == 7);

    // Delete 1 out of 6
    Command::cargo_bin("rusty-rolodex")
        .unwrap()
        .args(&["delete", "--name", "Patricia"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Contact deleted successfully"));

    // Delete 2 out of 6
    Command::cargo_bin("rusty-rolodex")
        .unwrap()
        .args(&["delete", "--name", "Diane"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Contact deleted successfully"));

    // LISTING REMAINING CONTACTS
    let normal_list_output = Command::cargo_bin("rusty-rolodex")
        .unwrap()
        .args(&["list"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let normal_list_str = String::from_utf8_lossy(&normal_list_output);
    let normal_list: Vec<_> = normal_list_str.lines().collect();

    // First line outputed "Current storage choice is: json"
    // Contact listing starts from second line
    // Total lines = total_contact + 1
    assert!(normal_list.len() == 5);

    // Verify that deleted contact no longer exist
    Command::cargo_bin("rusty-rolodex")
        .unwrap()
        .args(&["delete", "--name", "Patricia"])
        .assert()
        .success()
        .stderr(predicate::str::contains("Contact Not found"));

    Command::cargo_bin("rusty-rolodex")
        .unwrap()
        .args(&["delete", "--name", "Diane"])
        .assert()
        .success()
        .stderr(predicate::str::contains("Contact Not found"));

    // ATTEMPT TO DELETE CONTACT WITH IDENTICAL NAME "John" ADDED EARLIER
    Command::cargo_bin("rusty-rolodex")
        .unwrap()
        .args(&["delete", "--name", "John"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Found multiple contacts with this name: John, please provide number. See help",
        ));

    Command::cargo_bin("rusty-rolodex")
        .unwrap()
        .args(&["delete", "--name", "John", "--phone", "+2348031234567"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Contact deleted successfully"));

    // LISTING REMAINING CONTACTS
    let normal_list_output = Command::cargo_bin("rusty-rolodex")
        .unwrap()
        .args(&["list"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let normal_list_str = String::from_utf8_lossy(&normal_list_output);
    let normal_list: Vec<_> = normal_list_str.lines().collect();

    // First line outputed "Current storage choice is: json"
    // Contact listing starts from second line
    // Total lines = total_contact + 1
    assert!(normal_list.len() == 4);

    let _ = fs::remove_file("./.instance/contacts.json");
}
