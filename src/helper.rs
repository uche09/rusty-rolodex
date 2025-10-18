use chrono::{DateTime, Utc};

use crate::prelude::{AppError, Contact};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::str::FromStr;

pub fn serialize_contacts(contacts: &[Contact]) -> String {
    let mut data = String::new();

    for contact in contacts {
        let created_at_str = contact
            .created_at.to_string();

        let updated_at_str = contact
            .updated_at.to_string();

        let ser_contact = format!(
            "{{\n\
        name: {}\n\
        phone: {}\n\
        email: {}\n\
        tag: {}\n\
        created_at: {}\n\
        updated_at: {}\n\
        }}\n",
            contact.name, contact.phone, contact.email, contact.tag, created_at_str, updated_at_str,
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
) -> Result<Vec<Contact>, AppError> {
    let mut contacts = Vec::new();
    let mut test_contact = Contact {
        name: "".to_string(),
        phone: "".to_string(),
        email: "".to_string(),
        tag: "".to_string(),
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };
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
                name: name.clone(),
                phone: phone.clone(),
                email: email.clone(),
                tag: tag.clone(),
                created_at,
                updated_at,
            };
            contacts.push(contact);
            continue;
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
    use crate::store::txt::TxtStore;

    use super::*;

    #[test]
    // fn check_serialize_contact() -> Result<(), AppError> {
    //     let contacts = vec![Contact {
    //         name: "Uche".to_string(),
    //         phone: "012345678901".to_string(),
    //         email: "ucheuche@gmail.com".to_string(),
    //         tag: "".to_string(),
    //         created_at: Utc::now(),
    //         updated_at: Utc::now(),
    //     }];

    //     let ser_data = serialize_contacts(&contacts);

    //     assert_eq!(
    //         ser_data,
    //         "{\n\
    //         name: Uche\n\
    //         phone: 012345678901\n\
    //         email: ucheuche@gmail.com\n\
    //         tag: \n\
    //         created_at: \n\
    //         updated_at: \n\
    //     }\n"
    //         .to_string()
    //     );

    //     Ok(())
    // }

    #[test]
    fn check_deserialization_from_txt() -> Result<(), AppError> {
        let mut storage = TxtStore::new()?;

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

        storage.mem.push(contact1);
        storage.mem.push(contact2);

        storage.save(&storage.mem)?;
        storage.mem.clear();
        storage.mem = storage.load()?;

        assert_eq!(
            storage.mem[0],
            Contact::new(
                "Uche".to_string(),
                "012345678901".to_string(),
                String::new(),
                "".to_string(),
            )
        );

        assert_eq!(
            storage.mem[1],
            Contact::new(
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
