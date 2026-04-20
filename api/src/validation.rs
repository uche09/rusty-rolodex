use regex::Regex;
use serde::Deserialize;
use std::sync::LazyLock;
use validator::Validate;

// Must begin with alphabet
// Name may contain spaces, hyphens, and apostrophe between alphabets
// Name may end with number or alphabet
static NAME_RGX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^[A-Za-z][A-Za-z\s'-\.]*\w*$").unwrap());

// Must be between 10 to 15 digits digits
// Phone number may begin with + signifying a country code
// Every other character aside the "+" must be a digit.
static PHONE_RGX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^\+?\d{10,15}$").unwrap());

#[derive(Deserialize, Validate, Debug)]
pub struct NewContact {
    // Not more than 50 characters
    #[validate(
        regex(
            path = *NAME_RGX,
            message="Name must begin with alphabet\n\
            Name may contain spaces, hyphens, and apostrophe between alphabets\n\
            Name may end with number or alphabet"
        ),
        length(
            min=1, max=50,
        )
    )]
    pub name: String,

    #[validate(
        regex(
            path=*PHONE_RGX,
            message="Phone must be between 10 to 15 digits digits\n\
            Phone may begin with + signifying a country code"
        ) )]
    pub phone: String,

    #[validate(email)]
    pub email: Option<String>,

    #[validate(length(min = 1, max = 20))]
    pub tag: Option<String>,
}

#[derive(Deserialize, Validate, Debug)]
pub struct EditContact {
    #[validate(
        regex(
            path = *NAME_RGX,
            message="Name must begin with alphabet\n\
            Name may contain spaces, hyphens, and apostrophe between alphabets\n\
            Name may end with number or alphabet"
        ),
        length(
            min=1, max=50,
        )
    )]
    pub name: Option<String>,

    #[validate(
        regex(
            path=*PHONE_RGX,
            message="Phone must be between 10 to 15 digits digits\n\
            Phone may begin with + signifying a country code"
        ) )]
    pub phone: Option<String>,

    #[validate(email)]
    pub email: Option<String>,

    #[validate(length(min = 1, max = 20))]
    pub tag: Option<String>,
}
