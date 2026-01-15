use criterion::{BatchSize, Criterion, criterion_group, criterion_main};
use std::hint::black_box;

use rusty_rolodex::prelude::{
    Contact, ContactStore, Store, contact, store::filestore::Index, uuid::Uuid,
};
use std::fs;
use std::path::PathBuf;
use uuid::Uuid as BenchUuid;

// Helper to create a Store prepopulated with `n` contacts in-memory.
// Note: we avoid calling `save()` here so the measured benchmark focuses
// on CPU operations (list/search/edit/delete) rather than disk I/O.
fn make_store_with_n<'a>(n: usize) -> Store<'a> {
    let mut storage = Store::new().expect("Store not created");
    let created_at = contact::Utc::now();
    storage.mem = (0..n)
        .map(|i| {
            let id = Uuid::new_v4();
            let contact = Contact {
                id,
                name: format!("User{i}"),
                phone: format!("08885499529"),
                email: format!("user{i}@yahoo.com"),
                tag: if i % 2 == 0 {
                    "friends".to_string()
                } else {
                    "work".to_string()
                },
                created_at: created_at.clone(),
                updated_at: created_at.clone(),
            };
            (id, contact)
        })
        .collect();
    // Build index once. Use full path to Index to avoid importing filestore::Index.
    storage.index = Index::new(&storage).expect("index build");
    storage
}

// Add-benchmark: measure constructing & inserting one contact (in-memory).
fn bech_add(c: &mut Criterion) {
    c.bench_function("Adding 5k contact (in-memory single add)", |b| {
        // prepare an empty store once per iteration setup. We measure adding single contact.
        b.iter_batched(
            || make_store_with_n(5_000), // setup (expensive)
            |mut storage| {
                // measured closure: add one contact
                let new_contact = Contact::new(
                    "Zoe".to_string(),
                    "08885499529".to_string(),
                    "bryanwelch@gmail.com".to_string(),
                    "friends".to_string(),
                );
                storage.add_contact(new_contact);
                black_box(&storage.mem);
            },
            BatchSize::SmallInput, // choose a batch size appropriate for the work
        );
    });
}

// List-benchmark: measure one listing (collect + sort + filter) per iteration.
fn bench_list(c: &mut Criterion) {
    c.bench_function("listing 5k contact (collect + sort + filter)", |b| {
        let storage = make_store_with_n(5_000);
        b.iter(|| {
            // CPU work only: sort + reverse + filter once per iteration
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

// Search-benchmark: measure a single fuzzy search per iteration.
// Previously: the code called fuzzy_search many times inside one iteration.
fn bench_search(c: &mut Criterion) {
    c.bench_function("Searching 5k contact (single fuzzy search)", |b| {
        let storage = make_store_with_n(5_000);
        b.iter(|| {
            // measure a single search call (index already built in setup)
            let result = storage.fuzzy_search_name("User").expect("search failed");
            black_box(result);
        });
    });
}

// Edit-benchmark: measure editing a single contact per iteration.
// Previously: it did 5k edits inside one iteration and saved each time.
fn bench_edit(c: &mut Criterion) {
    c.bench_function("Editing 5k contact (single edit)", |b| {
        let mut storage = make_store_with_n(5_000);
        b.iter(|| {
            // pick one id to edit (stable behavior)
            let sample_name = "User100";
            if let Some(ids) = storage.get_ids_by_name(sample_name) {
                let id = ids[0];
                if let Some(contact) = storage.mem.get_mut(&id) {
                    contact.name = format!("{}-edited", contact.name);
                    contact.updated_at = contact::Utc::now();
                    black_box(contact);
                }
            }
        });
    });
}

// Delete-benchmark: measure deleting a single contact per iteration.
fn bench_delete(c: &mut Criterion) {
    c.bench_function("Deleting from 5k contact", |b| {
        b.iter_batched(
            || make_store_with_n(5_000),
            |mut storage| {
                // delete a single existing id
                let sample_name = "User200";
                if let Some(mut ids) = storage.get_ids_by_name(sample_name) {
                    let id = ids.remove(0);
                    let _ = storage.delete_contact(&id);
                    black_box(&storage.mem);
                }
            },
            BatchSize::SmallInput,
        );
    });
}

// Save-benchmark (Store::save) for JSON storage: create a unique temp dir, chdir into it,
// build store, then measure Store::save writing to the relative JSON path.
fn bench_save_store_json(c: &mut Criterion) {
    c.bench_function("save_5k_json_contacts", |b| {
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
                let storage = make_store_with_n(5_000);

                (storage, base)
            },
            |(storage, base)| {
                // Measured: call Store::save (timed)
                let _ = storage.save(&storage.mem);

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

// Read-benchmark (Store::load) for JSON storage: create a temp dir and save a file in setup,
// then measure Store::load reading and deserializing that file.
fn bench_read_store_json(c: &mut Criterion) {
    c.bench_function("read_5k_json_contacts", |b| {
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
                let storage = make_store_with_n(5_000);
                storage.save(&storage.mem).expect("setup save failed");

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
                let store = Store::new().expect("failed to create store for load");
                let _ = store.load();

                // Restore cwd to manifest dir BEFORE removing tempdir
                restore_to_manifest();

                let _ = std::fs::remove_dir_all(&base);
            },
            BatchSize::SmallInput,
        );
    });
}

// Save-benchmark (Store::save) for TXT storage: same pattern but set STORAGE_CHOICE to "txt".
fn bench_save_store_txt(c: &mut Criterion) {
    c.bench_function("save_5k_txt_contacts", |b| {
        b.iter_batched(
            || {
                let base = std::env::temp_dir()
                    .join(format!("rusty-rolodex-bench-txt-{}", BenchUuid::new_v4()));
                fs::create_dir_all(&base).expect("create temp dir");

                std::env::set_current_dir(&base).expect("chdir into tempdir");
                unsafe {
                    std::env::set_var("STORAGE_CHOICE", "txt");
                }

                let storage = make_store_with_n(5_000);

                (storage, base)
            },
            |(storage, base)| {
                let _ = storage.save(&storage.mem);

                // Restore original cwd (robustly) BEFORE removing temp dir
                restore_to_manifest();

                let _ = fs::remove_dir_all(&base);
                black_box(&storage.mem);
            },
            BatchSize::SmallInput,
        );
    });
}

// Read-benchmark (Store::load) for TXT storage.
fn bench_read_store_txt(c: &mut Criterion) {
    c.bench_function("read_5k_txt_contacts", |b| {
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

                let storage = make_store_with_n(5_000);
                storage.save(&storage.mem).expect("setup save failed");

                std::env::set_current_dir(&original_cwd).expect("restore cwd after setup");

                base
            },
            |base| {
                std::env::set_current_dir(&base).expect("chdir into tempdir for read");

                let store = Store::new().expect("failed to create store for load");
                let _ = store.load();

                // Restore original to manifest dir BEFORE removing temp dir
                restore_to_manifest();

                let _ = fs::remove_dir_all(&base);
            },
            BatchSize::SmallInput,
        );
    });
}

/// Restore to project's `CARGO_MANIFEST_DIR`, then `/`.
/// Logs warnings on failures but does not panic.
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
    // .measurement_time(std::time::Duration::from_secs(3))
}

criterion_group! {
    name = benches;
    config = configure();
    targets = bech_add, bench_list, bench_edit, bench_search, bench_delete, bench_save_store_json,
              bench_read_store_json, bench_save_store_txt, bench_read_store_txt
}
criterion_main!(benches);
