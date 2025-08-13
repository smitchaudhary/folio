use crate::data::{
    append_item_to_archive, load_archive_items, load_inbox_items, save_archive_items,
    save_inbox_items,
};
use crate::event::{AppEvent, EventHandler};
use crate::state::AppState;
use crate::terminal::{restore_terminal, setup_terminal};
use crate::widgets::ItemsTable;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::widgets::TableState;
use std::time::Duration;

pub struct App {
    pub should_quit: bool,
    pub state: AppState,
    pub table_state: TableState,
}

impl App {
    pub fn new() -> Self {
        Self {
            should_quit: false,
            state: AppState::new(),
            table_state: TableState::default(),
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
