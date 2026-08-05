mod db;

use clap::{Parser, Subcommand};
use std::net::SocketAddr;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "folio-server", about = "Folio server", version = "0.1.0")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Serve {
        #[arg(long, default_value = "127.0.0.1:8787", help = "Address to bind the server to")]
        bind: SocketAddr,

        #[arg(long, default_value = "~/.folio/folio.db", help = "Path to the database file")]
        db: PathBuf,
    },
    Import,
}

fn main() {
    let cli = Cli::parse();
    match cli.command {
        Commands::Serve { bind, db } => match db::open(&db) {
            Ok((_conn, db_path)) => {
                println!("database opened at {}", db_path.display());
                println!("bind: {bind}");
                println!("Not implemented yet");
            }
            Err(e) => {
                eprintln!("failed to open database: {e}");
                std::process::exit(1);
            }
        },
        Commands::Import => println!("Not implemented yet"),
    }
}
