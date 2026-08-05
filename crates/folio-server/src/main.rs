mod db;
mod types;

use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::routing::get;
use axum::{Json, Router};
use clap::{Parser, Subcommand};
use rusqlite::Connection;
use serde::Deserialize;
use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

#[derive(Clone)]
struct AppState {
    conn: Arc<Mutex<Connection>>,
}

#[derive(Deserialize)]
struct ListParams {
    view: Option<types::View>,
}

#[derive(Deserialize)]
struct AddItemRequest {
    name: String,
    #[serde(rename = "type", default)]
    item_type: Option<String>,
    #[serde(default)]
    author: Option<String>,
    #[serde(default)]
    link: Option<String>,
    #[serde(default)]
    note: Option<String>,
    #[serde(default)]
    kind: Option<String>,
}

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

    let state = AppState {
        conn: Arc::new(Mutex::new(conn)),
    };

    let app = Router::new()
        .route("/api/health", get(health))
        .route("/api/items", get(list_items).post(add_item))
        .with_state(state);

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

async fn list_items(
    State(state): State<AppState>,
    Query(params): Query<ListParams>,
) -> Result<Json<Vec<types::ApiItem>>, StatusCode> {
    let view = params.view.unwrap_or_default();
    let conn = state.conn.lock().unwrap();
    match db::load_items(&conn, view) {
        Ok(items) => Ok(Json(items)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn add_item(
    State(state): State<AppState>,
    Json(body): Json<AddItemRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), (StatusCode, Json<serde_json::Value>)> {
    let new_item = match folio_core::create_item(
        body.name,
        body.item_type,
        body.author,
        body.link,
        body.note,
        body.kind,
    ) {
        Ok(item) => item,
        Err(e) => {
            return Err(error_response(
                StatusCode::BAD_REQUEST,
                "validation_error",
                &e.to_string(),
            ))
        }
    };

    let mut conn = state.conn.lock().unwrap();
    match db::add_item(&mut conn, new_item) {
        Ok((item, overflow)) => Ok((
            StatusCode::CREATED,
            Json(serde_json::json!({ "item": item, "overflow": overflow })),
        )),
        Err(db::AddError::InboxFull) => Err(error_response(
            StatusCode::CONFLICT,
            "inbox_full",
            "Inbox limit reached",
        )),
        Err(db::AddError::Other(e)) => Err(error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            "internal_error",
            &e.to_string(),
        )),
    }
}

fn error_response(
    status: StatusCode,
    code: &str,
    message: &str,
) -> (StatusCode, Json<serde_json::Value>) {
    (
        status,
        Json(serde_json::json!({ "error": { "code": code, "message": message } })),
    )
}
