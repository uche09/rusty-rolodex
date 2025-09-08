use crate::domain::Contact;
use crate::errors::AppError;
use crate::helper;
use std::fs::{self, OpenOptions};
use std::io::{BufReader, Write};
use std::path::Path;

// pub const FILE_PATH: &str = "./.instance/contacts.txt";

pub struct FileStore {
    path: String,
}

pub struct MemStore {
    pub data: Vec<Contact>,
}

impl FileStore {
    pub fn new(path: &str) -> Result<Self, AppError> {
        FileStore::create_file_parent(path)?;

        Ok(FileStore {
            path: path.to_string(),
        })
    }

    fn create_file_parent(path: &str) -> Result<(), AppError> {
        let path = Path::new(path);

        if let Some(parent) = path.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent)?;
            }
        }
        Ok(())
    }
}

pub trait ContactStore {
    fn load(&self) -> Result<Vec<Contact>, AppError>;

    fn save(&self, contacts: &Vec<Contact>) -> Result<(), AppError>;
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

    fn save(&self, contacts: &Vec<Contact>) -> Result<(), AppError> {
        let mut file = OpenOptions::new()
            .write(true) // WRITE to file on save
            .truncate(true)
            .open(&self.path)?;

        let data = helper::serialize_contacts(contacts);
        file.write_all(data.as_bytes())?;

        Ok(())
    }
}

impl MemStore {
    pub fn new() -> Self {
        Self { data: Vec::new() }
    }
}

impl ContactStore for MemStore {
    fn load(&self) -> Result<Vec<Contact>, AppError> {
        Ok(self.data.clone())
    }

    fn save(&self, _contacts: &Vec<Contact>) -> Result<(), AppError> {
        Ok(())
    }
}
