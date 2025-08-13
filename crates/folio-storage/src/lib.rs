use folio_core::Item;
use serde_json;
use std::fs::File;
use std::io::{BufRead, BufReader, Read};
use std::path::Path;

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
