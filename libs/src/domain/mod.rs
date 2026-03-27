pub mod contact;
pub mod manager;

use crate::errors::AppError;
pub use contact::Contact;
use crate::storage::{self, ContactStore, file};
use uuid::Uuid;
