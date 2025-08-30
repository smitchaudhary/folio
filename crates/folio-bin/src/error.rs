use std::io;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CliError {
    #[error("Item with ID {id} not found. Use 'folio list' to see available items.")]
    ItemNotFound { id: usize },

    #[error("Invalid status '{status}'. Valid options are: todo, doing, done")]
    InvalidStatus { status: String },

    #[error("Invalid item type '{item_type}'. Valid options are: article, video, blog, other")]
    InvalidItemType { item_type: String },

    #[error("Invalid value for max_items: {value}. Must be a number between 1 and 1000")]
    InvalidMaxItems { value: String },

    #[error("Invalid value for archive_on_overflow: {value}. Valid options are: abort, todo, any")]
    InvalidOverflowStrategy { value: String },

    #[error("Unknown config key '{key}'. Valid keys are: max_items, archive_on_overflow")]
    UnknownConfigKey { key: String },

    #[error("Inbox limit ({limit}) reached. {suggestions}")]
    InboxFull { limit: u32, suggestions: String },

    #[error("Validation error: {message}")]
    ValidationError { message: String },

    #[error("IO error: {message}")]
    IoError { message: String },

    #[error("Configuration error: {message}")]
    ConfigError { message: String },
}

impl From<Box<dyn std::error::Error>> for CliError {
    fn from(error: Box<dyn std::error::Error>) -> Self {
        CliError::IoError {
            message: error.to_string(),
        }
    }
}

impl From<io::Error> for CliError {
    fn from(error: io::Error) -> Self {
        CliError::IoError {
            message: error.to_string(),
        }
    }
}

impl From<String> for CliError {
    fn from(message: String) -> Self {
        CliError::ValidationError { message }
    }
}

impl From<&str> for CliError {
    fn from(message: &str) -> Self {
        CliError::ValidationError {
            message: message.to_string(),
        }
    }
}

impl From<serde_json::Error> for CliError {
    fn from(error: serde_json::Error) -> Self {
        CliError::ConfigError {
            message: error.to_string(),
        }
    }
}

impl From<folio_core::CoreError> for CliError {
    fn from(error: folio_core::CoreError) -> Self {
        match error {
            folio_core::CoreError::ValidationError(msg) => {
                CliError::ValidationError { message: msg }
            }
            folio_core::CoreError::ItemNotFound => CliError::ValidationError {
                message: "Item not found".to_string(),
            },
            folio_core::CoreError::InvalidStatusTransition => CliError::ValidationError {
                message: "Invalid status transition".to_string(),
            },
            folio_core::CoreError::InboxFull => CliError::ValidationError {
                message: "Inbox is full".to_string(),
            },
        }
    }
}

pub fn print_error(error: &CliError) {
    eprintln!("Error: {}", error);
    eprintln!();
    eprintln!("For help, try:");
    eprintln!("folio --help");
    eprintln!("folio <command> --help");
}
