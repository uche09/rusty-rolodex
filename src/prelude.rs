pub use crate::cli::{command, run_app};
pub use crate::domain::{
    contact::{self, Contact},
    storage::{self, Storage},
};
pub use crate::errors::AppError;
pub use crate::store::{file, memory};
pub use crate::traits::ContactStore;
