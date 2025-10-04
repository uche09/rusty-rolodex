use super::*;
use crate::helper;
use std::fs::{self, OpenOptions};
use std::io::{BufReader, Read, Write};
use std::path::Path;

pub struct TxtStore {
    pub path: String,
}

pub struct JsonStore {
    pub path: String,
}

impl TxtStore {
    pub fn new(path: &str) -> Result<Self, AppError> {
        create_file_parent(path)?;

        Ok(TxtStore {
            path: path.to_string(),
        })
    }
}

impl JsonStore {
    pub fn new(path: &str) -> Result<Self, AppError> {
        create_file_parent(path)?;

        Ok(Self {
            path: path.to_string(),
        })
    }
}

impl ContactStore for TxtStore {
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

        // serde_json will give an error if data is empty
        if data.is_empty() {
            return Ok(Vec::new());
        }

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
    let mut txt_contacts: Vec<Contact> = Vec::new();
    let mut json_contacts: Vec<Contact> = Vec::new();

    if fs::exists(Path::new(&storage.txt_store.path))? {
        txt_contacts = storage.load_txt()?;
    }

    if fs::exists(Path::new(&storage.json_store.path))? {
        json_contacts = storage.load_json()?;
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

        let contact1 = Contact::new(
            "Uche".to_string(),
            "01234567890".to_string(),
            "ucheuche@gmail.com".to_string(),
            "".to_string(),
        );

        let contact2 = Contact::new(
            "Alex".to_string(),
            "+44731484372".to_string(),
            "".to_string(),
            "".to_string(),
        );

        storage.add_contact(contact1);
        storage.txt_store.save(&storage.mem_store.data)?;
        storage.mem_store.data.clear();

        storage.add_contact(contact2);
        storage.save_json(&storage.mem_store.data)?;
        storage.mem_store.data.clear();

        storage.mem_store.data = load_migrated_contact(&storage)?;

        assert!(storage.mem_store.data.len() == 2);

        assert_eq!(
            *storage.contact_list()[1],
            Contact::new(
                "Uche".to_string(),
                "01234567890".to_string(),
                "ucheuche@gmail.com".to_string(),
                "".to_string(),
            )
        );

        assert_eq!(
            *storage.contact_list()[0],
            Contact::new(
                "Alex".to_string(),
                "+44731484372".to_string(),
                "".to_string(),
                "".to_string(),
            )
        );

        storage.mem_store.data.clear();
        storage.save()?;

        Ok(())
    }
}
