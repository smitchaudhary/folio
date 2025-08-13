use crate::{Item, Status};

pub fn should_auto_archive(item: &Item) -> bool {
    matches!(item.status, Status::Done)
}

pub fn move_to_archive(inbox: &mut Vec<Item>, archive: &mut Vec<Item>, index: usize) -> Option<Item> {
    if index < inbox.len() {
        let item = inbox.remove(index);
        archive.push(item.clone());
        Some(item)
    } else {
        None
    }
}