use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;

#[test]
fn listing_contacts() {
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
            "Alice",
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

    let tagged_list_output = Command::cargo_bin("rusty-rolodex")
        .unwrap()
        .args(&["list", "--tag", "FRIENDS"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let sorted_list_output = Command::cargo_bin("rusty-rolodex")
        .unwrap()
        .args(&["list", "--sort", "name"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let normal_list_str = String::from_utf8_lossy(&normal_list_output);
    let tagged_list_str = String::from_utf8_lossy(&tagged_list_output);
    let sorted_list_str = String::from_utf8_lossy(&sorted_list_output);

    let normal_list: Vec<_> = normal_list_str.lines().collect();
    let tagged_list: Vec<_> = tagged_list_str.lines().collect();
    let sorted_list: Vec<_> = sorted_list_str.lines().collect();

    // First outputed line "Current storage choice is: json"
    // Contact listing starts from second line
    assert!(normal_list.len() == 7);
    assert!(tagged_list.len() == 3);

    assert!(tagged_list[1].contains("friend") && tagged_list[2].contains("friend"));

    assert!(normal_list.len() == sorted_list.len());
    assert!(sorted_list[1].contains("Alice") && sorted_list[2].contains("Diane"));

    let _ = fs::remove_file("./.instance/contacts.json");
}
