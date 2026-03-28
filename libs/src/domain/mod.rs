pub mod contact;
pub mod manager;

use crate::errors::AppError;
use crate::storage::{self, ContactStore, file};
pub use contact::Contact;
use uuid::Uuid;
