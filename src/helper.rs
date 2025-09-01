use std::fs::File;
use std::io::{BufRead, BufReader};

use crate::domain::Contact;
use crate::errors::AppError;
use crate::validation::{validate_email, validate_name, validate_number};

pub fn serialize_contacts(contacts: &Vec<Contact>) -> String {
    let mut data = String::new();

    for contact in contacts {
        let ser_contact = format!(
            "{{\n\
        {}\n\
        {}\n\
        {}\n\
        }}\n",
            contact.name, contact.phone, contact.email
        );

        data.push_str(&ser_contact);
    }
    data
}

pub fn deserialize_contacts_from_txt_buffer(
    buffer: BufReader<File>,
) -> Result<Vec<Contact>, AppError> {
    let mut contacts = Vec::new();
    let mut name = String::new();
    let mut phone = String::new();
    let mut email = String::new();

    for line in buffer.lines() {
        let line = line?;
        let line = line.trim();

        if line == "{" {
            // Start of a new contact format
            continue;
        } else if line == "}" {
            // End of a contact format
            let contact = Contact {
                name: name.clone(),
                phone: phone.clone(),
                email: email.clone(),
            };
            contacts.push(contact);
            continue;
        } else if validate_number(line) {
            phone = line.to_string();
            continue;
        } else if validate_email(line) {
            email = line.to_string();
            continue;
        } else if validate_name(line) {
            name = line.to_string();
            continue;
        }
    }

    Ok(contacts)
}

#[cfg(test)]
mod tests {
    use crate::store::{ContactStore, Store};

    use super::*;

    #[test]
    fn check_serialize_contact() {
        let contacts = vec![Contact {
            name: "Uche".to_string(),
            phone: "012345678901".to_string(),
            email: "ucheuche@gmail.com".to_string(),
        }];

        let ser_data = serialize_contacts(&contacts);

        assert_eq!(
            ser_data,
            "{\n\
            Uche\n\
            012345678901\n\
            ucheuche@gmail.com\n\
        }\n"
            .to_string()
        )
    }

    #[test]
    fn check_deserialization_from_file() -> Result<(), AppError> {
        let mut storage = Store::new()?;

        let contact1 = Contact {
            name: "Uche".to_string(),
            phone: "012345678901".to_string(),
            email: String::new(),
        };

        let contact2 = Contact {
            name: "Mom".to_string(),
            phone: "98765432109".to_string(),
            email: "ucheuche@gmail.com".to_string(),
        };

        storage.mem.push(contact1);
        storage.mem.push(contact2);

        let _ = storage.save();
        storage.mem.clear();
        let _ = storage.load();

        assert_eq!(
            storage.mem[0],
            Contact {
                name: "Uche".to_string(),
                phone: "012345678901".to_string(),
                email: String::new(),
            }
        );

        Ok(assert_eq!(
            storage.mem[1],
            Contact {
                name: "Mom".to_string(),
                phone: "98765432109".to_string(),
                email: "ucheuche@gmail.com".to_string(),
            }
        ))
    }
}
