pub mod file;
pub mod memory;

use crate::domain::{contact::Contact, storage::Storage};
use crate::errors::AppError;

pub trait ContactStore {
    fn load(&self) -> Result<Vec<Contact>, AppError>;

    fn save(&self, contacts: &[Contact]) -> Result<(), AppError>;
}
