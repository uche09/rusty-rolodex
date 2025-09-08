use crate::{
    errors::AppError,
    store::{FileStore, MemStore},
};
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct Contact {
    pub name: String,
    pub phone: String,
    pub email: String,
}

pub enum Command {
    AddContact,
    ListContacts,
    DeleteContact,
    Exit,
}

pub struct Storage {
    pub file_store: FileStore,
    pub mem_store: MemStore,
}

impl Storage {
    pub fn new(path: &str) -> Result<Storage, AppError> {
        Ok(Storage {
            file_store: FileStore::new(path)?,
            mem_store: MemStore::new(),
        })
    }

    pub fn add_contact(&mut self, contact: Contact) {
        self.mem_store.data.push(contact);
    }

    pub fn contact_list(&self) -> &Vec<Contact> {
        &self.mem_store.data
    }

    pub fn delete_contact(&mut self, index: usize) -> Result<(), AppError> {
        if index < self.mem_store.data.len() {
            self.mem_store.data.remove(index);
            Ok(())
        } else {
            Err(AppError::NotFound("Contact".to_string()))
        }
    }

    pub fn get_indices_by_name(&self, name: &String) -> Option<Vec<usize>> {
        let indices: Vec<usize> = self
            .contact_list()
            .iter()
            .enumerate()
            .filter(|(_, cont)| &cont.name == name)
            .map(|(idx, _)| idx)
            .collect();
        if indices.is_empty() {
            return None;
        }
        Some(indices)
    }
}

#[cfg(test)]
mod tests {
    use crate::store::ContactStore;

    use super::*;

    #[test]
    fn adds_persistent_contact() -> Result<(), AppError> {
        let mut storage = Storage::new("./.instance/contacts.txt")?;

        let new_contact = Contact {
            name: "Uche".to_string(),
            phone: "01234567890".to_string(),
            email: "ucheuche@gmail.com".to_string(),
        };

        storage.add_contact(new_contact);
        storage.file_store.save(&storage.mem_store.data)?;
        storage.mem_store.data.clear();
        storage.mem_store.data = storage.file_store.load()?;

        assert_eq!(
            storage.contact_list()[0],
            Contact {
                name: "Uche".to_string(),
                phone: "01234567890".to_string(),
                email: "ucheuche@gmail.com".to_string(),
            }
        );
        Ok(())
    }

    #[test]
    fn delete_persistent_contact() -> Result<(), AppError> {
        let mut storage = Storage::new("./.instance/contacts.txt")?;

        let contact1 = Contact {
            name: "Uche".to_string(),
            phone: "01234567890".to_string(),
            email: "ucheuche@gmail.com".to_string(),
        };

        let contact2 = Contact {
            name: "Alex".to_string(),
            phone: "01234567890".to_string(),
            email: "".to_string(),
        };

        storage.add_contact(contact1);
        storage.add_contact(contact2);

        storage.file_store.save(&storage.mem_store.data)?;
        storage.mem_store.data.clear();

        storage.mem_store.data = storage.file_store.load()?;
        storage.delete_contact(0)?;
        storage.file_store.save(&storage.mem_store.data)?;

        storage.mem_store.data.clear();
        storage.mem_store.data = storage.file_store.load()?;

        assert_eq!(storage.mem_store.data.len(), 1);

        assert_ne!(
            storage.contact_list()[0],
            Contact {
                name: "Uche".to_string(),
                phone: "01234567890".to_string(),
                email: "ucheuche@gmail.com".to_string(),
            }
        );

        Ok(())
    }
}
