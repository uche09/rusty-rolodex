use crate::prelude::Store;
use std::path::PathBuf;

use super::*;
use csv::Reader;

const IMPORT_PATH: &str = "./import_export/contacts.csv";

pub fn import_contacts_from_csv(src: Option<&str>) -> Result<(PathBuf, u64), AppError> {
    let mut file_path: PathBuf = PathBuf::from(IMPORT_PATH);

    if let Some(path) = src {
        file_path = PathBuf::from(path);
    }

    if !file_path.exists() {
        return Err(AppError::NotFound("CSV file".to_string()));
    }

    if file_path.extension().is_some_and(|ext| ext != "csv") {
        return Err(AppError::Validation("File not .csv".to_string()));
    }

    let mut reader = Reader::from_path(&file_path)?;

    let mut storage = Store::new()?;

    let mut counter: u64 = 0;
    for result in reader.deserialize() {
        let record: Contact = result?;
        storage.add_contact(record);
        counter += 1;
    }

    storage.save(&storage.mem)?;

    Ok((file_path, counter))
}
