use super::{
    Path,
    fs::{File, OpenOptions},
};
use fs2::FileExt;

pub struct FileLock {
    // we hold on the opened file, - dropping it releases the lock (RAII)
    _file: File,
    lock_path: std::path::PathBuf,
}

impl FileLock {
    /// Acquires an exclusive (write) lock.
    ///
    /// This function uses the `fs2` extention trait, **it blocks the thread until acquired.**
    pub fn exclusive(storage_path: &str) -> std::io::Result<Self> {
        let lock_path = Self::get_lock_path(storage_path);
        let lock_file = Self::open_lock_file(&lock_path)?;
        lock_file.lock_exclusive()?;
        Ok(Self {
            _file: lock_file,
            lock_path,
        })
    }

    /// Acquires an shared (read) lock.
    ///
    /// This function uses the `fs2` extention trait,
    /// **it only blocks the thread if a writer holds the lock.**
    pub fn shared(storage_path: &str) -> std::io::Result<Self> {
        let lock_path = Self::get_lock_path(storage_path);
        let lock_file = Self::open_lock_file(&lock_path)?;
        lock_file.lock_shared()?;
        Ok(Self {
            _file: lock_file,
            lock_path,
        })
    }

    fn get_lock_path(storage_path: &str) -> std::path::PathBuf {
        let path = Path::new(storage_path);

        path.with_extension(
            // e.g contacts.json.lock
            format!(
                "{}.lock",
                path.extension().unwrap_or_default().to_string_lossy()
            ),
        )
    }

    fn open_lock_file(lock_path: &std::path::Path) -> std::io::Result<File> {
        OpenOptions::new()
            .write(true)
            .truncate(true)
            .create(true)
            .open(lock_path)
    }
}

impl Drop for FileLock {
    fn drop(&mut self) {
        // Try to remove the lock file when FileLock is dropped
        let _ = std::fs::remove_file(&self.lock_path);
    }
}

#[cfg(test)]
mod tests {
    use super::FileLock;
    use fs2::FileExt;
    use std::fs::OpenOptions;
    use tempfile::tempdir;

    // Helper: derives what the lock path should be for a given storage path,
    // mirroring the logic in FileLock::get_lock_path.
    fn expected_lock_path(storage_path: &str) -> std::path::PathBuf {
        let path = std::path::Path::new(storage_path);
        path.with_extension(format!(
            "{}.lock",
            path.extension().unwrap_or_default().to_string_lossy()
        ))
    }

    //  Lock path derivation

    #[test]
    fn lock_path_appends_lock_extension() {
        let path = expected_lock_path("contacts.json");
        assert_eq!(path, std::path::PathBuf::from("contacts.json.lock"));
    }

    // Exclusive lock: contention

    #[test]
    fn exclusive_lock_blocks_second_exclusive() {
        let dir = tempdir().unwrap();
        let storage_path = dir.path().join("contacts.json");
        let path_str = storage_path.to_str().unwrap();

        // Thread A holds the exclusive lock.
        let _lock = FileLock::exclusive(path_str).expect("first exclusive lock should succeed");

        // Thread B tries to acquire another exclusive lock non-blockingly.
        // We open the same .lock file and call try_lock_exclusive on it.
        let lock_path = expected_lock_path(path_str);
        let competing_file = OpenOptions::new()
            .write(true)
            .create(true)
            .open(&lock_path)
            .unwrap();

        // try_lock_exclusive returns Err if the lock is already held.
        // This is the non-blocking probe: it tells us the OS would have blocked.
        assert!(
            competing_file.try_lock_exclusive().is_err(),
            "a second exclusive lock should be denied while the first is held"
        );
    }

    #[test]
    fn exclusive_lock_blocks_shared() {
        let dir = tempdir().unwrap();
        let storage_path = dir.path().join("contacts.json");
        let path_str = storage_path.to_str().unwrap();

        let _lock = FileLock::exclusive(path_str).expect("exclusive lock should succeed");

        let lock_path = expected_lock_path(path_str);
        let competing_file = OpenOptions::new()
            .write(true)
            .create(true)
            .open(&lock_path)
            .unwrap();

        assert!(
            competing_file.try_lock_shared().is_err(),
            "a shared lock should be denied while an exclusive lock is held"
        );
    }

    // Shared lock: coexistence

    #[test]
    fn multiple_shared_locks_can_coexist() {
        let dir = tempdir().unwrap();
        let storage_path = dir.path().join("contacts.json");
        let path_str = storage_path.to_str().unwrap();

        // Two shared locks on the same file should both succeed.
        let _lock1 = FileLock::shared(path_str).expect("first shared lock should succeed");
        let _lock2 = FileLock::shared(path_str).expect("second shared lock should succeed");
        // If we reach this point, coexistence is confirmed.
    }

    #[test]
    fn shared_lock_blocks_exclusive() {
        let dir = tempdir().unwrap();
        let storage_path = dir.path().join("contacts.json");
        let path_str = storage_path.to_str().unwrap();

        let _lock = FileLock::shared(path_str).expect("shared lock should succeed");

        let lock_path = expected_lock_path(path_str);
        let competing_file = OpenOptions::new()
            .write(true)
            .create(true)
            .open(&lock_path)
            .unwrap();

        assert!(
            competing_file.try_lock_exclusive().is_err(),
            "an exclusive lock should be denied while a shared lock is held"
        );
    }

    // RAII: drop releases the lock

    #[test]
    fn dropping_exclusive_lock_releases_it() {
        let dir = tempdir().unwrap();
        let storage_path = dir.path().join("contacts.json");
        let path_str = storage_path.to_str().unwrap();

        {
            let _lock = FileLock::exclusive(path_str).unwrap();
            // Lock is held inside this block.
        }
        // _lock is dropped here; the file is unlocked AND deleted.

        // A fresh exclusive lock should now succeed.
        let result = FileLock::exclusive(path_str);
        assert!(
            result.is_ok(),
            "exclusive lock should succeed after the previous one is dropped"
        );
    }

    #[test]
    fn dropping_lock_removes_lock_file() {
        let dir = tempdir().unwrap();
        let storage_path = dir.path().join("contacts.json");
        let path_str = storage_path.to_str().unwrap();

        let lock_path = expected_lock_path(path_str);

        {
            let _lock = FileLock::exclusive(path_str).unwrap();
            assert!(
                lock_path.exists(),
                "lock file should exist while lock is held"
            );
        }

        assert!(
            !lock_path.exists(),
            "lock file should be removed after lock is dropped"
        );
    }
}
