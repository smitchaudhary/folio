use crate::data::{load_archive_items, load_inbox_items};
use crate::event::{AppEvent, EventHandler};
use crate::state::AppState;
use crate::terminal::{restore_terminal, setup_terminal};
use crate::widgets::ItemsTable;
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

    pub async fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.load_data().await?;

        let mut terminal = setup_terminal()?;
        let mut events = EventHandler::new(Duration::from_millis(250));

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
                        if let crossterm::event::KeyCode::Char('q') = key_event.code {
                            self.should_quit = true;
                        }
                        if let crossterm::event::KeyCode::Esc = key_event.code {
                            self.should_quit = true;
                        }
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
