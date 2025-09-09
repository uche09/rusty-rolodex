use crate::domain::{Command, Contact};
use crate::errors::AppError;
use std::io::{self, Write};

// OUTPUT FUNCTIONS
pub fn parse_command_from_menu() -> Result<Command, AppError> {
    println!("\n");
    println!("1. Add Contact");
    println!("2. List Contacts");
    println!("3. Delete Contact");
    println!("4. Exit");
    print!("> ");
    io::stdout().flush()?;

    let action = match get_input() {
        Ok(input) => input,
        Err(e) => e.to_string(),
    };

    match action.as_str() {
        "1" => Ok(Command::AddContact),
        "2" => Ok(Command::ListContacts),
        "3" => Ok(Command::DeleteContact),
        "4" => Ok(Command::Exit),
        _ => Err(AppError::ParseCommand(action)),
    }
}

pub fn confirm_action(action: &str) -> Result<(), AppError> {
    println!("\nAre you sure you want to {}\n? (y/n)", action);
    print!("> ");
    io::stdout().flush()?;
    Ok(())
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
pub fn get_input() -> Result<String, AppError> {
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    Ok(input.trim().to_string())
}

pub fn get_input_to_lower() -> Result<String, AppError> {
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    Ok(input.trim().to_string().to_lowercase())
}

pub fn get_input_as_int() -> Result<i32, AppError> {
    let mut value: String = String::new();
    io::stdin().read_line(&mut value)?;

    Ok(value.trim().parse::<i32>()?)
}

pub fn retry<F, T, V>(prompt: &str, f: F, valid: Option<V>) -> T
where
    F: Fn() -> Result<T, AppError>,
    V: Fn(&str) -> Result<bool, AppError>,
    T: AsRef<str>,
{
    'input: loop {
        println!("\n{} \n* to go back: ", prompt);
        let input = f();

        // Check if input function returned error else destructure
        let input = match input {
            Ok(data) => data,
            Err(e) => {
                eprintln!("{e}");
                continue;
            }
        };

        if input.as_ref() == "*" {
            break 'input input; // return value from loop
        }

        // validate input
        if let Some(ref validator) = valid {
            if let Ok(t) = validator(input.as_ref()) {
                if !t {
                    eprintln!("{}", AppError::Validation("\nInvalid input.".to_string()));

                    continue;
                }
            }

            break 'input input; // return value from loop
        }
        break 'input input; // return value from loop
    }
}
