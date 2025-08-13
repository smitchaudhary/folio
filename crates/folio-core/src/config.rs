use serde::{Deserialize, Serialize};

/// Configuration for the folio application
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Maximum number of items allowed in the inbox
    pub max_items: u32,

    /// Strategy to use when the inbox is full
    pub archive_on_overflow: OverflowStrategy,

    /// Automatically archive done items after this many days (0 = disabled)
    pub auto_archive_done_days: u32,

    /// Schema version for migration purposes
    #[serde(rename = "_v")]
    pub version: u8,
}

/// Strategies for handling overflow when the inbox is full
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OverflowStrategy {
    #[serde(rename = "abort")]
    Abort,
    #[serde(rename = "todo")]
    Todo,
    #[serde(rename = "done")]
    Done,
    #[serde(rename = "any")]
    Any,
}

impl Config {
    /// Create a new Config with default values
    pub fn new() -> Self {
        Self {
            max_items: 100,
            archive_on_overflow: OverflowStrategy::Abort,
            auto_archive_done_days: 0,
            version: 1,
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}
