use super::*;

use rust_fuzzy_search::fuzzy_compare;
use std::{collections::HashMap, sync::{Arc, Mutex}, thread};
use crate::helper;

pub const JSON_STORAGE_PATH: &str = "./.instance/contacts.json";
pub const TXT_STORAGE_PATH: &str = "./.instance/contacts.txt";


pub struct Store<'a> {
    pub mem: Vec<Contact>,
    pub path: &'a str,
}

// pub struct JsonIter<'a> {
//     inner: &'a [Contact],
//     idx: usize,
// }


impl Store<'_> {
    pub fn new() -> Result<Self, AppError> {
        let path = match parse_storage_choice() {
            StoreChoice::Json => JSON_STORAGE_PATH,
            StoreChoice::Txt => TXT_STORAGE_PATH,
        };
        create_file_parent(path)?;

        Ok(Self {
            mem: Vec::new(),
            path,
        })
    }

    // pub fn iter(&self) -> JsonIter<'_> {
    //     JsonIter {
    //         inner: &self.mem,
    //         idx: 0,
    //     }
    // }


    pub fn contact_list(&self) -> Vec<&Contact> {
        self.mem.iter().collect()
    }

    
    pub fn get_indices_by_name(&self, name: &str) -> Option<Vec<usize>> {
        let contacts = self.contact_list();
        let mut key = name.to_ascii_lowercase().chars().next().unwrap_or_default();

        let index = Index::new(self);
        let index = match index {
            Ok(maps) => maps,
            Err(_) => {
                return None;
            }
        };

        if !key.is_alphabetic() {
            key = '#';
        }

        let indices: Vec<usize> = index
            .name.get(&key)?
            .iter()
            .filter(|&&idx| contacts[idx].name == name)
            .map(|idx| *idx)
            .collect();

        if indices.is_empty() {
            return None;
        }
        Some(indices)
    }

    pub fn add_contact(&mut self, contact: Contact) {
        self.mem.push(contact);
    }

    pub fn delete_contact(&mut self, index: usize) -> Result<(), AppError> {
        if index < self.mem.len() {
            self.mem.remove(index);
            Ok(())
        } else {
            Err(AppError::NotFound("Contact".to_string()))
        }
    }


    pub fn create_name_search_index(&self) -> Result<HashMap<char, Vec<usize>>, AppError> {
        let contact_list = Arc::new(self.contact_list());
        let mid = contact_list.len() / 2;
        
        let index: Arc<Mutex<HashMap<char, Vec<usize>>>> = Arc::new(Mutex::new(
            HashMap::new()
        ));


        thread::scope(|s| {
            let map1 = Arc::clone(&index);
            let list1 = Arc::clone(&contact_list);

            s.spawn(move || -> Result<(), AppError> {
                for idx in 0..mid {
                    if let Some(key) = list1[idx].name.chars().next(){

                        if key.is_alphabetic() {
                            let mut map1_lock = map1.lock()?;
                            map1_lock.entry(key.to_ascii_lowercase())
                            .or_default()
                            .push(idx);
                        } else {
                            // If contact name does not start with an alphabet
                            let mut map1_lock = map1.lock()?;
                            map1_lock.entry('#')
                            .or_default()
                            .push(idx);
                        }
                    }

                }

                Ok(())
            });

            let map2 = Arc::clone(&index);
            let list2 = Arc::clone(&contact_list);

            s.spawn(move || -> Result<(), AppError> {
                for idx in mid..contact_list.len() {
                    if let Some(key) = list2[idx].name.chars().next(){

                        if key.is_alphabetic() {
                            let mut map2_lock = map2.lock()?;
                            map2_lock.entry(key.to_ascii_lowercase())
                            .or_default()
                            .push(idx);
                        } else {
                            // If contact name does not start with an alphabet
                            let mut map2_lock = map2.lock()?;
                            map2_lock.entry('#')
                            .or_default()
                            .push(idx);
                        }
                    }

                }

                Ok(())
            });
        });
        

        
        let result = Arc::into_inner(index).unwrap_or_default().into_inner()?;
        Ok(result)
    }




    pub fn create_email_domain_search_index(&self) -> Result<HashMap<&str, Vec<usize>>, AppError> {
        let contact_list = Arc::new(self.contact_list());
        let mid = contact_list.len() / 2;

        let index: Arc<Mutex<HashMap<&str, Vec<usize>>>> = Arc::new(Mutex::new(
            HashMap::new()
        ));


        thread::scope(|s| {
            let map1 = Arc::clone(&index);
            let list1 = Arc::clone(&contact_list);

            s.spawn(move || -> Result<(), AppError> {
                for idx in 0..mid {
                    let email_parts: Vec<&str> = list1[idx].email.split('@').collect();
                    let domain = email_parts[email_parts.len() -1];
                    
                    let mut map1_lock = map1.lock()?;
                    map1_lock.entry(domain)
                    .or_default()
                    .push(idx);
                }

                Ok(())
            });

            let map2 = Arc::clone(&index);
            let list2 = Arc::clone(&contact_list);

            s.spawn(move || -> Result<(), AppError> {
                for idx in mid..contact_list.len() {
                    let email_parts: Vec<&str> = list2[idx].email.split('@').collect();
                    let domain = email_parts[email_parts.len() -1];
                    
                    let mut map2_lock = map2.lock()?;
                    map2_lock.entry(domain)
                    .or_default()
                    .push(idx);
                }

                Ok(())
            });
        });
        

        
        let result = Arc::into_inner(index).unwrap_or_default().into_inner()?;
        Ok(result)
    }


    pub fn fuzzy_search_name_index(&self, name: &str) -> Result<Vec<&Contact>, AppError> {
        const MAX_SEARCH_LENGTH: u8 = 30;
        let name = name.trim().to_ascii_lowercase();

        if name.is_empty() {
            return Err(AppError::Validation("No Name provided".to_string()));
        }

        if name.len() > MAX_SEARCH_LENGTH as usize {
            return Err(AppError::Validation("Search string too long".to_string()));
        }

        const MIN_DISTANCE: f32 = 0.7;
        let index = Index::new(self)?;
        let contact_list = self.contact_list();

        let empty_vec: Vec<usize> = Vec::new();

        let index_key = name.chars().next().unwrap_or_default();
        let indices_match = index.name.get(&index_key).unwrap_or(&empty_vec);

        let fuzzy_match: Vec<&Contact> = indices_match
            .iter()
            .map(|idx| contact_list[*idx])
            .filter(|&c| {
                fuzzy_compare(
                    &c.name.to_ascii_lowercase(),
                    name.as_str())
                >= MIN_DISTANCE
            })
            .collect();
        

        Ok(fuzzy_match)
    }


    pub fn fuzzy_search_email_domain_index(&self, domain: &str) -> Result<Vec<&Contact>, AppError> {
        const MAX_SEARCH_LENGTH: u8 = 15;
        let domain = domain.trim();

        if domain.is_empty() {
            return Err(AppError::Validation("No email domain provided".to_string()));
        }

        if domain.len() > MAX_SEARCH_LENGTH as usize {
            return Err(AppError::Validation("Please provide just email domain Eg. \"example.com\"".to_string()));
        }

        let index = Index::new(self)?;
        let contact_list = self.contact_list();

        let empty_vec: Vec<usize> = Vec::new();

        // let index = create_email_domain_search_index(contact_list)?;
        let index_match = index.domain.get(domain).unwrap_or(&empty_vec);

        let fuzzy_match: Vec<&Contact> = index_match
            .iter()
            .map(|c| contact_list[*c])
            .collect();

        Ok(fuzzy_match)
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
    fn load(&self) -> Result<Vec<Contact>, AppError> {
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
        json_contacts.sort();
        json_contacts.dedup();

        Ok(json_contacts)
    }


    fn save(&self, contacts: &[Contact]) -> Result<(), AppError> {
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

pub fn load_txt_contacts(path: &str) -> Result<Vec<Contact>, AppError> {
    if !fs::exists(Path::new(path))? {
        return Ok(Vec::new());
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

pub fn load_json_contacts(path: &str) -> Result<Vec<Contact>, AppError> {
    if !fs::exists(Path::new(path))? {
        return Ok(Vec::new());
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
        return Ok(Vec::new());
    }

    Ok(serde_json::from_str(&data)?)
}



pub struct Index<'a> {
    pub name: HashMap<char, Vec<usize>>,
    pub domain: HashMap<&'a str, Vec<usize>>,
}

impl<'a> Index<'a> {
    pub fn new(storage: &'a Store) -> Result<Self, AppError> {
        Ok(
            Self {
                name: storage.create_name_search_index()?,
                domain: storage.create_email_domain_search_index()?,
            }
        )
    }
}





#[cfg(test)]
mod tests {
    use super::*;


    #[test]
    fn adds_persistent_contact_with_txt() -> Result<(), AppError> {
        let mut storage = Store {
            mem: Vec::new(),
            path: TXT_STORAGE_PATH,
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
            mem: Vec::new(),
            path: TXT_STORAGE_PATH,
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

        let index = storage
            .get_indices_by_name(&"Uche".to_string())
            .unwrap_or_default();
        storage.delete_contact(index[0])?;
        storage.save(&storage.mem)?;

        storage.mem.clear();
        storage.mem = storage.load()?;

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

        assert_eq!(
            storage.mem[0],
            Contact::new(
                "Uche".to_string(),
                "01234567890".to_string(),
                "ucheuche@gmail.com".to_string(),
                "".to_string(),
            )
        );

        assert_eq!(
            storage.mem[1],
            Contact::new(
                "Alex".to_string(),
                "01234567890".to_string(),
                "".to_string(),
                "".to_string(),
            )
        );

        storage.delete_contact(0)?;
        storage.save(&storage.mem)?;

        storage.mem.clear();
        storage.mem = storage.load()?;

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
    fn migrates_contact() -> Result<(), AppError> {
        let mut txt_store = Store {
            mem: Vec::new(),
            path: TXT_STORAGE_PATH,
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

        assert!(json_store.contact_list().len() == 2);

        assert!(json_store.mem.contains(&Contact::new(
            "Uche".to_string(),
            "01234567890".to_string(),
            "ucheuche@gmail.com".to_string(),
            "".to_string(),
        )));

        assert!(json_store.mem.contains(&Contact::new(
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
}
