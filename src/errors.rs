use core::fmt;
use std::sync::PoisonError;

#[derive(Debug)]
pub enum AppError {
    CsvError(csv::Error),
    DateTime(chrono::ParseError),
    FailedRequest(reqwest::Error),
    Io(std::io::Error),
    JsonPerser(serde_json::Error),
    NotFound(String),
    Poison(String),
    RegexError(regex::Error),
    Synchronization(String),
    Validation(String),
}

impl From<csv::Error> for AppError {
    fn from(err: csv::Error) -> Self {
        AppError::CsvError(err)
    }
}
impl From<chrono::ParseError> for AppError {
    fn from(err: chrono::ParseError) -> Self {
        AppError::DateTime(err)
    }
}

impl From<std::io::Error> for AppError {
    fn from(err: std::io::Error) -> Self {
        AppError::Io(err)
    }
}

impl From<serde_json::Error> for AppError {
    fn from(err: serde_json::Error) -> Self {
        AppError::JsonPerser(err)
    }
}

impl<T> From<PoisonError<T>> for AppError {
    fn from(err: PoisonError<T>) -> Self {
        AppError::Poison(err.to_string())
    }
}

impl From<regex::Error> for AppError {
    fn from(err: regex::Error) -> Self {
        AppError::RegexError(err)
    }
}

impl From<reqwest::Error> for AppError {
    fn from(err: reqwest::Error) -> Self {
        AppError::FailedRequest(err)
    }
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AppError::CsvError(e) => {
                write!(f, "CSV parser failed: '{}'", e)
            }
            AppError::DateTime(e) => {
                write!(f, "Invalid Datetime format: {}", e)
            }
            AppError::FailedRequest(e) => {
                write!(f, "HTTP request failed: '{}'", e)
            }
            AppError::Io(e) => {
                write!(f, "I/O error while accessing a file or resource: {}", e)
            }
            AppError::JsonPerser(e) => {
                write!(f, "JSON parser failed '{}'", e)
            }
            AppError::NotFound(item) => {
                write!(f, "{} Not found", item)
            }
            AppError::Poison(e) => {
                write!(f, "Mutex poisoned: {}", e)
            }
            AppError::RegexError(e) => {
                write!(f, "Regex failed: {}", e)
            }
            AppError::Synchronization(msg) => {
                write!(f, "Synchronization Error: {}", msg)
            }
            AppError::Validation(msg) => {
                write!(f, "Validation failed: {}", msg)
            }
        }
    }
}

#[cfg(test)]
mod tests {

    use crate::domain::contact::Contact;

    use super::*;

    #[test]
    fn confirm_validation_error() -> Result<(), AppError> {
        let contact = Contact::new(
            "".to_string(),
            "abc".to_string(),
            "".to_string(),
            "".to_string(),
        );

        if let Ok(t) = contact.validate_number() {
            if !t {
                let err = AppError::Validation("\nInvalid Number input.".to_string());

                assert_eq!(
                    format!("{}", err),
                    format!("Validation failed: \nInvalid Number input.")
                );
            }
        } else {
            panic!();
        }

        Ok(())
    }
}
