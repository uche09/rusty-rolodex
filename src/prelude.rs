pub use crate::cli::{command, run_app};
pub use crate::domain::{
    contact::{self, Contact},
    search::{create_email_search_index, create_name_search_index},
};
pub use crate::errors::AppError;
pub use crate::store::{self, ContactStore, memory, parse_store};
