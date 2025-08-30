use crate::domain::Contact;
use crate::helper;
use std::fs::{self, File};
use std::io::{self, BufReader, Write};
use std::path::Path;

pub const FILE_PATH: &str = "./.instance/contacts.txt";

#[allow(unused)]
pub struct Store {
    pub mem: Vec<Contact>,
    pub file: File,
}

impl Store {
    pub fn new() -> Self {
        Store::create_file_parent();
        let file = File::create(FILE_PATH).unwrap();
        Store {
            mem: Vec::new(),
            file,
        }
    }

    fn create_file_parent() {
        let path = Path::new(FILE_PATH);

        if let Some(parent) = path.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent).unwrap();
            }
        }
    }
}

pub trait ContactStore {
    fn load(&mut self) -> io::Result<()>;

    fn save(&mut self) -> io::Result<()>;
}

impl ContactStore for Store {
    fn load(&mut self) -> io::Result<()> {
        // Read text from file
        let file = File::open(FILE_PATH)?;
        let reader = BufReader::new(file);
        let contacts = helper::deserialize_contacts_from_txt_buffer(reader)?;
        self.mem = contacts;
        Ok(())
    }

    fn save(&mut self) -> io::Result<()> {
        let data = helper::serialize_contacts(&self.mem);
        self.file.write_all(data.as_bytes())?;

        Ok(())
    }
}
