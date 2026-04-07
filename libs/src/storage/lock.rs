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
