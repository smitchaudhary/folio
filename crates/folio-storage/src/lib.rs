use folio_core::{Config, Item};
use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, BufReader, Read, Write};
use std::path::{Path, PathBuf};

pub mod error;
pub mod fs_atomic;

pub use error::{StorageError, StorageResult};

pub fn deserialize_jsonl_from_string(jsonl_str: &str) -> StorageResult<Vec<Item>> {
    let mut items = Vec::new();

    for line in jsonl_str.lines() {
        if line.trim().is_empty() {
            continue;
        }

        let item: Item = serde_json::from_str(line).map_err(|_| StorageError::JsonlParse)?;
        items.push(item);
    }

    Ok(items)
}

pub fn deserialize_jsonl_from_reader<R: Read>(reader: R) -> StorageResult<Vec<Item>> {
    let buf_reader = BufReader::new(reader);
    let mut items = Vec::new();

    for line in buf_reader.lines() {
        let line = line?;

        if line.trim().is_empty() {
            continue;
        }

        let item: Item = serde_json::from_str(&line).map_err(|_| StorageError::JsonlParse)?;
        items.push(item);
    }

    Ok(items)
}

pub fn load_items_from_file<P: AsRef<Path>>(path: P) -> StorageResult<Vec<Item>> {
    let path_ref = path.as_ref();
    match File::open(path_ref) {
        Ok(file) => {
            let items = deserialize_jsonl_from_reader(file)?;
            Ok(items)
        }
        Err(ref e) if e.kind() == std::io::ErrorKind::NotFound => {
            // File doesn't exist yet, return empty vec
            Ok(vec![])
        }
        Err(_) => Err(StorageError::FileRead {
            path: path_ref.to_path_buf(),
        }),
    }
}

pub fn serialize_items_to_jsonl(items: &[Item]) -> StorageResult<String> {
    let mut jsonl = String::new();

    for item in items {
        let line = serde_json::to_string(item).map_err(|_| StorageError::JsonSerialization)?;
        jsonl.push_str(&line);
        jsonl.push('\n');
    }

    Ok(jsonl)
}

pub fn ensure_folio_dir() -> StorageResult<()> {
    let folio_dir = get_folio_dir()?;

    if !folio_dir.exists() {
        fs::create_dir_all(&folio_dir).map_err(|_| StorageError::DirectoryCreation {
            path: folio_dir.clone(),
        })?;
    }

    Ok(())
}

pub fn get_folio_dir() -> StorageResult<PathBuf> {
    let folio_dir = dirs::home_dir()
        .ok_or(StorageError::HomeDirectoryNotFound)?
        .join(".folio");

    Ok(folio_dir)
}

pub fn get_inbox_path() -> StorageResult<PathBuf> {
    let inbox_path = get_folio_dir()?.join("inbox.jsonl");
    Ok(inbox_path)
}

pub fn get_archive_path() -> StorageResult<PathBuf> {
    let archive_path = get_folio_dir()?.join("archive.jsonl");
    Ok(archive_path)
}

pub fn get_config_path() -> StorageResult<PathBuf> {
    let config_path = get_folio_dir()?.join("config.json");
    Ok(config_path)
}

fn save_items(items: &[Item], path: PathBuf) -> StorageResult<()> {
    let jsonl = serialize_items_to_jsonl(items)?;
    fs_atomic::atomic_write(&path, jsonl.as_bytes())
        .map_err(|_| StorageError::FileWrite { path })?;
    Ok(())
}

pub fn save_inbox(items: &[Item]) -> StorageResult<()> {
    ensure_folio_dir()?;
    save_items(items, get_inbox_path()?)
}

pub fn save_archive(items: &[Item]) -> StorageResult<()> {
    ensure_folio_dir()?;
    save_items(items, get_archive_path()?)
}

pub fn append_to_archive(item: &Item) -> StorageResult<()> {
    ensure_folio_dir()?;
    let archive_path = get_archive_path()?;
    let json_line = serde_json::to_string(item).map_err(|_| StorageError::JsonSerialization)?;

    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&archive_path)
        .map_err(|_| StorageError::FileWrite {
            path: archive_path.clone(),
        })?;

    writeln!(file, "{}", json_line).map_err(|_| StorageError::FileWrite { path: archive_path })?;
    Ok(())
}

pub fn load_config() -> StorageResult<Config> {
    let config_path = get_config_path()?;

    if config_path.exists() {
        let file = File::open(&config_path).map_err(|_| StorageError::FileRead {
            path: config_path.clone(),
        })?;
        let config: Config =
            serde_json::from_reader(file).map_err(|_| StorageError::JsonDeserialization)?;
        Ok(config)
    } else {
        Ok(Config::default())
    }
}

pub fn save_config(config: &Config) -> StorageResult<()> {
    ensure_folio_dir()?;
    let config_path = get_config_path()?;
    let json = serde_json::to_string_pretty(config).map_err(|_| StorageError::JsonSerialization)?;
    fs_atomic::atomic_write(&config_path, json.as_bytes())
        .map_err(|_| StorageError::FileWrite { path: config_path })?;
    Ok(())
}

pub struct ConfigManager {
    config: Config,
}

impl ConfigManager {
    pub fn new() -> StorageResult<Self> {
        let config = load_config()?;
        Ok(Self { config })
    }

    pub fn get(&self) -> &Config {
        &self.config
    }

    pub fn save(&self) -> StorageResult<()> {
        save_config(&self.config)
    }

    pub fn update<F>(&mut self, updater: F) -> StorageResult<()>
    where
        F: FnOnce(&mut Config),
    {
        updater(&mut self.config);
        self.save()
    }
}
