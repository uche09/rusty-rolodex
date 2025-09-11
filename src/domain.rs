use dotenv::dotenv;

use crate::{
    errors::AppError,
    store::{ContactStore, FileStore, JsonStore, MemStore},
};
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize, Eq, PartialOrd, Ord)]
pub struct Contact {
    pub name: String,
    pub phone: String,
    pub email: String,
}

#[derive(Debug)]
pub enum StorageChoice {
    Mem,
    Txt,
    Json,
}

pub struct Storage {
    pub file_store: FileStore,
    pub json_store: JsonStore,
    pub mem_store: MemStore,
    pub storage_choice: StorageChoice,
}

impl Storage {
    pub fn new() -> Result<Storage, AppError> {
        Ok(Storage {
            file_store: FileStore::new("./.instance/contacts.txt")?,
            json_store: JsonStore::new("./.instance/contacts.json"),
            mem_store: MemStore::new(),
            storage_choice: parse_storage_choice(),
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
            // if contact exist in txt and contacts are now stored in json or vice versa,
            // Avoid PARTIAL DELETE
            if self.storage_choice.is_json()
                && self
                    .file_store
                    .load()?
                    .contains(&self.mem_store.data[index])
            {
                let selected_contact = &self.mem_store.data[index];
                let mut txt_contacts = self.file_store.load()?;

                // Match selected contact in legacy file
                if let Some(contact_index) = txt_contacts
                    .iter()
                    .position(|cont| cont == selected_contact)
                {
                    // Delete match from legacy file
                    txt_contacts.remove(contact_index);
                    self.file_store.save(&txt_contacts)?;
                }
            } else if self.storage_choice.is_txt()
                && self
                    .json_store
                    .load()?
                    .contains(&self.mem_store.data[index])
            {
                let selected_contact = &self.mem_store.data[index];
                let mut json_contacts = self.json_store.load()?;

                // Match selected contact in legacy file
                if let Some(contact_index) = json_contacts
                    .iter()
                    .position(|cont| cont == selected_contact)
                {
                    json_contacts.remove(contact_index);
                }
                self.json_store.save(&json_contacts)?;
            }

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
            match self.file_store.save(&self.mem_store.data) {
                Ok(_) => Ok(()),
                Err(e) => Err(e),
            }?;
        }

        if self.storage_choice.is_json() {
            match self.save_json(&self.mem_store.data) {
                Ok(_) => Ok(()),
                Err(e) => Err(e),
            }?;
        }

        Ok(())
    }

    pub fn load_txt(&self) -> Result<Vec<Contact>, AppError> {
        self.file_store.load()
    }

    pub fn load_json(&self) -> Result<Vec<Contact>, AppError> {
        self.json_store.load()
    }

    pub fn save_json(&self, contacts: &[Contact]) -> Result<(), AppError> {
        self.json_store.save(contacts)
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
    use crate::store::{ContactStore, load_migrated_contact};

    #[test]
    fn adds_persistent_contact_with_txt() -> Result<(), AppError> {
        let mut storage = Storage::new()?;

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

        storage.mem_store.data.clear();
        storage.file_store.save(&storage.mem_store.data)?;
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
        let index = storage.get_indices_by_name(&"Uche".to_string()).unwrap_or_default();
        storage.delete_contact(index[0])?;
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

        storage.mem_store.data.clear();
        storage.file_store.save(&storage.mem_store.data)?;
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
        };

        let contact2 = Contact {
            name: "Alex".to_string(),
            phone: "01234567890".to_string(),
            email: "".to_string(),
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
            }
        );

        assert_eq!(
            storage.mem_store.data[1],
            Contact {
                name: "Alex".to_string(),
                phone: "01234567890".to_string(),
                email: "".to_string(),
            }
        );

        storage.delete_contact(0)?;
        storage.json_store.save(&storage.mem_store.data)?;

        storage.mem_store.data.clear();
        storage.mem_store.data = storage.json_store.load()?;

        assert_eq!(storage.mem_store.data.len(), 1);

        assert_ne!(
            storage.contact_list()[0],
            Contact {
                name: "Uche".to_string(),
                phone: "01234567890".to_string(),
                email: "ucheuche@gmail.com".to_string(),
            }
        );

        storage.mem_store.data.clear();
        storage.json_store.save(&storage.mem_store.data)?;
        storage.file_store.save(&storage.mem_store.data)?;

        Ok(())
    }

    #[test]
    fn check_partial_delete() -> Result<(), AppError> {
        {
            unsafe {
                std::env::set_var("STORAGE_CHOICE", "txt");
            }

            let mut storage = Storage::new()?;
            storage.mem_store.data.clear();

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

            storage.save()?
        }

        unsafe {
            std::env::set_var("STORAGE_CHOICE", "json");
        }

        let mut storage = Storage::new()?;
        storage.mem_store.data.clear();

        storage.mem_store.data = load_migrated_contact(&storage)?;

        assert!(storage.mem_store.data.len() == 2);

        let index = storage.get_indices_by_name(&"Alex".to_string()).unwrap_or_default();

        assert!(index.len() == 1);

        storage.delete_contact(index[0])?;
        storage.save()?;

        storage.mem_store.data.clear();
        storage.mem_store.data = storage.file_store.load()?;

        assert_eq!(storage.mem_store.data.len(), 1);

        assert_eq!(
            storage.mem_store.data[0],
            Contact {
                name: "Uche".to_string(),
                phone: "01234567890".to_string(),
                email: "ucheuche@gmail.com".to_string(),
            }
        );

        storage.mem_store.data.clear();
        storage.save()?;

        Ok(())
    }
}
