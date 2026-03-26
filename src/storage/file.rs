use super::*;

use csv::{Reader, Writer};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::{fs as tokio_fs, io::AsyncWriteExt};

/// # TOML wrapper struct.
///
/// We are using TOML to Serialize/Deserialize into .txt file
/// while retaining readable file format.
///
/// TOML require a **table at the root of a TOML document**
/// for proper formating. The table is the `contacts` field.
#[derive(Serialize, Deserialize)]
struct TomlContacts {
    contacts: HashMap<Uuid, Contact>,
}

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

#[async_trait(?Send)]
impl ContactStore for JsonStorage {
    async fn load(&self) -> Result<HashMap<Uuid, Contact>, AppError> {
        let data = read_file(&self.path).await?;

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

    async fn save(&self, contacts: &HashMap<Uuid, Contact>) -> Result<(), AppError> {
        // user serde to serialize json data
        let json_contact = serde_json::to_string(&contacts)?;
        write_file(&self.path, &json_contact).await?;

        let txt_path =
            env::var("TXT_STORAGE_PATH").unwrap_or("./.instance/contacts.txt".to_string());
        let txt_path = Path::new(&txt_path);
        if tokio_fs::try_exists(txt_path).await? {
            tokio_fs::remove_file(txt_path).await?;
        }

        Ok(())
    }

    fn get_medium(&self) -> &str {
        &self.medium
    }
}

#[async_trait(?Send)]
impl ContactStore for TxtStorage {
    async fn load(&self) -> Result<HashMap<Uuid, Contact>, AppError> {
        let data = read_file(&self.path).await?;
        let toml_contacts: Result<TomlContacts, toml::de::Error> = toml::from_str(&data);

        if let Ok(toml_contacts) = toml_contacts {
            Ok(toml_contacts.contacts)
        } else {
            Ok(helper::deserialize_contacts_from_txt_buffer(data)?)
        }
    }

    async fn save(&self, contacts: &HashMap<Uuid, Contact>) -> Result<(), AppError> {
        let toml_contacts = TomlContacts {
            contacts: contacts.clone(),
        };
        let string_data = toml::to_string(&toml_contacts)?;

        // // use our helper to serialize data for txt file
        // let data = helper::serialize_contacts(contacts);

        write_file(&self.path, &string_data).await?;

        let json_path =
            env::var("JSON_STORAGE_PATH").unwrap_or("./.instance/contacts.json".to_string());
        let json_path = Path::new(&json_path);
        if tokio_fs::try_exists(json_path).await? {
            tokio_fs::remove_file(json_path).await?;
        }

        Ok(())
    }

    fn get_medium(&self) -> &str {
        &self.medium
    }
}

#[async_trait(?Send)]
impl ContactStore for CsvStorage {
    fn get_medium(&self) -> &str {
        &self.medium
    }

    async fn load(&self) -> Result<HashMap<Uuid, Contact>, AppError> {
        let file_path: PathBuf = PathBuf::from(&self.path);

        if !file_path.exists() {
            return Err(AppError::NotFound("CSV file".to_string()));
        }

        if file_path.extension().is_some_and(|ext| ext != "csv") {
            return Err(AppError::Validation("File not .csv".to_string()));
        }

        let data = read_file(&self.path).await?;

        let mut reader = Reader::from_reader(data.as_bytes());

        let mut contacts: HashMap<Uuid, Contact> = HashMap::new();

        for result in reader.deserialize() {
            let record: Contact = result?;
            contacts.insert(record.id, record);
        }

        Ok(contacts)
    }

    async fn save(&self, contacts: &HashMap<Uuid, Contact>) -> Result<(), AppError> {
        let mut writer = Writer::from_writer(vec![]);

        for contact in contacts.values() {
            writer.serialize(contact)?;
        }

        let data = writer
            .into_inner()
            .map_err(|e| AppError::Io(e.into_error()))?;
        let csv_string = str::from_utf8(&data).map_err(|e| AppError::Validation(e.to_string()))?;

        write_file(&self.path, csv_string).await?;

        Ok(())
    }
}

async fn read_file(path: &str) -> Result<String, AppError> {
    if !tokio_fs::try_exists(Path::new(path)).await? {
        return Ok(String::new());
    }
    let data = tokio_fs::read_to_string(path).await?;

    Ok(data)
}

async fn write_file(path_str: &str, data: &str) -> Result<(), AppError> {
    let path = Path::new(path_str);
    if !path.exists() {
        create_file_parent(path_str)?;
    }

    let mut file = tokio_fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(path)
        .await?;

    file.write_all(data.as_bytes()).await?;
    file.flush().await?;
    Ok(())
}
