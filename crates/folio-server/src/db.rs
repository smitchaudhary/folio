use rusqlite::Connection;
use std::path::{Path, PathBuf};

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
