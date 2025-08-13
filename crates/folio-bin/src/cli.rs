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

    List,

    SetStatus {
        id: usize,

        status: String,
    },

    Edit {
        id: usize,
    },

    Archive {
        id: usize,
    },

    Delete {
        id: usize,
    },

    MarkRef {
        id: usize,
    },

    Config {
        #[command(subcommand)]
        subcommand: ConfigSubcommands,
    },
}

#[derive(Subcommand, Debug)]
pub enum ConfigSubcommands {
    List,

    Get { key: String },

    Set { key: String, value: String },

    Reset,
}
