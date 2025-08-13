use crate::data::{
    append_item_to_archive, load_archive_items, load_inbox_items, save_archive_items,
    save_inbox_items,
};
use crate::event::{AppEvent, EventHandler};
use crate::forms::AddItemForm;
use crate::state::AppState;
use crate::terminal::{restore_terminal, setup_terminal};
use crate::widgets::ItemsTable;
use chrono::Utc;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::widgets::TableState;
use std::time::Duration;

pub struct App {
    pub should_quit: bool,
    pub state: AppState,
    pub table_state: TableState,
    pub add_form: AddItemForm,
}

impl App {
    pub fn new() -> Self {
        Self {
            should_quit: false,
            state: AppState::new(),
            table_state: TableState::default(),
            add_form: AddItemForm::new(),
        }
    }

    pub async fn load_data(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let inbox_items = load_inbox_items().await?;
        let archive_items = load_archive_items().await?;

        self.state.load_inbox_items(inbox_items);
        self.state.load_archive_items(archive_items);

        Ok(())
    }

    pub async fn save_data(&self) -> Result<(), Box<dyn std::error::Error>> {
        save_inbox_items(self.state.get_inbox_items()).await?;
        save_archive_items(self.state.get_archive_items()).await?;
        Ok(())
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) {
        // If the add form is visible, handle form-specific keys
        if self.add_form.is_visible {
            match key_event.code {
                KeyCode::Esc => {
                    self.add_form.toggle_visibility();
                }
                KeyCode::Enter => {
                    if self.submit_add_form() {
                        self.add_form.toggle_visibility();
                    }
                }
                _ => {
                    self.add_form.handle_key(key_event);
                }
            }
            return;
        }

        match key_event.code {
            KeyCode::Char('q') => self.should_quit = true,
            KeyCode::Esc => self.should_quit = true,
            KeyCode::Down => {
                self.state.next_item();
                self.table_state.select(Some(self.state.selected_index));
            }
            KeyCode::Up => {
                self.state.previous_item();
                self.table_state.select(Some(self.state.selected_index));
            }
            KeyCode::Char('s') => {
                if let Some(done_item) = self.state.cycle_selected_item_status() {
                    let _ = append_item_to_archive(&done_item);
                } else {
                    let _ = self.save_data();
                }
            }
            KeyCode::Char('i') => {
                if self.state.move_selected_to_doing() {
                    let _ = self.save_data();
                }
            }
            KeyCode::Char('d') => {
                if let Some(done_item) = self.state.move_selected_to_done() {
                    let _ = append_item_to_archive(&done_item);
                } else {
                    let _ = self.save_data();
                }
            }
            KeyCode::Char('a') => {
                self.add_form.toggle_visibility();
            }
            KeyCode::Enter => {
                if let Some(item) = self.state.selected_item() {
                    if !item.link.is_empty() {
                        let _ = opener::open(&item.link);
                    }
                }
            }
            _ => {}
        }
    }

    fn submit_add_form(&mut self) -> bool {
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

        let new_item = folio_core::Item {
            name,
            item_type: match item_type.as_str() {
                "video" => folio_core::ItemType::Video,
                "blog" => folio_core::ItemType::Blog,
                "other" => folio_core::ItemType::Other,
                _ => folio_core::ItemType::Article,
            },
            status: folio_core::Status::Todo,
            author,
            link,
            added_at: Utc::now(),
            started_at: None,
            finished_at: None,
            note,
            kind: folio_core::Kind::Normal,
            version: 1,
        };

        if new_item.validate().is_err() {
            return false;
        }

        self.state.inbox_items.push(new_item);

        let _ = self.save_data();

        true
    }

    pub async fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.load_data().await?;

        let mut terminal = setup_terminal()?;
        let mut events = EventHandler::new(Duration::from_millis(250));

        if !self.state.current_items().is_empty() {
            self.table_state.select(Some(0));
        }

        loop {
            terminal.draw(|f| {
                let chunks = ratatui::layout::Layout::default()
                    .constraints([ratatui::layout::Constraint::Percentage(100)])
                    .split(f.size());

                ItemsTable::render(f, &self.state, chunks[0], &self.table_state);
                self.add_form.render(f);
            })?;

            if let Some(event) = events.next().await {
                match event {
                    AppEvent::Key(key_event) => {
                        self.handle_key_event(key_event);
                    }
                    AppEvent::Tick => {
                        // Handle tick events if needed
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
}
