use crate::domain::{Contact, Storage};
use crate::errors::AppError;
use crate::helper;
use std::fs::{self, OpenOptions};
use std::io::{BufReader, Read, Write};
use std::path::Path;

// pub const FILE_PATH: &str = "./.instance/contacts.txt";

pub struct FileStore {
    pub path: String,
}

pub struct MemStore {
    pub data: Vec<Contact>,
}

pub struct JsonStore {
    pub path: String,
}

impl FileStore {
    pub fn new(path: &str) -> Result<Self, AppError> {
        create_file_parent(path)?;

        Ok(FileStore {
            path: path.to_string(),
        })
    }
}

impl MemStore {
    pub fn new() -> Self {
        Self { data: Vec::new() }
    }
}

impl JsonStore {
    pub fn new(path: &str) -> Self {
        Self {
            path: path.to_string(),
        }
    }
}

pub trait ContactStore {
    fn load(&self) -> Result<Vec<Contact>, AppError>;

    fn save(&self, contacts: &[Contact]) -> Result<(), AppError>;
}

impl ContactStore for FileStore {
    fn load(&self) -> Result<Vec<Contact>, AppError> {
        // Read text from file
        // Using OpenOptions to open file if already exist
        // Or create one
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .truncate(false)
            .create(true)
            .open(&self.path)?;
        let reader = BufReader::new(file);
        let contacts = helper::deserialize_contacts_from_txt_buffer(reader)?;
        Ok(contacts)
    }

    fn save(&self, contacts: &[Contact]) -> Result<(), AppError> {
        let mut file = OpenOptions::new()
            .read(true)
            .write(true) // WRITE to file on save
            .truncate(true)
            .create(true)
            .open(&self.path)?;

        let data = helper::serialize_contacts(contacts);
        file.write_all(data.as_bytes())?;

        Ok(())
    }
}

impl ContactStore for MemStore {
    fn load(&self) -> Result<Vec<Contact>, AppError> {
        Ok(self.data.clone())
    }

    fn save(&self, _contacts: &[Contact]) -> Result<(), AppError> {
        Ok(())
    }
}

impl ContactStore for JsonStore {
    fn load(&self) -> Result<Vec<Contact>, AppError> {
        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .truncate(false)
            .create(true)
            .open(&self.path)?;

        let mut data = String::new();
        file.read_to_string(&mut data)?;

        Ok(serde_json::from_str(&data)?)
    }

    fn save(&self, contacts: &[Contact]) -> Result<(), AppError> {
        let json_contact = serde_json::to_string(&contacts)?;

        let path = Path::new(&self.path);
        if !path.exists() {
            create_file_parent(&self.path)?;
            let _file = OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .open(path)?;
        }

        let mut file = OpenOptions::new().write(true).truncate(true).open(path)?;

        file.write_all(json_contact.as_bytes())?;
        Ok(())
    }
}

pub fn create_file_parent(path: &str) -> Result<(), AppError> {
    let path = Path::new(path);

    if let Some(parent) = path.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent)?;
        }
    }
    Ok(())
}

pub fn load_migrated_contact(storage: &Storage) -> Result<Vec<Contact>, AppError> {
    let txt_contacts = storage.load_txt()?;
    let json_contacts = storage.load_json()?;

    if txt_contacts.is_empty() && storage.storage_choice.is_json() {
        fs::remove_file(Path::new(&storage.file_store.path))?;
    } else if json_contacts.is_empty() && storage.storage_choice.is_txt() {
        fs::remove_file(Path::new(&storage.json_store.path))?;
    }

    let mut migrated_contact = json_contacts;
    migrated_contact.extend(txt_contacts);

    migrated_contact.sort();
    migrated_contact.dedup();

    Ok(migrated_contact)
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn migrates_contact() -> Result<(), AppError> {
        let mut storage = Storage::new()?;
        storage.mem_store.data.clear();

        let contact1 = Contact {
            name: "Uche".to_string(),
            phone: "01234567890".to_string(),
            email: "ucheuche@gmail.com".to_string(),
        };

        let contact2 = Contact {
            name: "Alex".to_string(),
            phone: "+44731484372".to_string(),
            email: "".to_string(),
        };

        storage.add_contact(contact1);
        storage.file_store.save(&storage.mem_store.data)?;
        storage.mem_store.data.clear();

        storage.add_contact(contact2);
        storage.save_json(&storage.mem_store.data)?;
        storage.mem_store.data.clear();

        storage.mem_store.data = load_migrated_contact(&storage)?;

        assert!(storage.mem_store.data.len() == 2);

        assert_eq!(
            storage.contact_list()[1], Contact {
                name: "Uche".to_string(),
                phone: "01234567890".to_string(),
                email: "ucheuche@gmail.com".to_string(),
            }
        );
            
        assert_eq!(
            storage.contact_list()[0], Contact {
                name: "Alex".to_string(),
                phone: "+44731484372".to_string(),
                email: "".to_string(),
            }
        );

        storage.mem_store.data.clear();
        storage.save()?;

        Ok(())
    }
}
