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
    pub fn new() -> Self {
        Storage {
            store: Store::new(),
        }
    }

    pub fn add_contact(&mut self, contact: Contact) {
        self.store.mem.push(contact);
    }

    pub fn contact_list(&self) -> &Vec<Contact> {
        &self.store.mem
    }

    pub fn delete_contact(&mut self, index: usize) -> Result<(), String> {
        if index < self.store.mem.len() {
            self.store.mem.remove(index);
            Ok(())
        } else {
            Err("No found".to_string())
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
