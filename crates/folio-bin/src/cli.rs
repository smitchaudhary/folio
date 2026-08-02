use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "folio")]
#[command(about = "A tool to manage your reading list", version = "0.1.0")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    Add {
        #[arg(long)]
        name: Option<String>,

        #[arg(short, long)]
        r#type: Option<String>,

        #[arg(short, long)]
        author: Option<String>,

        #[arg(short, long)]
        link: Option<String>,

        #[arg(long)]
        note: Option<String>,

        #[arg(long)]
        kind: Option<String>,
    },

    List {
        #[arg(long, value_delimiter = ',')]
        status: Option<Vec<String>>,

        #[arg(long, value_delimiter = ',')]
        r#type: Option<Vec<String>>,

        #[arg(long, conflicts_with = "all", help = "Show archived items")]
        archive: bool,

        #[arg(long, conflicts_with = "archive", help = "Show inbox and archive combined")]
        all: bool,
    },

    SetStatus {
        #[arg(help = "Item ID (position in the current view)")]
        id: usize,

        #[arg(help = "New status: todo, doing, done")]
        status: String,

        #[arg(long, help = "Target archived items — IDs are positions within the archive")]
        archive: bool,
    },

    Edit {
        #[arg(help = "Item ID (position in the current view)")]
        id: usize,

        #[arg(long, help = "Target archived items — IDs are positions within the archive")]
        archive: bool,
    },

    Archive {
        #[arg(help = "Item ID (position in the current view)")]
        id: usize,
    },

    Delete {
        #[arg(help = "Item ID (position in the current view)")]
        id: usize,

        #[arg(long, help = "Target archived items — IDs are positions within the archive")]
        archive: bool,
    },

    MarkRef {
        #[arg(help = "Item ID (position in the current view)")]
        id: usize,

        #[arg(long, help = "Target archived items — IDs are positions within the archive")]
        archive: bool,
    },

    Config {
        #[command(subcommand)]
        subcommand: ConfigSubcommands,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ListView {
    Inbox,
    Archive,
    All,
}

#[derive(Subcommand, Debug)]
pub enum ConfigSubcommands {
    List,

    Get { key: String },

    Set { key: String, value: String },

    Reset,
}
