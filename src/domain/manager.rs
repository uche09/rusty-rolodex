use crate::helper;

use super::*;

use chrono::{Duration, Utc};
use file::{JsonStorage, TxtStorage};
use rust_fuzzy_search::fuzzy_compare;
use std::{
    collections::{HashMap, HashSet},
    sync::{Arc, Mutex},
    thread,
};

const MAX_WORKER_THREADS: usize = 5;

#[derive(Debug)]
pub struct Index {
    pub name: HashMap<String, HashSet<Uuid>>,
    pub domain: HashMap<String, HashSet<Uuid>>,
}

pub enum IndexUpdateType {
    Add,
    Remove,
}

pub struct LastWriteWinsPolicy;

pub enum SyncDecision {
    LocalWins,
    RemoteWins,
}

pub enum SyncPolicy {
    LastWriteWinsPolicy(LastWriteWinsPolicy),
}

pub struct ContactManager {
    pub mem: HashMap<Uuid, Contact>,
    pub storage: Box<dyn ContactStore>,
    pub index: Index,
}

impl Index {
    pub fn new(storage: &ContactManager) -> Result<Self, AppError> {
        let mut index = Self {
            name: storage.create_name_search_index()?,
            domain: storage.create_email_domain_search_index()?,
        };

        index.name.reserve(storage.mem.len() * 2); // Assume each contact has two unique name parts on average
        index.domain.reserve(storage.mem.len() / 5); // Assume 1 in 5 contacts share the same email domain
        Ok(index)
    }

    pub fn updated_name_index(&mut self, contact: &Contact, update_type: &IndexUpdateType) {
        let name = &contact.name;

        if name.is_empty() {
            return;
        }

        let names = name.split_ascii_whitespace();

        match update_type {
            IndexUpdateType::Add => {
                for name_slice in names {
                    self.name
                        .entry(name_slice.to_lowercase())
                        .or_default()
                        .insert(contact.id);
                }
            }
            IndexUpdateType::Remove => {
                for name_slice in names {
                    let name_slice = name_slice.to_lowercase();
                    if let Some(indices) = self.name.get_mut(&name_slice) {
                        indices.remove(&contact.id);

                        if indices.is_empty() {
                            self.name.remove(&name_slice);
                        }
                    }
                }
            }
        }
    }

    pub fn update_domain_index(&mut self, contact: &Contact, update_type: &IndexUpdateType) {
        if contact.email.is_empty() {
            return;
        }

        let email_parts: Vec<&str> = contact.email.split('@').collect();
        let domain = email_parts[email_parts.len() - 1].to_string();
        let domain = domain.to_ascii_lowercase();

        match update_type {
            IndexUpdateType::Add => {
                self.domain.entry(domain).or_default().insert(contact.id);
            }
            IndexUpdateType::Remove => {
                if let Some(indices) = self.domain.get_mut(&domain) {
                    indices.remove(&contact.id);

                    if indices.is_empty() {
                        self.domain.remove(&domain);
                    }
                }
            }
        }
    }

    pub fn update_both_indexes(&mut self, contact: &Contact, update_type: &IndexUpdateType) {
        self.updated_name_index(contact, update_type);
        self.update_domain_index(contact, update_type);
    }
}

impl LastWriteWinsPolicy {
    pub fn verify_match(&self, local: &Contact, remote: &Contact) -> bool {
        local.created_at == remote.created_at
    }

    pub fn conflict_resolution(&self, local: &mut Contact, remote: &mut Contact) -> SyncDecision {
        // If local contact has been deleted, ensure remote counterpart is also marked deleted
        if local.deleted {
            remote.updated_at = if !remote.deleted {
                local.updated_at
            } else {
                remote.updated_at
            };
            remote.deleted = local.deleted;
            return SyncDecision::LocalWins;
        }

        // If local contact has the latest update, no need to update
        if local.updated_at >= remote.updated_at {
            return SyncDecision::LocalWins;
        }

        SyncDecision::RemoteWins
    }

    pub fn merge_changes(&self, local: &mut Contact, remote: &mut Contact) {
        // Update name and phone if either has changed
        if local.name != remote.name || !contact::phone_number_matches(&local.phone, &remote.phone)
        {
            local.name = remote.name.clone();
            local.phone = remote.phone.clone();
            local.updated_at = remote.updated_at;
        }

        local.email = remote.email.clone();
        local.tag = remote.tag.clone();

        // Handle deletion
        if remote.deleted {
            local.deleted = true;
        }

        local.updated_at = remote.updated_at;
    }

    pub fn is_duplicate(&self, contacts: &HashMap<Uuid, Contact>, remote: &Contact) -> bool {
        contacts.values().any(|c| c == remote)
    }
}

impl ContactManager {
    pub fn new() -> Result<Self, AppError> {
        let storage = storage::parse_storage_type_env_config(None)?;

        let mut manager = Self {
            mem: HashMap::new(),
            storage,
            index: Index {
                name: HashMap::new(),
                domain: HashMap::new(),
            },
        };
        manager.load()?;
        manager.index = Index::new(&manager)?;

        if manager.storage.get_medium() == "txt" {
            manager.migrate_from_storage(&JsonStorage::new()?)?;
        } else {
            manager.migrate_from_storage(&TxtStorage::new()?)?;
        }
        Ok(manager)
    }

    pub fn contact_list(&self) -> Vec<&Contact> {
        self.mem
            .values()
            .filter(|&c| !c.deleted)
            .collect::<Vec<&Contact>>()
    }

    pub fn get_ids_by_name(&self, name: &str) -> Option<Vec<Uuid>> {
        let names = name.split_ascii_whitespace();

        let index = &self.index;
        let mut ids_as_set: HashSet<Uuid> = HashSet::new();

        for name_slice in names {
            let ids = index.name.get(&name_slice.to_ascii_lowercase())?;
            ids_as_set = ids_as_set.union(ids).copied().collect()
        }

        let ids: Vec<Uuid> = ids_as_set
            .iter()
            .filter_map(|&id| {
                self.mem.get(&id).and_then(|contact| {
                    if contact.name.eq_ignore_ascii_case(name) && !contact.deleted {
                        Some(id)
                    } else {
                        None
                    }
                })
            })
            .collect();

        if ids.is_empty() { None } else { Some(ids) }
    }

    pub fn add_contact(&mut self, contact: Contact) {
        self.index
            .update_both_indexes(&contact, &IndexUpdateType::Add);

        self.mem.insert(contact.id, contact);
    }

    pub fn delete_contact(&mut self, id: &Uuid) -> Result<(), AppError> {
        match self.mem.get_mut(id) {
            Some(deleted_contact) => {
                deleted_contact.deleted = true;
                deleted_contact.updated_at = Utc::now();
                self.index
                    .update_both_indexes(deleted_contact, &IndexUpdateType::Remove);
                Ok(())
            }
            None => Err(AppError::NotFound("Contact".to_string())),
        }
    }
    /// Permanently remove contacts that were soft-deleted more than `days` days ago.
    /// Returns the number of contacts removed.
    pub fn purge_soft_deleted_older_than(&mut self, days: i64) {
        let now = Utc::now().date_naive();
        let cutoff_date = now - Duration::days(days);

        // Collect ids to remove to avoid mutating the map while iterating
        let to_remove: Vec<Uuid> = self
            .mem
            .iter()
            .filter_map(|(&id, contact)| {
                let contact_date = contact.updated_at.date_naive();
                if contact.deleted && contact_date <= cutoff_date {
                    Some(id)
                } else {
                    None
                }
            })
            .collect();

        // Remove contacts
        for id in &to_remove {
            self.mem.remove(id);
        }
    }

    pub fn migrate_from_storage(&mut self, storage: &dyn ContactStore) -> Result<(), AppError> {
        let contacts = storage.load()?;

        for contact in contacts.values() {
            self.index
                .update_both_indexes(contact, &IndexUpdateType::Add);
            self.mem.insert(contact.id, contact.clone());
        }

        Ok(())
    }

    pub fn load(&mut self) -> Result<(), AppError> {
        self.mem = self.storage.load()?;
        Ok(())
    }

    pub fn save(&mut self) -> Result<(), AppError> {
        // Purge soft-deleted contacts older than configured days before persisting.
        // Default: 1 days.
        let purge_days: i64 = helper::get_env_value_by_key("PURGE_DAYS")
            .ok()
            .and_then(|s| s.parse::<i64>().ok())
            .unwrap_or(1);

        self.purge_soft_deleted_older_than(purge_days);

        self.storage.save(&self.mem)
    }

    pub fn import_contacts_from_storage(
        &mut self,
        storage: Box<dyn ContactStore>,
    ) -> Result<(), AppError> {
        let mut base = self.mem.clone();

        let sync_status = self.sync_from_storage(
            &mut base,
            storage,
            SyncPolicy::LastWriteWinsPolicy(LastWriteWinsPolicy),
        );

        if sync_status.is_err() {
            self.save()?; // rollback to previous state on error

            return Err(sync_status.err().unwrap());
        } else {
            self.mem = base;
            self.index = Index::new(self)?
        }

        let mut saved: Result<(), AppError> = Err(AppError::Synchronization(
            "Error saving synced data".to_string(),
        ));

        for _ in 0..3 {
            saved = self.save();
            if saved.is_ok() {
                break;
            }
        }
        if saved.is_err() {
            return Err(AppError::Synchronization(
                "Error saving synced data".to_string(),
            ));
        }
        Ok(())
    }

    pub fn export_contacts_to_storage(
        &self,
        storage: Box<dyn ContactStore>,
    ) -> Result<(), AppError> {
        storage.save(&self.mem)
    }

    pub fn sync_from_storage(
        &self,
        base: &mut HashMap<Uuid, Contact>,
        storage: Box<dyn ContactStore>,
        policy: SyncPolicy,
    ) -> Result<(), AppError> {
        let mut remote_contacts = storage.load()?;

        let SyncPolicy::LastWriteWinsPolicy(policy) = policy;

        for remote_contact in remote_contacts.values_mut() {
            // Check if contact exist in local storage
            if let Some(local_contact) = base.get_mut(&remote_contact.id) {
                // Confirm contact were created the same time to be sure they are the same
                if !policy.verify_match(local_contact, remote_contact) {
                    return Err(AppError::Synchronization(format!(
                        "Date conflict for contact:{{{}, {}}}",
                        remote_contact.name, remote_contact.phone
                    )));
                }

                match policy.conflict_resolution(local_contact, remote_contact) {
                    SyncDecision::LocalWins => continue,
                    SyncDecision::RemoteWins => policy.merge_changes(local_contact, remote_contact),
                }
            } else {
                // Ignore duplicate contacts (same name && phone)
                if policy.is_duplicate(base, remote_contact) {
                    continue;
                }

                base.insert(remote_contact.id, remote_contact.clone());
            }
        }
        Ok(())
    }

    pub fn create_name_search_index(&self) -> Result<HashMap<String, HashSet<Uuid>>, AppError> {
        let index: Arc<Mutex<HashMap<String, HashSet<Uuid>>>> =
            Arc::new(Mutex::new(HashMap::new()));
        let contact_list = Arc::new(self.contact_list());

        let length = contact_list.len();
        if length < 1 {
            return Ok(Arc::into_inner(index).unwrap_or_default().into_inner()?);
        }

        let worker_threads = determine_num_of_workers_thread_for_a_work_size(length);

        thread::scope(|s| {
            for i in 1..=worker_threads {
                let map1 = Arc::clone(&index);
                let contact_list = Arc::clone(&contact_list);

                s.spawn(move || -> Result<(), AppError> {
                    let mut local_map: HashMap<String, HashSet<Uuid>> = HashMap::new();

                    let (start, end) =
                        allocate_work_size_for_single_thread(i, length, worker_threads);

                    for contact in &contact_list[start..end] {
                        // All parts of the contact name (seperated by space) is inserted as a new key
                        // To ensure that searching any part of a contact name (not just the first name) will also
                        // provide the expected contact
                        let contact_names: Vec<&str> =
                            contact.name.split_ascii_whitespace().collect();

                        for name in contact_names {
                            local_map
                                .entry(name.to_ascii_lowercase())
                                .or_default()
                                .insert(contact.id);
                        }
                    }

                    if !local_map.is_empty() {
                        let mut map1_lock = map1.lock()?;
                        map1_lock.extend(local_map);
                    }

                    Ok(())
                });
            }
        });

        let result = Arc::into_inner(index).unwrap_or_default().into_inner()?;
        Ok(result)
    }

    pub fn create_email_domain_search_index(
        &self,
    ) -> Result<HashMap<String, HashSet<Uuid>>, AppError> {
        let index: Arc<Mutex<HashMap<String, HashSet<Uuid>>>> =
            Arc::new(Mutex::new(HashMap::new()));
        let contact_list = Arc::new(self.contact_list());

        let length = contact_list.len();
        if length < 1 {
            return Ok(Arc::into_inner(index).unwrap_or_default().into_inner()?);
        }

        let worker_threads: usize = determine_num_of_workers_thread_for_a_work_size(length);

        thread::scope(|s| {
            for i in 1..=worker_threads {
                let map1 = Arc::clone(&index);
                let contact_list = Arc::clone(&contact_list);

                s.spawn(move || -> Result<(), AppError> {
                    let mut local_map: HashMap<String, HashSet<Uuid>> = HashMap::new();
                    let (start, end) =
                        allocate_work_size_for_single_thread(i, length, worker_threads);

                    for contact in &contact_list[start..end] {
                        let email_parts: Vec<&str> = contact.email.split('@').collect();
                        let domain = email_parts[email_parts.len() - 1].to_string();

                        local_map
                            .entry(domain.to_ascii_lowercase())
                            .or_default()
                            .insert(contact.id);
                    }

                    if !local_map.is_empty() {
                        let mut map1_lock = map1.lock()?;
                        map1_lock.extend(local_map);
                    }
                    Ok(())
                });
            }
        });

        let result = Arc::into_inner(index).unwrap_or_default().into_inner()?;
        Ok(result)
    }

    pub fn fuzzy_search_name(&self, name: &str) -> Result<Vec<&Contact>, AppError> {
        let max_search_length: u8 = 30;
        let top_results: usize = 10;
        let name = Arc::new(name.trim().to_ascii_lowercase());

        if name.is_empty() {
            return Err(AppError::Validation("No Name provided".to_string()));
        }
        if name.len() > max_search_length as usize {
            return Err(AppError::Validation("Search string too long".to_string()));
        }

        let min_distance: f32 = 0.4;

        let contact_list = Arc::new(self.contact_list());
        let length = contact_list.len();

        if length < 1 {
            return Ok(Vec::new());
        }

        let worker_threads: usize = determine_num_of_workers_thread_for_a_work_size(length);

        let fuzzy_match_contact_set: Arc<Mutex<HashSet<(i32, &Contact)>>> = Arc::new(
            Mutex::new(HashSet::new()), // This would hold &contact and the corresponding Lavenchtine distance of all contact that passes the MIN_DISTANCE threshold.
        );

        thread::scope(|s| {
            for i in 1..=worker_threads {
                let name = Arc::clone(&name);
                let fuzzy_match_contact_set = Arc::clone(&fuzzy_match_contact_set);
                let contact_list = Arc::clone(&contact_list);

                s.spawn(move || -> Result<(), AppError> {
                    let (start, end) =
                        allocate_work_size_for_single_thread(i, length, worker_threads);

                    // collect matches locally to reduce lock contention
                    let mut local_matches: Vec<(i32, &Contact)> = Vec::new();

                    for &contact in &contact_list[start..end] {
                        // HashSet values must implement Eq and Hash traits. Float does not implement the Eq and Hash trait
                        // That is the reason we are using a tuple of i32 instead of float (i32, &Contact) here
                        // fuzzy_compare() returns a f32 value ranging from 0.0 to 1.0. To convert it to i32 for hashing and Eqality, we multiply by 1000.0

                        let distance = (fuzzy_compare(&contact.name.to_ascii_lowercase(), &name)
                            * 1000.0) as i32;

                        if distance >= (min_distance * 1000.0) as i32 {
                            local_matches.push((distance, contact));
                        }
                    }

                    // Merge local matches into the shared set
                    if !local_matches.is_empty() {
                        let mut matches = fuzzy_match_contact_set.lock()?;
                        matches.extend(local_matches);
                    }

                    Ok(())
                });
            }
        });

        // get the data of the Arc (Arc::into_inner()) a Mutex data, then get the value of the Mutex (.into_inner())
        let result = Arc::into_inner(fuzzy_match_contact_set)
            .unwrap_or_default()
            .into_inner()?;

        // Sort by top 10 highest distance value
        let mut result = result.into_iter().collect::<Vec<(i32, &Contact)>>();

        result.sort_by_key(|&(dist, _)| std::cmp::Reverse(dist));

        let result = result
            .iter()
            .take(top_results)
            .map(|(_, contact)| *contact)
            .collect::<Vec<&Contact>>();
        Ok(result)
    }

    pub fn fuzzy_search_email_domain_index(&self, domain: &str) -> Result<Vec<&Contact>, AppError> {
        let max_search_length: u8 = 15;
        let domain = &domain.trim().to_lowercase();

        if domain.is_empty() {
            return Err(AppError::Validation("No email domain provided".to_string()));
        }
        if domain.len() > max_search_length as usize {
            return Err(AppError::Validation(
                "Please provide just email domain Eg. \"example.com\"".to_string(),
            ));
        }

        let index = &self.index;

        // using &self in thread produces and error:
        // "`(dyn store::ContactStore + 'static)` cannot be shared between threads safely" because of storage field
        // Hence I try to borrow the data will be using from self below, instead of using self in thread
        let contacts_map = Arc::new(&self.mem);

        let default_set: HashSet<Uuid> = HashSet::new();

        let ids_as_set = index.domain.get(domain).unwrap_or(&default_set);
        let index_match = Arc::new(ids_as_set.iter().collect::<Vec<&Uuid>>()); // Convert to Vec

        let length = index_match.len();
        if length < 1 {
            return Ok(Vec::new());
        }

        let worker_threads: usize = determine_num_of_workers_thread_for_a_work_size(length);

        let fuzzy_match: Arc<Mutex<Vec<&Contact>>> = Arc::new(Mutex::new(Vec::new()));

        thread::scope(|s| {
            for i in 1..=worker_threads {
                let match1 = Arc::clone(&fuzzy_match);
                let contacts_map = Arc::clone(&contacts_map);
                let uuids = Arc::clone(&index_match);

                s.spawn(move || -> Result<(), AppError> {
                    let (start, end) =
                        allocate_work_size_for_single_thread(i, length, worker_threads);
                    let mut local_set: HashSet<&Contact> = HashSet::new();

                    for &id in &uuids[start..end] {
                        if let Some(contact) = contacts_map.get(id) {
                            local_set.insert(contact);
                        }
                    }

                    let to_vec = local_set.into_iter().collect::<Vec<&Contact>>();
                    if !to_vec.is_empty() {
                        let mut matches = match1.lock()?;
                        matches.extend(to_vec);
                    }

                    Ok(())
                });
            }
        });

        // get the data of the Arc (Arc::into_inner()) a Metex data, the get the value of the Mutex (.into_inner())
        let result = Arc::into_inner(fuzzy_match)
            .unwrap_or_default()
            .into_inner()?;
        Ok(result)
    }
}

fn determine_num_of_workers_thread_for_a_work_size(work_length: usize) -> usize {
    if work_length < 1 {
        return 0;
    }
    match work_length {
        0..=100 => 1,
        101..=200 => 2,
        201..=500 => 3,
        501..=1000 => 4,
        _ => MAX_WORKER_THREADS,
    }
}
fn allocate_work_size_for_single_thread(
    current_thread: usize,
    total_work_length: usize,
    total_workers_thread: usize,
) -> (usize, usize) {
    let average_work_per_thread = total_work_length / total_workers_thread;

    // Get next starting index multiplying chunk with current iteration
    let start = average_work_per_thread * (current_thread - 1); // -1 to start from index zero and also catch unincluded end index from previous iteration

    let end: usize = if current_thread == total_workers_thread {
        // Last thread takes the remainder if any
        (average_work_per_thread * current_thread).max(total_work_length)
    } else {
        average_work_per_thread * current_thread
    };

    (start, end)
}

#[cfg(test)]
mod tests {
    use chrono::Utc;

    use super::*;

    #[test]
    fn adds_persistent_contact_with_txt() -> Result<(), AppError> {
        let storage_backend = Box::new(TxtStorage::new()?);
        let mut storage = ContactManager {
            mem: HashMap::new(),
            storage: storage_backend,
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
        storage.save()?;
        storage.mem.clear();
        storage.load()?;
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
        storage.save()?;
        Ok(())
    }

    #[test]
    fn delete_persistent_contact_with_txt() -> Result<(), AppError> {
        let storage_backend = Box::new(TxtStorage::new()?);
        let mut storage = ContactManager {
            mem: HashMap::new(),
            storage: storage_backend,
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

        storage.save()?;
        storage.mem.clear();

        storage.load()?;
        storage.index = Index::new(&storage)?;

        let index = storage
            .get_ids_by_name(&"Uche".to_string())
            .unwrap_or_default();
        storage.delete_contact(&index[0])?; // delete contact1 (Soft delete)
        storage.save()?;

        storage.mem.clear();
        storage.load()?;
        storage.index = Index::new(&storage)?;

        assert_eq!(storage.mem.len(), 2); // contact is soft deleted
        assert!(storage.mem.get(&index[0]).unwrap().deleted);

        storage.mem.clear();
        storage.save()?;

        Ok(())
    }

    #[test]
    fn json_store_is_persistent() -> Result<(), AppError> {
        let mut storage = ContactManager::new()?;

        let created = Utc::now();
        let id_1 = Uuid::new_v4();
        let id_2 = Uuid::new_v4();

        let contact1 = Contact {
            id: id_1.clone(),
            name: "Uche".to_string(),
            phone: "01234567890".to_string(),
            email: "ucheuche@gmail.com".to_string(),
            tag: "".to_string(),
            deleted: false,
            created_at: created.clone(),
            updated_at: created.clone(),
        };

        let contact2 = Contact {
            id: id_2,
            name: "Alex".to_string(),
            phone: "01234567890".to_string(),
            email: "".to_string(),
            tag: "".to_string(),
            deleted: false,
            created_at: created.clone(),
            updated_at: created.clone(),
        };

        storage.add_contact(contact1);
        storage.add_contact(contact2);

        storage.save()?;
        storage.mem.clear();

        storage.load()?;
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
        storage.save()?;

        storage.mem.clear();
        storage.load()?;
        storage.index = Index::new(&storage)?;

        assert_eq!(storage.mem.len(), 2); // contact is soft deleted

        assert!(storage.mem.get(&id_1).unwrap().deleted);

        storage.mem.clear();
        storage.save()?;

        Ok(())
    }

    #[test]
    fn migrates_contact() -> Result<(), AppError> {
        let mut txt_store = ContactManager {
            mem: HashMap::new(),
            storage: Box::new(TxtStorage::new()?),
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
        txt_store.save()?;
        txt_store.mem.clear();

        let mut json_store = ContactManager::new()?;

        // json_store.mem = json_store.load()?;

        json_store.add_contact(contact2);
        json_store.save()?;
        json_store.mem.clear();

        json_store.load()?;
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
        json_store.save()?;

        txt_store.mem.clear();
        txt_store.save()?;

        Ok(())
    }

    #[test]
    fn index_updates_on_add_and_delete() -> Result<(), AppError> {
        let mut store = ContactManager::new()?;

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
    fn index_updates_on_edit() -> Result<(), AppError> {
        let mut store = ContactManager::new()?;

        let contact = Contact::new(
            "John Doe".to_string(),
            "01234567890".to_string(),
            "john@example.com".to_string(),
            "".to_string(),
        );
        let id = contact.id;
        store.add_contact(contact.clone());

        // Verify initial index state
        if let Some(ids_for_john) = store.index.name.get("john") {
            assert!(ids_for_john.contains(&id));
        }
        if let Some(ids_for_doe) = store.index.name.get("doe") {
            assert!(ids_for_doe.contains(&id));
        }
        let ids_for_jane = store.index.name.get("jane");
        assert!(ids_for_jane.is_none());

        if let Some(domain_set) = store.index.domain.get("example.com") {
            assert!(domain_set.contains(&id));
        }
        let domain_set = store.index.domain.get("new.com");
        assert!(domain_set.is_none());

        // Simulate edit: change name to "Jane Doe" and email to "jane@new.com"
        let contact_mut = store.mem.get_mut(&id).unwrap();
        // Remove old name and email from index
        store
            .index
            .updated_name_index(contact_mut, &IndexUpdateType::Remove);
        store
            .index
            .update_domain_index(contact_mut, &IndexUpdateType::Remove);
        // Update contact fields
        contact_mut.name = "Jane Doe".to_string();
        contact_mut.email = "jane@new.com".to_string();
        contact_mut.updated_at = Utc::now();
        // Add new name and email to index
        store
            .index
            .update_both_indexes(contact_mut, &IndexUpdateType::Add);

        // Verify updated index state
        let ids_for_john_after = store.index.name.get("john");
        assert!(ids_for_john_after.is_none());

        if let Some(ids_for_doe_after) = store.index.name.get("Doe") {
            assert!(ids_for_doe_after.contains(&id)); // "Doe" should still be there
        }

        if let Some(ids_for_jane_after) = store.index.name.get("Jane") {
            assert!(ids_for_jane_after.contains(&id));
        }

        let domain_set = store.index.domain.get("example.com");
        assert!(domain_set.is_none());

        if let Some(domain_set) = store.index.domain.get("new.com") {
            assert!(domain_set.contains(&id));
        }

        store.mem.clear();
        Ok(())
    }

    #[test]
    fn fuzzy_search_name_matches_on_partial() -> Result<(), AppError> {
        let mut store = ContactManager::new()?;

        let contact = Contact::new(
            "Uche Johnson".to_string(),
            "01234567890".to_string(),
            "uche@example.com".to_string(),
            "".to_string(),
        );
        let expected_name = contact.name.clone();
        store.add_contact(contact);

        // Search by a portion of the name (partial)
        let results = store.fuzzy_search_name("uche j")?;
        assert!(!results.is_empty());
        assert!(results.iter().any(|c| c.name == expected_name));

        let results = store.fuzzy_search_name("johnson")?;
        assert!(!results.is_empty());
        assert!(results.iter().any(|c| c.name == expected_name));

        // Cleanup persistent files
        store.mem.clear();
        Ok(())
    }

    #[test]
    fn fuzzy_search_email_domain_returns_contact() -> Result<(), AppError> {
        let mut store = ContactManager::new()?;

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
