use crate::prelude::{AppError, Contact};
use std::fs::File;
use std::io::{BufRead, BufReader};

pub fn serialize_contacts(contacts: &[Contact]) -> String {
    let mut data = String::new();

    for contact in contacts {
        let ser_contact = format!(
            "{{\n\
        name: {}\n\
        phone: {}\n\
        email: {}\n\
        tag: {}\n\
        }}\n",
            contact.name, contact.phone, contact.email, contact.tag
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
    };
    let mut name = "".to_string();
    let mut phone = "".to_string();
    let mut email = "".to_string();
    let mut tag: String = "".to_string();

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
    }

    Ok(contacts)
}

#[cfg(test)]
mod tests {
    use crate::domain::storage::Storage;
    use crate::prelude::ContactStore;

    use super::*;

    #[test]
    fn check_serialize_contact() {
        let contacts = vec![Contact {
            name: "Uche".to_string(),
            phone: "012345678901".to_string(),
            email: "ucheuche@gmail.com".to_string(),
            tag: "".to_string(),
        }];

        let ser_data = serialize_contacts(&contacts);

        assert_eq!(
            ser_data,
            "{\n\
            name: Uche\n\
            phone: 012345678901\n\
            email: ucheuche@gmail.com\n\
            tag: \n\
        }\n"
            .to_string()
        )
    }

    #[test]
    fn check_deserialization_from_txt() -> Result<(), AppError> {
        let mut storage = Storage::new()?;

        let contact1 = Contact {
            name: "Uche".to_string(),
            phone: "012345678901".to_string(),
            email: String::new(),
            tag: "".to_string(),
        };

        let contact2 = Contact {
            name: "Mom".to_string(),
            phone: "98765432109".to_string(),
            email: "ucheuche@gmail.com".to_string(),
            tag: "".to_string(),
        };

        storage.mem_store.data.push(contact1);
        storage.mem_store.data.push(contact2);

        storage.txt_store.save(&storage.mem_store.data)?;
        storage.mem_store.data.clear();
        storage.mem_store.data = storage.txt_store.load()?;

        assert_eq!(
            storage.mem_store.data[0],
            Contact {
                name: "Uche".to_string(),
                phone: "012345678901".to_string(),
                email: String::new(),
                tag: "".to_string(),
            }
        );

        assert_eq!(
            storage.mem_store.data[1],
            Contact {
                name: "Mom".to_string(),
                phone: "98765432109".to_string(),
                email: "ucheuche@gmail.com".to_string(),
                tag: "".to_string(),
            }
        );

        storage.mem_store.data.clear();
        storage.txt_store.save(&storage.mem_store.data)?;
        storage.json_store.save(&storage.mem_store.data)?;
        Ok(())
    }
}
