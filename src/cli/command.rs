use clap::{Parser, Subcommand, ValueEnum};

#[derive(Parser, Debug)]
#[command(name = "rolodex", version, about = "Simple Contact Book")]
pub struct Cli {
    /// Storage choice (mem, txt, json) are available
    #[arg(long, env = "STORAGE_CHOICE", default_value_t = String::from("json"))]
    pub storage_choice: String,

    #[command(subcommand)]
    pub command: Commands,
}

/// Subcommand and their flags
#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Add a new contact
    Add {
        /// Contact name
        #[arg(long)]
        name: String,

        /// Contact phone number
        #[arg(long)]
        phone: String,

        /// Contact email address
        #[arg(long)]
        email: Option<String>,

        /// Contact tag (school, work, gym)
        #[arg(long)]
        tag: Option<String>,
    },
    /// List contacts
    List {
        /// Sort ordering (default is unsorted)
        #[arg(long)]
        sort: Option<SortKey>,

        /// List only specific tags
        #[arg(long)]
        tag: Option<String>,

        /// Reverse order
        #[arg(short, long)]
        reverse: bool,
    },
    /// Edit the data of an existing contact
    /// Provide current contact name and number
    /// followed by optional arguments of as many field you wish to update
    Edit {
        /// Contact current name
        #[arg(long)]
        name: String,

        /// Contact current phone number
        #[arg(long)]
        phone: String,

        /// Update name
        #[arg(long)]
        new_name: Option<String>,

        /// Update phone number
        #[arg(long)]
        new_phone: Option<String>,

        /// Update email address
        #[arg(long)]
        new_email: Option<String>,

        /// Update tag (school, work, gym)
        #[arg(long)]
        new_tag: Option<String>,
    },
    /// Delete a contact by name
    /// provide optional number in cases where name matches multiple contacts
    Delete {
        /// Name of contact to delete
        #[arg(long)]
        name: String,

        /// Contact number to delete
        #[arg(long)]
        phone: Option<String>,
    },

    /// Import contacts from .csv file
    Import {
        /// File path to the source .csv file
        #[arg(short, long)]
        src: Option<String>,
    },

    /// Export contacts to a .csv file
    Export {
        /// File path to the destination location for export file
        #[arg(short, long)]
        des: Option<String>,
    },
}

/// Supported sort keys
#[derive(Copy, Clone, Debug, ValueEnum)]
pub enum SortKey {
    Name,
    Email,
    Created,
    Updated,
}
