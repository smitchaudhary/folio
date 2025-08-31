use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use strum::EnumString;

mod error;
pub use error::{CapError, CoreError};

mod config;
pub use config::{Config, OverflowStrategy};

mod status;
pub use status::{StatusTransitionResult, change_item_status, update_timestamps};

mod cap;
pub use cap::add_with_cap;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, EnumString)]
pub enum ItemType {
    #[serde(rename = "blog_post")]
    #[strum(serialize = "blog_post")]
    BlogPost,
    #[serde(rename = "video")]
    #[strum(serialize = "video")]
    Video,
    #[serde(rename = "podcast")]
    #[strum(serialize = "podcast")]
    Podcast,
    #[serde(rename = "news")]
    #[strum(serialize = "news")]
    News,
    #[serde(rename = "thread")]
    #[strum(serialize = "thread")]
    Thread,
    #[serde(rename = "academic_paper")]
    #[strum(serialize = "academic_paper")]
    AcademicPaper,
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

pub fn create_item(
    name: String,
    item_type: Option<String>,
    author: Option<String>,
    link: Option<String>,
    note: Option<String>,
    kind: Option<String>,
) -> Result<Item, CoreError> {
    if name.is_empty() {
        return Err(CoreError::ValidationError(
            "Name cannot be empty".to_string(),
        ));
    }

    let parsed_type = match item_type.as_deref() {
        Some(t) => ItemType::from_str(t).unwrap_or(ItemType::BlogPost),
        None => ItemType::BlogPost,
    };

    let parsed_kind = match kind.as_deref() {
        Some(k) => Kind::from_str(k).unwrap_or(Kind::Normal),
        None => Kind::Normal,
    };

    let item = Item {
        name,
        item_type: parsed_type,
        status: Status::Todo,
        author: author.unwrap_or_default(),
        link: link.unwrap_or_default(),
        added_at: Utc::now(),
        started_at: None,
        finished_at: None,
        note: note.unwrap_or_default(),
        kind: parsed_kind,
        version: 1,
    };

    item.validate()?;
    Ok(item)
}

pub fn add_item_to_inbox(
    inbox: Vec<Item>,
    new_item: Item,
    config: &Config,
) -> Result<(Vec<Item>, Vec<Item>), CoreError> {
    new_item.validate()?;

    add_with_cap(
        inbox,
        new_item,
        config.max_items as usize,
        config.archive_on_overflow.clone(),
    )
    .map_err(|_| CoreError::InboxFull)
}
