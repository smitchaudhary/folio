use folio_core::{Item, Status};

#[derive(PartialEq)]
pub enum View {
    Inbox,
    Archive,
}

pub struct AppState {
    pub inbox_items: Vec<Item>,
    pub archive_items: Vec<Item>,
    pub selected_index: usize,
    pub current_view: View,
    pub filter: Option<String>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            inbox_items: vec![],
            archive_items: vec![],
            selected_index: 0,
            current_view: View::Inbox,
            filter: None,
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

    pub fn move_selected_to_done(&mut self) -> Option<Item> {
        if self.change_selected_status(Status::Done) {
            if self.current_view == View::Inbox {
                return self.remove_selected_item();
            }
        }
        None
    }

    pub fn move_selected_to_doing(&mut self) -> bool {
        self.change_selected_status(Status::Doing)
    }

    pub fn move_selected_to_todo(&mut self) -> bool {
        self.change_selected_status(Status::Todo)
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

    pub fn next_page(&mut self, page_size: usize) {
        let items_len = self.current_items().len();
        if items_len > 0 {
            self.selected_index = (self.selected_index + page_size).min(items_len - 1);
        }
    }

    pub fn previous_page(&mut self, page_size: usize) {
        self.selected_index = self.selected_index.saturating_sub(page_size);
    }

    pub fn jump_to_first(&mut self) {
        self.selected_index = 0;
    }

    pub fn jump_to_last(&mut self) {
        let items_len = self.current_items().len();
        if items_len > 0 {
            self.selected_index = items_len - 1;
        }
    }

    pub fn add_item_to_archive(&mut self, item: Item) {
        self.archive_items.push(item);
    }

    pub fn get_inbox_items(&self) -> &[Item] {
        &self.inbox_items
    }

    pub fn get_archive_items(&self) -> &[Item] {
        &self.archive_items
    }

    pub fn filtered_items(&self) -> Vec<usize> {
        let items = self.current_items();

        if let Some(filter) = &self.filter {
            items
                .iter()
                .enumerate()
                .filter(|(_, item)| {
                    let text_match = item.name.to_lowercase().contains(&filter.to_lowercase())
                        || item.author.to_lowercase().contains(&filter.to_lowercase());

                    text_match
                })
                .map(|(index, _)| index)
                .collect()
        } else {
            (0..items.len()).collect()
        }
    }

    pub fn set_filter(&mut self, filter: Option<String>) {
        self.filter = filter;
    }

    fn change_selected_status(&mut self, new_status: Status) -> bool {
        if self.current_items().is_empty() || self.selected_index >= self.current_items().len() {
            return false;
        }

        let item_id = match self.current_view {
            View::Inbox => self.selected_index + 1,
            View::Archive => self.inbox_items.len() + self.selected_index + 1,
        };

        if let Ok(config_manager) = folio_storage::ConfigManager::new() {
            let config = config_manager.get();

            match folio_core::update_item_status(
                item_id,
                new_status,
                self.inbox_items.clone(),
                self.archive_items.clone(),
                config,
            ) {
                Ok(result) => {
                    if result.item_found {
                        self.inbox_items = result.inbox_items;
                        self.archive_items = result.archive_items;

                        // Adjust selected index if needed
                        if self.current_view == View::Archive && result.moved_to_inbox {
                            if self.selected_index >= self.archive_items.len()
                                && !self.archive_items.is_empty()
                            {
                                self.selected_index = self.archive_items.len() - 1;
                            }
                        }

                        return true;
                    }
                }
                Err(_) => {
                    return false;
                }
            }
        }

        false
    }
}
