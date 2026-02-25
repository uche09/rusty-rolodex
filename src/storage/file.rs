use super::*;

use csv::{Reader, Writer};
use serde_json::Value;

pub struct JsonStorage {
    pub medium: String,
    pub path: String,
}

impl JsonStorage {
    pub fn new() -> Result<Self, AppError> {
        Ok(Self {
            medium: "json".to_string(),
            path: env::var("JSON_STORAGE_PATH").unwrap_or("./.instance/contacts.json".to_string()),
        })
    }
}

pub struct TxtStorage {
    pub medium: String,
    pub path: String,
}

impl TxtStorage {
    pub fn new() -> Result<Self, AppError> {
        Ok(Self {
            medium: "txt".to_string(),
            path: env::var("TXT_STORAGE_PATH").unwrap_or("./.instance/contacts.txt".to_string()),
        })
    }
}

pub struct CsvStorage {
    pub medium: String,
    pub path: String,
}

impl CsvStorage {
    pub fn new(path: &str) -> Result<Self, AppError> {
        let mut path = path;
        let mut file_path = PathBuf::from(path);

        if file_path.is_dir() || file_path.extension().is_some_and(|ext| ext != "csv") {
            if file_path.is_dir() {
                file_path = file_path.join("exported.csv");
                path = file_path.to_str().unwrap();
            } else {
                return Err(AppError::Validation(
                    "Export file must be a .csv file".to_string(),
                ));
            }
        }

        Ok(Self {
            medium: "csv".to_string(),
            path: if !(path.is_empty()) {
                path.to_string()
            } else {
                ("./csv/contacts.csv").to_string()
            },
        })
    }
}

impl ContactStore for JsonStorage {
    fn load(&self) -> Result<HashMap<Uuid, Contact>, AppError> {
        if !fs::exists(Path::new(&self.path))? {
            return Ok(HashMap::new());
        }
        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .truncate(false)
            .create(true)
            .open(&self.path)?;

        let mut data = String::new();
        file.read_to_string(&mut data)?;

        // serde_json will give an error if data is empty
        if data.is_empty() {
            return Ok(HashMap::new());
        }

        let value: Value = serde_json::from_str(&data)?;

        // New Format: Contacts are now stored in HashMap.
        // Try if new format has been effected
        if value.is_object() {
            let contacts: HashMap<Uuid, Contact> = serde_json::from_value(value)?;
            Ok(contacts)
        } else if value.is_array() {
            // Old Format: Contacts were stored in Vec
            let contacts: Vec<Contact> = serde_json::from_value(value)?;

            // Convert Vec to HashMap for new feature backward compatibility
            let mapped_contacts = contacts
                .into_iter()
                .map(|cont| (cont.id, cont))
                .collect::<HashMap<Uuid, Contact>>();
            Ok(mapped_contacts)
        } else {
            Err(AppError::Validation(
                "Invalid JSON structure: expected object or array".to_string(),
            ))
        }
    }

    fn save(&self, contacts: &HashMap<Uuid, Contact>) -> Result<(), AppError> {
        let path = Path::new(&self.path);
        if !path.exists() {
            create_file_parent(&self.path)?;
            // let _file = OpenOptions::new()
            //     .write(true)
            //     .create(true)
            //     .truncate(true)
            //     .open(path)?;
        }

        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(path)?;

        // user serde to serialize json data
        let json_contact = serde_json::to_string(&contacts)?;
        file.write_all(json_contact.as_bytes())?;

        let txt_path =
            env::var("TXT_STORAGE_PATH").unwrap_or("./.instance/contacts.txt".to_string());
        let txt_path = Path::new(&txt_path);
        if fs::exists(txt_path)? {
            fs::remove_file(txt_path)?;
        }

        Ok(())
    }

    fn get_medium(&self) -> &str {
        &self.medium
    }
}

impl ContactStore for TxtStorage {
    fn load(&self) -> Result<HashMap<Uuid, Contact>, AppError> {
        if !fs::exists(Path::new(&self.path))? {
            return Ok(HashMap::new());
        }
        // Read text fom file
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

    fn save(&self, contacts: &HashMap<Uuid, Contact>) -> Result<(), AppError> {
        let path = Path::new(&self.path);
        if !path.exists() {
            create_file_parent(&self.path)?;
            // let _file = OpenOptions::new()
            //     .write(true)
            //     .create(true)
            //     .truncate(true)
            //     .open(path)?;
        }

        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(path)?;

        // use our helper to serialize data for txt file
        let data = helper::serialize_contacts(contacts);
        file.write_all(data.as_bytes())?;

        let json_path =
            env::var("JSON_STORAGE_PATH").unwrap_or("./.instance/contacts.json".to_string());
        let json_path = Path::new(&json_path);
        if fs::exists(json_path)? {
            fs::remove_file(json_path)?;
        }
        Ok(())
    }

    fn get_medium(&self) -> &str {
        &self.medium
    }
}

impl ContactStore for CsvStorage {
    fn get_medium(&self) -> &str {
        &self.medium
    }

    fn load(&self) -> Result<HashMap<Uuid, Contact>, AppError> {
        let file_path: PathBuf = PathBuf::from(&self.path);

        if !file_path.exists() {
            return Err(AppError::NotFound("CSV file".to_string()));
        }

        if file_path.extension().is_some_and(|ext| ext != "csv") {
            return Err(AppError::Validation("File not .csv".to_string()));
        }

        let mut reader = Reader::from_path(&file_path)?;

        let mut contacts: HashMap<Uuid, Contact> = HashMap::new();

        for result in reader.deserialize() {
            let record: Contact = result?;
            contacts.insert(record.id, record);
        }

        Ok(contacts)
    }

    fn save(&self, contacts: &HashMap<Uuid, Contact>) -> Result<(), AppError> {
        let file_path = PathBuf::from(&self.path);

        if !file_path.exists() {
            let _file = fs::OpenOptions::new()
                .write(true)
                .truncate(true)
                .create(true)
                .open(&file_path)?;
        }

        let mut writer = Writer::from_path(&file_path)?;

        for contact in contacts.values() {
            writer.serialize(contact)?;
        }

        writer.flush()?;

        Ok(())
    }
}

pub fn load_txt_contacts(path: &str) -> Result<HashMap<Uuid, Contact>, AppError> {
    if !fs::exists(Path::new(path))? {
        return Ok(HashMap::new());
    }
    // Read text fom file
    // Using OpenOptions to open file if already exist
    // Or create one
    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .truncate(false)
        .create(true)
        .open(path)?;
    let reader = BufReader::new(file);
    let contacts = helper::deserialize_contacts_from_txt_buffer(reader)?;
    Ok(contacts)
}

pub fn load_json_contacts(path: &str) -> Result<HashMap<Uuid, Contact>, AppError> {
    if !fs::exists(Path::new(path))? {
        return Ok(HashMap::new());
    }
    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .truncate(false)
        .create(true)
        .open(path)?;

    let mut data = String::new();
    file.read_to_string(&mut data)?;

    // serde_json will give an error if data is empty
    if data.is_empty() {
        return Ok(HashMap::new());
    }

    let value: Value = serde_json::from_str(&data)?;

    // New Format: Contacts are now stored in HashMap.
    // Try if new format has been effected
    if value.is_object() {
        let contacts: HashMap<Uuid, Contact> = serde_json::from_value(value)?;
        Ok(contacts)
    } else if value.is_array() {
        // Old Format: Contacts were stored in Vec
        let contacts: Vec<Contact> = serde_json::from_value(value)?;

        // Convert Vec to HashMap for new feature backward compatibility
        let mapped_contacts = contacts
            .into_iter()
            .map(|cont| (cont.id, cont))
            .collect::<HashMap<Uuid, Contact>>();
        Ok(mapped_contacts)
    } else {
        Err(AppError::Validation(
            "Invalid JSON structure: expected object or array".to_string(),
        ))
    }
}
