pub mod memory;
pub mod storage_port;
pub mod filestore;

use crate::prelude::{AppError, Contact, uuid::Uuid, HashMap};
use dotenv::dotenv;
use std::fs::{self, OpenOptions};
use std::io::{BufReader, Read, Write};
use std::path::Path;

pub trait ContactStore {
    fn load(&self) -> Result<HashMap<Uuid, Contact>, AppError>;

    fn save(&self, contacts: &HashMap<Uuid, Contact>) -> Result<(), AppError>;

}


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
