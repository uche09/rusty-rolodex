use super::contact::Contact;
use crate::{
    errors::AppError,
    store::{ContactStore, file, memory},
};
use dotenv::dotenv;
use std::{fs, path::Path};

#[derive(Debug)]
pub enum StorageChoice {
    Mem,
    Txt,
    Json,
}

pub struct Storage {
    pub txt_store: file::TxtStore,
    pub json_store: file::JsonStore,
    pub mem_store: memory::MemStore,
    pub storage_choice: StorageChoice,
}

impl Storage {
    pub fn new() -> Result<Storage, AppError> {
        Ok(Storage {
            txt_store: file::TxtStore::new("./.instance/contacts.txt")?,
            json_store: file::JsonStore::new("./.instance/contacts.json")?,
            mem_store: memory::MemStore::new(),
            storage_choice: parse_storage_choice(),
        })
    }

    pub fn add_contact(&mut self, contact: Contact) {
        self.mem_store.data.push(contact);
    }

    pub fn contact_list(&self) -> Vec<&Contact> {
        let contacts = self.mem_store.iter().collect();
        contacts
    }

    pub fn delete_contact(&mut self, index: usize) -> Result<(), AppError> {
        if index < self.mem_store.data.len() {
            self.mem_store.data.remove(index);
            Ok(())
        } else {
            Err(AppError::NotFound("Contact".to_string()))
        }
    }

    pub fn save(&self) -> Result<(), AppError> {
        if self.storage_choice.is_mem() {
            return Ok(());
        }

        if self.storage_choice.is_txt() {
            self.save_txt(&self.mem_store.data)?;
        }

        if self.storage_choice.is_json() {
            self.save_json(&self.mem_store.data)?;
        }

        // Delete migrated storage file for COMPLETE MIGRATION
        if self.storage_choice.is_json() && Path::new(&self.txt_store.path).exists() {
            fs::remove_file(Path::new(&self.txt_store.path))?;
        } else if self.storage_choice.is_txt() && Path::new(&self.json_store.path).exists() {
            fs::remove_file(Path::new(&self.json_store.path))?;
        }

        Ok(())
    }

    pub fn load_txt(&self) -> Result<Vec<Contact>, AppError> {
        self.txt_store.load()
    }

    pub fn load_json(&self) -> Result<Vec<Contact>, AppError> {
        self.json_store.load()
    }

    pub fn save_json(&self, contacts: &[Contact]) -> Result<(), AppError> {
        self.json_store.save(contacts)
    }

    pub fn save_txt(&self, contacts: &[Contact]) -> Result<(), AppError> {
        self.txt_store.save(contacts)
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

impl StorageChoice {
    pub fn is_json(&self) -> bool {
        matches!(self, StorageChoice::Json)
    }
    pub fn is_mem(&self) -> bool {
        matches!(self, StorageChoice::Mem)
    }

    pub fn is_txt(&self) -> bool {
        matches!(self, StorageChoice::Txt)
    }

    pub fn is_which(&self) -> String {
        if self.is_json() {
            "json".to_string()
        } else if self.is_txt() {
            return "txt".to_string();
        } else {
            return "mem".to_string();
        }
    }
}

pub fn parse_storage_choice() -> StorageChoice {
    dotenv().ok();

    let choice = std::env::var("STORAGE_CHOICE").unwrap_or("mem".to_string());
    match choice.to_lowercase().as_str() {
        "json" => StorageChoice::Json,
        "mem" => StorageChoice::Mem,
        "txt" => StorageChoice::Txt,
        _ => StorageChoice::Mem,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::ContactStore;

    #[test]
    fn adds_persistent_contact_with_txt() -> Result<(), AppError> {
        let mut storage = Storage::new()?;

        let new_contact = Contact {
            name: "Uche".to_string(),
            phone: "01234567890".to_string(),
            email: "ucheuche@gmail.com".to_string(),
            tag: "".to_string(),
        };

        storage.add_contact(new_contact);
        storage.txt_store.save(&storage.mem_store.data)?;
        storage.mem_store.data.clear();
        storage.mem_store.data = storage.txt_store.load()?;

        assert_eq!(
            storage.contact_list()[0],
            &Contact {
                name: "Uche".to_string(),
                phone: "01234567890".to_string(),
                email: "ucheuche@gmail.com".to_string(),
                tag: "".to_string(),
            }
        );

        storage.mem_store.data.clear();
        storage.txt_store.save(&storage.mem_store.data)?;
        storage.json_store.save(&storage.mem_store.data)?;
        Ok(())
    }

    #[test]
    fn delete_persistent_contact_with_txt() -> Result<(), AppError> {
        let mut storage = Storage::new()?;

        let contact1 = Contact {
            name: "Uche".to_string(),
            phone: "01234567890".to_string(),
            email: "ucheuche@gmail.com".to_string(),
            tag: "".to_string(),
        };

        let contact2 = Contact {
            name: "Alex".to_string(),
            phone: "01234567890".to_string(),
            email: "".to_string(),
            tag: "".to_string(),
        };

        storage.add_contact(contact1);
        storage.add_contact(contact2);

        storage.txt_store.save(&storage.mem_store.data)?;
        storage.mem_store.data.clear();

        storage.mem_store.data = storage.txt_store.load()?;
        let index = storage
            .get_indices_by_name(&"Uche".to_string())
            .unwrap_or_default();
        storage.delete_contact(index[0])?;
        storage.txt_store.save(&storage.mem_store.data)?;

        storage.mem_store.data.clear();
        storage.mem_store.data = storage.txt_store.load()?;

        assert_eq!(storage.mem_store.data.len(), 1);

        assert_ne!(
            *storage.contact_list()[0],
            Contact {
                name: "Uche".to_string(),
                phone: "01234567890".to_string(),
                email: "ucheuche@gmail.com".to_string(),
                tag: "".to_string(),
            }
        );

        storage.mem_store.data.clear();
        storage.txt_store.save(&storage.mem_store.data)?;
        storage.json_store.save(&storage.mem_store.data)?;

        Ok(())
    }

    #[test]
    fn json_store_is_persistent() -> Result<(), AppError> {
        let mut storage = Storage::new()?;

        let contact1 = Contact {
            name: "Uche".to_string(),
            phone: "01234567890".to_string(),
            email: "ucheuche@gmail.com".to_string(),
            tag: "".to_string(),
        };

        let contact2 = Contact {
            name: "Alex".to_string(),
            phone: "01234567890".to_string(),
            email: "".to_string(),
            tag: "".to_string(),
        };

        storage.add_contact(contact1);
        storage.add_contact(contact2);

        storage.json_store.save(&storage.mem_store.data)?;
        storage.mem_store.data.clear();

        storage.mem_store.data = storage.json_store.load()?;

        assert_eq!(
            storage.mem_store.data[0],
            Contact {
                name: "Uche".to_string(),
                phone: "01234567890".to_string(),
                email: "ucheuche@gmail.com".to_string(),
                tag: "".to_string(),
            }
        );

        assert_eq!(
            storage.mem_store.data[1],
            Contact {
                name: "Alex".to_string(),
                phone: "01234567890".to_string(),
                email: "".to_string(),
                tag: "".to_string(),
            }
        );

        storage.delete_contact(0)?;
        storage.json_store.save(&storage.mem_store.data)?;

        storage.mem_store.data.clear();
        storage.mem_store.data = storage.json_store.load()?;

        assert_eq!(storage.mem_store.data.len(), 1);

        assert_ne!(
            *storage.contact_list()[0],
            Contact {
                name: "Uche".to_string(),
                phone: "01234567890".to_string(),
                email: "ucheuche@gmail.com".to_string(),
                tag: "".to_string(),
            }
        );

        storage.mem_store.data.clear();
        storage.json_store.save(&storage.mem_store.data)?;
        storage.txt_store.save(&storage.mem_store.data)?;

        Ok(())
    }
}
