use crate::domain::{Command, Contact};
use std::{
    io::{self, Write},
    num::ParseIntError,
};

// OUTPUT FUNCTIONS
pub fn parse_command_from_menu() -> Result<Command, String> {
    println!("\n");
    println!("1. Add Contact");
    println!("2. List Contacts");
    println!("3. Delete Contact");
    println!("4. Exit");
    print!("> ");
    io::stdout().flush().unwrap();

    let action = get_input();

    match action.as_str() {
        "1" => Ok(Command::AddContact),
        "2" => Ok(Command::ListContacts),
        "3" => Ok(Command::DeleteContact),
        "4" => Ok(Command::Exit),
        _ => Err("Invalid command.".to_string()),
    }
}

pub fn confirm_action(action: &str) {
    println!("\nAre you sure you want to {}\n? (y/n)", action);
    print!("> ");
    io::stdout().flush().unwrap();
}

pub fn display_contact(contact: &Contact) -> String {
    let output = format!(
        "Name: {}\n\
        Number: {}\n\
        Email: {}",
        contact.name, contact.phone, contact.email
    );
    output
}

// INPUT FUNCTIONS
pub fn get_input() -> String {
    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .expect("Failed to read line");
    input.trim().to_string()
}

pub fn get_input_to_lower() -> String {
    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .expect("Failed to read line");
    input.trim().to_string().to_lowercase()
}

pub fn get_input_as_int() -> Result<i32, ParseIntError> {
    let mut value: String = String::new();
    io::stdin().read_line(&mut value).expect("Input failed");

    value.trim().parse::<i32>()
}
