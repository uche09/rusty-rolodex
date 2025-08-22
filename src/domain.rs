use crate::store::Store;

#[derive(Debug)]
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

pub struct ContactStore {
    store: Store,
}

impl ContactStore {
    pub fn new() -> Self {
        ContactStore {
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

    pub fn get_index_by_name(&self, name: &String) -> Option<usize> {
        let index = self
            .contact_list()
            .iter()
            .position(|cont| &cont.name == name);
        index
    }
}
