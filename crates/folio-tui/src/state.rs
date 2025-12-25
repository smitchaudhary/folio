use folio_core::{CoreError, Item, Status, StatusUpdateResult};

#[derive(PartialEq)]
pub enum View {
    Inbox,
    Archive,
}

pub struct AppState {
    pub inbox_items: Vec<Item>,
    pub archive_items: Vec<Item>,
    pub selected_item_id: Option<usize>,
    pub current_view: View,
    pub filter: Option<String>,
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

impl AppState {
    pub fn new() -> Self {
        Self {
            inbox_items: vec![],
            archive_items: vec![],
            selected_item_id: None,
            current_view: View::Inbox,
            filter: None,
        }
    }

    pub fn load_inbox_items(&mut self, items: Vec<Item>) {
        self.inbox_items = items;
        self.selected_item_id = self.inbox_items.first().map(|item| item.id());
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
        let selected_id = self.selected_item_id?;
        self.current_items()
            .iter()
            .find(|item| item.id() == selected_id)
    }

    pub fn selected_item_mut(&mut self) -> Option<&mut Item> {
        let selected_id = self.selected_item_id?;
        self.current_items_mut()
            .iter_mut()
            .find(|item| item.id() == selected_id)
    }

    pub fn move_selected_to_done(&mut self) -> Result<Option<Item>, CoreError> {
        let result = self.change_selected_status(Status::Done)?;

        if self.current_view == View::Inbox && !result.moved_to_archive.is_empty() {
            Ok(result.moved_to_archive.into_iter().next())
        } else {
            Ok(None)
        }
    }

    pub fn move_selected_to_doing(&mut self) -> Result<StatusUpdateResult, CoreError> {
        self.change_selected_status(Status::Doing)
    }

    pub fn move_selected_to_todo(&mut self) -> Result<StatusUpdateResult, CoreError> {
        self.change_selected_status(Status::Todo)
    }

    pub fn next_item(&mut self) {
        let visible: Vec<usize> = self.visible_items().into_iter().map(|(id, _)| id).collect();

        if visible.is_empty() {
            self.selected_item_id = None;
            return;
        }

        match self.selected_item_id {
            None => {
                self.selected_item_id = visible.first().copied();
            }
            Some(current_id) => {
                if let Some(pos) = visible.iter().position(|&id| id == current_id) {
                    let next_pos = (pos + 1).min(visible.len() - 1);
                    self.selected_item_id = visible.get(next_pos).copied();
                } else {
                    self.selected_item_id = visible.first().copied();
                }
            }
        }
    }

    pub fn previous_item(&mut self) {
        let visible: Vec<usize> = self.visible_items().into_iter().map(|(id, _)| id).collect();

        if visible.is_empty() {
            self.selected_item_id = None;
            return;
        }

        match self.selected_item_id {
            None => {
                self.selected_item_id = visible.first().copied();
            }
            Some(current_id) => {
                if let Some(pos) = visible.iter().position(|&id| id == current_id) {
                    let prev_pos = pos.saturating_sub(1);
                    self.selected_item_id = visible.get(prev_pos).copied();
                } else {
                    self.selected_item_id = visible.first().copied();
                }
            }
        }
    }

    pub fn next_page(&mut self, page_size: usize) {
        let visible: Vec<usize> = self.visible_items().into_iter().map(|(id, _)| id).collect();

        if visible.is_empty() {
            self.selected_item_id = None;
            return;
        }

        match self.selected_item_id {
            None => {
                self.selected_item_id = visible.first().copied();
            }
            Some(current_id) => {
                if let Some(pos) = visible.iter().position(|&id| id == current_id) {
                    let next_pos = (pos + page_size).min(visible.len() - 1);
                    self.selected_item_id = visible.get(next_pos).copied();
                } else {
                    self.selected_item_id = visible.first().copied();
                }
            }
        }
    }

    pub fn previous_page(&mut self, page_size: usize) {
        let visible: Vec<usize> = self.visible_items().into_iter().map(|(id, _)| id).collect();

        if visible.is_empty() {
            self.selected_item_id = None;
            return;
        }

        match self.selected_item_id {
            None => {
                self.selected_item_id = visible.first().copied();
            }
            Some(current_id) => {
                if let Some(pos) = visible.iter().position(|&id| id == current_id) {
                    let prev_pos = pos.saturating_sub(page_size);
                    self.selected_item_id = visible.get(prev_pos).copied();
                } else {
                    self.selected_item_id = visible.first().copied();
                }
            }
        }
    }

    pub fn jump_to_first(&mut self) {
        let visible: Vec<usize> = self.visible_items().into_iter().map(|(id, _)| id).collect();

        self.selected_item_id = visible.first().copied();
    }

    pub fn jump_to_last(&mut self) {
        let visible: Vec<usize> = self.visible_items().into_iter().map(|(id, _)| id).collect();

        self.selected_item_id = visible.last().copied();
    }

    pub fn move_item_up(&mut self) -> Result<(), CoreError> {
        let selected_id = self.selected_item_id.ok_or(CoreError::ItemNotFound)?;
        let items = self.current_items_mut();

        let pos = items
            .iter()
            .position(|item| item.id() == selected_id)
            .ok_or(CoreError::ItemNotFound)?;

        if pos == 0 {
            return Err(CoreError::ValidationError("Already at top".to_string()));
        }

        items.swap(pos, pos - 1);
        Ok(())
    }

    pub fn move_item_down(&mut self) -> Result<(), CoreError> {
        let selected_id = self.selected_item_id.ok_or(CoreError::ItemNotFound)?;
        let items = self.current_items_mut();

        let pos = items
            .iter()
            .position(|item| item.id() == selected_id)
            .ok_or(CoreError::ItemNotFound)?;

        if pos >= items.len() - 1 {
            return Err(CoreError::ValidationError("Already at bottom".to_string()));
        }

        items.swap(pos, pos + 1);
        Ok(())
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

    pub fn set_filter(&mut self, filter: Option<String>) {
        self.filter = filter;

        if let Some(selected_id) = self.selected_item_id {
            let still_visible = self
                .visible_items()
                .iter()
                .any(|(id, _)| *id == selected_id);

            if !still_visible {
                self.selected_item_id = self.visible_items().first().map(|(id, _)| *id);
            }
        } else {
            self.selected_item_id = self.visible_items().first().map(|(id, _)| *id);
        }
    }

    pub fn visible_items(&self) -> Vec<(usize, &Item)> {
        let items = self.current_items();

        items
            .iter()
            .filter(|item| self.matches_filter(item))
            .map(|item| (item.id(), item))
            .collect()
    }

    fn matches_filter(&self, item: &Item) -> bool {
        match &self.filter {
            None => true,
            Some(f) => {
                let filter_lower = f.to_lowercase();
                item.name.to_lowercase().contains(&filter_lower)
                    || item.author.to_lowercase().contains(&filter_lower)
            }
        }
    }

    pub fn selected_table_row(&self) -> Option<usize> {
        let selected_id = self.selected_item_id?;

        self.visible_items()
            .iter()
            .position(|(id, _)| *id == selected_id)
    }

    pub fn reselect_visible_row(&mut self, preferred_row: Option<usize>) {
        let visible: Vec<usize> = self.visible_items().into_iter().map(|(id, _)| id).collect();

        if visible.is_empty() {
            self.selected_item_id = None;
            return;
        }

        let idx = preferred_row.unwrap_or(0).min(visible.len() - 1);
        self.selected_item_id = visible.get(idx).copied();
    }

    fn change_selected_status(
        &mut self,
        new_status: Status,
    ) -> Result<StatusUpdateResult, CoreError> {
        let selected_id = self.selected_item_id.ok_or(CoreError::ItemNotFound)?;
        let previous_row = self.selected_table_row();

        let (item_id, found_in_view) = if let Some(pos) = self
            .inbox_items
            .iter()
            .position(|item| item.id() == selected_id)
        {
            (pos + 1, View::Inbox)
        } else if let Some(pos) = self
            .archive_items
            .iter()
            .position(|item| item.id() == selected_id)
        {
            (self.inbox_items.len() + pos + 1, View::Archive)
        } else {
            return Err(CoreError::ItemNotFound);
        };

        let config_manager = folio_storage::ConfigManager::new()
            .map_err(|_| CoreError::ValidationError("Failed to load config".to_string()))?;
        let config = config_manager.get();

        let result = folio_core::update_item_status(
            item_id,
            new_status,
            self.inbox_items.clone(),
            self.archive_items.clone(),
            config,
        )?;

        if result.item_found {
            self.inbox_items = result.inbox_items.clone();
            self.archive_items = result.archive_items.clone();

            if found_in_view == View::Archive && result.moved_to_inbox {
                if self.current_view == View::Archive {
                    self.reselect_visible_row(previous_row);
                }
            } else if self.current_view == View::Inbox
                && result
                    .moved_to_archive
                    .iter()
                    .any(|item| item.id() == selected_id)
            {
                self.reselect_visible_row(previous_row);
            }

            Ok(result)
        } else {
            Err(CoreError::ItemNotFound)
        }
    }
}
