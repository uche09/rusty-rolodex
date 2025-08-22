use crate::domain::Contact;

pub fn validate_name(name: &str) -> bool {
    // Must be alphabetic and non-empty
    // Name may contain spaces between alphabets
    name.chars().count() > 0 && name.chars().all(|c| c.is_alphabetic() || c.is_whitespace())
}

pub fn validate_number(phone: &str) -> bool {
    // Must be at least 10 digits
    // Must contain only digits
    phone.chars().count() >= 10 && phone.chars().all(|c| c.is_ascii_digit())
}

pub fn validate_email(email: &str) -> bool {
    // Email can be empty
    // Or email must contain '@' char and contain '.' char after
    email.chars().count() < 1
        || (email.contains('@')
            && email.contains('.')
            && email.chars().any(|c| {
                c == '@'
                    && (email.find(c).unwrap_or_default() < email.find('.').unwrap_or_default())
            }))
}

pub fn contact_exist(contact: &Contact, contactlist: &[Contact]) -> bool {
    // Check if contact alread exist in contactlist
    contactlist
        .iter()
        .any(|cont| cont.name == contact.name && cont.phone == contact.phone)
}
