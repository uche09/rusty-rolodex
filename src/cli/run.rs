use crate::{
    domain::contact,
    prelude::{
        AppError, Storage,
        command::{Cli, Commands, SortKey},
        contact::{Contact, ValidationReq, phone_number_matches},
        file::load_migrated_contact,
    },
};

use clap::Parser;
use std::{env, process::exit};

pub fn run_app() -> Result<(), AppError> {
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
            let new_contact = Contact::new(
                name,
                phone,
                email.unwrap_or_default(),
                tag.unwrap_or_default(),
            );

            if !new_contact.validate_name()? {
                return Err(AppError::Validation(ValidationReq::name_req()));
            }

            if !new_contact.validate_number()? {
                return Err(AppError::Validation(ValidationReq::phone_req()));
            }

            if !new_contact.validate_email()? {
                return Err(AppError::Validation(ValidationReq::email_req()));
            }

            if new_contact.already_exist(&storage.contact_list()[0..]) {
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
        Commands::List { sort, tag, reverse } => {
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
                    SortKey::Created => storage
                        .mem_store
                        .data
                        .sort_by(|a, b| a.created_at.cmp(&b.created_at)),
                    SortKey::Updated => storage
                        .mem_store
                        .data
                        .sort_by(|a, b| a.updated_at.cmp(&b.updated_at)),
                }
            }

            if reverse {
                storage.mem_store.data.reverse();
            }

            if let Some(tag) = tag {
                let filtered_contacts: Vec<&Contact> = storage
                    .mem_store
                    .iter()
                    .filter(|c| c.tag.to_lowercase() == tag.to_lowercase())
                    .collect();

                if filtered_contacts.is_empty() {
                    println!("Found no contact with this {{{}}} tag", tag);
                    return Ok(());
                }

                for (mut i, c) in filtered_contacts.iter().enumerate() {
                    i += 1;
                    println!(
                        "{i:>3}. {:<20} {:15} {:^30} {:<15}",
                        c.name, c.phone, c.email, c.tag
                    );
                }
                return Ok(());
            }

            for (mut i, c) in storage.contact_list().iter().enumerate() {
                i += 1;
                println!(
                    "{i:>3}. {:<20} {:15} {:^30} {:<15}",
                    c.name, c.phone, c.email, c.tag
                );
            }

            Ok(())
        }

        // Edit Contact
        Commands::Edit {
            name,
            phone,
            new_name,
            new_phone,
            new_email,
            new_tag,
        } => {
            let desired_contact = Contact::new(name, phone, "".to_string(), "".to_string());
            let found_contact = storage
                .mem_store
                .data
                .iter_mut()
                .find(|c| **c == desired_contact);

            if let Some(contact) = found_contact {
                if let Some(name) = new_name {
                    contact.name = name;
                }
                if let Some(phone) = new_phone {
                    contact.phone = phone;
                }
                if let Some(email) = new_email {
                    contact.email = email;
                }
                if let Some(tag) = new_tag {
                    contact.tag = tag;
                }

                contact.updated_at = Some(contact::Utc::now());
            }

            storage.save()?;
            println!("Contact updated successfully");
            Ok(())
        }

        // Delete Contact
        Commands::Delete { name, phone } => {
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
                                let contact = storage.contact_list()[index];
                                if contact.name == name
                                    && phone_number_matches(&contact.phone, &phone)
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
