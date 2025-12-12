use super::*;

use rust_fuzzy_search::fuzzy_compare;
use std::{collections::{HashMap, HashSet}, sync::{Arc, Mutex}, thread};
use crate::helper;

pub const JSON_STORAGE_PATH: &str = "./.instance/contacts.json";
pub const TXT_STORAGE_PATH: &str = "./.instance/contacts.txt";


pub struct Store<'a> {
    pub mem: HashMap<Uuid, Contact>,
    pub path: &'a str,
    pub index: Index,
}

// pub struct JsonIter<'a> {
//     inner: &'a [Contact],
//     idx: usize,
// }


// TODO: use string (jack) as name index key rather than char (j)
#[derive(Debug)]
pub struct Index {
    pub name: HashMap<String, HashSet<Uuid>>,
    pub domain: HashMap<String, HashSet<Uuid>>,
}

impl Index {
    pub fn new(storage: &Store) -> Result<Self, AppError> {
        Ok(
            Self {
                name: storage.create_name_search_index()?,
                domain: storage.create_email_domain_search_index()?,
            }
        )
    }

    pub fn increment_index(&mut self, contact: &Contact) {
        let name = &contact.name;

        if !name.is_empty(){
            let names = name.split_ascii_whitespace();
            
            for name_slice in names {
                self.name.entry(name_slice.to_lowercase())
                    .or_default()
                    .insert(contact.id.clone());
            }

        }

        let email_parts: Vec<&str> = contact.email.split('@').collect();
        let domain = email_parts[email_parts.len() -1].to_string();

        self.domain.entry(domain.to_lowercase())
            .or_default()
            .insert(contact.id.clone());
    }

    pub fn decrement_index(&mut self, contact: &Contact) {
        let name = &contact.name;

        if !name.is_empty() {
            let names = name.split_ascii_whitespace();

            for name_slice in names {
                if let Some(indices) = self.name.get_mut(&name_slice.to_lowercase()) {
                    indices.retain(|&i| i != contact.id);
                }
            }

        }

        let email_parts: Vec<&str> = contact.email.split('@').collect();
        let domain = email_parts[email_parts.len() -1].to_string();

        if let Some(indices) = self.domain.get_mut(&domain.to_ascii_lowercase()) {
            indices.retain(|&i| i != contact.id);
        }
    }
}


impl Store<'_> {
    pub fn new() -> Result<Self, AppError> {
        let path = match parse_storage_choice() {
            StoreChoice::Json => JSON_STORAGE_PATH,
            StoreChoice::Txt => TXT_STORAGE_PATH,
        };
        create_file_parent(path)?;

        Ok(Self {
            mem: HashMap::new(),
            path,
            index: Index {
                name: HashMap::new(),
                domain: HashMap::new(),
            },
        })
    }

    // pub fn iter(&self) -> JsonIter<'_> {
    //     JsonIter {
    //         inner: &self.mem,
    //         idx: 0,
    //     }
    // }


    pub fn contact_list(&self) -> Vec<&Contact> {
        self.mem.iter().map(|(_, cont)| cont).collect::<Vec<&Contact>>() 
    }

    
    pub fn get_ids_by_name(&self, name: &str) -> Option<Vec<Uuid>> {
        let names = name.split_ascii_whitespace();

        // If the index is not built or invalidated, build it
        let index = &self.index;
            
         // If the first character is not alphabetic, use '#' as key
        // if !key.is_alphabetic() {
        //     key = '#';
        //}

        let mut ids_as_set: HashSet<Uuid> = HashSet::new();

        for name_slice in names {
            let ids = index.name.get(&name_slice.to_ascii_lowercase())?;
            ids_as_set = ids_as_set.union(ids)
                .map(|&id| id)
                .collect()
        }

        let ids: Vec<Uuid> = ids_as_set
            .iter()
            .filter_map(|id| {
                self.mem.get(id).and_then(|contact| {
                    if contact.name.eq_ignore_ascii_case(name) { Some(id.clone()) } else { None }
                })
            })
            .collect();

        if ids.is_empty() { None } else { Some(ids) }
    }

    pub fn add_contact(&mut self, contact: Contact) {
        self.index.increment_index(&contact);

        self.mem.insert(contact.id.clone(), contact);

    }

    pub fn delete_contact(&mut self, id: &Uuid) -> Result<(), AppError> {

        match self.mem.remove(id) {

            Some(deleted_contact) => {
                self.index.decrement_index(&deleted_contact);
                Ok(())
            }
            None => Err(AppError::NotFound("Contact".to_string()))
        }
    }


    pub fn create_name_search_index(&self) -> Result<HashMap<String, HashSet<Uuid>>, AppError> {
        const MAX_WORKER_THREADS: usize = 5;
        let contact_list = Arc::new(self.contact_list());
        let worker_threads: usize;
        let length = contact_list.len();
        
        match length {
            0..=10 => worker_threads = 1,
            11..=50 => worker_threads = 2,
            51..=200 => worker_threads = 3,
            201..=500 => worker_threads = 4,
            _ => worker_threads = MAX_WORKER_THREADS,
        }

        let chunk = length / worker_threads;
        let index: Arc<Mutex<HashMap<String, HashSet<Uuid>>>> = Arc::new(Mutex::new(
            HashMap::new()
        ));

        thread::scope(|s| {
            for i in 1..=worker_threads {
                let map1 = Arc::clone(&index);
                let list1 = Arc::clone(&contact_list);

                s.spawn(move || -> Result<(), AppError> {
                    // Get next starting index multiplying chunk with current iteration
                    let start = chunk * (i-1); // -1 to start from index zero and also catch unincluded end index from previous iteration
                    let end: usize;

                    if i == worker_threads {
                        // Last thread takes the remainder if any
                        end = (chunk * i).max(length);
                    } else {
                        end = chunk * i;
                    }

                    let mut map1_lock = map1.lock()?;

                    for idx in start..end {
                        let contact = list1[idx];

                        // All parts of the contact name (seperated by space) is inserted as a new key
                        // To ensure that searching any part of a contact name (not just the first name) will also
                        // provide the expected contact
                        let contact_names: Vec<&str> = contact.name.split_ascii_whitespace().collect();

                        for name in contact_names {
                            // if name.is_alphabetic() {
                            map1_lock.entry(name.to_ascii_lowercase())
                            .or_default()
                            .insert(contact.id);

                            // } else {
                            //     // If contact name does not start with an alphabet
                            //     map1_lock.entry('#')
                            //     .or_default()
                            //     .insert(contact.id);
                            // }
                        }
                    }

                    Ok(())
                });
            }
            
        });
        

        
        let result = Arc::into_inner(index).unwrap_or_default().into_inner()?;
        Ok(result)
    }




    pub fn create_email_domain_search_index(&self) -> Result<HashMap<String, HashSet<Uuid>>, AppError> {
        const MAX_WORKER_THREADS: usize = 5;
        let contact_list = Arc::new(self.contact_list());
        let worker_threads: usize;
        let length = contact_list.len();

        match length {
            0..=10 => worker_threads = 1,
            11..=50 => worker_threads = 2,
            51..=200 => worker_threads = 3,
            201..=500 => worker_threads = 4,
            _ => worker_threads = MAX_WORKER_THREADS,
        }

        let chunk = length / worker_threads;
        let index: Arc<Mutex<HashMap<String, HashSet<Uuid>>>> = Arc::new(Mutex::new(
            HashMap::new()
        ));

        thread::scope(|s| {
            for i in 1..=worker_threads {
                let map1 = Arc::clone(&index);
                let list1 = Arc::clone(&contact_list);

                s.spawn(move || -> Result<(), AppError> {
                    // Get next starting index multiplying chunk with current iteration
                    let start = chunk * (i-1); // -1 to start from index zero and also catch unincluded end index from previous iteration
                    let end: usize;

                    if i == worker_threads {
                        // Last thread takes the remainder if any
                        end = (chunk * i).max(length);
                    } else {
                        end = chunk * i;
                    }

                    let mut map1_lock = map1.lock()?;

                    for idx in start..end {
                        let contact = list1[idx];
                        let email_parts: Vec<&str> = contact.email.split('@').collect();
                        let domain = email_parts[email_parts.len() -1].to_string();
                        
                        map1_lock.entry(domain.to_ascii_lowercase())
                        .or_default()
                        .insert(contact.id);
                    }

                    Ok(())
                });
            }
            

        });
        

        
        let result = Arc::into_inner(index).unwrap_or_default().into_inner()?;
        Ok(result)
    }


    pub fn fuzzy_search_name_index(&self, name: &str) -> Result<Vec<&Contact>, AppError> {
        const MAX_SEARCH_LENGTH: u8 = 30;
        const MAX_WORKER_THREADS: usize = 3;
        let name = Arc::new(name.trim().to_ascii_lowercase());

        if name.is_empty() {
            return Err(AppError::Validation("No Name provided".to_string()));
        }
        if name.len() > MAX_SEARCH_LENGTH as usize {
            return Err(AppError::Validation("Search string too long".to_string()));
        }

        const MIN_DISTANCE: f32 = 0.5;

        let index = &self.index;


        // let mut index_key = name.chars().next().unwrap_or_default();
        // if !index_key.is_alphabetic() {
        //     index_key = '#';
        // }
         
        
        let names: Vec<&str> = name.trim().split_ascii_whitespace().collect();
        let keys: Arc<Vec<_>> = Arc::new(index.name.keys().collect());
        let length = keys.len();
        let worker_threads: usize;

        match length {
            0..=10 => worker_threads = 1,
            11..=50 => worker_threads = 2,
            _ => worker_threads = MAX_WORKER_THREADS,
        }

        let chunk = length / worker_threads;
        let fuzzy_match_id_set: Arc<Mutex<HashSet<Uuid>>> = Arc::new(
            Mutex::new(HashSet::new()) // This would hold contact ids of all contacts where the search string matches their membership key in the Index. 
        );

        if length < 1 {
            return Ok(Vec::new());
        }

        thread::scope(|s| {
            // search for all parts of the search string (seperated by space if any) for an inclusive search. 
            for name_slice in &names {

                for i in 1..=worker_threads {
                    let fuzzy_match_id_set = Arc::clone(&fuzzy_match_id_set);
                    let keys = Arc::clone(&keys);

                    s.spawn(move || -> Result<(), AppError> {
                        // Get next starting index multiplying chunk with current iteration
                        let start = chunk * (i-1); // -1 to start from index zero and also catch unincluded end index from previous iteration
                        let end: usize;

                        if i == worker_threads {
                            // Last thread takes the remainder if any
                            end = (chunk * i).max(length);
                        } else {
                            end = chunk * i;
                        }

                        let mut matches = fuzzy_match_id_set.lock()?;

                        for &key in &keys[start..end] {
                            let distance = fuzzy_compare(key, name_slice);

                            if distance >= MIN_DISTANCE {
                                if let Some(ids) = index.name.get(key) {
                                    matches.extend(ids.iter());
                                }

                            }
                        }

                        Ok(())
                    });
                }
            }
        });
        
        // get the data of the Arc (Arc::into_inner()) a Mutex data, the get the value of the Mutex (.into_inner())
        let result = Arc::into_inner(fuzzy_match_id_set).unwrap_or_default().into_inner()?;
        let result = result.iter()
                .filter_map(|id| {
                    self.mem.get(id).and_then(|contact| Some(contact))
                }).collect();
        Ok(result)
    }


    pub fn fuzzy_search_email_domain_index(&self, domain: &str) -> Result<Vec<&Contact>, AppError> {
        const MAX_SEARCH_LENGTH: u8 = 15;
        const MAX_WORKER_THREADS: usize = 3;

        let domain = &domain.trim().to_lowercase();

        if domain.is_empty() {
            return Err(AppError::Validation("No email domain provided".to_string()));
        }

        if domain.len() > MAX_SEARCH_LENGTH as usize {
            return Err(AppError::Validation("Please provide just email domain Eg. \"example.com\"".to_string()));
        }

        let index = &self.index;

        let default_set: HashSet<Uuid> = HashSet::new();

        let ids_as_set = index.domain.get(domain).unwrap_or(&default_set);

        let index_match = Arc::new(ids_as_set.into_iter().collect::<Vec<&Uuid>>());
        let worker_threads: usize;
        let length = index_match.len();

        match length {
            0..=10 => worker_threads = 1,
            11..=50 => worker_threads = 2,
            _ => worker_threads = MAX_WORKER_THREADS,
        }

        let chunk = length / worker_threads;

        let fuzzy_match: Arc<Mutex<Vec<&Contact>>> = Arc::new(
            Mutex::new(Vec::new())
        );

        if length < 1 {
            let result = Arc::into_inner(fuzzy_match).unwrap_or_default().into_inner()?;
            return Ok(result);
        }

        thread::scope(|s| {
            for i in 1..=worker_threads {
                let match1 = Arc::clone(&fuzzy_match);
                let uuids = Arc::clone(&index_match);

                s.spawn(move || -> Result<(), AppError> {

                    let start = chunk * (i-1); // -1 to start from index zero and also catch unincluded end index from previous iteration
                    let end: usize;

                    if i == worker_threads {
                        // Last thread takes the remainder if any
                        end = (chunk * i).max(length);
                    } else {
                        end = chunk * i;
                    }

                    let mut matches = match1.lock()?;

                    for &id in &uuids[start..end] {
                        if let Some(contact) = self.mem.get(&id){
                            matches.push(contact);
                        }
                    }
                    
                    Ok(())
                });
            }
        });

        // get the data of the Arc (Arc::into_inner()) a Metex data, the get the value of the Mutex (.into_inner())
        let result = Arc::into_inner(fuzzy_match).unwrap_or_default().into_inner()?;
        Ok(result)
    }
}

// impl<'a> Iterator for JsonIter<'a> {
//     type Item = &'a Contact;

//     fn next(&mut self) -> Option<Self::Item> {
//         if self.idx >= self.inner.len() {
//             return None;
//         }
//         let contact = &self.inner[self.idx];
//         self.idx += 1;
//         Some(contact)
//     }
// }



impl ContactStore for Store<'_> {
    fn load(&self) -> Result<HashMap<Uuid, Contact>, AppError> {
        let txt_contacts = load_txt_contacts(TXT_STORAGE_PATH)?;
        let mut json_contacts = load_json_contacts(JSON_STORAGE_PATH)?;
        let storage_choice = parse_storage_choice();

        if storage_choice.is_json() && txt_contacts.is_empty() {
            return Ok(json_contacts);
        }

        if storage_choice.is_txt() && json_contacts.is_empty() {
            return Ok(txt_contacts);
        }

        json_contacts.extend(txt_contacts);
        
        Ok(json_contacts)
    }


    fn save(&self, contacts: &HashMap<Uuid, Contact>) -> Result<(), AppError> {
        let path = Path::new(&self.path);
        if !path.exists() {
            create_file_parent(self.path)?;
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

        if parse_storage_choice().is_txt() {
            // use our helper to serialize data for txt file
            let data = helper::serialize_contacts(contacts);
            file.write_all(data.as_bytes())?;

            if fs::exists(Path::new(JSON_STORAGE_PATH))? {
                fs::remove_file(Path::new(JSON_STORAGE_PATH))?;
            }

        } else {
            // user serde to serialize json data
            let json_contact = serde_json::to_string(&contacts)?;
            file.write_all(json_contact.as_bytes())?;

            if fs::exists(Path::new(TXT_STORAGE_PATH))? {
                fs::remove_file(Path::new(TXT_STORAGE_PATH))?;
            }
        }
        
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

    // New feature: Contacts are now stored in HashMap.
    // Try if new feature has been effected
    if let Ok(contacts) = serde_json::from_str::<HashMap<Uuid, Contact>>(&data) {
        return Ok(contacts);
    }

    let contacts: Vec<Contact> = serde_json::from_str(&data)?;

    // Convert Vec to HashMap for new feature backward compatibility
    let mapped_contacts = contacts
        .into_iter()
        .map(|cont| (cont.id, cont))
        .collect::<HashMap<Uuid, Contact>>();
    Ok(mapped_contacts)
}






#[cfg(test)]
mod tests {
    use chrono::Utc;

    use super::*;


    #[test]
    fn adds_persistent_contact_with_txt() -> Result<(), AppError> {
        let mut storage = Store {
            mem: HashMap::new(),
            path: TXT_STORAGE_PATH,
            index: Index {
                name: HashMap::new(),
                domain: HashMap::new(),
            },
        };

        let new_contact = Contact::new(
            "Uche".to_string(),
            "01234567890".to_string(),
            "ucheuche@gmail.com".to_string(),
            "".to_string(),
        );

        storage.add_contact(new_contact);
        storage.save(&storage.mem)?;
        storage.mem.clear();
        storage.mem = storage.load()?;
        storage.index = Index::new(&storage)?;

        assert_eq!(
            storage.contact_list()[0],
            &Contact::new(
                "Uche".to_string(),
                "01234567890".to_string(),
                "ucheuche@gmail.com".to_string(),
                "".to_string(),
            )
        );

        storage.mem.clear();
        storage.save(&storage.mem)?;
        Ok(())
    }

    #[test]
    fn delete_persistent_contact_with_txt() -> Result<(), AppError> {
        let mut storage = Store {
            mem: HashMap::new(),
            path: TXT_STORAGE_PATH,
            index: Index {
                name: HashMap::new(),
                domain: HashMap::new(),
            },
        };

        let contact1 = Contact::new(
            "Uche".to_string(),
            "01234567890".to_string(),
            "ucheuche@gmail.com".to_string(),
            "".to_string(),
        );

        let contact2 = Contact::new(
            "Alex".to_string(),
            "01234567890".to_string(),
            "".to_string(),
            "".to_string(),
        );

        storage.add_contact(contact1);
        storage.add_contact(contact2);

        storage.save(&storage.mem)?;
        storage.mem.clear();

        storage.mem = storage.load()?;
        storage.index = Index::new(&storage)?;

        let index = storage
            .get_ids_by_name(&"Uche".to_string())
            .unwrap_or_default();
        storage.delete_contact(&index[0])?;
        storage.save(&storage.mem)?;

        storage.mem.clear();
        storage.mem = storage.load()?;
        storage.index = Index::new(&storage)?;

        assert_eq!(storage.mem.len(), 1);

        assert_ne!(
            *storage.contact_list()[0],
            Contact::new(
                "Uche".to_string(),
                "01234567890".to_string(),
                "ucheuche@gmail.com".to_string(),
                "".to_string(),
            )
        );

        storage.mem.clear();
        storage.save(&storage.mem)?;

        Ok(())
    }

    #[test]
    fn json_store_is_persistent() -> Result<(), AppError> {
        let mut storage = Store::new()?;

        let created = Utc::now();
        let id_1 = Uuid::new_v4();
        let id_2 = Uuid::new_v4();

        let contact1 = Contact {
            id: id_1.clone(),
            name: "Uche".to_string(),
            phone: "01234567890".to_string(),
            email: "ucheuche@gmail.com".to_string(),
            tag: "".to_string(),
            created_at: created.clone(),
            updated_at: created.clone(),
        };
        

        let contact2 = Contact{
            id: id_2,
            name: "Alex".to_string(),
            phone: "01234567890".to_string(),
            email: "".to_string(),
            tag: "".to_string(),
            created_at: created.clone(),
            updated_at: created.clone(),
        };

        storage.add_contact(contact1);
        storage.add_contact(contact2);

        storage.save(&storage.mem)?;
        storage.mem.clear();

        storage.mem = storage.load()?;
        storage.index = Index::new(&storage)?;

        assert_eq!(
            storage.mem.get(&id_1).unwrap(),
            &Contact::new(
                "Uche".to_string(),
                "01234567890".to_string(),
                "ucheuche@gmail.com".to_string(),
                "".to_string(),
            )
        );

        assert_eq!(
            storage.mem.get(&id_2).unwrap(),
            &Contact::new(
                "Alex".to_string(),
                "01234567890".to_string(),
                "".to_string(),
                "".to_string(),
            )
        );


        storage.delete_contact(&id_1)?;
        storage.save(&storage.mem)?;

        storage.mem.clear();
        storage.mem = storage.load()?;
        storage.index = Index::new(&storage)?;

        assert_eq!(storage.mem.len(), 1);

        assert_ne!(
            *storage.contact_list()[0],
            Contact {
                id: id_1,
                name: "Uche".to_string(),
                phone: "01234567890".to_string(),
                email: "ucheuche@gmail.com".to_string(),
                tag: "".to_string(),
                created_at: created.clone(),
                updated_at: created.clone(),
            }
        );

        storage.mem.clear();
        storage.save(&storage.mem)?;

        Ok(())
    }

    #[test]
    fn migrates_contact() -> Result<(), AppError> {
        let mut txt_store = Store {
            mem: HashMap::new(),
            path: TXT_STORAGE_PATH,
            index: Index {
                name: HashMap::new(),
                domain: HashMap::new(),
            },
        };
        txt_store.mem.clear();

        let contact1 = Contact::new(
            "Uche".to_string(),
            "01234567890".to_string(),
            "ucheuche@gmail.com".to_string(),
            "".to_string(),
        );

        let contact2 = Contact::new(
            "Alex".to_string(),
            "+44731484372".to_string(),
            "".to_string(),
            "".to_string(),
        );

        txt_store.add_contact(contact1);
        txt_store.save(&txt_store.mem)?;
        txt_store.mem.clear();

        let mut json_store = Store::new()?;
        
        json_store.mem = json_store.load()?;

        json_store.add_contact(contact2);
        json_store.save(&json_store.mem)?;
        json_store.mem.clear();

        json_store.mem = json_store.load()?;
        let contact_list = json_store.contact_list();

        assert!(contact_list.len() == 2);

        assert!(contact_list.contains(&&Contact::new(
            "Uche".to_string(),
            "01234567890".to_string(),
            "ucheuche@gmail.com".to_string(),
            "".to_string(),
        )));

        assert!(contact_list.contains(&&Contact::new(
            "Alex".to_string(),
            "+44731484372".to_string(),
            "".to_string(),
            "".to_string(),
        )));

        json_store.mem.clear();
        json_store.save(&json_store.mem)?;

        txt_store.mem.clear();
        txt_store.save(&txt_store.mem)?;

        Ok(())
    }

    
    #[test]
    fn index_updates_on_add_and_delete() -> Result<(), AppError> {
        let mut store = Store::new()?;

        let contact1 = Contact::new(
            "Uche".to_string(),
            "01234567890".to_string(),
            "u1@example.com".to_string(),
            "".to_string(),
        );
        let id1 = contact1.id;
        store.add_contact(contact1);

        // After adding, index should contain the contact id for the name "Uche"
        let ids_for_uche = store.get_ids_by_name("Uche").unwrap_or_default();
        assert!(ids_for_uche.contains(&id1));

        let contact2 = Contact::new(
            "Uche".to_string(),
            "111222333".to_string(),
            "u2@example.com".to_string(),
            "".to_string(),
        );
        let id2 = contact2.id;
        store.add_contact(contact2);

        // Now two ids should be returned for "Uche"
        let ids_for_uche = store.get_ids_by_name("Uche").unwrap_or_default();
        assert_eq!(ids_for_uche.len(), 2);
        assert!(ids_for_uche.contains(&id1));
        assert!(ids_for_uche.contains(&id2));

        // Delete the first contact and ensure index is updated
        store.delete_contact(&id1)?;
        let ids_after_delete = store.get_ids_by_name("Uche").unwrap_or_default();
        assert_eq!(ids_after_delete.len(), 1);
        assert!(ids_after_delete.contains(&id2));
        assert!(!ids_after_delete.contains(&id1));

        // Ensure domain index no longer references the deleted id
        if let Some(domain_set) = store.index.domain.get("example.com") {
            assert!(!domain_set.contains(&id1));
        }

        store.mem.clear();
        Ok(())
    }

    #[test]
    fn fuzzy_search_name_matches_on_partial() -> Result<(), AppError> {
        let mut store = Store::new()?;

        let contact = Contact::new(
            "Uche Johnson".to_string(),
            "01234567890".to_string(),
            "uche@example.com".to_string(),
            "".to_string(),
        );
        let expected_name = contact.name.clone();
        store.add_contact(contact);

        // Search by a portion of the name (partial)
        let results = store.fuzzy_search_name_index("uch")?;
        assert!(!results.is_empty());
        assert!(results.iter().any(|c| c.name == expected_name));

        let results = store.fuzzy_search_name_index("john")?;
        assert!(!results.is_empty());
        assert!(results.iter().any(|c| c.name == expected_name));

        // Cleanup persistent files
        store.mem.clear();
        Ok(())
    }

    #[test]
    fn fuzzy_search_email_domain_returns_contact() -> Result<(), AppError> {
        let mut store = Store::new()?;

        let contact = Contact::new(
            "Alice".to_string(),
            "+447700900123".to_string(),
            "alice@example.com".to_string(),
            "".to_string(),
        );
        let expected_email = contact.email.clone();
        store.add_contact(contact);

        // Domain search expects the domain part exactly
        let results = store.fuzzy_search_email_domain_index("example.com")?;
        assert!(!results.is_empty());
        assert!(results.iter().any(|c| c.email == expected_email));

        store.mem.clear();
        Ok(())
    }
}
