use folio_core::Item;
use serde_json;
use std::io::{BufRead, BufReader, Read};

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
