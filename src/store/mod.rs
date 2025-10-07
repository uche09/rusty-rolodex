pub mod json;
pub mod memory;
pub mod txt;

use crate::prelude::{AppError, Contact};
use crate::store::json::JsonStore;
use crate::store::txt::TxtStore;
use dotenv::dotenv;
use std::fs::{self, OpenOptions};
use std::io::{BufReader, Read, Write};
use std::path::Path;

pub trait ContactStore {
    fn load(&self) -> Result<Vec<Contact>, AppError>;

    fn save(&self, contacts: &[Contact]) -> Result<(), AppError>;
    fn load_migrated_contact(&mut self) -> Result<(), AppError>;
}

pub trait Store {
    type Item;

    fn get_mem(&self) -> &Vec<Self::Item>;
    fn mut_mem(&mut self) -> &mut Vec<Self::Item>;
    fn contact_list(&self) -> Vec<&Contact>;
    fn get_indices_by_name(&self, name: &str) -> Option<Vec<usize>>;

    fn add_contact(&mut self, contact: Self::Item) {
        self.mut_mem().push(contact);
    }

    fn delete_contact(&mut self, index: usize) -> Result<(), AppError> {
        let mem = self.mut_mem();
        if index < mem.len() {
            mem.remove(index);
            Ok(())
        } else {
            Err(AppError::NotFound("Contact".to_string()))
        }
    }
}

pub trait Stoarage: Store + ContactStore {}
impl<T: Store + ContactStore> Stoarage for T {}

#[derive(Debug)]
pub enum StoreChoice {
    Txt,
    Json,
}

impl StoreChoice {
    pub fn is_json(&self) -> bool {
        matches!(self, StoreChoice::Json)
    }

    pub fn is_txt(&self) -> bool {
        matches!(self, StoreChoice::Txt)
    }

    pub fn is_which(&self) -> &str {
        if self.is_json() { "json" } else { "txt" }
    }
}

pub fn parse_storage_choice() -> StoreChoice {
    dotenv().ok();

    let choice = std::env::var("STORAGE_CHOICE").unwrap_or("json".to_string());
    match choice.to_lowercase().as_str() {
        "json" => StoreChoice::Json,
        _ => StoreChoice::Txt,
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

pub fn parse_store() -> Result<Box<dyn Stoarage<Item = Contact>>, AppError> {
    let store_choice = parse_storage_choice();

    match store_choice {
        StoreChoice::Json => Ok(Box::new(JsonStore::new()?)),
        StoreChoice::Txt => Ok(Box::new(TxtStore::new()?)),
    }
}
