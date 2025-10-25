use std::hint::black_box;

use criterion::{criterion_group, criterion_main, Criterion};

use rusty_rolodex::{prelude::{
    Contact, Store, contact::{self, phone_number_matches},
    command::SearchKey,
}, store::ContactStore};



fn bech_add(c: &mut Criterion) {
    c.bench_function("Adding 100k contact", |b| {
        let mut storage = Store::new().expect("Store not created");
        storage.mem = storage.load().expect("Failed to load");
        
        b.iter(|| {

            storage.mem.clear();
            let mut counter = 0;

            for _ in 0..100_000{
                counter += 1;
                let name = "Zoe";
                let phone = "08885499529";
                let email = "bryanwelch@gmail.com";
                let tag = ["friends", "work"];

                let new_contact = Contact::new(
                    name.to_string(),
                    phone.to_string(),
                    email.to_string(),
                    tag[counter % 2].to_string(),
                );

                if !new_contact.validate_name().expect("Name Validation failed") {
                    return;
                }

                if !new_contact.validate_number().expect("Name Validation failed") {
                    return;
                }

                if !new_contact.validate_email().expect("Name Validation failed") {
                    return;
                }

                storage.add_contact(new_contact);

                storage.save(&storage.mem).expect("Failue to save contacts");

            }
            
            
        });

    });
}

fn bench_list(c: &mut Criterion){

    c.bench_function("listing 100k contact", |b| {
        let mut storage = Store::new().expect("Store not created");
        storage.mem = storage.load().expect("Failed to load");
        storage.mem.clear();
        let created_at = contact::Utc::now();
        
        storage.mem = (0..100_000)
            .map(|i| Contact {
                name: format!("User{i}"),
                phone: format!("08885499529"),
                email: format!("user{i}@yahoo.com"),
                tag: ["friends", "work"][i % 2].to_string(),
                created_at: created_at.clone(),
                updated_at: created_at.clone()
            })
            .collect();

        storage.save(&storage.mem).expect("Failed to save");

        b.iter( || {
            
            let mut contact_list = storage.contact_list();

            if contact_list.is_empty() {
                println!("No contact yet");
                return;
            }

            contact_list.sort_by(|a, b| a.email.to_lowercase().cmp(&b.email.to_lowercase()));
                    

            contact_list.reverse();

            let filtered_contacts: Vec<&Contact> = contact_list
                .iter()
                .filter(|&c| c.tag.to_lowercase() == "friends".to_lowercase())
                .map(|c| *c)
                .collect();

            black_box(filtered_contacts);

        });
        
            
    });

}


fn bench_search(c: &mut Criterion){

    c.bench_function("Searching 100k contact", |b| {
        let mut storage = Store::new().expect("Store not created");
        storage.mem = storage.load().expect("Failed to load");
        storage.mem.clear();
        let created_at = contact::Utc::now();
        
        storage.mem = (0..100_000)
            .map(|i| Contact {
                name: format!("User{i}"),
                phone: format!("08885499529"),
                email: format!("user{i}@yahoo.com"),
                tag: ["friends", "work"][i % 2].to_string(),
                created_at: created_at.clone(),
                updated_at: created_at.clone()
            })
            .collect();

        storage.save(&storage.mem).expect("Failed to save");

        b.iter(|| {

            let name = "Zeo";
            let by = None;
            let domain: Option<String> = None;

            let search_by = by.unwrap_or(SearchKey::N);


            match search_by {
                // Search using email domain
                SearchKey::D => {
                    // user's provided email strig is assigned to "search_for"
                    let searched_for = domain.unwrap_or_default();
                    

                    let result = storage.fuzzy_search_email_domain_index(&searched_for).expect("Failed to search domain");
                    black_box(result);
                }
                _ => {
                    // Default to search by name
                    let searched_for = name;
                
                    let result = storage.fuzzy_search_name_index(searched_for).expect("Failed to search name");
                    
                    black_box(result);
                }
            }
            
            
        });
    });

}


fn bench_edit(c: &mut Criterion){

    c.bench_function("Editing 100k contact", |b| {
        let mut storage = Store::new().expect("Store not created");
        storage.mem = storage.load().expect("Failed to load");
        storage.mem.clear();
        let created_at = contact::Utc::now();
        
        storage.mem = (0..105_000)
            .map(|i| Contact {
                name: format!("User{i}"),
                phone: format!("08885499529"),
                email: format!("user{i}@yahoo.com"),
                tag: ["friends", "work"][i % 2].to_string(),
                created_at: created_at.clone(),
                updated_at: created_at.clone()
            })
            .collect();

        storage.save(&storage.mem).expect("Failed to save");

        b.iter(|| {
            let revers_counter = 100_000;
            for i in 0..100_000 {

                let name = format!("User{i}");
                let phone = "08885499529";

                let new_name = format!("User{}", revers_counter-i);
                let new_phone = "08885499520";
                let new_email = format!("user{}@yahoo.com", revers_counter - i);
                let new_tag = ["friends", "work"];

                let desired_contact = Contact::new(
                    name,
                    phone.to_string(),
                    "".to_string(),
                    "".to_string(),
                );

                let found_contact = storage.mem
                    .iter_mut()
                    .find(|c| **c == desired_contact);

                if let Some(contact) = found_contact {
                    contact.name = new_name;

                    contact.phone = new_phone.to_string();

                    contact.email = new_email;

                    contact.tag = new_tag[i % 2].to_string();

                    contact.updated_at = contact::Utc::now();

                } else {
                    println!("No match");
                    return;
                }

                storage.save(&storage.mem).expect("Failed to save");
            }
            
        });
    });

}


fn bench_delete(c: &mut Criterion){

    c.bench_function("Deleting 100k contact", |b| {
        let mut storage = Store::new().expect("Store not created");
        storage.mem.clear();
        let created_at = contact::Utc::now();
        
        storage.mem = (0..105_000)
            .map(|i| Contact {
                name: format!("User{i}"),
                phone: format!("08885499529"),
                email: format!("user{i}@yahoo.com"),
                tag: ["friends", "work"][i % 2].to_string(),
                created_at: created_at.clone(),
                updated_at: created_at.clone()
            })
            .collect();

        storage.save(&storage.mem).expect("Failed to save");


        b.iter(|| {

            for i in 0..100_000 {

                let name = format!("User{i}");
                let phone = "+2348885499529";

                let indices = storage.get_indices_by_name(&name);

                match indices {
                    Some(indices) => {
                        if indices.len() > 1 {
                            if phone.is_empty() {
                                println!("No name match");
                                continue;
                            } else {
                                for index in indices {
                                    let contact = storage.contact_list()[index];
                                    if contact.name == name
                                        && phone_number_matches(&contact.phone, phone)
                                    {
                                        storage.delete_contact(index).expect("Failed to delete contact");
                                        storage.save(&storage.mem).expect("Failed to save");
                                        continue;
                                    }
                                }
                                println!("No number match");
                                continue;
                            }
                        } else {
                            storage.delete_contact(indices[0]).expect("Failed to delete contact");
                        }

                        storage.save(&storage.mem).expect("Failed to save");
                        continue;
                    }
                    None => {
                        println!("not found");
                        continue;
                    }
                }
            }
            
        });

    });

}


fn configure() -> Criterion {
    Criterion::default()
        .without_plots() 
        .sample_size(10) 
        .measurement_time(std::time::Duration::from_secs(2))
}

criterion_group!{
    name = benches;
    config = configure();
    targets = bech_add, bench_list, bench_edit, bench_search, bench_delete
}
criterion_main!(benches);