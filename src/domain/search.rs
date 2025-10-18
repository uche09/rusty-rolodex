use rust_fuzzy_search::fuzzy_compare;

use crate::prelude::{AppError, Contact};
use std::{collections::HashMap, sync::{Arc, Mutex}, thread};


pub fn create_name_search_index<'a>(contact_list: &'a Vec<&Contact>) -> Result<HashMap<char, Vec<&'a Contact>>, AppError> {
    let mid = contact_list.len() / 2;
    let (first_half, second_half) = contact_list.split_at(mid);

    let index: Arc<Mutex<HashMap<char, Vec<&Contact>>>> = Arc::new(Mutex::new(
        HashMap::new()
    ));


    thread::scope(|s| {
        let map1 = Arc::clone(&index);

        s.spawn(move || -> Result<(), AppError> {
            for &cont in first_half {
                if let Some(key) = cont.name.chars().next(){

                    if key.is_alphabetic() {
                        let mut map1_lock = map1.lock()?;
                        map1_lock.entry(key.to_ascii_lowercase())
                        .or_default()
                        .push(cont);
                    } else {
                        // If contact name does not start with an alphabet
                        let mut map1_lock = map1.lock()?;
                        map1_lock.entry('#')
                        .or_default()
                        .push(cont);
                    }
                }

            }

            Ok(())
        });

        let map2 = Arc::clone(&index);

        s.spawn(move || -> Result<(), AppError> {
            for &cont in second_half {
                if let Some(key) = cont.name.chars().next(){

                    if key.is_alphabetic() {
                        let mut map2_lock = map2.lock()?;
                        map2_lock.entry(key.to_ascii_lowercase())
                        .or_default()
                        .push(cont);
                    } else {
                        // If contact name does not start with an alphabet
                        let mut map2_lock = map2.lock()?;
                        map2_lock.entry('#')
                        .or_default()
                        .push(cont);
                    }
                }

            }

            Ok(())
        });
    });
    

    
    let result = Arc::into_inner(index).unwrap_or_default().into_inner()?;
    Ok(result)
}

pub fn create_email_domain_search_index<'a>(contact_list: &'a Vec<&Contact>) -> Result<HashMap<&'a str, Vec<&'a Contact>>, AppError> {
    let mid = contact_list.len() / 2;
    let (first_half, second_half) = contact_list.split_at(mid);

    let index: Arc<Mutex<HashMap<&str, Vec<&Contact>>>> = Arc::new(Mutex::new(
        HashMap::new()
    ));


    thread::scope(|s| {
        let map1 = Arc::clone(&index);

        s.spawn(move || -> Result<(), AppError> {
            for &cont in first_half {
                let email_parts: Vec<&str> = cont.email.split('@').collect();
                let domain = email_parts[email_parts.len() -1];
                
                let mut map1_lock = map1.lock()?;
                map1_lock.entry(domain)
                .or_default()
                .push(cont);
            }

            Ok(())
        });

        let map2 = Arc::clone(&index);

        s.spawn(move || -> Result<(), AppError> {
            for &cont in second_half {
                let email_parts: Vec<&str> = cont.email.split('@').collect();
                let domain = email_parts[email_parts.len() -1];
                
                let mut map2_lock = map2.lock()?;
                map2_lock.entry(domain)
                .or_default()
                .push(cont);
            }

            Ok(())
        });
    });
    

    
    let result = Arc::into_inner(index).unwrap_or_default().into_inner()?;
    Ok(result)
}


pub fn fuzzy_search_name_index<'a>(name: &str, contact_list: &'a Vec<&Contact>) -> Result<Vec<&'a Contact>, AppError> {
    if name.is_empty() {
        return Err(AppError::Validation("No Name provided".to_string()));
    }

    if name.len() > 30 {
        return Err(AppError::Validation("Search string too long".to_string()));
    }

    let empty_vec: Vec<&'a Contact> = Vec::new();

    let index_key = name.to_lowercase().chars().next().unwrap_or_default();
    let index = create_name_search_index(contact_list)?;
    let index_match = index.get(&index_key).unwrap_or(&empty_vec);

    let fuzzy_match: Vec<&Contact> = index_match
        .iter()
        .filter(|&&c| {
            fuzzy_compare(name.to_ascii_lowercase().as_str(), 
                &c.name.to_lowercase()) >= 0.7
        })
        .map(|c| *c)
        .collect();

    Ok(fuzzy_match)
}


pub fn fuzzy_search_email_domain_index<'a>(domain: &str, contact_list: &'a Vec<&Contact>) -> Result<Vec<&'a Contact>, AppError> {
    if domain.is_empty() {
        return Err(AppError::Validation("No email domain provided".to_string()));
    }

    if domain.len() > 15 {
        return Err(AppError::Validation("Please provide just email domain Eg. \"example.com\"".to_string()));
    }

    let empty_vec: Vec<&'a Contact> = Vec::new();

    let index = create_email_domain_search_index(contact_list)?;
    let index_match = index.get(domain).unwrap_or(&empty_vec);

    let fuzzy_match: Vec<&Contact> = index_match
        .iter()
        .map(|c| *c)
        .collect();

    Ok(fuzzy_match)
}