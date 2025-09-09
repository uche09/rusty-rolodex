use core::fmt;

#[derive(Debug)]
pub enum AppError {
    Io(std::io::Error),
    JsonPerser(serde_json::Error),
    NotFound(String),
    ParseCommand(String),
    ParseInt(std::num::ParseIntError),
    RegexError(regex::Error),
    Validation(String),
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

impl From<std::num::ParseIntError> for AppError {
    fn from(err: std::num::ParseIntError) -> Self {
        AppError::ParseInt(err)
    }
}

impl From<regex::Error> for AppError {
    fn from(err: regex::Error) -> Self {
        AppError::RegexError(err)
    }
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AppError::Io(e) => {
                write!(f, "I/O error while accessing a file or resource: {}", e)
            }

            AppError::JsonPerser(e) => {
                write!(f, "JSON parser failed '{}'", e)
            }
            AppError::NotFound(item) => {
                write!(f, "{} Not found", item)
            }
            AppError::ParseCommand(cmd) => {
                write!(f, "Unrecognized command: '{}'", cmd)
            }
            AppError::ParseInt(e) => {
                write!(f, "Invalid number format: {}", e)
            }
            AppError::RegexError(e) => {
                write!(f, "Regex failed: {}", e)
            }
            AppError::Validation(msg) => {
                write!(f, "Validation failed: {}", msg)
            }
        }
    }
}

#[cfg(test)]
mod tests {

    use crate::validation::validate_number;

    use super::*;

    #[test]
    fn confirm_parse_int_error_message() {
        let wrong_string = "abc".parse::<i32>().unwrap_err();
        let err = AppError::ParseInt(wrong_string);

        assert!(format!("{}", err).contains("Invalid number format: "));
    }

    #[test]
    fn confirm_validation_error() {
        if let Ok(t) = validate_number(&"abc".to_string()) {
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
    }
}
