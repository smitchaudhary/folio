use chrono::{DateTime, Utc};
use folio_core::{Item, View};
use folio_core::{ItemType, Kind, OverflowStrategy, Status};
use rusqlite::{Connection, params};
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
            archive_on_overflow TEXT NOT NULL
        );

        INSERT OR IGNORE INTO config (id, max_items, archive_on_overflow)
        VALUES (1, 30, 'abort');
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

pub fn load_items(conn: &Connection, view: View) -> Result<Vec<Item>, Box<dyn std::error::Error>> {
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
        items.push(Item {
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

pub fn parse_strategy(s: &str) -> Option<OverflowStrategy> {
    match s {
        "abort" => Some(OverflowStrategy::Abort),
        "todo" => Some(OverflowStrategy::Todo),
        "any" => Some(OverflowStrategy::Any),
        _ => None,
    }
}

pub fn strategy_to_str(s: &OverflowStrategy) -> &'static str {
    match s {
        OverflowStrategy::Abort => "abort",
        OverflowStrategy::Todo => "todo",
        OverflowStrategy::Any => "any",
    }
}

pub fn load_config(conn: &Connection) -> Result<folio_core::Config, Box<dyn std::error::Error>> {
    let (max_items, strategy_str): (u32, String) = conn.query_row(
        "SELECT max_items, archive_on_overflow FROM config WHERE id = 1",
        [],
        |row| Ok((row.get(0)?, row.get(1)?)),
    )?;

    let archive_on_overflow = parse_strategy(&strategy_str)
        .ok_or_else(|| format!("unknown overflow strategy: {strategy_str}"))?;

    Ok(folio_core::Config {
        max_items,
        archive_on_overflow,
    })
}

pub fn save_config(
    conn: &mut Connection,
    max_items: u32,
    archive_on_overflow: OverflowStrategy,
) -> Result<folio_core::Config, DbError> {
    if !(1..=1000).contains(&max_items) {
        return Err(DbError::Validation(
            "max_items must be between 1 and 1000".to_string(),
        ));
    }

    let tx = conn.transaction().map_err(|e| DbError::Other(e.into()))?;
    let inbox_count: i64 = tx
        .query_row(
            "SELECT COUNT(*) FROM items WHERE status != 'done'",
            [],
            |row| row.get(0),
        )
        .map_err(|e| DbError::Other(e.into()))?;

    if (max_items as i64) < inbox_count {
        return Err(DbError::InboxOverCap(inbox_count as u32));
    }

    tx.execute(
        "UPDATE config SET max_items = ?1, archive_on_overflow = ?2 WHERE id = 1",
        params![max_items, strategy_to_str(&archive_on_overflow)],
    )
    .map_err(|e| DbError::Other(e.into()))?;

    tx.commit().map_err(|e| DbError::Other(e.into()))?;

    Ok(folio_core::Config {
        max_items,
        archive_on_overflow,
    })
}

fn next_position(conn: &Connection, archive: bool) -> rusqlite::Result<i64> {
    let sql = if archive {
        "SELECT COALESCE(MAX(position), 0) + 1 FROM items WHERE status = 'done'"
    } else {
        "SELECT COALESCE(MAX(position), 0) + 1 FROM items WHERE status != 'done'"
    };
    conn.query_row(sql, [], |row| row.get(0))
}

pub enum DbError {
    NotFound,
    InboxFull,
    Validation(String),
    NotArchived,
    InboxOverCap(u32),
    AlreadyAtEdge,
    Other(Box<dyn std::error::Error>),
}

pub fn add_item(
    conn: &mut Connection,
    mut new_item: folio_core::Item,
) -> Result<(Item, Vec<Item>), DbError> {
    let tx = conn.transaction().map_err(|e| DbError::Other(e.into()))?;
    let config = load_config(&tx).map_err(|e| DbError::Other(e.into()))?;

    let mut overflow_api = Vec::new();

    if new_item.kind == folio_core::Kind::Reference {
        new_item.status = folio_core::Status::Done;
        folio_core::update_timestamps(&mut new_item);
    } else {
        let inbox_rows = load_items(&tx, View::Inbox).map_err(|e| DbError::Other(e.into()))?;
        let inbox_core = inbox_rows;

        let (_, to_archive) = folio_core::add_item_to_inbox(inbox_core, new_item.clone(), &config)
            .map_err(|e| match e {
                folio_core::CoreError::InboxFull => DbError::InboxFull,
                other => DbError::Other(other.into()),
            })?;

        let now = Utc::now();
        for item in &to_archive {
            let position = next_position(&tx, true).map_err(|e| DbError::Other(e.into()))?;
            tx.execute(
                "UPDATE items SET status = 'done', finished_at = ?1, position = ?2, updated_at = ?3 \
                 WHERE id = ?4",
                params![
                    item.finished_at.map(|dt| dt.to_rfc3339()),
                    position,
                    now.to_rfc3339(),
                    item.id,
                ],
            )
            .map_err(|e| DbError::Other(e.into()))?;
            overflow_api.push(item.clone());
        }
    }

    let api_item = new_item.clone();
    let archive = api_item.status == folio_core::Status::Done;
    let position = next_position(&tx, archive).map_err(|e| DbError::Other(e.into()))?;
    let now = Utc::now();
    let kind_str = match &api_item.kind {
        folio_core::Kind::Normal => "normal",
        folio_core::Kind::Reference => "reference",
    };

    tx.execute(
        "INSERT INTO items (id, name, type, status, kind, author, link, note, added_at, started_at, \
         finished_at, position, updated_at) \
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
        params![
            &api_item.id,
            &api_item.name,
            api_item.item_type.as_string(),
            api_item.status.as_string(),
            kind_str,
            &api_item.author,
            &api_item.link,
            &api_item.note,
            api_item.added_at.to_rfc3339(),
            api_item.started_at.map(|dt| dt.to_rfc3339()),
            api_item.finished_at.map(|dt| dt.to_rfc3339()),
            position,
            now.to_rfc3339(),
        ],
    )
    .map_err(|e| DbError::Other(e.into()))?;

    tx.commit().map_err(|e| DbError::Other(e.into()))?;
    Ok((api_item, overflow_api))
}

pub struct StatusResult {
    pub item: Item,
    pub moved_to_archive: bool,
    pub moved_to_inbox: bool,
    pub overflow: Vec<Item>,
}

pub fn update_item_status(
    conn: &mut Connection,
    id: &str,
    new_status: folio_core::Status,
) -> Result<StatusResult, DbError> {
    let tx = conn.transaction().map_err(|e| DbError::Other(e.into()))?;
    let config = load_config(&tx).map_err(|e| DbError::Other(e.into()))?;

    let inbox_rows = load_items(&tx, View::Inbox).map_err(|e| DbError::Other(e.into()))?;
    let archive_rows = load_items(&tx, View::Archive).map_err(|e| DbError::Other(e.into()))?;
    let inbox_core = inbox_rows;
    let archive_core = archive_rows;

    let result = folio_core::update_item_status(id, new_status, inbox_core, archive_core, &config)
        .map_err(|e| match e {
            folio_core::CoreError::InboxFull => DbError::InboxFull,
            other => DbError::Other(other.into()),
        })?;

    if !result.item_found {
        return Err(DbError::NotFound);
    }

    let target = result
        .inbox_items
        .iter()
        .chain(result.archive_items.iter())
        .find(|item| item.id == id)
        .ok_or(DbError::NotFound)?;

    let now = Utc::now();
    let target_archive = target.status == folio_core::Status::Done;
    let crossed_views = !result.moved_to_archive.is_empty() || result.moved_to_inbox;

    let current_pos: i64 = tx
        .query_row("SELECT position FROM items WHERE id = ?1", [id], |row| {
            row.get(0)
        })
        .map_err(|e| DbError::Other(e.into()))?;
    let position = if crossed_views {
        next_position(&tx, target_archive).map_err(|e| DbError::Other(e.into()))?
    } else {
        current_pos
    };

    tx.execute(
        "UPDATE items SET status = ?1, started_at = ?2, finished_at = ?3, position = ?4, \
         updated_at = ?5 WHERE id = ?6",
        params![
            target.status.as_string(),
            target.started_at.map(|dt| dt.to_rfc3339()),
            target.finished_at.map(|dt| dt.to_rfc3339()),
            position,
            now.to_rfc3339(),
            id,
        ],
    )
    .map_err(|e| DbError::Other(e.into()))?;

    let mut overflow_api = Vec::new();
    for item in &result.overflow_items {
        let position = next_position(&tx, true).map_err(|e| DbError::Other(e.into()))?;
        tx.execute(
            "UPDATE items SET status = 'done', finished_at = ?1, position = ?2, updated_at = ?3 \
             WHERE id = ?4",
            params![
                item.finished_at.map(|dt| dt.to_rfc3339()),
                position,
                now.to_rfc3339(),
                item.id,
            ],
        )
        .map_err(|e| DbError::Other(e.into()))?;
        overflow_api.push(item.clone());
    }

    tx.commit().map_err(|e| DbError::Other(e.into()))?;

    Ok(StatusResult {
        item: target.clone(),
        moved_to_archive: !result.moved_to_archive.is_empty(),
        moved_to_inbox: result.moved_to_inbox,
        overflow: overflow_api,
    })
}

pub fn edit_item(
    conn: &mut Connection,
    id: &str,
    name: Option<String>,
    item_type: Option<String>,
    author: Option<String>,
    link: Option<String>,
    note: Option<String>,
) -> Result<Item, DbError> {
    let tx = conn.transaction().map_err(|e| DbError::Other(e.into()))?;
    let all_rows = load_items(&tx, View::All).map_err(|e| DbError::Other(e.into()))?;
    let item = all_rows
        .iter()
        .find(|row| row.id == id)
        .cloned()
        .ok_or(DbError::NotFound)?;

    let mut core_item = item;
    if let Some(name) = name {
        core_item.name = name;
    }
    if let Some(item_type) = item_type {
        core_item.item_type = ItemType::from_str(&item_type).unwrap_or(ItemType::BlogPost);
    }
    if let Some(author) = author {
        core_item.author = author;
    }
    if let Some(link) = link {
        core_item.link = link;
    }
    if let Some(note) = note {
        core_item.note = note;
    }

    core_item
        .validate()
        .map_err(|e| DbError::Validation(e.to_string()))?;

    let updated = core_item;
    let now = Utc::now();
    let kind_str = match &updated.kind {
        folio_core::Kind::Normal => "normal",
        folio_core::Kind::Reference => "reference",
    };

    tx.execute(
        "UPDATE items SET name = ?1, type = ?2, status = ?3, kind = ?4, author = ?5, link = ?6, \
         note = ?7, added_at = ?8, started_at = ?9, finished_at = ?10, updated_at = ?11 \
         WHERE id = ?12",
        params![
            &updated.name,
            updated.item_type.as_string(),
            updated.status.as_string(),
            kind_str,
            &updated.author,
            &updated.link,
            &updated.note,
            updated.added_at.to_rfc3339(),
            updated.started_at.map(|dt| dt.to_rfc3339()),
            updated.finished_at.map(|dt| dt.to_rfc3339()),
            now.to_rfc3339(),
            id,
        ],
    )
    .map_err(|e| DbError::Other(e.into()))?;

    tx.commit().map_err(|e| DbError::Other(e.into()))?;
    Ok(updated)
}

pub fn delete_item(conn: &mut Connection, id: &str) -> Result<(), DbError> {
    let tx = conn.transaction().map_err(|e| DbError::Other(e.into()))?;
    let deleted = tx
        .execute("DELETE FROM items WHERE id = ?1", [id])
        .map_err(|e| DbError::Other(e.into()))?;
    if deleted == 0 {
        return Err(DbError::NotFound);
    }
    tx.commit().map_err(|e| DbError::Other(e.into()))?;
    Ok(())
}

pub fn toggle_kind(conn: &mut Connection, id: &str) -> Result<Item, DbError> {
    let tx = conn.transaction().map_err(|e| DbError::Other(e.into()))?;
    let all_rows = load_items(&tx, View::All).map_err(|e| DbError::Other(e.into()))?;
    let mut item = all_rows
        .iter()
        .find(|row| row.id == id)
        .cloned()
        .ok_or(DbError::NotFound)?;

    if item.status != folio_core::Status::Done {
        return Err(DbError::NotArchived);
    }

    item.kind = match item.kind {
        folio_core::Kind::Normal => folio_core::Kind::Reference,
        folio_core::Kind::Reference => folio_core::Kind::Normal,
    };

    let kind_str = match &item.kind {
        folio_core::Kind::Normal => "normal",
        folio_core::Kind::Reference => "reference",
    };
    let now = Utc::now();

    tx.execute(
        "UPDATE items SET kind = ?1, updated_at = ?2 WHERE id = ?3",
        params![kind_str, now.to_rfc3339(), id],
    )
    .map_err(|e| DbError::Other(e.into()))?;

    tx.commit().map_err(|e| DbError::Other(e.into()))?;
    Ok(item)
}

pub enum MoveDirection {
    Up,
    Down,
}

pub fn move_item(
    conn: &mut Connection,
    id: &str,
    direction: MoveDirection,
) -> Result<Item, DbError> {
    let tx = conn.transaction().map_err(|e| DbError::Other(e.into()))?;

    let all_rows = load_items(&tx, View::All).map_err(|e| DbError::Other(e.into()))?;
    let target = all_rows
        .iter()
        .find(|item| item.id == id)
        .ok_or(DbError::NotFound)?;

    let in_archive = target.status == folio_core::Status::Done;
    let view_items: Vec<&Item> = all_rows
        .iter()
        .filter(|item| (item.status == folio_core::Status::Done) == in_archive)
        .collect();

    let pos_in_view = view_items
        .iter()
        .position(|item| item.id == id)
        .ok_or(DbError::NotFound)?;

    let swap_index = match direction {
        MoveDirection::Up => {
            if pos_in_view == 0 {
                return Err(DbError::AlreadyAtEdge);
            }
            pos_in_view - 1
        }
        MoveDirection::Down => {
            if pos_in_view + 1 >= view_items.len() {
                return Err(DbError::AlreadyAtEdge);
            }
            pos_in_view + 1
        }
    };

    let mine = view_items[pos_in_view];
    let neighbor = view_items[swap_index];

    let my_pos: i64 = tx
        .query_row("SELECT position FROM items WHERE id = ?1", [id], |row| {
            row.get(0)
        })
        .map_err(|e| DbError::Other(e.into()))?;
    let neighbor_pos: i64 = tx
        .query_row(
            "SELECT position FROM items WHERE id = ?1",
            [&neighbor.id],
            |row| row.get(0),
        )
        .map_err(|e| DbError::Other(e.into()))?;

    let now = Utc::now();
    tx.execute(
        "UPDATE items SET position = ?1, updated_at = ?2 WHERE id = ?3",
        params![neighbor_pos, now.to_rfc3339(), id],
    )
    .map_err(|e| DbError::Other(e.into()))?;
    tx.execute(
        "UPDATE items SET position = ?1, updated_at = ?2 WHERE id = ?3",
        params![my_pos, now.to_rfc3339(), &neighbor.id],
    )
    .map_err(|e| DbError::Other(e.into()))?;

    tx.commit().map_err(|e| DbError::Other(e.into()))?;
    Ok(mine.clone())
}
