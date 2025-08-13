use thiserror::Error;

#[derive(Error, Debug)]
pub enum CoreError {
    #[error("Validation error: {0}")]
    ValidationError(String),

    #[error("Item not found")]
    ItemNotFound,

    #[error("Invalid status transition")]
    InvalidStatusTransition,

    #[error("Inbox is full")]
    InboxFull,
}
