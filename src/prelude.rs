pub use crate::cli::{command, run_app};
pub use crate::domain::{
    contact::{self, Contact},
    search::{fuzzy_search_email_domain_index, fuzzy_search_name_index},
};
pub use crate::errors::AppError;
pub use crate::store::{self, ContactStore, memory, parse_store};
