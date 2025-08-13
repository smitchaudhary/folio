use crate::event::{AppEvent, EventHandler};
use crate::terminal::{restore_terminal, setup_terminal};
use std::time::Duration;

pub struct App {
    pub should_quit: bool,
}

impl App {
    pub fn new() -> Self {
        Self { should_quit: false }
    }

    pub async fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let mut terminal = setup_terminal()?;
        let mut events = EventHandler::new(Duration::from_millis(250));

        loop {
            terminal.draw(|f| {
                f.render_widget(
                    ratatui::widgets::Paragraph::new("Hello, folio! Press 'q' or 'Esc' to quit."),
                    f.size(),
                );
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
