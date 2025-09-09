use crate::{domain::Contact, errors::AppError};
use regex::Regex;

pub fn validate_name(name: &str) -> Result<bool, AppError> {
    // Must begin with alphabet
    // Name may contain spaces, hyphens, and apostrophe between alphabets
    // Name may end with number or alphabet
    let re = Regex::new(r"^[A-Za-z][A-Za-z\s'-]*\w*$")?;
    Ok(re.is_match(name))
}

pub fn validate_number(phone: &str) -> Result<bool, AppError> {
    // Must be between 10 to 15 digits digits
    // Phone number may begin with + signifying a country code
    // Every other character aside the "+" must be a digit.
    let re = Regex::new(r"^\+?\d{10,15}$")?;
    Ok(re.is_match(phone))
}

pub fn validate_email(email: &str) -> Result<bool, AppError> {
    // Email can be empty
    // Or email must contain '@' char and contain '.' char somewhere after after
    let re = Regex::new(r"^[^@\s]+@[^@\s]+\.[^@\s]+$")?;
    Ok(email.is_empty() || re.is_match(email))
}

pub fn contact_exist(contact: &Contact, contactlist: &[Contact]) -> bool {
    // Check if contact alread exist in contactlist
    contactlist
        .iter()
        .any(|cont| cont.name == contact.name && phone_number_matches(&cont.phone, &contact.phone))
}

fn phone_number_matches(phone1: &str, phone2: &str) -> bool {
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

// TEST
#[cfg(test)]
mod tests {

    use crate::validation::phone_number_matches;

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
}
