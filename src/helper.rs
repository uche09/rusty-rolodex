use chrono::{DateTime, Utc};

use crate::prelude::{AppError, Contact, uuid::Uuid, HashMap};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::str::FromStr;

pub fn serialize_contacts(contacts: &HashMap<Uuid, Contact>) -> String {
    let mut data = String::new();

    for (_, contact) in contacts {
        let created_at_str = contact
            .created_at.to_string();

        let updated_at_str = contact
            .updated_at.to_string();

        let ser_contact = format!(
            "{{\n\
        id: {}\n\
        name: {}\n\
        phone: {}\n\
        email: {}\n\
        tag: {}\n\
        created_at: {}\n\
        updated_at: {}\n\
        }}\n",
            contact.id.to_string(), contact.name, contact.phone, contact.email, contact.tag, created_at_str, updated_at_str,
        );

        data.push_str(&ser_contact);
    }
    data
}

fn split_annotation(line: &str) -> (Option<&str>, &str) {
    if let Some((key, value)) = line.split_once(':') {
        (Some(key.trim()), value.trim())
    } else {
        (None, line.trim())
    }
}

pub fn deserialize_contacts_from_txt_buffer(
    buffer: BufReader<File>,
) -> Result<HashMap<Uuid, Contact>, AppError> {
    let mut contacts = HashMap::new();
    let mut test_contact = Contact {
        id: Uuid::new_v4(),
        name: "".to_string(),
        phone: "".to_string(),
        email: "".to_string(),
        tag: "".to_string(),
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };
    let mut id = Uuid::new_v4();
    let mut name = "".to_string();
    let mut phone = "".to_string();
    let mut email = "".to_string();
    let mut tag: String = "".to_string();
    let mut created_at = Utc::now();
    let mut updated_at = Utc::now();

    for line in buffer.lines() {
        let line = line?;
        let (key, value) = split_annotation(&line);

        if value == "{" {
            // Start of a new contact format
            continue;
        }

        if value == "}" {
            // End of a contact format
            let contact = Contact {
                id: id.clone(),
                name: name.clone(),
                phone: phone.clone(),
                email: email.clone(),
                tag: tag.clone(),
                created_at,
                updated_at,
            };
            contacts.insert(contact.id.clone(), contact);
            continue;
        }

        if key == Some("id") {
            let parse_result =  Uuid::try_parse(value);
            
            if parse_result.is_ok(){
                id = parse_result.unwrap();
            }
        }

        if key == Some("name") {
            name = value.to_string();
            continue;
        } else if key.is_none() {
            test_contact.name = value.to_string();
            if test_contact.validate_name()? {
                name = value.to_string();
                continue;
            }
        }

        if key == Some("phone") {
            phone = value.to_string();
            continue;
        } else if key.is_none() {
            test_contact.phone = value.to_string();
            if test_contact.validate_number()? {
                phone = value.to_string();
                continue;
            }
        }

        if key.is_some() && key == Some("email") {
            email = value.to_string();
            continue;
        } else if key.is_none() {
            test_contact.email = value.to_string();
            if test_contact.validate_email()? {
                email = value.to_string();
                continue;
            }
            print!("Failed email validation");
        }

        if key.is_some() && key == Some("tag") {
            tag = value.to_string();
            continue;
        }

        if key.is_some() && key == Some("created_at") {
            if value.is_empty() {
                continue;
            } else {
                created_at = DateTime::<Utc>::from_str(value)?.to_utc();
            }
            continue;
        }

        if key.is_some() && key == Some("updated_at") {
            if value.is_empty() {
                continue;
            } else {
                updated_at = DateTime::<Utc>::from_str(value)?.to_utc()
            }
            continue;
        }
    }

    Ok(contacts)
}

#[cfg(test)]

mod tests {
    use crate::prelude::ContactStore;
    use crate::store::filestore::Store;

    use super::*;
    use std::env;

    

    #[test]
    fn check_serialize_contact() -> Result<(), AppError> {
        let dt_now = Utc::now();
        let id = Uuid::new_v4();

        let contact = Contact {
            id: id.clone(),
            name: "Uche".to_string(),
            phone: "012345678901".to_string(),
            email: "ucheuche@gmail.com".to_string(),
            tag: "".to_string(),
            created_at: dt_now.clone(),
            updated_at: dt_now.clone(),
        };

        let mut contacts = HashMap::new();
        contacts.insert(contact.id, contact);

        let ser_data = serialize_contacts(&contacts);

        assert_eq!(
            ser_data,
            format!("{{\n\
                id: {}\n\
                name: Uche\n\
                phone: 012345678901\n\
                email: ucheuche@gmail.com\n\
                tag: \n\
                created_at: {}\n\
                updated_at: {}\n\
            }}\n",
            id.clone(), dt_now.to_string(), dt_now.to_string())
            
        );

        Ok(())
    }

    #[test]
    fn check_deserialization_from_txt() -> Result<(), AppError> {

        // Testing should be ran explicitly on a single thread to avoid race condition from multiply test threads
        unsafe {
            env::set_var("STORAGE_CHOICE", "txt");
        }

        let mut storage = Store::new()?;

        let contact1 = Contact::new(
            "Uche".to_string(),
            "012345678901".to_string(),
            String::new(),
            "".to_string(),
        );

        let contact2 = Contact::new(
            "Mom".to_string(),
            "98765432109".to_string(),
            "ucheuche@gmail.com".to_string(),
            "".to_string(),
        );

        let id_1 = contact1.id.clone();
        let id_2 = contact2.id.clone();

        storage.mem.insert(contact1.id.clone(), contact1);
        storage.mem.insert(contact2.id.clone(), contact2);

        storage.save(&storage.mem)?;
        storage.mem.clear();
        storage.mem = storage.load()?;

        assert_eq!(
            storage.mem.get(&id_1).unwrap(),
            &Contact::new(
                "Uche".to_string(),
                "012345678901".to_string(),
                String::new(),
                "".to_string(),
            )
        );

        assert_eq!(
            storage.mem.get(&id_2).unwrap(),
            &Contact::new(
                "Mom".to_string(),
                "98765432109".to_string(),
                "ucheuche@gmail.com".to_string(),
                "".to_string(),
            )
        );

        storage.mem.clear();
        storage.save(&storage.mem)?;
        Ok(())
    }
}
