use std::path::PathBuf;

use super::*;
use csv::Writer;

const EXPORT_PATH: &str = "./import_export/exported.csv";

pub fn export_contacts_to_csv(
    contacts: &Vec<Contact>,
    des: Option<&str>,
) -> Result<(PathBuf, u64), AppError> {
    let mut file_path = PathBuf::from(EXPORT_PATH);

    if let Some(path) = des {
        file_path = PathBuf::from(path);

        if file_path.is_dir() || file_path.extension().is_some_and(|ext| ext != "csv") {
            if file_path.is_dir() {
                file_path = file_path.join("exported.csv");
            } else {
                return Err(AppError::Validation(
                    "Export file must be a .csv file".to_string(),
                ));
            }
        }
    }

    if !file_path.exists() {
        let _file = fs::OpenOptions::new()
            .write(true)
            .truncate(true)
            .create(true)
            .open(&file_path)?;
    }

    let mut writer = Writer::from_path(&file_path)?;

    let mut counter: u64 = 0;

    for contact in contacts {
        writer.serialize(contact)?;
        counter += 1;
    }

    writer.flush()?;

    Ok((file_path, counter))
}
