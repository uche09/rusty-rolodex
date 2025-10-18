// use assert_cmd::Command;
// use predicates::prelude::*;
// use std::fs;

// fn listing_format(i: i32, name: &str, phone: &str, email: &str, tag: &str) -> String {
//     format!("{i:>3}. {name:<20} {phone:15} {email:^30} {tag:<15}")
// }

// #[test]
// // fn add_contact() {
// //     let _ = fs::remove_file("./.instance/contacts.json");
// //     // Add a contact
// //     Command::cargo_bin("rusty-rolodex")
// //         .unwrap()
// //         .args(&[
// //             "add",
// //             "--name",
// //             "Alice",
// //             "--phone",
// //             "08031234567",
// //             "--email",
// //             "alice@example.com",
// //             "--tag",
// //             "work",
// //         ])
// //         .assert()
// //         .success()
// //         .stdout(predicate::str::contains("Contact added successfully"));

// //     // Confirm newly added contact exist
// //     Command::cargo_bin("rusty-rolodex")
// //         .unwrap()
// //         .args(&["list"])
// //         .assert()
// //         .success()
// //         .stdout(predicate::str::contains(listing_format(
// //             1,
// //             "Alice",
// //             "08031234567",
// //             "alice@example.com",
// //             "work",
// //         )));

// //     // Attempt to Add duplicate contacts
// //     Command::cargo_bin("rusty-rolodex")
// //         .unwrap()
// //         .args(&[
// //             "add",
// //             "--name",
// //             "Alice",
// //             "--phone",
// //             "+2348031234567",
// //             "--email",
// //             "alice@example.com",
// //             "--tag",
// //             "work",
// //         ])
// //         .assert()
// //         .failure()
// //         .stderr(predicate::str::contains(
// //             "Error: Validation(\
// //             \"Contact with this name and number already exist\")\n",
// //         ));

// //     // Clear memory
// //     Command::cargo_bin("rusty-rolodex")
// //         .unwrap()
// //         .args(&["delete", "--name", "Alice"])
// //         .assert()
// //         .success()
// //         .stdout(predicate::str::contains("Contact deleted successfully"));
// // }

// #[test]
// fn invalid_inputs() {
//     // INVALID COMMAND
//     Command::cargo_bin("rusty-rolodex")
//         .unwrap()
//         .args(&[
//             "and",
//             "--name",
//             "Alice",
//             "--phone",
//             "08031234567",
//             "--email",
//             "alice@example.com",
//             "--tag",
//             "work",
//         ])
//         .assert()
//         .failure()
//         .stderr(predicate::str::contains("unrecognized subcommand 'and'"));

//     // INVALID NAME
//     Command::cargo_bin("rusty-rolodex")
//         .unwrap()
//         .args(&[
//             "add",
//             "--name",
//             "123",
//             "--phone",
//             "08031234567",
//             "--email",
//             "alice@example.com",
//             "--tag",
//             "work",
//         ])
//         .assert()
//         .failure()
//         .stderr(predicate::str::contains(
//             "Error: Validation(\"Name must begin with alphabet, \
//                         may contain spaces, dot, hyphen, and apostrophe between \
//                         alphabets and may end with number or alphabet. \
//                         Name must not exceed 50 characters\")\n",
//         ));

//     Command::cargo_bin("rusty-rolodex")
//         .unwrap()
//         .args(&[
//             "add",
//             "--name",
//             "A very very very very very very very very very \
//                         very very very very very very very very very very \
//                         very long name", // 120 CHARS
//             "--phone",
//             "08031234567",
//             "--email",
//             "alice@example.com",
//             "--tag",
//             "work",
//         ])
//         .assert()
//         .failure()
//         .stderr(predicate::str::contains(
//             "Error: Validation(\"Name must begin with alphabet, \
//                         may contain spaces, dot, hyphen, and apostrophe between \
//                         alphabets and may end with number or alphabet. \
//                         Name must not exceed 50 characters\")\n",
//         ));

//     // INVALID PHONE NUMBER
//     Command::cargo_bin("rusty-rolodex")
//         .unwrap()
//         .args(&[
//             "add",
//             "--name",
//             "Alice",
//             "--phone",
//             "+234813abcd",
//             "--email",
//             "alice@example.com",
//             "--tag",
//             "work",
//         ])
//         .assert()
//         .failure()
//         .stderr(predicate::str::contains(
//             "Number must contain 10 to 15 digits, \
//                 may begin with + and all digits",
//         ));

//     // INVALID EMAIL
//     Command::cargo_bin("rusty-rolodex")
//         .unwrap()
//         .args(&[
//             "add",
//             "--name",
//             "Alice",
//             "--phone",
//             "+08031234567",
//             "--email",
//             "foo@bar",
//             "--tag",
//             "coursemate",
//         ])
//         .assert()
//         .failure()
//         .stderr(predicate::str::contains(
//             "Error: Validation(\"Email can be empty, or must be a valid email. \
//                 Must not exceed 254 characters\")\n",
//         ));

//     Command::cargo_bin("rusty-rolodex")
//         .unwrap()
//         .args(&[
//             "add",
//             "--name",
//             "Alice",
//             "--phone",
//             "+08031234567",
//             "--email",
//             "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\
//                         aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\
//                         aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\
//                         aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\
//                         aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\
//                         aaaaaa@example.com", // 240 `a`s + @gmail.com = 300 CHARS
//             "--tag",
//             "coursemate",
//         ])
//         .assert()
//         .failure()
//         .stderr(predicate::str::contains(
//             "Error: Validation(\"Email can be empty, or must be a valid email. \
//                 Must not exceed 254 characters\")\n",
//         ));
// }
