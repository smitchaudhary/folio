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

        #[arg(long, conflicts_with = "all")]
        archive: bool,

        #[arg(long, conflicts_with = "archive")]
        all: bool,
    },

    SetStatus {
        id: usize,

        status: String,

        #[arg(long)]
        archive: bool,
    },

    Edit {
        id: usize,

        #[arg(long)]
        archive: bool,
    },

    Archive {
        id: usize,
    },

    Delete {
        id: usize,

        #[arg(long)]
        archive: bool,
    },

    MarkRef {
        id: usize,

        #[arg(long)]
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
