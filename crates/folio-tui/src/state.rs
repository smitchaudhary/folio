use folio_core::{Item, Status};

pub struct AppState {
    pub inbox_items: Vec<Item>,
    pub archive_items: Vec<Item>,
    pub selected_index: usize,
    pub current_view: View,
}

#[derive(PartialEq)]
pub enum View {
    Inbox,
    Archive,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            inbox_items: vec![],
            archive_items: vec![],
            selected_index: 0,
            current_view: View::Inbox,
        }
    }

    pub fn load_inbox_items(&mut self, items: Vec<Item>) {
        self.inbox_items = items;
        self.selected_index = 0;
    }

    pub fn load_archive_items(&mut self, items: Vec<Item>) {
        self.archive_items = items;
    }

    pub fn current_items(&self) -> &Vec<Item> {
        match self.current_view {
            View::Inbox => &self.inbox_items,
            View::Archive => &self.archive_items,
        }
    }

    pub fn current_items_mut(&mut self) -> &mut Vec<Item> {
        match self.current_view {
            View::Inbox => &mut self.inbox_items,
            View::Archive => &mut self.archive_items,
        }
    }

    pub fn selected_item(&self) -> Option<&Item> {
        let items = match self.current_view {
            View::Inbox => &self.inbox_items,
            View::Archive => &self.archive_items,
        };
        if items.is_empty() || self.selected_index >= items.len() {
            None
        } else {
            Some(&items[self.selected_index])
        }
    }

    pub fn selected_item_mut(&mut self) -> Option<&mut Item> {
        let items_len = match self.current_view {
            View::Inbox => self.inbox_items.len(),
            View::Archive => self.archive_items.len(),
        };
        if items_len == 0 || self.selected_index >= items_len {
            return None;
        }
        match self.current_view {
            View::Inbox => Some(&mut self.inbox_items[self.selected_index]),
            View::Archive => Some(&mut self.archive_items[self.selected_index]),
        }
    }

    pub fn cycle_selected_item_status(&mut self) -> Option<Item> {
        if let Some(item) = self.selected_item_mut() {
            let old_status = item.status.clone();
            folio_core::cycle_status(item);
            folio_core::update_timestamps(item);

            if old_status != Status::Done && item.status == Status::Done {
                return self.remove_selected_item();
            }
        }
        None
    }

    pub fn move_selected_to_done(&mut self) -> Option<Item> {
        if let Some(item) = self.selected_item_mut() {
            let old_status = item.status.clone();
            item.status = Status::Done;
            folio_core::update_timestamps(item);

            if old_status != Status::Done {
                return self.remove_selected_item();
            }
        }
        None
    }

    pub fn move_selected_to_doing(&mut self) -> bool {
        if let Some(item) = self.selected_item_mut() {
            item.status = Status::Doing;
            folio_core::update_timestamps(item);
            true
        } else {
            false
        }
    }

    pub fn move_selected_to_todo(&mut self) {
        if let Some(item) = self.selected_item_mut() {
            item.status = Status::Todo;
            folio_core::update_timestamps(item);
        }
    }

    fn remove_selected_item(&mut self) -> Option<Item> {
        if self.current_view != View::Inbox {
            return None;
        }

        if self.inbox_items.is_empty() || self.selected_index >= self.inbox_items.len() {
            return None;
        }

        let removed_item = self.inbox_items.remove(self.selected_index);

        if self.selected_index >= self.inbox_items.len() && !self.inbox_items.is_empty() {
            self.selected_index = self.inbox_items.len() - 1;
        }

        Some(removed_item)
    }

    pub fn next_item(&mut self) {
        let items_len = self.current_items().len();
        if items_len > 0 {
            self.selected_index = (self.selected_index + 1).min(items_len - 1);
        }
    }

    pub fn previous_item(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
        }
    }

    pub fn add_item_to_archive(&mut self, item: Item) {
        self.archive_items.push(item);
    }
}
