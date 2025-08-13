use folio_core::Item;
use serde_json;
use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, BufReader, Read, Write};
use std::path::{Path, PathBuf};

pub mod fs_atomic;

pub fn deserialize_jsonl_from_string(jsonl_str: &str) -> Result<Vec<Item>, serde_json::Error> {
    let mut items = Vec::new();

    for line in jsonl_str.lines() {
        if line.trim().is_empty() {
            continue;
        }

        let item: Item = serde_json::from_str(line)?;
        items.push(item);
    }

    Ok(items)
}

pub fn deserialize_jsonl_from_reader<R: Read>(
    reader: R,
) -> Result<Vec<Item>, Box<dyn std::error::Error>> {
    let buf_reader = BufReader::new(reader);
    let mut items = Vec::new();

    for line in buf_reader.lines() {
        let line = line?;

        if line.trim().is_empty() {
            continue;
        }

        let item: Item = serde_json::from_str(&line)?;
        items.push(item);
    }

    Ok(items)
}

pub fn load_items_from_file<P: AsRef<Path>>(
    path: P,
) -> Result<Vec<Item>, Box<dyn std::error::Error>> {
    match File::open(path) {
        Ok(file) => {
            let items = deserialize_jsonl_from_reader(file)?;
            Ok(items)
        }
        Err(_) => Ok(vec![]),
    }
}

pub fn serialize_items_to_jsonl(items: &[Item]) -> Result<String, serde_json::Error> {
    let mut jsonl = String::new();

    for item in items {
        let line = serde_json::to_string(item)?;
        jsonl.push_str(&line);
        jsonl.push('\n');
    }

    Ok(jsonl)
}

pub fn ensure_folio_dir() -> Result<(), Box<dyn std::error::Error>> {
    let folio_dir = get_folio_dir()?;

    if !folio_dir.exists() {
        fs::create_dir_all(&folio_dir)?;
    }

    Ok(())
}

pub fn get_folio_dir() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let folio_dir = dirs::home_dir()
        .ok_or("Could not determine home directory")?
        .join(".folio");

    Ok(folio_dir)
}

pub fn get_inbox_path() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let inbox_path = get_folio_dir()?.join("inbox.jsonl");
    Ok(inbox_path)
}

pub fn get_archive_path() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let archive_path = get_folio_dir()?.join("archive.jsonl");
    Ok(archive_path)
}

pub fn get_config_path() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let config_path = get_folio_dir()?.join("config.json");
    Ok(config_path)
}

fn save_items(items: &[Item], path: PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    let jsonl = serialize_items_to_jsonl(items)?;
    fs_atomic::atomic_write(path, jsonl.as_bytes())?;
    Ok(())
}

pub fn save_inbox(items: &[Item]) -> Result<(), Box<dyn std::error::Error>> {
    ensure_folio_dir()?;
    save_items(items, get_inbox_path()?)
}

pub fn save_archive(items: &[Item]) -> Result<(), Box<dyn std::error::Error>> {
    ensure_folio_dir()?;
    save_items(items, get_archive_path()?)
}

pub fn append_to_archive(item: &Item) -> Result<(), Box<dyn std::error::Error>> {
    ensure_folio_dir()?;
    let archive_path = get_archive_path()?;
    let json_line = serde_json::to_string(item)?;
    
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(archive_path)?;
        
    writeln!(file, "{}", json_line)?;
    Ok(())
}