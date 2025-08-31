use crate::data::{load_archive_items, load_inbox_items, save_archive_items, save_inbox_items};
use crate::error::TuiResult;
use crate::event::{AppEvent, EventHandler};
use crate::forms::{FormType, ItemForm};
use crate::state::{AppState, View};
use crate::terminal::{restore_terminal, setup_terminal};
use crate::widgets::ItemsTable;
use crossterm::event::{KeyCode, KeyEvent};
use folio_core::{Item, OverflowStrategy};
use folio_storage::ConfigManager;
use ratatui::widgets::TableState;
use std::str::FromStr;
use std::time::{Duration, Instant};

pub struct App {
    pub should_quit: bool,
    pub state: AppState,
    pub table_state: TableState,
    pub add_form: ItemForm,
    pub edit_form: ItemForm,
    pub show_delete_confirmation: bool,
    pub show_help: bool,
    pub status_message: Option<(String, Instant)>,
    pub filter_input_mode: bool,
    pub filter_input: String,
    pub start_with_add_form: bool,
    pub show_cap_warning: bool,
    pub cap_warning_message: String,
    pub pending_add_item: Option<Item>,
    pub show_done_confirmation: bool,
    pub show_config_dialog: bool,
    pub config_max_items_input: String,
    pub config_overflow_strategy: usize,
}

impl App {
    pub fn new() -> Self {
        Self {
            should_quit: false,
            state: AppState::new(),
            table_state: TableState::default(),
            add_form: ItemForm::new(FormType::Add),
            edit_form: ItemForm::new(FormType::Edit),
            show_delete_confirmation: false,
            show_help: false,
            status_message: None,
            filter_input_mode: false,
            filter_input: String::new(),
            start_with_add_form: false,
            show_cap_warning: false,
            cap_warning_message: String::new(),
            pending_add_item: None,
            show_done_confirmation: false,
            show_config_dialog: false,
            config_max_items_input: String::new(),
            config_overflow_strategy: 0,
        }
    }

    pub fn new_with_add_form() -> Self {
        let mut app = Self::new();
        app.start_with_add_form = true;
        app
    }

    pub async fn load_data(&mut self) -> TuiResult<()> {
        let inbox_items = load_inbox_items().await?;
        let archive_items = load_archive_items().await?;

        self.state.load_inbox_items(inbox_items);
        self.state.load_archive_items(archive_items);

        Ok(())
    }

    pub async fn save_data(&mut self) -> TuiResult<()> {
        save_inbox_items(self.state.get_inbox_items()).await?;
        save_archive_items(self.state.get_archive_items()).await?;
        self.show_status_message("Saved".to_string());
        Ok(())
    }

    fn show_status_message(&mut self, message: String) {
        self.status_message = Some((message, Instant::now()));
    }

    async fn handle_key_event(&mut self, key_event: KeyEvent) {
        if self.show_help {
            if let KeyCode::Char('?') | KeyCode::Esc = key_event.code {
                self.show_help = false;
            }
            return;
        }

        if self.show_cap_warning {
            if let KeyCode::Esc | KeyCode::Char('q') | KeyCode::Enter = key_event.code {
                self.show_cap_warning = false;
            }
            return;
        }

        if self.show_delete_confirmation {
            match key_event.code {
                KeyCode::Char('y') | KeyCode::Char('Y') => {
                    self.delete_selected_item().await;
                    self.show_delete_confirmation = false;
                }
                KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                    self.show_delete_confirmation = false;
                }
                _ => {}
            }
            return;
        }

        if self.show_done_confirmation {
            match key_event.code {
                KeyCode::Char('y') | KeyCode::Char('Y') => {
                    match self.state.move_selected_to_done() {
                        Ok(maybe_item) => {
                            if maybe_item.is_some() {
                                self.show_status_message("Item archived".to_string());
                            }
                            let _ = self.save_data().await;
                        }
                        Err(_) => {
                            self.show_status_message("Failed to archive item".to_string());
                        }
                    }
                    self.show_done_confirmation = false;
                }
                KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                    self.show_done_confirmation = false;
                }
                _ => {}
            }
            return;
        }

        if self.show_config_dialog {
            match key_event.code {
                KeyCode::Esc => {
                    self.show_config_dialog = false;
                }
                KeyCode::Enter => {
                    self.save_config_changes();
                }
                KeyCode::Char('j') | KeyCode::Down => {
                    self.config_overflow_strategy = (self.config_overflow_strategy + 1) % 3;
                }
                KeyCode::Char('k') | KeyCode::Up => {
                    self.config_overflow_strategy = (self.config_overflow_strategy + 2) % 3;
                }
                KeyCode::Char(c) if c.is_ascii_digit() && key_event.code != KeyCode::Enter => {
                    if self.config_max_items_input.len() < 4 {
                        self.config_max_items_input.push(c);
                    }
                }
                KeyCode::Backspace => {
                    self.config_max_items_input.pop();
                }
                _ => {}
            }
            return;
        }

        if self.add_form.is_visible {
            match key_event.code {
                KeyCode::Esc => {
                    self.add_form.toggle_visibility();
                }
                KeyCode::Enter => {
                    if self.submit_add_form().await {
                        self.add_form.toggle_visibility();
                    }
                }
                _ => {
                    self.add_form.handle_key(key_event);
                }
            }
            return;
        }

        if self.edit_form.is_visible {
            match key_event.code {
                KeyCode::Esc => {
                    self.edit_form.toggle_visibility();
                }
                KeyCode::Enter => {
                    if self.submit_edit_form().await {
                        self.edit_form.toggle_visibility();
                    }
                }
                _ => {
                    self.edit_form.handle_key(key_event);
                }
            }
            return;
        }

        if self.filter_input_mode {
            match key_event.code {
                KeyCode::Esc => {
                    self.filter_input_mode = false;
                    self.filter_input.clear();
                    self.state.set_filter(None);
                }
                KeyCode::Enter => {
                    self.filter_input_mode = false;
                    if self.filter_input.is_empty() {
                        self.state.set_filter(None);
                    } else {
                        self.state.set_filter(Some(self.filter_input.clone()));
                    }
                }
                KeyCode::Backspace => {
                    self.filter_input.pop();
                }
                KeyCode::Char(c) => {
                    self.filter_input.push(c);
                }
                _ => {}
            }
            return;
        }

        match key_event.code {
            KeyCode::Char('q') => self.should_quit = true,
            KeyCode::Esc => self.should_quit = true,
            KeyCode::Char('/') => {
                self.filter_input_mode = true;
                self.filter_input.clear();
            }
            KeyCode::Down => {
                self.state.next_item();
                self.table_state.select(Some(self.state.selected_index));
            }
            KeyCode::Up => {
                self.state.previous_item();
                self.table_state.select(Some(self.state.selected_index));
            }
            KeyCode::Char('j') => {
                self.state.next_item();
                self.table_state.select(Some(self.state.selected_index));
            }
            KeyCode::Char('k') => {
                self.state.previous_item();
                self.table_state.select(Some(self.state.selected_index));
            }
            KeyCode::PageDown => {
                self.state.next_page(10);
                self.table_state.select(Some(self.state.selected_index));
            }
            KeyCode::PageUp => {
                self.state.previous_page(10);
                self.table_state.select(Some(self.state.selected_index));
            }
            KeyCode::Home => {
                self.state.jump_to_first();
                self.table_state.select(Some(self.state.selected_index));
            }
            KeyCode::End => {
                self.state.jump_to_last();
                self.table_state.select(Some(self.state.selected_index));
            }
            KeyCode::Char('t') => match self.state.move_selected_to_todo() {
                Ok(result) => {
                    let _ = self.save_data().await;
                    if result.moved_to_inbox && !result.overflow_items.is_empty() {
                        self.show_status_message(format!(
                            "Moved to inbox. {} item(s) archived due to overflow",
                            result.overflow_items.len()
                        ));
                    } else {
                        self.show_status_message("Status set to Todo".to_string());
                    }
                }
                Err(folio_core::CoreError::InboxFull) => {
                    self.show_status_message(
                        "Cannot move to inbox: capacity limit reached".to_string(),
                    );
                }
                Err(_) => {
                    self.show_status_message("Failed to update status".to_string());
                }
            },
            KeyCode::Char('i') => match self.state.move_selected_to_doing() {
                Ok(result) => {
                    let _ = self.save_data().await;
                    if result.moved_to_inbox && !result.overflow_items.is_empty() {
                        self.show_status_message(format!(
                            "Moved to inbox. {} item(s) archived due to overflow",
                            result.overflow_items.len()
                        ));
                    } else {
                        self.show_status_message("Status set to Doing".to_string());
                    }
                }
                Err(folio_core::CoreError::InboxFull) => {
                    self.show_status_message(
                        "Cannot move to inbox: capacity limit reached".to_string(),
                    );
                }
                Err(_) => {
                    self.show_status_message("Failed to update status".to_string());
                }
            },
            KeyCode::Char('d') => {
                if self.state.selected_item().is_some() {
                    self.show_done_confirmation = true;
                    self.show_status_message("Status set to Done".to_string());
                }
            }
            KeyCode::Char('a') => {
                self.check_cap_before_add();
            }
            KeyCode::Char('x') => {
                self.show_delete_confirmation = true;
            }
            KeyCode::Char('r') => {
                self.toggle_reference_status().await;
                self.show_status_message("Reference status toggled".to_string());
            }
            KeyCode::Char('e') => {
                self.start_edit_item();
            }
            KeyCode::Char('?') => {
                self.show_help = true;
            }
            KeyCode::Char('C') => {
                self.show_config_dialog = true;
                if let Ok(config_manager) = ConfigManager::new() {
                    let config = config_manager.get();
                    self.config_max_items_input = config.max_items.to_string();
                    self.config_overflow_strategy = match config.archive_on_overflow {
                        OverflowStrategy::Abort => 0,
                        OverflowStrategy::Todo => 1,
                        OverflowStrategy::Any => 2,
                    };
                } else {
                    self.config_max_items_input = "30".to_string();
                    self.config_overflow_strategy = 0;
                }
            }
            KeyCode::Tab => {
                match self.state.current_view {
                    View::Inbox => self.state.current_view = View::Archive,
                    View::Archive => self.state.current_view = View::Inbox,
                }
                self.state.selected_index = 0;
                self.table_state.select(Some(0));
                self.show_status_message(
                    match self.state.current_view {
                        View::Inbox => "Switched to Inbox",
                        View::Archive => "Switched to Archive",
                    }
                    .to_string(),
                );
            }
            KeyCode::Enter => {
                if let Some(item) = self.state.selected_item() {
                    if !item.link.is_empty() {
                        // Ensure link has a protocol for better compatibility
                        let link_to_open = if item.link.starts_with("http://")
                            || item.link.starts_with("https://")
                        {
                            item.link.clone()
                        } else {
                            format!("https://{}", item.link)
                        };

                        match opener::open(&link_to_open) {
                            Ok(_) => {
                                self.show_status_message("Opening link...".to_string());
                            }
                            Err(_) => {
                                // Try without https:// prefix if we added it
                                if link_to_open != item.link {
                                    let _ = opener::open(&item.link);
                                }
                                self.show_status_message("Opening link...".to_string());
                            }
                        }
                    }
                }
            }
            _ => {}
        }
    }

    fn check_cap_before_add(&mut self) {
        match ConfigManager::new() {
            Ok(config_manager) => {
                let config = config_manager.get();
                if self.state.inbox_items.len() >= config.max_items as usize {
                    match config.archive_on_overflow {
                        OverflowStrategy::Abort => {
                            self.cap_warning_message = format!(
                                "Inbox limit ({}) reached.\n\n\
                                Choose an action:\n\
                                • Delete an existing item (use 'x' key to delete)\n\
                                • Archive an item (change status to 'done' or use 'A' key)\n\
                                • Adjust inbox size: `folio config set max_items N`\n\
                                • Change overflow strategy: `folio config set archive_on_overflow [todo|any]`",
                                config.max_items
                            );
                            self.show_cap_warning = true;
                        }
                        OverflowStrategy::Todo | OverflowStrategy::Any => {
                            self.add_form.toggle_visibility();
                        }
                    }
                } else {
                    self.add_form.toggle_visibility();
                }
            }
            Err(_) => {
                self.add_form.toggle_visibility();
            }
        }
    }

    async fn toggle_reference_status(&mut self) {
        if self.state.current_view != View::Archive {
            return;
        }

        if let Some(item) = self.state.selected_item_mut() {
            match item.kind {
                folio_core::Kind::Normal => item.kind = folio_core::Kind::Reference,
                folio_core::Kind::Reference => item.kind = folio_core::Kind::Normal,
            }
            let _ = self.save_data().await;
        }
    }

    fn save_config_changes(&mut self) {
        match self.config_max_items_input.parse::<u32>() {
            Ok(max_items) => {
                if max_items > 0 && max_items <= 1000 {
                    let strategy = match self.config_overflow_strategy {
                        0 => OverflowStrategy::Abort,
                        1 => OverflowStrategy::Todo,
                        2 => OverflowStrategy::Any,
                        _ => OverflowStrategy::Abort,
                    };

                    match ConfigManager::new() {
                        Ok(mut config_manager) => {
                            match config_manager.update(|config| {
                                config.max_items = max_items;
                                config.archive_on_overflow = strategy;
                            }) {
                                Ok(_) => {
                                    self.show_status_message(
                                        "Config saved successfully".to_string(),
                                    );
                                    self.show_config_dialog = false;
                                }
                                Err(_) => {
                                    self.show_status_message("Failed to save config".to_string());
                                }
                            }
                        }
                        Err(_) => {
                            self.show_status_message("Failed to load config".to_string());
                        }
                    }
                } else {
                    self.show_status_message("Max items must be 1-1000".to_string());
                }
            }
            Err(_) => {
                self.show_status_message("Invalid number format".to_string());
            }
        }
    }

    fn start_edit_item(&mut self) {
        if let Some(item) = self.state.selected_item() {
            self.edit_form.populate_fields(item);
            self.edit_form.toggle_visibility();
        } else {
            self.show_status_message("No item selected".to_string());
        }
    }

    async fn delete_selected_item(&mut self) {
        match self.state.current_view {
            View::Inbox => {
                if !self.state.inbox_items.is_empty()
                    && self.state.selected_index < self.state.inbox_items.len()
                {
                    self.state.inbox_items.remove(self.state.selected_index);
                    if self.state.selected_index >= self.state.inbox_items.len()
                        && !self.state.inbox_items.is_empty()
                    {
                        self.state.selected_index = self.state.inbox_items.len() - 1;
                    }
                    let _ = self.save_data().await;
                    self.show_status_message("Item deleted".to_string());
                }
            }
            View::Archive => {
                if !self.state.archive_items.is_empty()
                    && self.state.selected_index < self.state.archive_items.len()
                {
                    self.state.archive_items.remove(self.state.selected_index);
                    if self.state.selected_index >= self.state.archive_items.len()
                        && !self.state.archive_items.is_empty()
                    {
                        self.state.selected_index = self.state.archive_items.len() - 1;
                    }
                    let _ = self.save_data().await;
                    self.show_status_message("Item deleted".to_string());
                }
            }
        }
    }

    async fn submit_add_form(&mut self) -> bool {
        let name = self
            .add_form
            .get_field_value("name")
            .cloned()
            .unwrap_or_default();
        if name.is_empty() {
            return false;
        }

        let item_type = self
            .add_form
            .get_field_value("type")
            .cloned()
            .unwrap_or_default();
        let author = self
            .add_form
            .get_field_value("author")
            .cloned()
            .unwrap_or_default();
        let link = self
            .add_form
            .get_field_value("link")
            .cloned()
            .unwrap_or_default();
        let note = self
            .add_form
            .get_field_value("note")
            .cloned()
            .unwrap_or_default();

        let new_item = match folio_core::create_item(
            name,
            Some(item_type),
            Some(author),
            Some(link),
            Some(note),
            None,
        ) {
            Ok(item) => item,
            Err(_) => return false,
        };

        match ConfigManager::new() {
            Ok(config_manager) => {
                let config = config_manager.get();
                match folio_core::add_item_to_inbox(
                    self.state.inbox_items.clone(),
                    new_item,
                    config,
                ) {
                    Ok((new_inbox, to_archive)) => {
                        self.state.inbox_items = new_inbox;

                        for item in &to_archive {
                            self.state.add_item_to_archive(item.clone());
                        }

                        let _ = self.save_data().await;

                        if !to_archive.is_empty() {
                            self.show_status_message(format!(
                                "Item added. {} item(s) archived due to overflow",
                                to_archive.len()
                            ));
                        } else {
                            self.show_status_message("Item added".to_string());
                        }

                        true
                    }
                    Err(_) => {
                        self.cap_warning_message = format!(
                            "Inbox limit ({}) reached.\n\n\
                            Choose an action:\n\
                            • Delete an existing item (use 'x' key to delete)\n\
                            • Archive an item (change status to 'done' or use 'A' key)\n\
                            • Adjust inbox size: `folio config set max_items N`\n\
                            • Change overflow strategy: `folio config set archive_on_overflow [todo|any]`",
                            config.max_items
                        );
                        self.show_cap_warning = true;
                        false
                    }
                }
            }
            Err(_) => {
                self.state.inbox_items.push(new_item);
                let _ = self.save_data().await;
                self.show_status_message("Item added".to_string());
                true
            }
        }
    }

    async fn submit_edit_form(&mut self) -> bool {
        let name = self
            .edit_form
            .get_field_value("name")
            .cloned()
            .unwrap_or_default();
        if name.is_empty() {
            return false;
        }

        let item_type = self
            .edit_form
            .get_field_value("type")
            .cloned()
            .unwrap_or_default();
        let author = self
            .edit_form
            .get_field_value("author")
            .cloned()
            .unwrap_or_default();
        let link = self
            .edit_form
            .get_field_value("link")
            .cloned()
            .unwrap_or_default();
        let note = self
            .edit_form
            .get_field_value("note")
            .cloned()
            .unwrap_or_default();

        if let Some(item) = self.state.selected_item_mut() {
            item.name = name;
            item.item_type =
                folio_core::ItemType::from_str(&item_type).unwrap_or(folio_core::ItemType::Other);
            item.author = author;
            item.link = link;
            item.note = note;

            if item.validate().is_err() {
                return false;
            }

            let _ = self.save_data().await;
            self.show_status_message("Item updated".to_string());
            true
        } else {
            false
        }
    }

    pub async fn run(&mut self) -> TuiResult<()> {
        self.load_data().await?;

        let mut terminal = setup_terminal()?;
        let mut events = EventHandler::new(Duration::from_millis(250));

        if !self.state.current_items().is_empty() {
            self.table_state.select(Some(0));
        }

        if self.start_with_add_form {
            self.add_form.toggle_visibility();
        }

        loop {
            terminal.draw(|f| {
                let chunks = ratatui::layout::Layout::default()
                    .constraints([
                        ratatui::layout::Constraint::Min(3),
                        ratatui::layout::Constraint::Length(1),
                    ])
                    .split(f.size());

                ItemsTable::render(f, &self.state, chunks[0], &self.table_state);
                self.add_form.render(f);
                self.edit_form.render(f);

                if self.filter_input_mode {
                    Self::render_filter_input(f, &self.filter_input);
                }

                if self.show_delete_confirmation {
                    Self::render_delete_confirmation(f);
                }

                if self.show_help {
                    Self::render_help_dialog(f);
                }

                if self.show_cap_warning {
                    Self::render_cap_warning_dialog(f, &self.cap_warning_message);
                }

                if self.show_done_confirmation {
                    Self::render_done_confirmation_dialog(f);
                }

                if self.show_config_dialog {
                    Self::render_config_dialog(
                        f,
                        &self.config_max_items_input,
                        self.config_overflow_strategy,
                    );
                }

                Self::render_status_bar(
                    f,
                    chunks[1],
                    &self.status_message,
                    &self.state.current_view,
                );
            })?;

            if let Some(event) = events.next().await {
                match event {
                    AppEvent::Key(key_event) => {
                        self.handle_key_event(key_event).await;
                    }
                    AppEvent::Tick => {
                        if let Some((_, time)) = self.status_message {
                            if time.elapsed() > Duration::from_secs(2) {
                                self.status_message = None;
                            }
                        }
                    }
                }
            }

            if self.should_quit {
                break;
            }
        }

        restore_terminal(&mut terminal)?;
        Ok(())
    }

    fn render_delete_confirmation(frame: &mut ratatui::Frame) {
        let area = frame.size();
        let popup_area = ratatui::layout::Rect {
            x: area.width / 2 - 20,
            y: area.height / 2 - 2,
            width: 40.min(area.width),
            height: 4.min(area.height),
        };

        frame.render_widget(ratatui::widgets::Clear, popup_area);

        let block = ratatui::widgets::Block::default()
            .title("Confirm Delete")
            .borders(ratatui::widgets::Borders::ALL);

        let text = vec![
            ratatui::text::Line::from("Delete this item permanently?"),
            ratatui::text::Line::from("(Y)es / (N)o"),
        ];

        let paragraph = ratatui::widgets::Paragraph::new(text)
            .block(block)
            .alignment(ratatui::layout::Alignment::Center);

        frame.render_widget(paragraph, popup_area);
    }

    fn render_help_dialog(frame: &mut ratatui::Frame) {
        let area = frame.size();
        let popup_area = ratatui::layout::Rect {
            x: area.width / 2 - 30,
            y: area.height / 2 - 12,
            width: 60.min(area.width),
            height: 24.min(area.height),
        };

        frame.render_widget(ratatui::widgets::Clear, popup_area);

        let block = ratatui::widgets::Block::default()
            .title("Help")
            .borders(ratatui::widgets::Borders::ALL);

        let help_text = vec![
            ratatui::text::Line::from("Navigation:"),
            ratatui::text::Line::from("  ↑/↓ or j/k        Move selection"),
            ratatui::text::Line::from("  PgUp/PgDn         Jump pages"),
            ratatui::text::Line::from("  Home/End          Jump to top/bottom"),
            ratatui::text::Line::from("  Tab               Switch between Inbox/Archive"),
            ratatui::text::Line::from(""),
            ratatui::text::Line::from("Item Actions:"),
            ratatui::text::Line::from("  Enter    Open link"),
            ratatui::text::Line::from("  t        Set status to todo"),
            ratatui::text::Line::from("  i        Set status to in progress"),
            ratatui::text::Line::from("  d        Set status to done"),
            ratatui::text::Line::from("  a        Add new item"),
            ratatui::text::Line::from("  e        Edit item"),
            ratatui::text::Line::from("  x        Delete item"),
            ratatui::text::Line::from("  r        Toggle reference (Archive only)"),
            ratatui::text::Line::from(""),
            ratatui::text::Line::from("System:"),
            ratatui::text::Line::from("  C        Configure settings"),
            ratatui::text::Line::from(""),
            ratatui::text::Line::from("General:"),
            ratatui::text::Line::from("  ?        Show this help"),
            ratatui::text::Line::from("  /        Filter items"),
            ratatui::text::Line::from("  q/Esc    Quit"),
        ];

        let paragraph = ratatui::widgets::Paragraph::new(help_text)
            .block(block)
            .alignment(ratatui::layout::Alignment::Left);

        frame.render_widget(paragraph, popup_area);
    }

    fn render_cap_warning_dialog(frame: &mut ratatui::Frame, message: &str) {
        let area = frame.size();
        let popup_area = ratatui::layout::Rect {
            x: area.width / 2 - 35,
            y: area.height / 2 - 8,
            width: 70.min(area.width),
            height: 16.min(area.height),
        };

        frame.render_widget(ratatui::widgets::Clear, popup_area);

        let block = ratatui::widgets::Block::default()
            .title("Inbox Full")
            .borders(ratatui::widgets::Borders::ALL);

        let text_lines: Vec<ratatui::text::Line> = message
            .lines()
            .map(|line| ratatui::text::Line::from(line.to_string()))
            .collect();

        let mut content_lines = text_lines;
        content_lines.push(ratatui::text::Line::from(""));
        content_lines.push(ratatui::text::Line::from("Press Enter or Esc to continue"));

        let paragraph = ratatui::widgets::Paragraph::new(content_lines)
            .block(block)
            .alignment(ratatui::layout::Alignment::Left);

        frame.render_widget(paragraph, popup_area);
    }

    fn render_done_confirmation_dialog(frame: &mut ratatui::Frame) {
        let area = frame.size();
        let popup_area = ratatui::layout::Rect {
            x: area.width / 2 - 25,
            y: area.height / 2 - 3,
            width: 50.min(area.width),
            height: 6.min(area.height),
        };

        frame.render_widget(ratatui::widgets::Clear, popup_area);

        let block = ratatui::widgets::Block::default()
            .title("Confirm Mark as Done")
            .borders(ratatui::widgets::Borders::ALL);

        let text = vec![
            ratatui::text::Line::from("Mark item as Done?"),
            ratatui::text::Line::from("(This will move it to Archive)"),
            ratatui::text::Line::from(""),
            ratatui::text::Line::from("(Y)es / (N)o"),
        ];

        let paragraph = ratatui::widgets::Paragraph::new(text)
            .block(block)
            .alignment(ratatui::layout::Alignment::Center);

        frame.render_widget(paragraph, popup_area);
    }

    fn render_config_dialog(
        frame: &mut ratatui::Frame,
        max_items_input: &str,
        overflow_strategy_index: usize,
    ) {
        let area = frame.size();
        let popup_area = ratatui::layout::Rect {
            x: area.width / 2 - 30,
            y: area.height / 2 - 8,
            width: 60.min(area.width),
            height: 16.min(area.height),
        };

        frame.render_widget(ratatui::widgets::Clear, popup_area);

        let block = ratatui::widgets::Block::default()
            .title("Configuration")
            .borders(ratatui::widgets::Borders::ALL);

        let strategies = ["abort", "todo", "any"];
        let selected_strategy = strategies[overflow_strategy_index];

        let text = vec![
            ratatui::text::Line::from(""),
            ratatui::text::Line::from(format!("Max Items: {}", max_items_input)),
            ratatui::text::Line::from(""),
            ratatui::text::Line::from(format!(
                "Overflow Strategy: {} (↑/↓ to cycle)",
                selected_strategy
            )),
            ratatui::text::Line::from(""),
            ratatui::text::Line::from("Controls:"),
            ratatui::text::Line::from("  Enter  - Save changes"),
            ratatui::text::Line::from("  Esc    - Cancel"),
            ratatui::text::Line::from("  ↑/↓    - Cycle strategy"),
            ratatui::text::Line::from("  0-9    - Edit max items"),
            ratatui::text::Line::from("  Backspace - Delete last digit"),
        ];

        let paragraph = ratatui::widgets::Paragraph::new(text)
            .block(block)
            .alignment(ratatui::layout::Alignment::Left);

        frame.render_widget(paragraph, popup_area);
    }

    fn render_status_bar(
        frame: &mut ratatui::Frame,
        area: ratatui::layout::Rect,
        status_message: &Option<(String, Instant)>,
        current_view: &View,
    ) {
        let view_text = match current_view {
            View::Inbox => " Inbox ",
            View::Archive => " Archive ",
        };

        let message_text = if let Some((message, _)) = status_message {
            message.clone()
        } else {
            String::new()
        };

        let status_text = if message_text.is_empty() {
            format!("View: {} | Press ? for help", view_text)
        } else {
            format!("{} | View: {} | Press ? for help", message_text, view_text)
        };

        let paragraph = ratatui::widgets::Paragraph::new(status_text).style(
            ratatui::style::Style::default()
                .fg(ratatui::style::Color::White)
                .bg(ratatui::style::Color::Black),
        );

        frame.render_widget(paragraph, area);
    }

    fn render_filter_input(frame: &mut ratatui::Frame, filter_input: &str) {
        let area = frame.size();
        let input_area = ratatui::layout::Rect {
            x: 0,
            y: area.height.saturating_sub(3),
            width: area.width.min(50),
            height: 3.min(area.height),
        };

        frame.render_widget(ratatui::widgets::Clear, input_area);

        let block = ratatui::widgets::Block::default()
            .title("Filter")
            .borders(ratatui::widgets::Borders::ALL);

        let text = vec![
            ratatui::text::Line::from(format!("Filter: {}", filter_input)),
            ratatui::text::Line::from("Press Enter to apply, Esc to cancel"),
        ];

        let paragraph = ratatui::widgets::Paragraph::new(text)
            .block(block)
            .alignment(ratatui::layout::Alignment::Left);

        frame.render_widget(paragraph, input_area);
    }
}
