mod cli;
mod domain;
mod store;
mod validation;

use std::io::{self, Write};
use std::process::exit;

use crate::domain::{Command, Contact, ContactStore};
use crate::validation::{contact_exist, validate_email, validate_name, validate_number};

fn main() {
    let mut storage = ContactStore::new();

    println!("\n\n--- Contact BOOK ---\n");

    'outerloop: loop {

        match cli::parse_command_from_menu() {
            Ok(command) => {
                // User entered valid command

                match command {
                    Command::AddContact => {
                        'add_contact: loop {
                            // Get contact name
                            println!("\nEnter contact name \n* to go back: ");
                            let name = cli::get_input();

                            if name == '*'.to_string() {
                                continue 'outerloop;
                            }

                            if !validate_name(&name) {
                                println!("\nInvalid Name input.");
                                continue 'add_contact;
                            }

                            // Get contact number
                            println!("\nEnter contact number:");
                            let phone = cli::get_input();

                            if !validate_number(&phone) {
                                println!("\nInvalid Number input.");
                                continue 'add_contact;
                            }

                            // Get contact email
                            println!("\nEnter contact email.");
                            let email = cli::get_input();

                            if !validate_email(&email) {
                                println!("\nInvalid email input.");
                                continue 'add_contact;
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
                            cli::confirm_action(&message);

                            let consent = cli::get_input_to_lower();
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
                            println!("Search contact by name to DELETE or * to go back");
                            let name = cli::get_input();

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
                                        io::stdout().flush().unwrap();

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

                                        cli::confirm_action(&message);

                                        let consent = cli::get_input_to_lower();
                                        if consent != 'y'.to_string() {
                                            continue 'outerloop;
                                        }

                                        let _ = storage
                                            .delete_contact(indices[selected as usize - 1_usize]);
                                        println!("Contact deleted successfully!");
                                        break 'delete_contact;
                                    } else {
                                        // Handle single single contact match

                                        // Confirm action
                                        let message = format!(
                                            "delete this contact from your contact list \n{}\n",
                                            cli::display_contact(&contact_list[indices[0]])
                                        );
                                        cli::confirm_action(&message);

                                        let consent = cli::get_input_to_lower();
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
                        println!("\nBye!");
                        exit(0);
                    }
                }
            }
            Err(message) => {
                // User entered invalid command
                println!("{}", message);
                continue 'outerloop;
            }
        }
    }
}
