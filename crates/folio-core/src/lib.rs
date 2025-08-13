use serde::{Deserialize, Serialize};
use strum::EnumString;

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
