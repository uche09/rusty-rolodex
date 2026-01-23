use std::hash::{Hash, Hasher};

use super::*;
pub use chrono::{DateTime, Utc};
use regex::Regex;
use serde::{Deserialize, Deserializer, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialOrd, Ord, Clone)]
pub struct Contact {
    #[serde(default = "Uuid::new_v4")] // For backward compatibility with contacts without id.
    pub id: Uuid,

    pub name: String,
    pub phone: String,
    pub email: String,
    pub tag: String,

    #[serde(
        default = "bool::default",
        deserialize_with = "deserialize_deleted_field"
    )]
    pub deleted: bool,

    #[serde(
        default = "default_timestamp",
        deserialize_with = "deserialize_timestamp"
    )]
    pub created_at: DateTime<Utc>,

    #[serde(
        default = "default_timestamp",
        deserialize_with = "deserialize_timestamp"
    )]
    pub updated_at: DateTime<Utc>,
}

pub enum ValidationReq {
    __,
}

impl ValidationReq {
    pub fn name_req() -> String {
        "Name must begin with alphabet, may contain spaces, dot, hyphen, and apostrophe between alphabets \
        and may end with number or alphabet. Name must not exceed 50 characters"
        .to_string()
    }

    pub fn phone_req() -> String {
        "Number must contain 10 to 15 digits, may begin with + and all digits".to_string()
    }

    pub fn email_req() -> String {
        "Email can be empty, or must be a valid email. Must not exceed 254 characters".to_string()
    }
}

impl Contact {
    pub fn new(name: String, phone: String, email: String, tag: String) -> Self {
        Contact {
            id: Uuid::new_v4(),
            name,
            phone,
            email,
            tag,
            deleted: false,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }
    pub fn validate_name(&self) -> Result<bool, AppError> {
        // Must begin with alphabet
        // Name may contain spaces, hyphens, and apostrophe between alphabets
        // Name may end with number or alphabet
        // Not more than 50 characters
        let re = Regex::new(r"^[A-Za-z][A-Za-z\s'-\.]*\w*$")?;
        Ok((self.name.len() <= 50) && re.is_match(&self.name))
    }

    pub fn validate_number(&self) -> Result<bool, AppError> {
        // Must be between 10 to 15 digits digits
        // Phone number may begin with + signifying a country code
        // Every other character aside the "+" must be a digit.
        let re = Regex::new(r"^\+?\d{10,15}$")?;
        Ok(re.is_match(&self.phone))
    }

    pub fn validate_email(&self) -> Result<bool, AppError> {
        // Email can be empty
        // Or email must contain '@' char and contain '.' char somewhere after after
        // Not more than 254 characters
        let re = Regex::new(r"^[^@\s]+@[^@\s]+\.[^@\s]+$")?;
        Ok(self.email.is_empty() || (re.is_match(&self.email) && self.email.len() <= 254))
    }

    pub fn already_exist(&self, contactlist: &[&Contact]) -> bool {
        // Check if contact alread exist in contactlist
        contactlist.iter().any(|cont| cont == &self)
    }
}

impl PartialEq for Contact {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name && phone_number_matches(&self.phone, &other.phone)
    }
}

impl Eq for Contact {}

impl Hash for Contact {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name.hash(state);
        self.phone.hash(state);
    }
}

pub fn phone_number_matches(phone1: &str, phone2: &str) -> bool {
    let phone1: Vec<char> = phone1.chars().collect();
    let phone2: Vec<char> = phone2.chars().collect();

    // Quick exit if number if entire number matches
    if phone1.len() > 8 && phone1 == phone2 {
        return true;
    }

    let rest_of_phone1: &[char] = if phone1[0] == '+' {
        // filter country code eg. +234 and collect the rest of phone number
        let [_plus, _code1, _code2, _code3, rest @ ..] = phone1.as_slice() else {
            return false;
        };

        rest
    } else {
        let [_zero, rest @ ..] = phone1.as_slice() else {
            return false;
        };
        rest
    };

    let rest_of_phone2: &[char] = if phone2[0] == '+' {
        let [_plus, _code1, _code2, _code3, rest @ ..] = phone2.as_slice() else {
            return false;
        };

        rest
    } else {
        let [_zero, rest @ ..] = phone2.as_slice() else {
            return false;
        };
        rest
    };

    if rest_of_phone1.is_empty() || rest_of_phone2.is_empty() {
        return false;
    }
    rest_of_phone1 == rest_of_phone2
}

fn default_timestamp() -> DateTime<Utc> {
    Utc::now()
}

fn deserialize_timestamp<'de, D>(deserializer: D) -> Result<DateTime<Utc>, D::Error>
where
    D: Deserializer<'de>,
{
    let opt = Option::<String>::deserialize(deserializer)?;
    match opt {
        Some(s) => DateTime::parse_from_rfc3339(&s)
            .map(|dt| dt.with_timezone(&Utc))
            .map_err(serde::de::Error::custom),
        None => Ok(Utc::now()), // fallback for old contacts
    }
}

fn deserialize_deleted_field<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: Deserializer<'de>,
{
    let value = Option::<bool>::deserialize(deserializer)?;
    match value {
        Some(b) => Ok(b),
        None => Ok(false), // fallback for old contacts
    }
}

// TEST
#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn confirm_phone_number_matches() {
        let phone_a1 = "08123456789".to_string();
        let phone_a2 = "08123456789".to_string(); // Should match

        let phone_b1 = "08123456789".to_string();
        let phone_b2 = "08163456789".to_string(); // Should not match

        let phone_c1 = "+2348123456789".to_string();
        let phone_c2 = "+2348123456789".to_string(); // Should match

        let phone_d1 = "+2348123456789".to_string();
        let phone_d2 = "08123456789".to_string(); // Should match

        let phone_e1 = "08123456789".to_string();
        let phone_e2 = "+2348123456789".to_string(); // Should match

        let phone_f1 = "08163456789".to_string();
        let phone_f2 = "+2348123456789".to_string(); // Should not match

        let phone_g1 = "+234".to_string();
        let phone_g2 = "+234".to_string(); // Fail (Should not match)

        assert!(phone_number_matches(&phone_a1, &phone_a2));
        assert!(!phone_number_matches(&phone_b1, &phone_b2)); // Take note of '!' operator
        assert!(phone_number_matches(&phone_c1, &phone_c2));
        assert!(phone_number_matches(&phone_d1, &phone_d2));
        assert!(phone_number_matches(&phone_e1, &phone_e2));
        assert!(!phone_number_matches(&phone_f1, &phone_f2)); // Take note of '!' operator
        assert!(!phone_number_matches(&phone_g1, &phone_g2)); // Take not of '!' operator
    }

    #[test]
    fn email_validation() -> Result<(), AppError> {
        let contact = Contact::new(
            "Uche".to_string(),
            "08132165498".to_string(),
            "foo@bar".to_string(),
            "".to_string(),
        );

        assert!(!contact.validate_email()?);
        Ok(())
    }
}
