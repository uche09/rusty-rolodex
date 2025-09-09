mod cli;
mod domain;
mod errors;
mod helper;
mod store;
mod validation;

use std::io::{self, Write};
use std::process::exit;

use crate::cli::get_input;
use crate::domain::{Command, Contact, Storage};
use crate::errors::AppError;
use crate::store::{ContactStore, load_migrated_contact};
use crate::validation::{contact_exist, validate_email, validate_name, validate_number};

fn main() -> Result<(), AppError> {
    let mut storage = Storage::new()?;

    storage.mem_store.data = load_migrated_contact(&storage)?;

    println!("\n\n--- Contact BOOK ---\n");

    'outerloop: loop {
        match cli::parse_command_from_menu() {
            Ok(command) => {
                // User entered valid command

                match command {
                    Command::AddContact => {
                        'add_contact: loop {
                            // Get contact name
                            let name = cli::retry(
                                "Enter contact name",
                                cli::get_input,
                                Some(validate_name),
                            );

                            if name == '*'.to_string() {
                                continue 'outerloop;
                            }

                            // Get contact number
                            let phone = cli::retry(
                                "Enter contact number",
                                get_input,
                                Some(validate_number),
                            );

                            if phone == '*'.to_string() {
                                continue 'outerloop;
                            }

                            // Get contact email
                            let email = cli::retry(
                                "Enter contact email address",
                                get_input,
                                Some(validate_email),
                            );

                            if email == '*'.to_string() {
                                continue 'outerloop;
                            }

                            // Create a contact
                            let new_contact = Contact { name, phone, email };

                            if contact_exist(&new_contact, storage.contact_list()) {
                                println!("Contact with this name and number already exist");
                                continue 'add_contact;
                            }

                            // Confirm action
                            let message = format!(
                                "add this contact to your contact list \n{}\n",
                                cli::display_contact(&new_contact)
                            );
                            if let Err(e) = cli::confirm_action(&message) {
                                eprintln!("{}", e);
                            }

                            let consent = cli::retry(
                                "",
                                cli::get_input_to_lower,
                                None::<fn(&str) -> Result<bool, AppError>>,
                            );
                            if consent != 'y'.to_string() {
                                continue 'outerloop;
                            }

                            storage.add_contact(new_contact);

                            println!("Contact added successfully!");
                            break 'add_contact;
                        }
                    }
                    Command::ListContacts => {
                        if storage.contact_list().iter().count() < 1 {
                            println!("No contact in contact list! ");
                            continue 'outerloop;
                        }

                        println!("***YOUR CONTACTS***");

                        for contact in storage.contact_list().iter() {
                            println!();
                            println!("{}", cli::display_contact(contact));
                        }
                    }
                    Command::DeleteContact => {
                        'delete_contact: loop {
                            // Search contact by name
                            let name = cli::retry(
                                "Search contact by name to DELETE",
                                cli::get_input,
                                None::<fn(&str) -> Result<bool, AppError>>,
                            );

                            if name == '*'.to_string() {
                                continue 'outerloop;
                            }

                            // Get index for all contact having exactly same name
                            let indices = storage.get_indices_by_name(&name);

                            match indices {
                                Some(indices) => {
                                    let contact_list = storage.contact_list();
                                    let mut match_count = 1;

                                    // Handle contact with identical name match
                                    if indices.len() > 1 {
                                        println!(
                                            "\nWe found multiple match for this name \"{}\"",
                                            &name
                                        );
                                        println!(
                                            "Enter the correspondint number to delete desired contact"
                                        );

                                        for index in &indices {
                                            println!(
                                                "\n{}. {}",
                                                match_count,
                                                cli::display_contact(&contact_list[*index])
                                            );
                                            match_count += 1;
                                        }
                                        print!("> ");
                                        io::stdout().flush()?;

                                        let selected = cli::get_input_as_int();
                                        let selected = selected.unwrap_or(0);

                                        if selected < 1 || selected > indices.len() as i32 {
                                            println!("Invalid choice");
                                            continue 'delete_contact;
                                        }

                                        // confirm action
                                        let message = format!(
                                            "delete this contact from your contact list \n{}\n",
                                            cli::display_contact(
                                                &contact_list[indices[selected as usize - 1_usize]]
                                            )
                                        );

                                        if let Err(e) = cli::confirm_action(&message) {
                                            eprintln!("{}", e);
                                            break 'delete_contact;
                                        }

                                        let consent = cli::retry(
                                            "",
                                            cli::get_input_to_lower,
                                            None::<fn(&str) -> Result<bool, AppError>>,
                                        );
                                        if consent != 'y'.to_string() {
                                            continue 'outerloop;
                                        }

                                        // if contact exist in txt and contacts are now stored in json or vice versa,
                                        // Avoid PARTIAL DELETE
                                        if storage.storage_choice.is_json()
                                            && storage.file_store.load()?.contains(
                                                &storage.mem_store.data[selected as usize - 1],
                                            )
                                        {
                                            let contact =
                                                &storage.mem_store.data[selected as usize - 1];
                                            let mut txt_contacts = storage.file_store.load()?;

                                            if let Some(contact_index) =
                                                txt_contacts.iter().position(|cont| cont == contact)
                                            {
                                                txt_contacts.remove(contact_index);
                                                storage.file_store.save(&txt_contacts)?;
                                            }
                                        } else if storage.storage_choice.is_txt()
                                            && storage.json_store.load()?.contains(
                                                &storage.mem_store.data[selected as usize - 1],
                                            )
                                        {
                                            let contact =
                                                &storage.mem_store.data[selected as usize - 1];
                                            let mut json_contacts = storage.json_store.load()?;

                                            if let Some(contact_index) = json_contacts
                                                .iter()
                                                .position(|cont| cont == contact)
                                            {
                                                json_contacts.remove(contact_index);
                                            }
                                            storage.json_store.save(&json_contacts)?;
                                        }

                                        match storage
                                            .delete_contact(indices[selected as usize - 1_usize])
                                        {
                                            Ok(_) => {
                                                println!("Contact deleted successfully!");
                                                break 'delete_contact;
                                            }
                                            Err(e) => {
                                                eprintln!("{}", e);
                                                break 'delete_contact;
                                            }
                                        }
                                    } else {
                                        // Handle single single contact match

                                        // Confirm action
                                        let message = format!(
                                            "delete this contact from your contact list \n{}\n",
                                            cli::display_contact(&contact_list[indices[0]])
                                        );
                                        if let Err(e) = cli::confirm_action(&message) {
                                            eprintln!("{}", e);
                                        }

                                        let consent = cli::retry(
                                            "",
                                            cli::get_input_to_lower,
                                            None::<fn(&str) -> Result<bool, AppError>>,
                                        );
                                        if consent != 'y'.to_string() {
                                            continue 'outerloop;
                                        }

                                        let _ = storage.delete_contact(indices[0]);

                                        println!("Contact deleted successfully!");
                                        break 'delete_contact;
                                    }
                                }
                                _ => {
                                    println!("Name not found in contact list");
                                    continue 'delete_contact;
                                }
                            }
                        }
                    }
                    Command::Exit => {
                        if storage.storage_choice.is_mem() {
                            println!("\nBye!");
                            exit(0);
                        }

                        if storage.storage_choice.is_txt() {
                            match storage.file_store.save(&storage.mem_store.data) {
                                Ok(_) => {
                                    println!("\nBye!");
                                    exit(0);
                                }
                                Err(e) => {
                                    eprintln!("Unable to store contacts '{}'", e);
                                    println!("\nBye!");
                                    exit(1);
                                }
                            }
                        }

                        if storage.storage_choice.is_json() {
                            match storage.save_json(&storage.mem_store.data) {
                                Ok(_) => {
                                    println!("\nBye!");
                                    exit(0);
                                }
                                Err(e) => {
                                    eprintln!("Unable to store contacts '{}'", e);
                                    println!("\nBye!");
                                    exit(1);
                                }
                            }
                        }
                    }
                }
            }
            Err(message) => {
                // User entered invalid command
                eprintln!("{}", message);
                continue 'outerloop;
            }
        }
    }
}
