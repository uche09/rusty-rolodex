use crate::{
    errors::AppError,
    store::{ContactStore, Store},
};

#[derive(Debug, PartialEq)]
pub struct Contact {
    pub name: String,
    pub phone: String,
    pub email: String,
}

pub enum Command {
    AddContact,
    ListContacts,
    DeleteContact,
    Exit,
}

pub struct Storage {
    store: Store,
}

impl Storage {
    pub fn new() -> Result<Storage, AppError> {
        Ok(Storage {
            store: Store::new()?,
        })
    }

    pub fn add_contact(&mut self, contact: Contact) {
        self.store.mem.push(contact);
    }

    pub fn contact_list(&self) -> &Vec<Contact> {
        &self.store.mem
    }

    pub fn delete_contact(&mut self, index: usize) -> Result<(), AppError> {
        if index < self.store.mem.len() {
            self.store.mem.remove(index);
            Ok(())
        } else {
            Err(AppError::NotFound("Contact".to_string()))
        }
    }

    pub fn get_indices_by_name(&self, name: &String) -> Option<Vec<usize>> {
        let indices: Vec<usize> = self
            .contact_list()
            .iter()
            .enumerate()
            .filter(|(_, cont)| &cont.name == name)
            .map(|(idx, _)| idx)
            .collect();
        if indices.is_empty() {
            return None;
        }
        Some(indices)
    }
}

impl ContactStore for Storage {
    fn load(&mut self) -> Result<(), AppError> {
        self.store.load()
    }

    fn save(&mut self) -> Result<(), AppError> {
        self.store.save()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn adds_persistent_contact() -> Result<(), AppError> {
        let mut storage = Storage::new()?;

        let new_contact = Contact {
            name: "Uche".to_string(),
            phone: "01234567890".to_string(),
            email: "ucheuche@gmail.com".to_string(),
        };

        storage.add_contact(new_contact);
        storage.save()?;
        storage.store.mem.clear();
        storage.load()?;

        assert_eq!(
            storage.contact_list()[0],
            Contact {
                name: "Uche".to_string(),
                phone: "01234567890".to_string(),
                email: "ucheuche@gmail.com".to_string(),
            }
        );
        Ok(())
    }

    #[test]
    fn delete_persistent_contact() -> Result<(), AppError> {
        let mut storage = Storage::new()?;

        let contact1 = Contact {
            name: "Uche".to_string(),
            phone: "01234567890".to_string(),
            email: "ucheuche@gmail.com".to_string(),
        };

        let contact2 = Contact {
            name: "Alex".to_string(),
            phone: "01234567890".to_string(),
            email: "".to_string(),
        };

        storage.add_contact(contact1);
        storage.add_contact(contact2);

        storage.save()?;
        storage.store.mem.clear();

        storage.load()?;
        storage.delete_contact(0)?;
        storage.save()?;

        storage.store.mem.clear();
        storage.load()?;

        assert_eq!(storage.store.mem.len(), 1);

        assert_ne!(
            storage.contact_list()[0],
            Contact {
                name: "Uche".to_string(),
                phone: "01234567890".to_string(),
                email: "ucheuche@gmail.com".to_string(),
            }
        );

        Ok(())
    }
}
