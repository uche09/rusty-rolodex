use super::*;

use csv::{Reader, Writer};
use lock::FileLock;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::{fs as tokio_fs, io::AsyncWriteExt, task};

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
        let mut default_dir = resolve_storage_dir();
        default_dir.push_str("contacts.json");
        helper::set_env_value_in_file("JSON_STORAGE_PATH", &default_dir)?;

        Ok(Self {
            medium: "json".to_string(),
            path: env::var("JSON_STORAGE_PATH").unwrap_or(default_dir),
        })
    }
}

pub struct TxtStorage {
    pub medium: String,
    pub path: String,
}

impl TxtStorage {
    pub fn new() -> Result<Self, AppError> {
        let mut default_dir = resolve_storage_dir();
        default_dir.push_str("contacts.txt");
        helper::set_env_value_in_file("TXT_STORAGE_PATH", &default_dir)?;

        Ok(Self {
            medium: "txt".to_string(),
            path: env::var("TXT_STORAGE_PATH").unwrap_or(default_dir),
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
                let mut default_dir = resolve_storage_dir();
                default_dir.push_str("csv/contacts.csv");

                default_dir
            },
        })
    }
}

#[async_trait(?Send)]
impl ContactStore for JsonStorage {
    async fn load(&self) -> Result<HashMap<Uuid, Contact>, AppError> {
        let data = {
            let storage_path = self.path.clone();

            // Locking storage file for shared access coodination
            let _shared_lock = task::spawn_blocking(move || FileLock::shared(&storage_path))
                .await
                .map_err(|e| AppError::Poison(format!("FileLock task panicked: {}", e)))??;

            read_file(&self.path).await?
            // Drop FileLock instance ASAP to free lock
        };

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
        {
            // user serde to serialize json data
            let json_contact = serde_json::to_string(&contacts)?;
            let storage_path = self.path.clone();

            // Locking storage file for exclusive access coodination
            let _xlock = task::spawn_blocking(move || FileLock::exclusive(&storage_path))
                .await
                .map_err(|e| AppError::Poison(format!("FileLock task panicked: {}", e)))??;

            write_file(&self.path, &json_contact).await?;
            // Drop FileLock instance ASAP to free lock
        }

        let txt_path = env::var("TXT_STORAGE_PATH").unwrap_or_else(|_| {
            let mut path = resolve_storage_dir();
            path.push_str("contacts.txt");
            path
        });
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
        let data = {
            let storage_path = self.path.clone();

            // Locking storage file for shared access coodination
            let _shared_lock = task::spawn_blocking(move || FileLock::shared(&storage_path))
                .await
                .map_err(|e| AppError::Poison(format!("FileLock task panicked: {}", e)))??;

            read_file(&self.path).await?
            // Drop FileLock instance ASAP to free lock
        };

        let toml_contacts: Result<TomlContacts, toml::de::Error> = toml::from_str(&data);

        if let Ok(toml_contacts) = toml_contacts {
            Ok(toml_contacts.contacts)
        } else {
            Ok(helper::deserialize_contacts_from_txt_buffer(data)?)
        }
    }

    async fn save(&self, contacts: &HashMap<Uuid, Contact>) -> Result<(), AppError> {
        {
            let toml_contacts = TomlContacts {
                contacts: contacts.clone(),
            };
            let string_data = toml::to_string(&toml_contacts)?;

            // // use our helper to serialize data for txt file
            // let data = helper::serialize_contacts(contacts);

            let storage_path = self.path.clone();

            // Locking storage file for exclusive access coodination
            let _xlock = task::spawn_blocking(move || FileLock::exclusive(&storage_path))
                .await
                .map_err(|e| AppError::Poison(format!("FileLock task panicked: {}", e)))??;

            write_file(&self.path, &string_data).await?;
            // Drop FileLock instance ASAP to free lock
        }

        let json_path = env::var("JSON_STORAGE_PATH").unwrap_or_else(|_| {
            let mut path = resolve_storage_dir();
            path.push_str("contacts.json");
            path
        });
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

pub fn resolve_storage_dir() -> String {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let workspace_root = std::path::Path::new(manifest_dir).parent().unwrap();
    workspace_root
        .join(".instance/")
        .to_string_lossy()
        .to_string()
}
