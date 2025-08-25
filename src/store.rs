use crate::domain::Contact;
use std::fs::{self, File};
use std::path::Path;

pub const FILE_PATH: &str = "./.instance/contacts.json";

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
