pub use crate::cli::{command, run_app};
pub use crate::domain::{
    contact::{self, Contact},
    manager::{self, ContactManager},
};
pub use crate::errors::AppError;
pub use crate::storage::{self, ContactStore, file, memory, remote::RemoteStorage};
pub use std::collections::HashMap;
pub use uuid;
