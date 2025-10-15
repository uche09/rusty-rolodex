use super::*;

pub const STORAGE_PATH: &str = "./.instance/contacts.json";
pub struct JsonStore<'a> {
    pub mem: Vec<Contact>,
    pub path: &'a str,
}

// pub struct JsonIter<'a> {
//     inner: &'a [Contact],
//     idx: usize,
// }

impl JsonStore<'_> {
    pub fn new() -> Result<Self, AppError> {
        create_file_parent(STORAGE_PATH)?;

        Ok(Self {
            mem: Vec::new(),
            path: STORAGE_PATH,
        })
    }

    // pub fn iter(&self) -> JsonIter<'_> {
    //     JsonIter {
    //         inner: &self.mem,
    //         idx: 0,
    //     }
    // }
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

impl ContactStore for JsonStore<'_> {
    fn load(&self) -> Result<Vec<Contact>, AppError> {
        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .truncate(false)
            .create(true)
            .open(self.path)?;

        let mut data = String::new();
        file.read_to_string(&mut data)?;

        // serde_json will give an error if data is empty
        if data.is_empty() {
            return Ok(Vec::new());
        }

        Ok(serde_json::from_str(&data)?)
    }

    fn save(&self, contacts: &[Contact]) -> Result<(), AppError> {
        let json_contact = serde_json::to_string(&contacts)?;

        let path = Path::new(&self.path);
        if !path.exists() {
            create_file_parent(self.path)?;
            let _file = OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .open(path)?;
        }

        let mut file = OpenOptions::new().write(true).truncate(true).open(path)?;

        file.write_all(json_contact.as_bytes())?;

        if fs::exists(Path::new(txt::STORAGE_PATH))? {
            fs::remove_file(Path::new(txt::STORAGE_PATH))?;
        }
        Ok(())
    }

    fn load_migrated_contact(&mut self) -> Result<(), AppError> {
        self.mem = self.load()?;

        if fs::exists(Path::new(txt::STORAGE_PATH))? {
            let txt_contacts = TxtStore::new()?.load()?;

            self.mem.extend(txt_contacts);
            self.mem.sort();
            self.mem.dedup();

            self.save(&self.mem)?;
        }

        Ok(())
    }


    fn contact_list(&self) -> Vec<&Contact> {
        self.mem.iter().collect()
    }

    fn mut_contact_list(&mut self) -> Vec<&mut Contact> {
        self.mem.iter_mut().collect::<Vec<&mut Contact>>()
    }

    fn get_mem(&self) -> &Vec<Contact> {
        &self.mem
    }
    
    fn get_indices_by_name(&self, name: &str) -> Option<Vec<usize>> {
        let indices: Vec<usize> = self
            .mem
            .iter()
            .enumerate()
            .filter(|(_, cont)| cont.name == name)
            .map(|(idx, _)| idx)
            .collect();
        if indices.is_empty() {
            return None;
        }
        Some(indices)
    }

    fn add_contact(&mut self, contact: Contact) {
        self.mem.push(contact);
    }

    fn delete_contact(&mut self, index: usize) -> Result<(), AppError> {
        if index < self.mem.len() {
            self.mem.remove(index);
            Ok(())
        } else {
            Err(AppError::NotFound("Contact".to_string()))
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn json_store_is_persistent() -> Result<(), AppError> {
        let mut storage = JsonStore::new()?;

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
        let mut txt_store = TxtStore::new()?;
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
        txt_store.save(txt_store.get_mem())?;
        txt_store.mem.clear();

        let mut json_store = JsonStore::new()?;
        json_store.load_migrated_contact()?;

        json_store.add_contact(contact2);
        json_store.save(json_store.get_mem())?;
        json_store.mem.clear();

        json_store.load_migrated_contact()?;

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
