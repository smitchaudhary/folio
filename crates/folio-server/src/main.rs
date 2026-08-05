mod db;

use axum::routing::get;
use axum::{Json, Router};
use clap::{Parser, Subcommand};
use std::net::SocketAddr;
use std::path::{Path, PathBuf};

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

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    match cli.command {
        Commands::Serve { bind, db } => serve(bind, &db).await,
        Commands::Import => println!("Not implemented yet"),
    }
}

async fn serve(bind: SocketAddr, db_arg: &Path) {
    let (conn, db_path) = match db::open(db_arg) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("failed to open database: {e}");
            std::process::exit(1);
        }
    };
    println!("database opened at {}", db_path.display());

    if let Err(e) = db::init_schema(&conn) {
        eprintln!("failed to initialize schema: {e}");
        std::process::exit(1);
    }
    println!("schema initialized");

    let app = Router::new().route("/api/health", get(health));

    let listener = match tokio::net::TcpListener::bind(bind).await {
        Ok(l) => l,
        Err(e) => {
            eprintln!("failed to bind {bind}: {e}");
            std::process::exit(1);
        }
    };
    println!("listening on {bind}");

    axum::serve(listener, app).await.expect("server error");
}

async fn health() -> Json<serde_json::Value> {
    Json(serde_json::json!({"status": "ok"}))
}
