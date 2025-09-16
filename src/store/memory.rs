use super::*;

pub struct MemStore {
    pub data: Vec<Contact>,
}

impl MemStore {
    pub fn new() -> Self {
        Self { data: Vec::new() }
    }
}

impl ContactStore for MemStore {
    fn load(&self) -> Result<Vec<Contact>, AppError> {
        Ok(self.data.clone())
    }

    fn save(&self, _contacts: &[Contact]) -> Result<(), AppError> {
        Ok(())
    }
}
