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

impl Kind {
    pub fn display_emoji(&self) -> &'static str {
        match self {
            Kind::Normal => "✅",
            Kind::Reference => "🔖",
        }
    }
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

impl ItemType {
    pub fn abbreviation(&self) -> &'static str {
        match self {
            ItemType::BlogPost => "blog",
            ItemType::Video => "vid.",
            ItemType::Podcast => "pod.",
            ItemType::News => "news",
            ItemType::Thread => "thrd",
            ItemType::AcademicPaper => "acad",
            ItemType::Other => "oth.",
        }
    }

    pub fn as_string(&self) -> &'static str {
        match self {
            ItemType::BlogPost => "blog_post",
            ItemType::Video => "video",
            ItemType::Podcast => "podcast",
            ItemType::News => "news",
            ItemType::Thread => "thread",
            ItemType::AcademicPaper => "academic_paper",
            ItemType::Other => "other",
        }
    }
}

impl Status {
    pub fn display_char(&self) -> &'static str {
        match self {
            Status::Todo => "T",
            Status::Doing => "D",
            Status::Done => "✓",
        }
    }

    pub fn display_emoji(&self) -> &'static str {
        match self {
            Status::Todo => "📝",
            Status::Doing => "⏳",
            Status::Done => "✅",
        }
    }

    pub fn as_string(&self) -> &'static str {
        match self {
            Status::Todo => "todo",
            Status::Doing => "doing",
            Status::Done => "done",
        }
    }
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

    pub fn format_for_list(&self, index: usize) -> String {
        let name_display = if self.name.len() > 28 {
            format!("{}..", &self.name[..26])
        } else {
            self.name.clone()
        };

        let author_display = if self.author.len() > 13 {
            format!("{}..", &self.author[..11])
        } else {
            self.author.clone()
        };

        let added_date = self.added_at.format("%Y-%m-%d").to_string();

        format!(
            "{:<4} {:<6} {:<30} {:<10} {:<20} {:<15}",
            index,
            self.status.display_char(),
            name_display,
            self.item_type.abbreviation(),
            added_date,
            author_display
        )
    }

    pub fn format_list_header() -> String {
        format!(
            "{:<4} {:<6} {:<30} {:<10} {:<20} {:<15}",
            "ID", "Status", "Name", "Type", "Added", "Author"
        )
    }

    pub fn format_list_separator() -> String {
        "-".repeat(100)
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

#[derive(Debug)]
pub struct StatusUpdateResult {
    pub inbox_items: Vec<Item>,
    pub archive_items: Vec<Item>,
    pub moved_to_archive: Vec<Item>,
    pub moved_to_inbox: bool,
    pub overflow_items: Vec<Item>,
    pub item_found: bool,
}

pub fn update_item_status(
    item_id: usize,
    new_status: Status,
    mut inbox_items: Vec<Item>,
    mut archive_items: Vec<Item>,
    config: &Config,
) -> Result<StatusUpdateResult, CoreError> {
    let mut result = StatusUpdateResult {
        inbox_items: inbox_items.clone(),
        archive_items: archive_items.clone(),
        moved_to_archive: Vec::new(),
        moved_to_inbox: false,
        overflow_items: Vec::new(),
        item_found: false,
    };

    if item_id > 0 && item_id <= inbox_items.len() {
        let item_index = item_id - 1;
        let item = &mut inbox_items[item_index];

        let status_result = change_item_status(item, new_status);
        result.item_found = true;

        if status_result.should_archive {
            let done_item = inbox_items.remove(item_index);
            result.moved_to_archive.push(done_item.clone());
            archive_items.push(done_item);
        }

        result.inbox_items = inbox_items;
        result.archive_items = archive_items;
        return Ok(result);
    }

    if item_id > inbox_items.len() && item_id <= inbox_items.len() + archive_items.len() {
        let item_index = item_id - inbox_items.len() - 1;
        let item = &mut archive_items[item_index];

        let status_result = change_item_status(item, new_status);
        result.item_found = true;

        if status_result.should_move_to_inbox {
            let item_to_move = archive_items.remove(item_index);
            result.moved_to_inbox = true;

            match add_item_to_inbox(inbox_items.clone(), item_to_move.clone(), config) {
                Ok((new_inbox, to_archive)) => {
                    inbox_items = new_inbox;
                    result.overflow_items = to_archive.clone();

                    for overflow_item in to_archive {
                        archive_items.push(overflow_item);
                    }
                }
                Err(_) => {
                    // Rollback - put item back in archive
                    archive_items.insert(item_index, item_to_move);
                    result.inbox_items = inbox_items;
                    result.archive_items = archive_items;
                    return Err(CoreError::InboxFull);
                }
            }
        } else {
            result.archive_items = archive_items;
        }

        result.inbox_items = inbox_items;
        return Ok(result);
    }

    Ok(result)
}
