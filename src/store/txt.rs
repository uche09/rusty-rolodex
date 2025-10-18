use super::*;
use crate::helper;

pub const STORAGE_PATH: &str = "./.instance/contacts.txt";
pub struct TxtStore<'a> {
    pub mem: Vec<Contact>,
    pub path: &'a str,
}

impl TxtStore<'_> {
    pub fn new() -> Result<Self, AppError> {
        create_file_parent(STORAGE_PATH)?;

        Ok(TxtStore {
            mem: Vec::new(),
            path: STORAGE_PATH,
        })
    }
}

impl ContactStore for TxtStore<'_> {
    fn load(&self) -> Result<Vec<Contact>, AppError> {
        // Read text from file
        // Using OpenOptions to open file if already exist
        // Or create one
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .truncate(false)
            .create(true)
            .open(self.path)?;
        let reader = BufReader::new(file);
        let contacts = helper::deserialize_contacts_from_txt_buffer(reader)?;
        Ok(contacts)
    }

    fn save(&self, contacts: &[Contact]) -> Result<(), AppError> {
        let mut file = OpenOptions::new()
            .read(true)
            .write(true) // WRITE to file on save
            .truncate(true)
            .create(true)
            .open(self.path)?;

        let data = helper::serialize_contacts(contacts);
        file.write_all(data.as_bytes())?;

        Ok(())
    }

    fn load_migrated_contact(&mut self) -> Result<(), AppError> {
        self.mem = self.load()?;

        if fs::exists(Path::new(json::STORAGE_PATH))? {
            let json_contacts = JsonStore::new()?.load()?;

            self.mem.extend(json_contacts);
            self.mem.sort();
            self.mem.dedup();

            self.save(&self.mem)?;

            fs::remove_file(Path::new(json::STORAGE_PATH))?;
        }

        Ok(())
    }

    fn contact_list(&self) -> Vec<&Contact> {
        self.mem.iter().collect()
    }

    fn mut_contact_list(&mut self) -> &mut Vec<Contact> {
        &mut self.mem
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
    fn adds_persistent_contact_with_txt() -> Result<(), AppError> {
        let mut storage = TxtStore::new()?;

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
        let mut storage = TxtStore::new()?;

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
}
