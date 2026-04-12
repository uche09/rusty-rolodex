pub mod file;
mod lock;
pub mod memory;
pub mod remote;

use crate::domain::Contact;
use crate::errors::AppError;
use crate::helper;
use async_trait::async_trait;
pub use std::collections::HashMap;
use std::fs;
use std::{
    env,
    path::{Path, PathBuf},
};
use uuid::Uuid;

#[async_trait(?Send)]
pub trait ContactStore: Send + Sync {
    async fn load(&self) -> Result<HashMap<Uuid, Contact>, AppError>;

    async fn save(&self, contacts: &HashMap<Uuid, Contact>) -> Result<(), AppError>;

    fn get_medium(&self) -> &str;
}

#[derive(Debug)]
pub enum StorageMediums {
    Csv,
    Txt,
    Json,
    Remote,
}

impl StorageMediums {
    pub fn is_json(&self) -> bool {
        matches!(self, StorageMediums::Json)
    }

    pub fn is_txt(&self) -> bool {
        matches!(self, StorageMediums::Txt)
    }

    pub fn is_remote(&self) -> bool {
        matches!(self, StorageMediums::Remote)
    }

    pub fn is_which(&self) -> &str {
        if self.is_json() {
            "json"
        } else if self.is_txt() {
            "txt"
        } else {
            "remote"
        }
    }
}

impl TryFrom<&str> for StorageMediums {
    type Error = AppError;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "csv" => Ok(StorageMediums::Csv),
            "json" => Ok(StorageMediums::Json),
            "txt" => Ok(StorageMediums::Txt),
            "remote" => Ok(StorageMediums::Remote),
            _ => Err(AppError::Validation(
                "Not a recognized storage medium".to_string(),
            )),
        }
    }
}

pub fn parse_storage_type_env_config(
    storage_medium: Option<StorageMediums>,
) -> Result<Box<dyn ContactStore>, AppError> {
    let medium: StorageMediums;
    if let Some(storage_medium) = storage_medium {
        medium = storage_medium;
    } else {
        let choice = helper::get_env_value_by_key("STORAGE_CHOICE").unwrap_or("json".to_string());
        medium = choice.as_str().try_into()?;
    }

    match medium {
        StorageMediums::Json => Ok(Box::new(file::JsonStorage::new()?)),
        StorageMediums::Csv => Ok(Box::new(file::CsvStorage::new("")?)),
        StorageMediums::Txt => Ok(Box::new(file::TxtStorage::new()?)),
        StorageMediums::Remote => Ok(Box::new(remote::RemoteStorage::new()?)),
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
