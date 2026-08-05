use rusqlite::Connection;
use std::path::{Path, PathBuf};

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
