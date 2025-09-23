use crate::prelude::{AppError, Contact};

pub trait ContactStore {
    fn load(&self) -> Result<Vec<Contact>, AppError>;

    fn save(&self, contacts: &[Contact]) -> Result<(), AppError>;
}
