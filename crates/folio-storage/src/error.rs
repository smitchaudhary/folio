use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum StorageError {
    #[error("Failed to create folio directory at {path}")]
    DirectoryCreation { path: PathBuf },

    #[error("Failed to determine home directory")]
    HomeDirectoryNotFound,

    #[error("Failed to read file: {path}")]
    FileRead { path: PathBuf },

    #[error("Failed to write file: {path}")]
    FileWrite { path: PathBuf },

    #[error("Failed to parse JSONL data")]
    JsonlParse,

    #[error("Failed to serialize item to JSON")]
    JsonSerialization,

    #[error("Failed to deserialize JSON data")]
    JsonDeserialization,

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}

pub type StorageResult<T> = Result<T, StorageError>;