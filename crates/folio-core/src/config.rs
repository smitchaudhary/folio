use serde::{Deserialize, Serialize};
use strum::EnumString;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub max_items: u32,

    pub archive_on_overflow: OverflowStrategy,
}

#[derive(Debug, Clone, Serialize, Deserialize, EnumString)]
pub enum OverflowStrategy {
    #[serde(rename = "abort")]
    #[strum(serialize = "abort")]
    Abort,
    #[serde(rename = "todo")]
    #[strum(serialize = "todo")]
    Todo,
    #[serde(rename = "any")]
    #[strum(serialize = "any")]
    Any,
}

impl OverflowStrategy {
    pub fn as_string(&self) -> &'static str {
        match self {
            OverflowStrategy::Abort => "abort",
            OverflowStrategy::Todo => "todo",
            OverflowStrategy::Any => "any",
        }
    }
}

impl Config {
    pub fn new() -> Self {
        Self {
            max_items: 30,
            archive_on_overflow: OverflowStrategy::Abort,
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}
