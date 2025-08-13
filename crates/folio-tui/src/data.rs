use folio_core::Item;
use folio_storage::{get_archive_path, get_inbox_path, load_items_from_file};

pub async fn load_inbox_items() -> Result<Vec<Item>, Box<dyn std::error::Error>> {
    let inbox_path = get_inbox_path()?;
    load_items_from_file(inbox_path)
}

pub async fn load_archive_items() -> Result<Vec<Item>, Box<dyn std::error::Error>> {
    let archive_path = get_archive_path()?;
    load_items_from_file(archive_path)
}
