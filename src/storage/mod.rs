pub mod memory;
pub mod stores;

use crate::prelude::{AppError, Contact, HashMap, uuid::Uuid};
use dotenv::dotenv;
use std::fs::{self, OpenOptions};
use std::io::{BufReader, Read, Write};
use std::path::Path;

pub trait ContactStore {
    fn load(&self) -> Result<HashMap<Uuid, Contact>, AppError>;

    fn save(&self, contacts: &HashMap<Uuid, Contact>) -> Result<(), AppError>;

    fn get_medium(&self) -> &str;
}

#[derive(Debug)]
pub enum StorageMediums {
    Txt,
    Json,
}

impl StorageMediums {
    pub fn is_json(&self) -> bool {
        matches!(self, StorageMediums::Json)
    }

    pub fn is_txt(&self) -> bool {
        matches!(self, StorageMediums::Txt)
    }

    pub fn is_which(&self) -> &str {
        if self.is_json() { "json" } else { "txt" }
    }
    pub fn from(str: &str) -> Result<Self, AppError> {
        match str {
            "json" => Ok(StorageMediums::Json),
            "txt" => Ok(StorageMediums::Txt),
            _ => Err(AppError::Validation(
                "Not a recognized storage medium".to_string(),
            )),
        }
    }
}

pub fn parse_storage_type(
    storage_medium: Option<StorageMediums>,
) -> Result<Box<dyn ContactStore>, AppError> {
    let medium: StorageMediums;
    if let Some(storage_medium) = storage_medium {
        medium = storage_medium;
    } else {
        dotenv().ok();

        let choice = std::env::var("STORAGE_CHOICE").unwrap_or("json".to_string());
        medium = StorageMediums::from(&choice)?;
    }

    match medium {
        StorageMediums::Json => Ok(Box::new(stores::JsonStorage::new()?)),
        StorageMediums::Txt => Ok(Box::new(stores::TxtStorage::new()?)),
    }
}

pub fn create_file_parent(path: &str) -> Result<(), AppError> {
    let path = Path::new(path);

    if let Some(parent) = path.parent()
        && !parent.exists()
    {
        fs::create_dir_all(parent)?;
    }
    Ok(())
}
