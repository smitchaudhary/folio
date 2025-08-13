use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use strum::EnumString;

mod error;
pub use error::{CoreError, CapError};

mod config;
pub use config::{Config, OverflowStrategy};

mod status;
pub use status::{cycle_status, update_timestamps};

mod cap;
pub use cap::add_with_cap;

mod archive;
pub use archive::{should_auto_archive, move_to_archive};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, EnumString)]
pub enum ItemType {
    #[serde(rename = "article")]
    #[strum(serialize = "article")]
    Article,
    #[serde(rename = "video")]
    #[strum(serialize = "video")]
    Video,
    #[serde(rename = "blog")]
    #[strum(serialize = "blog")]
    Blog,
    #[serde(rename = "other")]
    #[strum(serialize = "other")]
    Other,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, EnumString)]
pub enum Status {
    #[serde(rename = "todo")]
    #[strum(serialize = "todo")]
    Todo,
    #[serde(rename = "doing")]
    #[strum(serialize = "doing")]
    Doing,
    #[serde(rename = "done")]
    #[strum(serialize = "done")]
    Done,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, EnumString)]
pub enum Kind {
    #[serde(rename = "normal")]
    #[strum(serialize = "normal")]
    Normal,
    #[serde(rename = "reference")]
    #[strum(serialize = "reference")]
    Reference,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Item {
    pub name: String,
    #[serde(rename = "type")]
    pub item_type: ItemType,
    pub status: Status,
    pub author: String,
    pub link: String,
    pub added_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub finished_at: Option<DateTime<Utc>>,
    pub note: String,
    pub kind: Kind,
    #[serde(rename = "_v")]
    pub version: u8,
}

impl Item {
    pub fn validate(&self) -> Result<(), CoreError> {
        if self.name.is_empty() {
            return Err(CoreError::ValidationError(
                "Name cannot be empty".to_string(),
            ));
        }
        Ok(())
    }
}
