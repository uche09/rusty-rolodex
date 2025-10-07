// use super::*;
// use std::{fs, path::Path};

// pub struct Storage {
//     pub txt_store: file::TxtStore,
//     pub json_store: file::JsonStore,
//     pub mem_store: memory::MemStore,
//     pub storage_choice: StorageChoice,
// }

// impl Storage {
//     pub fn new() -> Result<Storage, AppError> {
//         Ok(Storage {
//             txt_store: file::TxtStore::new()?,
//             json_store: file::JsonStore::new()?,
//             mem_store: memory::MemStore::default(),
//             storage_choice: parse_storage_choice(),
//         })
//     }

//     pub fn add_contact(&mut self, contact: Contact) {
//         self.mem_store.data.push(contact);
//     }

//     pub fn contact_list(&self) -> Vec<&Contact> {
//         let contacts = self.mem_store.iter().collect();
//         contacts
//     }

//     pub fn delete_contact(&mut self, index: usize) -> Result<(), AppError> {
//         if index < self.mem_store.data.len() {
//             self.mem_store.data.remove(index);
//             Ok(())
//         } else {
//             Err(AppError::NotFound("Contact".to_string()))
//         }
//     }

//     pub fn save(&self) -> Result<(), AppError> {
//         if self.storage_choice.is_mem() {
//             return Ok(());
//         }

//         if self.storage_choice.is_txt() {
//             self.save_txt(&self.mem_store.data)?;
//         }

//         if self.storage_choice.is_json() {
//             self.save_json(&self.mem_store.data)?;
//         }

//         // Delete migrated storage file for COMPLETE MIGRATION
//         if self.storage_choice.is_json() && Path::new(&self.txt_store.path).exists() {
//             fs::remove_file(Path::new(&self.txt_store.path))?;
//         } else if self.storage_choice.is_txt() && Path::new(&self.json_store.path).exists() {
//             fs::remove_file(Path::new(&self.json_store.path))?;
//         }

//         Ok(())
//     }

//     pub fn load_txt(&self) -> Result<Vec<Contact>, AppError> {
//         self.txt_store.load()
//     }

//     pub fn load_json(&self) -> Result<Vec<Contact>, AppError> {
//         self.json_store.load()
//     }

//     pub fn save_json(&self, contacts: &[Contact]) -> Result<(), AppError> {
//         self.json_store.save(contacts)
//     }

//     pub fn save_txt(&self, contacts: &[Contact]) -> Result<(), AppError> {
//         self.txt_store.save(contacts)
//     }

//     pub fn get_indices_by_name(&self, name: &String) -> Option<Vec<usize>> {
//         let indices: Vec<usize> = self
//             .contact_list()
//             .iter()
//             .enumerate()
//             .filter(|(_, cont)| &cont.name == name)
//             .map(|(idx, _)| idx)
//             .collect();
//         if indices.is_empty() {
//             return None;
//         }
//         Some(indices)
//     }
// }
