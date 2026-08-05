use chrono::{DateTime, Utc};
use folio_core::{ItemType, Kind, Status};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiItem {
    pub id: String,
    pub name: String,
    #[serde(rename = "type")]
    pub item_type: ItemType,
    pub status: Status,
    pub kind: Kind,
    pub author: String,
    pub link: String,
    pub note: String,
    pub added_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub finished_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Copy, Default, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum View {
    #[default]
    Inbox,
    Archive,
    All,
}

impl From<ApiItem> for folio_core::Item {
    fn from(a: ApiItem) -> Self {
        folio_core::Item {
            id: a.id,
            name: a.name,
            item_type: a.item_type,
            status: a.status,
            author: a.author,
            link: a.link,
            added_at: a.added_at,
            started_at: a.started_at,
            finished_at: a.finished_at,
            note: a.note,
            kind: a.kind,
            version: 1,
        }
    }
}

impl ApiItem {
    pub fn from_core(item: folio_core::Item) -> Self {
        ApiItem {
            id: item.id,
            name: item.name,
            item_type: item.item_type,
            status: item.status,
            kind: item.kind,
            author: item.author,
            link: item.link,
            note: item.note,
            added_at: item.added_at,
            started_at: item.started_at,
            finished_at: item.finished_at,
        }
    }
}
