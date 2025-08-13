use folio_core::Item;
use folio_storage::{
    append_to_archive, get_archive_path, get_inbox_path, load_items_from_file, save_archive,
    save_inbox,
};

pub async fn load_inbox_items() -> Result<Vec<Item>, Box<dyn std::error::Error>> {
    let inbox_path = get_inbox_path()?;
    load_items_from_file(inbox_path)
}

pub async fn load_archive_items() -> Result<Vec<Item>, Box<dyn std::error::Error>> {
    let archive_path = get_archive_path()?;
    load_items_from_file(archive_path)
}

pub async fn save_inbox_items(items: &[Item]) -> Result<(), Box<dyn std::error::Error>> {
    save_inbox(items)
}

pub async fn save_archive_items(items: &[Item]) -> Result<(), Box<dyn std::error::Error>> {
    save_archive(items)
}

pub async fn append_item_to_archive(item: &Item) -> Result<(), Box<dyn std::error::Error>> {
    append_to_archive(item)
}
