use crate::{
    domain::contact,
    prelude::{
        AppError,
        command::{Cli, Commands, SearchKey, SortKey},
        contact::{Contact, ValidationReq, phone_number_matches},
        create_email_search_index, create_name_search_index, parse_store,
        store::{
            self,
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

    let mut storage = parse_store()?;

    storage.load_migrated_contact()?;

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

            storage.save(storage.get_mem())?;

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
                        .mut_mem()
                        .sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase())),
                    SortKey::Email => storage
                        .mut_mem()
                        .sort_by(|a, b| a.email.to_lowercase().cmp(&b.email.to_lowercase())),
                    SortKey::Created => storage
                        .mut_mem()
                        .sort_by(|a, b| a.created_at.cmp(&b.created_at)),
                    SortKey::Updated => storage
                        .mut_mem()
                        .sort_by(|a, b| a.updated_at.cmp(&b.updated_at)),
                }
            }

            if reverse {
                storage.mut_mem().reverse();
            }

            if let Some(tag) = tag {
                let filtered_contacts: Vec<&Contact> = storage
                    .get_mem()
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
                .mut_mem()
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

            storage.save(storage.get_mem())?;
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

                    storage.save(storage.get_mem())?;
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
        Commands::Search { by, name, email } => {
            // Default search = name (if not provided)
            let mut search_by = "name";

            if let Some(search_key) = by {
                match search_key {
                    SearchKey::N => {
                        search_by = "name";
                    }
                    SearchKey::E => {
                        search_by = "email";
                    }
                }
            }

            // Search using email address
            if search_by == "email" {
                // user's provided email strig is assigned to "search_for"
                let mut searched_for = "".to_string();

                // Validate user provided string before assigning
                if let Some(addr) = email {
                    searched_for = addr.trim().to_owned();

                    if addr.is_empty() {
                        return Err(AppError::Validation("No email provided".to_string()));
                    }

                    if addr.len() > 15 {
                        return Err(AppError::Validation("Search string too long".to_string()));
                    }
                }

                // key is the first later of the string
                let index_key = searched_for.to_ascii_lowercase().chars().next().unwrap_or_default();

                storage.mut_mem().sort_by(|a, b| {
                    a.email
                        .trim()
                        .to_ascii_lowercase()
                        .cmp(&b.email.trim().to_ascii_lowercase())
                });

                let index = create_email_search_index()?;

                // Get index range for search
                let (start, end) = index.get(&index_key).unwrap_or(&(0usize, 0usize));

                // fuzzy search filter
                let filtered_contacts: Vec<&Contact> = storage.get_mem()[*start..*end]
                    .iter()
                    .filter(|con| {
                        con.email
                            .to_lowercase()
                            .contains(searched_for.to_lowercase().as_str())
                    })
                    .collect();

                if filtered_contacts.is_empty() {
                    println!("Couldn't find an email with {searched_for}");
                    return Ok(());
                }

                for (mut i, c) in filtered_contacts.iter().enumerate() {
                    i += 1;

                    let date = c
                        .updated_at
                        .map(|dt| dt.date_naive().to_string())
                        .unwrap_or_else(|| "".to_string());

                    println!(
                        "{i:>3}. {:<20} {:15} {:^30} {:<15} 'Updated on:' {:<12}",
                        c.name, c.phone, c.email, c.tag, date
                    );
                }
            } else {
                // Same logic for name

                let mut searched_for = "".to_string();
                if let Some(n) = name {
                    searched_for = n.trim().to_owned();

                    if n.is_empty() {
                        return Err(AppError::Validation("No Name provided".to_string()));
                    }

                    if n.len() > 10 {
                        return Err(AppError::Validation("Search string too long".to_string()));
                    }
                }
                
                let index_key = searched_for.to_lowercase().chars().next().unwrap_or_default();
                
                let index = create_name_search_index()?;

                let mut storage = parse_store()?;
                
                storage.load_migrated_contact()?;

                storage.mut_mem().sort_by(|a, b| {
                    a.name
                        .trim()
                        .to_ascii_lowercase()
                        .cmp(&b.name.trim().to_ascii_lowercase())
                });

                let (start, end) = index.get(&index_key).unwrap_or(&(0usize, 0usize));

                let filtered_contacts: Vec<&Contact> = storage.get_mem()[*start..*end]
                    .iter()
                    .filter(|con| {
                        con.name
                            .to_lowercase()
                            .contains(searched_for.to_lowercase().as_str())
                    })
                    .collect();

                    
                if filtered_contacts.is_empty() {
                    println!("Couldn't find a name with {searched_for}");
                    return Ok(());
                }

                for (mut i, c) in filtered_contacts.iter().enumerate() {
                    i += 1;

                    let date = c
                        .updated_at
                        .map(|dt| dt.date_naive().to_string())
                        .unwrap_or_else(|| "".to_string());

                    println!(
                        "{i:>3}. {:<20} {:15} {:^30} {:<15} 'Updated on:' {:<12}",
                        c.name, c.phone, c.email, c.tag, date
                    );
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

            if file_path.is_empty() {
                let (path, total) = export_contacts_to_csv(storage.get_mem(), None)?;

                println!("Successfully exported {} contacts to {:?}.", total, path);
                return Ok(());
            }

            let (path, total) = export_contacts_to_csv(storage.get_mem(), Some(&file_path))?;

            println!("Successfully exported {} contacts to {:?}.", total, path);
            Ok(())
        }
    }
}
