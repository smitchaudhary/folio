use thiserror::Error;

#[derive(Error, Debug)]
pub enum TuiError {
    #[error("Storage error: {0}")]
    Storage(#[from] folio_storage::StorageError),

    #[error("Core error: {0}")]
    Core(#[from] folio_core::CoreError),

    #[error("Terminal I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Configuration error: {message}")]
    Config { message: String },

    #[error("Application error: {message}")]
    Application { message: String },
}

pub type TuiResult<T> = Result<T, TuiError>;

impl From<Box<dyn std::error::Error>> for TuiError {
    fn from(error: Box<dyn std::error::Error>) -> Self {
        TuiError::Application {
            message: error.to_string(),
        }
    }
}
