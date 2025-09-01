use crate::domain::Contact;
use crate::errors::AppError;
use crate::helper;
use std::fs::{self, File, OpenOptions};
use std::io::{BufReader, Write};
use std::path::Path;

pub const FILE_PATH: &str = "./.instance/contacts.txt";

#[allow(unused)]
pub struct Store {
    pub mem: Vec<Contact>,
    pub file: File,
}

impl Store {
    pub fn new() -> Result<Self, AppError> {
        Store::create_file_parent()?;

        // Now using OpenOptions to open file if already exist
        // Or create one
        let file = OpenOptions::new()
        .read(true)   // READ from file during instanciation
        .write(true)
        .create(true)
        .open(FILE_PATH)?;
        Ok(
            Store {
                mem: Vec::new(),
                file,
            }
        )
    }

    fn create_file_parent() -> Result<(), AppError> {
        let path = Path::new(FILE_PATH);

        if let Some(parent) = path.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent)?;
            }
        }
        Ok(())
    }
}

pub trait ContactStore {
    fn load(&mut self) -> Result<(), AppError>;

    fn save(&mut self) -> Result<(), AppError>;
}

impl ContactStore for Store {
    fn load(&mut self) -> Result<(), AppError> {
        // Read text from file
        let file = File::open(FILE_PATH)?;
        let reader = BufReader::new(file);
        let contacts = helper::deserialize_contacts_from_txt_buffer(reader)?;
        self.mem = contacts;
        Ok(())
    }

    fn save(&mut self) -> Result<(), AppError> {
        self.file = OpenOptions::new()
        .write(true)   // WRITE to file on save
        .truncate(true)
        .open(FILE_PATH)?;
    
        let data = helper::serialize_contacts(&self.mem);
        self.file.write_all(data.as_bytes())?;

        Ok(())
    }
}
