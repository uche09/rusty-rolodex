use criterion::{BatchSize, Criterion, criterion_group, criterion_main};
use rusty_rolodex::domain::contact::phone_number_matches;
use std::hint::black_box;

use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use rusty_rolodex::prelude::{Contact, ContactManager, contact, manager::IndexUpdateType};
use std::fs;
use std::path::PathBuf;
use uuid::Uuid as BenchUuid;

fn make_store_with_n(n: usize) -> ContactManager {
    let mut rng = StdRng::seed_from_u64(42); // Seeded for reproducibility in benchmarks

    // Randomized and more realistic data pools
    let first_names = vec![
        "Alice", "Bob", "Charlie", "Diana", "Eve", "Frank", "Grace", "Henry", "Ivy", "Jack",
        "Kate", "Liam", "Mia", "Noah", "Olivia", "Peter", "Quinn", "Ryan", "Sophia", "Tyler",
        "Uma", "Victor", "Wendy", "Xander", "Yara", "Zoe",
    ];
    let last_names = vec![
        "Smith",
        "Johnson",
        "Williams",
        "Brown",
        "Jones",
        "Garcia",
        "Miller",
        "Davis",
        "Rodriguez",
        "Martinez",
        "Hernandez",
        "Lopez",
        "Gonzalez",
        "Wilson",
        "Anderson",
        "Thomas",
        "Taylor",
        "Moore",
        "Jackson",
        "Martin",
    ];
    let domains = vec![
        "gmail.com",
        "yahoo.com",
        "hotmail.com",
        "outlook.com",
        "example.com",
        "test.org",
        "company.net",
    ];
    let tags = vec![
        "friends",
        "work",
        "family",
        "acquaintance",
        "colleague",
        "neighbor",
        "",
    ];

    let mut storage = ContactManager::new().expect("Store not created");
    storage.mem = (0..n)
        .map(|_| {
            let first = first_names[rng.gen_range(0..first_names.len())];
            let last = last_names[rng.gen_range(0..last_names.len())];
            let name = format!("{} {}", first, last);
            let email_domain = domains[rng.gen_range(0..domains.len())];
            let email = format!(
                "{}.{}@{}",
                first.to_lowercase(),
                last.to_lowercase(),
                email_domain
            );
            let phone = format!("{:010}", rng.gen_range(1000000000..9999999999u64)); // Random 10-digit phone
            let tag = tags[rng.gen_range(0..tags.len())].to_string();
            let contact = Contact::new(name, phone, email, tag);

            (contact.id, contact)
        })
        .collect();
    // storage.index = Index::new(&storage).expect("index build");
    storage
}

fn bech_add(c: &mut Criterion) {
    c.bench_function("Adding 20k contact (in-memory single add)", |b| {
        b.iter_batched(
            || make_store_with_n(20_000),
            |mut storage| {
                let new_contact = Contact::new(
                    "Zoe".to_string(),
                    "08885499529".to_string(),
                    "bryanwelch@gmail.com".to_string(),
                    "friends".to_string(),
                );
                storage.add_contact(new_contact);
                black_box(&storage.mem);
            },
            BatchSize::SmallInput,
        );
    });
}

fn bench_list(c: &mut Criterion) {
    c.bench_function("listing 20k contact (collect + sort + filter)", |b| {
        let storage = make_store_with_n(20_000);
        b.iter(|| {
            let mut filtered_contacts: Vec<&Contact> = storage
                .mem
                .iter()
                .filter_map(|(_, cont)| {
                    if cont.tag.to_lowercase() == "friends" {
                        Some(cont)
                    } else {
                        None
                    }
                })
                .collect();
            filtered_contacts.sort_by(|a, b| a.email.to_lowercase().cmp(&b.email.to_lowercase()));
            filtered_contacts.reverse();

            black_box(filtered_contacts);
        });
    });
}

fn bench_search(c: &mut Criterion) {
    c.bench_function("Searching 20k contact (single fuzzy search)", |b| {
        let storage = make_store_with_n(20_000);
        b.iter(|| {
            let result = storage.fuzzy_search_name("zoe").expect("search failed");
            black_box(result);
        });
    });
}

fn bench_edit(c: &mut Criterion) {
    c.bench_function("Editing 20k contact (single edit)", |b| {
        b.iter_batched(
            || {
                let mut storage = make_store_with_n(20_000);

                let new_contact = Contact::new(
                    "Zoe".to_string(),
                    "08885499529".to_string(),
                    "bryanwelch@gmail.com".to_string(),
                    "friends".to_string(),
                );
                storage.add_contact(new_contact);
                storage
            },
            |mut storage| {
                let sample_name = "zoe";
                let sample_phone = "08885499529";
                if let Some(ids) = storage.get_ids_by_name(sample_name) {
                    for id in ids {
                        if let Some(contact) = storage.mem.get_mut(&id)
                            && phone_number_matches(&contact.phone, sample_phone)
                        {
                            storage
                                .index
                                .updated_name_index(contact, &IndexUpdateType::Remove);

                            contact.name = format!("{}-edited", contact.name);
                            contact.updated_at = contact::Utc::now();

                            storage
                                .index
                                .updated_name_index(contact, &IndexUpdateType::Add);
                            black_box(contact);
                            continue;
                        }
                    }
                }
            },
            BatchSize::SmallInput,
        );
    });
}

fn bench_delete(c: &mut Criterion) {
    c.bench_function("Deleting 20k contact (single delete)", |b| {
        b.iter_batched(
            || {
                let mut storage = make_store_with_n(20_000);

                let new_contact = Contact::new(
                    "Zoe".to_string(),
                    "08885499529".to_string(),
                    "bryanwelch@gmail.com".to_string(),
                    "friends".to_string(),
                );
                storage.add_contact(new_contact);
                storage
            },
            |mut storage| {
                let sample_name = "Zoe";
                let sample_phone = "08885499529";
                if let Some(ids) = storage.get_ids_by_name(sample_name) {
                    for id in ids {
                        if let Some(contact) = storage.mem.get(&id)
                            && phone_number_matches(&contact.phone, sample_phone)
                        {
                            let _ = storage.delete_contact(&id);
                            black_box(&storage.mem);
                        }
                    }
                }
            },
            BatchSize::SmallInput,
        );
    });
}

fn bench_increment_index(c: &mut Criterion) {
    c.bench_function("Increment index for 20k store", |b| {
        b.iter_batched(
            || make_store_with_n(20_000),
            |mut storage| {
                let new_contact = Contact::new(
                    "NewUser".to_string(),
                    "08885499529".to_string(),
                    "newuser@example.com".to_string(),
                    "test".to_string(),
                );
                storage
                    .index
                    .update_both_indexes(&new_contact, &IndexUpdateType::Add);
                black_box(&storage.index);
            },
            BatchSize::SmallInput,
        );
    });
}

fn bench_decrement_index(c: &mut Criterion) {
    c.bench_function("Decrement index for 20k store", |b| {
        b.iter_batched(
            || make_store_with_n(20_000),
            |mut storage| {
                // Take the first contact from the store to decrement
                if let Some((_, contact)) = storage.mem.iter().next() {
                    let contact_clone = (*contact).clone();
                    storage
                        .index
                        .update_both_indexes(&contact_clone, &IndexUpdateType::Remove);
                    black_box(&storage.index);
                }
            },
            BatchSize::SmallInput,
        );
    });
}

// IO
fn bench_save_store_json(c: &mut Criterion) {
    c.bench_function("save_20k_json_contacts", |b| {
        b.iter_batched(
            || {
                // Setup: create temp dir and enter it
                let base = std::env::temp_dir()
                    .join(format!("rusty-rolodex-bench-json-{}", BenchUuid::new_v4()));
                fs::create_dir_all(&base).expect("create temp dir");

                // chdir into temp dir so relative storage path is isolated
                std::env::set_current_dir(&base).expect("chdir into tempdir");

                unsafe {
                    std::env::set_var("STORAGE_CHOICE", "json");
                }

                // Build store in setup (excluded from measured timing)
                let storage = make_store_with_n(20_000);

                (storage, base)
            },
            |(storage, base)| {
                // Measured: call Store::save (timed)
                let _ = storage.save();

                // Restore original cwd (robustly) BEFORE removing temp dir
                restore_to_manifest();

                // Now remove the temporary directory (best-effort)
                let _ = fs::remove_dir_all(&base);
                black_box(&storage.mem);
            },
            BatchSize::SmallInput,
        );
    });
}

fn bench_read_store_json(c: &mut Criterion) {
    c.bench_function("read_20k_json_contacts", |b| {
        b.iter_batched(
            || {
                // Setup: create temp dir and enter it
                let base = std::env::temp_dir()
                    .join(format!("rusty-rolodex-bench-json-{}", BenchUuid::new_v4()));
                fs::create_dir_all(&base).expect("create temp dir");
                let original_cwd = std::env::current_dir().expect("cwd");

                std::env::set_current_dir(&base).expect("chdir into tempdir");
                unsafe {
                    std::env::set_var("STORAGE_CHOICE", "json");
                }

                // Build and save the store so there's something to load
                let storage = make_store_with_n(20_000);
                storage.save().expect("setup save failed");

                // restore cwd so setup leaves global state clean; measured closure will chdir into base
                std::env::set_current_dir(&original_cwd).expect("restore cwd after setup");

                base
            },
            |base| {
                // Measured: chdir into temp dir and call load()
                if let Err(e) = std::env::set_current_dir(&base) {
                    eprintln!("warning: failed to chdir into bench base: {}", e);
                    return;
                }

                // Create a Store instance that picks up JSON path via env var/current dir
                let mut manager = ContactManager::new().expect("failed to create store for load");
                let _ = manager.load();

                // Restore cwd to manifest dir BEFORE removing tempdir
                restore_to_manifest();

                let _ = std::fs::remove_dir_all(&base);
            },
            BatchSize::SmallInput,
        );
    });
}

fn bench_save_store_txt(c: &mut Criterion) {
    c.bench_function("save_20k_txt_contacts", |b| {
        b.iter_batched(
            || {
                let base = std::env::temp_dir()
                    .join(format!("rusty-rolodex-bench-txt-{}", BenchUuid::new_v4()));
                fs::create_dir_all(&base).expect("create temp dir");

                std::env::set_current_dir(&base).expect("chdir into tempdir");
                unsafe {
                    std::env::set_var("STORAGE_CHOICE", "txt");
                }

                let storage = make_store_with_n(20_000);

                (storage, base)
            },
            |(storage, base)| {
                let _ = storage.save();

                // Restore original cwd (robustly) BEFORE removing temp dir
                restore_to_manifest();

                let _ = fs::remove_dir_all(&base);
                black_box(&storage.mem);
            },
            BatchSize::SmallInput,
        );
    });
}

fn bench_read_store_txt(c: &mut Criterion) {
    c.bench_function("read_20k_txt_contacts", |b| {
        b.iter_batched(
            || {
                let base = std::env::temp_dir()
                    .join(format!("rusty-rolodex-bench-txt-{}", BenchUuid::new_v4()));
                fs::create_dir_all(&base).expect("create temp dir");
                let original_cwd = std::env::current_dir().expect("cwd");

                std::env::set_current_dir(&base).expect("chdir into tempdir");
                unsafe {
                    std::env::set_var("STORAGE_CHOICE", "txt");
                }

                let storage = make_store_with_n(20_000);
                storage.save().expect("setup save failed");

                std::env::set_current_dir(&original_cwd).expect("restore cwd after setup");

                base
            },
            |base| {
                std::env::set_current_dir(&base).expect("chdir into tempdir for read");

                let mut manager = ContactManager::new().expect("failed to create store for load");
                let _ = manager.load();

                // Restore original to manifest dir BEFORE removing temp dir
                restore_to_manifest();

                let _ = fs::remove_dir_all(&base);
            },
            BatchSize::SmallInput,
        );
    });
}

fn restore_to_manifest() {
    let manifest_dir: PathBuf = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    if std::env::set_current_dir(&manifest_dir).is_ok() {
        return;
    }
    eprintln!(
        "warning: fallback restore to manifest_dir {:?} failed. trying '/' fallback.",
        manifest_dir
    );
    let _ = std::env::set_current_dir("/");
}

fn configure() -> Criterion {
    Criterion::default()
    // .without_plots()
    // .sample_size(10)
    // .measurement_time(std::time::Duration::from_secs(2))
}

criterion_group! {
    name = benches;
    config = configure();
    targets = bech_add, bench_list, bench_edit, bench_search, bench_delete, bench_save_store_json, bench_read_store_json,
                        bench_save_store_txt, bench_read_store_txt, bench_increment_index, bench_decrement_index
}
criterion_main!(benches);
