use crate::{
    domain::{contact, manager::Index},
    prelude::{
        AppError, ContactStore,
        command::{Cli, Commands, SearchKey, SortKey},
        contact::{Contact, EMAIL_REQ_MESSAGE, NAME_REQ_MESSAGE, PHONE_REQ_MESSAGE},
        manager::{ContactManager, IndexUpdateType, LastWriteWinsPolicy, SyncPolicy},
        stores::{JsonStorage, TxtStorage},
    },
    storage::StorageMediums,
};
use clap::Parser;
use std::{env, path::PathBuf, process::exit};

pub fn run_app() -> Result<(), AppError> {
    let cli = Cli::parse();

    unsafe {
        env::set_var("STORAGE_CHOICE", &cli.storage_choice);
    }

    let mut manager = ContactManager::new()?;

    println!(
        "Current storage choice is: {}",
        manager.storage.get_medium()
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
                return Err(AppError::Validation(NAME_REQ_MESSAGE.to_string()));
            }

            if !new_contact.validate_number()? {
                return Err(AppError::Validation(PHONE_REQ_MESSAGE.to_string()));
            }

            if !new_contact.validate_email()? {
                return Err(AppError::Validation(EMAIL_REQ_MESSAGE.to_string()));
            }

            if new_contact.already_exist(&manager.contact_list()[0..]) {
                return Err(AppError::Validation(
                    "Contact with this name and number already exist".to_string(),
                ));
            }

            manager.add_contact(new_contact);

            manager.save()?;

            println!("Contact added successfully");
            Ok(())
        }

        // Listing contacts
        Commands::List { sort, tag, reverse } => {
            let mut contact_list: Vec<&Contact>;

            if let Some(tag) = tag {
                contact_list = manager
                    .mem
                    .iter()
                    .filter_map(|(_, cont)| {
                        if cont.tag.to_lowercase() == tag.to_lowercase() && !cont.deleted {
                            Some(cont)
                        } else {
                            None
                        }
                    })
                    .collect();
            } else {
                contact_list = manager.contact_list();
            }

            if contact_list.is_empty() {
                println!("No contact yet");
                exit(0);
            }
            if let Some(key) = sort {
                match key {
                    SortKey::Name => {
                        contact_list.sort_by(|a, b| parse_list_order(reverse, &a.name, &b.name))
                    }
                    SortKey::Email => {
                        contact_list.sort_by(|a, b| parse_list_order(reverse, &a.email, &b.email))
                    }
                    SortKey::Created => contact_list
                        .sort_by(|a, b| parse_list_order(reverse, &a.created_at, &b.created_at)),
                    SortKey::Updated => contact_list
                        .sort_by(|a, b| parse_list_order(reverse, &a.updated_at, &b.updated_at)),
                }
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
            let ids = manager
                .get_ids_by_name(&desired_contact.name)
                .unwrap_or_default();

            let matching_id = ids
                .iter()
                .find(|c| manager.mem.get(c) == Some(&desired_contact));

            let found_contact = matching_id.and_then(|id| manager.mem.get_mut(id));

            if let Some(contact) = found_contact {
                if let Some(name) = new_name {
                    manager
                        .index
                        .updated_name_index(contact, &IndexUpdateType::Remove);

                    contact.name = name;

                    manager
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
                        manager
                            .index
                            .update_domain_index(contact, &IndexUpdateType::Remove);

                        contact.email = email;

                        manager
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

            manager.save()?;
            println!("Contact updated successfully");
            Ok(())
        }

        // Delete Contact
        Commands::Delete { name, phone } => {
            let ids = manager.get_ids_by_name(&name);

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
                                if let Some(contact) = manager.mem.get(id)
                                    && contact == &desired_contact
                                {
                                    manager.delete_contact(id)?;
                                    manager.save()?;
                                    println!("Contact deleted successfully");
                                    exit(0);
                                }
                            }

                            eprintln!("{}", AppError::NotFound("Contact".to_string()));
                            return Ok(());
                        }
                    } else {
                        manager.delete_contact(&ids[0])?;
                    }

                    manager.save()?;
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

                    let result = manager.fuzzy_search_email_domain_index(&searched_for)?;

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

                    let result = manager.fuzzy_search_name(&searched_for)?;

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
                manager.import_contacts_from_csv(None)?;

                println!("Successfully imported contacts.");
                return Ok(());
            }

            manager.import_contacts_from_csv(Some(&file_path))?;
            manager.save()?;

            println!("Successfully imported contacts from {:?}.", file_path);
            Ok(())
        }

        Commands::Export { des } => {
            let mut file_path: String = "".to_string();

            if let Some(path) = des {
                file_path = path;
            }

            if file_path.is_empty() {
                manager.export_contacts_to_csv(None)?;

                println!("Successfully exported contacts.");
                return Ok(());
            }

            manager.export_contacts_to_csv(Some(&file_path))?;

            println!("Successfully exported contacts to {:?}.", file_path);
            Ok(())
        }

        Commands::Sync { src } => {
            let path = PathBuf::from(&src);
            if !path.exists() {
                return Err(AppError::NotFound("Data source".to_string()));
            }

            let file_extension = path.extension();
            if file_extension.is_none() {
                return Err(AppError::Validation("Can't decode file type".to_string()));
            }

            let file_extension = file_extension.unwrap();
            let extension_str = file_extension.to_str();
            if extension_str.is_none() {
                return Err(AppError::Validation("Can't decode file type".to_string()));
            }

            let path = extension_str.unwrap().to_string();
            let src_medium: StorageMediums = path.as_str().try_into()?;

            let storage: Box<dyn ContactStore> = match src_medium {
                StorageMediums::Json => Box::new(JsonStorage {
                    medium: "json".to_string(),
                    path,
                }),
                StorageMediums::Txt => Box::new(TxtStorage {
                    medium: "txt".to_string(),
                    path,
                }),
            };

            let mut base = manager.mem.clone();
            let sync_status = manager.sync_from_storage(
                &mut base,
                storage,
                SyncPolicy::LastWriteWinsPolicy(LastWriteWinsPolicy),
            );

            if sync_status.is_err() {
                manager.save()?; // rollback to previous state on error

                return Err(sync_status.err().unwrap());
            } else {
                manager.mem = base;
                manager.index = Index::new(&manager)?
            }

            println!("Contacts synchronized successfully");

            manager.save()?;

            Ok(())
        }
    }
}

fn parse_list_order<T: std::cmp::Ord>(reverse: bool, a: T, b: T) -> std::cmp::Ordering {
    let cmp = a.cmp(&b);
    if reverse { cmp.reverse() } else { cmp }
}
