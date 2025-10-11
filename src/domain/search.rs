use crate::prelude::{AppError, parse_store};
use std::collections::HashMap;

// A B C D E, F G H I J, K L M N O, P Q R S T U V W X Y Z

pub fn create_name_search_index() -> Result<HashMap<char, (usize, usize)>, AppError> {
    let mut storage = parse_store()?;
    storage.load_migrated_contact()?;

    storage.mut_mem().sort_by(|a, b| {
        a.name
            .trim()
            .to_ascii_lowercase()
            .cmp(&b.name.trim().to_ascii_lowercase())
    });

    let mut index = HashMap::new();
    let contact_list = storage.get_mem();

    for alpha in 'a'..='z' {
        let next = (alpha as u8 + 1) as char;
        let start = contact_list.partition_point(|cont| {
            cont.name
                .chars()
                .next()
                .unwrap_or_default()
                .to_ascii_lowercase()
                < alpha
        });
        let end = contact_list.partition_point(|cont| {
            cont.name
                .chars()
                .next()
                .unwrap_or_default()
                .to_ascii_lowercase()
                < next
        });

        index.insert(alpha, (start, end));
    }

    Ok(index)
}

pub fn create_email_search_index() -> Result<HashMap<char, (usize, usize)>, AppError> {
    let mut storage = parse_store()?;
    storage.load_migrated_contact()?;

    storage.mut_mem().sort_by(|a, b| {
        a.email
            .trim()
            .to_ascii_lowercase()
            .cmp(&b.email.trim().to_ascii_lowercase())
    });

    let mut index = HashMap::new();
    let contact_list = storage.get_mem();

    for alpha in 'a'..='z' {
        let next = (alpha as u8 + 1) as char;
        let start = contact_list.partition_point(|cont| {
            cont.email
                .chars()
                .next()
                .unwrap_or_default()
                .to_ascii_lowercase()
                < alpha
        });

        let end = contact_list.partition_point(|cont| {
            cont.email
                .chars()
                .next()
                .unwrap_or_default()
                .to_ascii_lowercase()
                < next
        });
        index.insert(alpha, (start, end));
    }

    Ok(index)
}
