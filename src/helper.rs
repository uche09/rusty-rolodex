use std::fs::File;
use std::io::{BufRead, BufReader};

use crate::domain::contact::Contact;
use crate::errors::AppError;

pub fn serialize_contacts(contacts: &[Contact]) -> String {
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
    let mut test_contact = Contact {
        name: "".to_string(),
        phone: "".to_string(),
        email: "".to_string(),
    };
    let mut name = String::new();
    let mut phone = String::new();
    let mut email = String::new();

    for line in buffer.lines() {
        let line = line?;
        let line = line.trim();

        if line == "{" {
            // Start of a new contact format
            continue;
        }

        if line == "}" {
            // End of a contact format
            let contact = Contact {
                name: name.clone(),
                phone: phone.clone(),
                email: email.clone(),
            };
            contacts.push(contact);
            continue;
        }

        test_contact.name = line.to_string();
        if test_contact.validate_name()? {
            name = line.to_string();
            continue;
        }

        test_contact.phone = line.to_string();
        if test_contact.validate_number()? {
            phone = line.to_string();
            continue;
        }

        test_contact.email = line.to_string();
        if test_contact.validate_email()? {
            email = line.to_string();
            continue;
        }
    }

    Ok(contacts)
}

#[cfg(test)]
mod tests {
    use crate::domain::storage::Storage;
    use crate::store::ContactStore;

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
    fn check_deserialization_from_txt() -> Result<(), AppError> {
        let mut storage = Storage::new()?;

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
            }
        );

        Ok(assert_eq!(
            storage.mem_store.data[1],
            Contact {
                name: "Mom".to_string(),
                phone: "98765432109".to_string(),
                email: "ucheuche@gmail.com".to_string(),
            }
        ))
    }
}
