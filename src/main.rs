mod cli;
mod domain;
mod errors;
mod helper;
mod store;

use std::{env, process::exit};

use clap::Parser;

use crate::cli::{command::Cli, command::Commands, command::SortKey};
use crate::domain::{
    contact::{self, Contact, ValidationReq},
    storage::Storage,
};
use crate::errors::AppError;
use store::file::load_migrated_contact;

fn main() -> Result<(), AppError> {
    let cli = Cli::parse();

    unsafe {
        env::set_var("STORAGE_CHOICE", &cli.storage_choice);
    }

    let mut storage = Storage::new()?;

    storage.mem_store.data = load_migrated_contact(&storage)?;
    println!(
        "Current storage choice is: {}",
        storage.storage_choice.is_which()
    );

    match cli.command {
        Commands::Add {
            name,
            phone,
            email,
            tag,
        } => {
            let new_contact = Contact {
                name,
                phone,
                email: email.unwrap_or_default(),
                tag: tag.unwrap_or_default(),
            };

            if !new_contact.validate_name()? {
                return Err(AppError::Validation(ValidationReq::name_req()));
            }

            if !new_contact.validate_number()? {
                return Err(AppError::Validation(ValidationReq::phone_req()));
            }

            if !new_contact.validate_email()? {
                return Err(AppError::Validation(ValidationReq::email_req()));
            }

            if new_contact.already_exist(storage.contact_list()) {
                return Err(AppError::Validation(
                    "Contact with this name and number already exist".to_string(),
                ));
            }

            storage.add_contact(new_contact);

            storage.save()?;
            println!("Contact added successfully");
            Ok(())
        }

        // Listing contacts
        Commands::List { sort, tag } => {
            let _tag = tag; // TODO

            if storage.contact_list().is_empty() {
                println!("No contact yet");
                exit(0);
            }
            if let Some(key) = sort {
                match key {
                    SortKey::Name => storage
                        .mem_store
                        .data
                        .sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase())),
                    SortKey::Email => storage
                        .mem_store
                        .data
                        .sort_by(|a, b| a.email.to_lowercase().cmp(&b.email.to_lowercase())),
                }
            }

            for (mut i, c) in storage.contact_list().iter().enumerate() {
                i += 1;
                println!("{i:>3}. {:<20} {:<15} {}", c.name, c.phone, c.email);
            }

            Ok(())
        }

        // Delete Contact
        Commands::Delete { name, phone } => {
            let contacts = storage.contact_list().clone();
            let indices = storage.get_indices_by_name(&name);

            let phone = phone.unwrap_or_default();

            match indices {
                Some(indices) => {
                    if indices.len() > 1 {
                        if phone.is_empty() {
                            println!("Deleting failed");
                            println!(
                                "Found multiple contacts with this name: {}, please provide number. See help",
                                name
                            );
                            exit(0);
                        } else {
                            for index in indices {
                                if contacts[index].name == name
                                    && contact::phone_number_matches(&contacts[index].phone, &phone)
                                {
                                    storage.delete_contact(index)?;
                                }
                            }
                        }
                    } else {
                        storage.delete_contact(indices[0])?;
                    }

                    storage.save()?;
                    println!("Contact deleted successfully");
                    Ok(())
                }
                None => {
                    eprintln!("{}", AppError::NotFound("Contact".to_string()));
                    Ok(())
                }
            }
        }
    }
}
