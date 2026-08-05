use crate::types::{ApiItem, View};
use chrono::{DateTime, Utc};
use folio_core::{ItemType, Kind, Status};
use rusqlite::Connection;
use std::path::{Path, PathBuf};
use std::str::FromStr;

const SCHEMA_VERSION: i32 = 1;

pub fn expand_tilde(path: &Path) -> PathBuf {
    if path == Path::new("~") {
        return dirs::home_dir().unwrap_or_else(|| path.to_path_buf());
    }
    if let Ok(rest) = path.strip_prefix("~")
        && let Some(home) = dirs::home_dir()
    {
        return home.join(rest);
    }
    path.to_path_buf()
}

pub fn open(path: &Path) -> Result<(Connection, PathBuf), Box<dyn std::error::Error>> {
    let db_path = expand_tilde(path);

    if let Some(parent) = db_path.parent()
        && !parent.as_os_str().is_empty()
        && !parent.exists()
    {
        std::fs::create_dir_all(parent)?;
    }

    let conn = Connection::open(&db_path)?;
    Ok((conn, db_path))
}

pub fn init_schema(conn: &Connection) -> rusqlite::Result<()> {
    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS items (
            id          TEXT PRIMARY KEY,
            name        TEXT NOT NULL,
            type        TEXT NOT NULL,
            status      TEXT NOT NULL,
            kind        TEXT NOT NULL DEFAULT 'normal',
            author      TEXT NOT NULL DEFAULT '',
            link        TEXT NOT NULL DEFAULT '',
            note        TEXT NOT NULL DEFAULT '',
            added_at    TEXT NOT NULL,
            started_at  TEXT,
            finished_at TEXT,
            position    INTEGER NOT NULL,
            updated_at  TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS config (
            id                  INTEGER PRIMARY KEY CHECK (id = 1),
            max_items           INTEGER NOT NULL,
            archive_on_overflow TEXT NOT NULL,
            _v                  INTEGER NOT NULL
        );

        INSERT OR IGNORE INTO config (id, max_items, archive_on_overflow, _v)
        VALUES (1, 30, 'abort', 1);
        ",
    )?;
    conn.pragma_update(None, "user_version", SCHEMA_VERSION)?;
    Ok(())
}

struct ItemRow {
    id: String,
    name: String,
    item_type: String,
    status: String,
    kind: String,
    author: String,
    link: String,
    note: String,
    added_at: String,
    started_at: Option<String>,
    finished_at: Option<String>,
}

pub fn load_items(
    conn: &Connection,
    view: View,
) -> Result<Vec<ApiItem>, Box<dyn std::error::Error>> {
    let where_clause = match view {
        View::Inbox => " WHERE status != 'done'",
        View::Archive => " WHERE status = 'done'",
        View::All => "",
    };

    let sql = format!(
        "SELECT id, name, type, status, kind, author, link, note, added_at, started_at, finished_at \
         FROM items{where_clause} ORDER BY position"
    );

    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map([], |row| {
        Ok(ItemRow {
            id: row.get(0)?,
            name: row.get(1)?,
            item_type: row.get(2)?,
            status: row.get(3)?,
            kind: row.get(4)?,
            author: row.get(5)?,
            link: row.get(6)?,
            note: row.get(7)?,
            added_at: row.get(8)?,
            started_at: row.get(9)?,
            finished_at: row.get(10)?,
        })
    })?;

    let mut items = Vec::new();
    for row in rows {
        let row = row?;
        items.push(ApiItem {
            id: row.id,
            name: row.name,
            item_type: ItemType::from_str(&row.item_type)?,
            status: Status::from_str(&row.status)?,
            kind: Kind::from_str(&row.kind)?,
            author: row.author,
            link: row.link,
            note: row.note,
            added_at: parse_ts(&row.added_at)?,
            started_at: parse_ts_opt(row.started_at)?,
            finished_at: parse_ts_opt(row.finished_at)?,
        });
    }
    Ok(items)
}

fn parse_ts(s: &str) -> Result<DateTime<Utc>, Box<dyn std::error::Error>> {
    Ok(DateTime::parse_from_rfc3339(s)?.with_timezone(&Utc))
}

fn parse_ts_opt(s: Option<String>) -> Result<Option<DateTime<Utc>>, Box<dyn std::error::Error>> {
    s.map(|v| parse_ts(&v)).transpose()
}
