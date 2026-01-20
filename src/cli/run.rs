use crate::{
    domain::contact,
    prelude::{
        AppError,
        command::{Cli, Commands, SearchKey, SortKey},
        contact::{Contact, ValidationReq},
        store::{
            self, ContactStore,
            filestore::{Index, IndexUpdateType, Store},
            storage_port::{export_contacts_to_csv, import_contacts_from_csv},
        },
    },
};
use clap::Parser;
use std::{env, process::exit};

pub fn run_app() -> Result<(), AppError> {
    let cli = Cli::parse();

    unsafe {
        env::set_var("STORAGE_CHOICE", &cli.storage_choice);
    }

    let mut storage = Store::new()?;

    storage.mem = storage.load()?;
    storage.index = Index::new(&storage)?;

    println!(
        "Current storage choice is: {}",
        store::parse_storage_choice().is_which()
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

            storage.save(&storage.mem)?;

            println!("Contact added successfully");
            Ok(())
        }

        // Listing contacts
        Commands::List { sort, tag, reverse } => {
            let mut contact_list: Vec<&Contact>;

            if let Some(tag) = tag {
                contact_list = storage
                    .mem
                    .iter()
                    .filter_map(|(_, cont)| {
                        if cont.tag.to_lowercase() == tag.to_lowercase() {
                            Some(cont)
                        } else {
                            None
                        }
                    })
                    // .filter(|&c| c.tag.to_lowercase() == tag.to_lowercase())
                    // .map(|c| *c)
                    .collect();
            } else {
                contact_list = storage.contact_list();
            }

            if contact_list.is_empty() {
                println!("No contact yet");
                exit(0);
            }
            if let Some(key) = sort {
                match key {
                    SortKey::Name => contact_list
                        .sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase())),
                    SortKey::Email => contact_list
                        .sort_by(|a, b| a.email.to_lowercase().cmp(&b.email.to_lowercase())),
                    SortKey::Created => {
                        contact_list.sort_by(|a, b| a.created_at.cmp(&b.created_at))
                    }
                    SortKey::Updated => {
                        contact_list.sort_by(|a, b| a.updated_at.cmp(&b.updated_at))
                    }
                }
            }

            if reverse {
                contact_list.reverse();
            }

            for (mut i, c) in contact_list.iter().enumerate() {
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
            let ids = storage
                .get_ids_by_name(&desired_contact.name)
                .unwrap_or_default();

            let matching_id = ids
                .iter()
                .find(|c| storage.mem.get(c) == Some(&desired_contact));

            let found_contact = matching_id.and_then(|id| storage.mem.get_mut(id));

            if let Some(contact) = found_contact {
                if let Some(name) = new_name {
                    storage
                        .index
                        .updated_name_index(contact, &IndexUpdateType::Remove);

                    contact.name = name;

                    storage
                        .index
                        .updated_name_index(contact, &IndexUpdateType::Add);
                }
                if let Some(phone) = new_phone {
                    contact.phone = phone;
                }
                if let Some(email) = new_email {
                    let new_domain: Vec<&str> = email.split('@').collect();
                    let current_domain: Vec<&str> = contact.email.split("@").collect();

                    if current_domain[current_domain.len() - 1] != new_domain[new_domain.len() - 1]
                    {
                        storage
                            .index
                            .update_domain_index(contact, &IndexUpdateType::Remove);

                        contact.email = email;

                        storage
                            .index
                            .update_domain_index(contact, &IndexUpdateType::Add);
                    } else {
                        contact.email = email;
                    }
                }
                if let Some(tag) = new_tag {
                    contact.tag = tag;
                }

                contact.updated_at = contact::Utc::now();
            } else {
                return Err(AppError::NotFound("Contact".to_string()));
            }

            storage.save(&storage.mem)?;
            println!("Contact updated successfully");
            Ok(())
        }

        // Delete Contact
        Commands::Delete { name, phone } => {
            let ids = storage.get_ids_by_name(&name);

            let phone = phone.unwrap_or_default();
            let desired_contact =
                Contact::new(name.clone(), phone.clone(), "".to_string(), "".to_string());

            match ids {
                Some(ids) => {
                    if ids.len() > 1 {
                        if phone.is_empty() {
                            println!("Deleting failed");
                            println!(
                                "Found multiple contacts with this name: {}, please provide number. See help",
                                name
                            );
                            exit(0);
                        } else {
                            for id in &ids {
                                if let Some(contact) = storage.mem.get(id)
                                    && contact == &desired_contact
                                {
                                    storage.delete_contact(id)?;
                                    storage.save(&storage.mem)?;
                                    println!("Contact deleted successfully");
                                    exit(0);
                                }
                            }

                            eprintln!("{}", AppError::NotFound("Contact".to_string()));
                            return Ok(());
                        }
                    } else {
                        storage.delete_contact(&ids[0])?;
                    }

                    storage.save(&storage.mem)?;
                    println!("Contact deleted successfully");
                    Ok(())
                }
                None => {
                    eprintln!("{}", AppError::NotFound("Contact".to_string()));
                    Ok(())
                }
            }
        }

        // Search for a contact
        Commands::Search { by, name, domain } => {
            // Default search = name (if not provided)
            let search_by = by.unwrap_or(SearchKey::N);

            match search_by {
                // Search using email domain
                SearchKey::D => {
                    // user's provided email strig is assigned to "search_for"
                    let searched_for = domain.unwrap_or_default();

                    let result = storage.fuzzy_search_email_domain_index(&searched_for)?;

                    for (mut i, c) in result.iter().enumerate() {
                        i += 1;

                        let date = c.updated_at.date_naive().to_string();

                        println!(
                            "{i:>3}. {:<20} {:15} {:^30} {:<15} 'Updated on:' {:<12}",
                            c.name, c.phone, c.email, c.tag, date
                        );
                    }
                }
                _ => {
                    // Default to search by name
                    let searched_for = name.unwrap_or_default();

                    let result = storage.fuzzy_search_name(&searched_for)?;

                    for (mut i, &c) in result.iter().enumerate() {
                        i += 1;

                        let date = c.updated_at.date_naive().to_string();

                        println!(
                            "{i:>3}. {:<20} {:15} {:^30} {:<15} 'Updated on:' {:<12}",
                            c.name, c.phone, c.email, c.tag, date
                        );
                    }
                }
            }

            Ok(())
        }

        // Import contacts into storage from .csv file
        Commands::Import { src } => {
            let mut file_path: String = "".to_string();

            if let Some(path) = src {
                file_path = path;
            }

            if file_path.is_empty() {
                let (path, total) = import_contacts_from_csv(None)?;

                println!("Successfully imported {} contacts from {:?}.", total, path);
                return Ok(());
            }

            let (path, total) = import_contacts_from_csv(Some(&file_path))?;

            println!("Successfully imported {} contacts from {:?}.", total, path);
            Ok(())
        }

        Commands::Export { des } => {
            let mut file_path: String = "".to_string();

            if let Some(path) = des {
                file_path = path;
            }

            let contact_list = storage.contact_list();

            if file_path.is_empty() {
                let (path, total) = export_contacts_to_csv(&contact_list, None)?;

                println!("Successfully exported {} contacts to {:?}.", total, path);
                return Ok(());
            }

            let (path, total) = export_contacts_to_csv(&contact_list, Some(&file_path))?;

            println!("Successfully exported {} contacts to {:?}.", total, path);
            Ok(())
        }
    }
}
