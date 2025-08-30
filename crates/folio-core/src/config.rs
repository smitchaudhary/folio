use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub max_items: u32,

    pub archive_on_overflow: OverflowStrategy,

    #[serde(rename = "_v")]
    pub version: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OverflowStrategy {
    #[serde(rename = "abort")]
    Abort,
    #[serde(rename = "todo")]
    Todo,
    #[serde(rename = "any")]
    Any,
}

impl Config {
    pub fn new() -> Self {
        Self {
            max_items: 30,
            archive_on_overflow: OverflowStrategy::Abort,
            version: 1,
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}
