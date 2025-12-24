use crate::error::TuiResult;
use crossterm::event::{DisableMouseCapture, EnableMouseCapture};
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use std::io;

pub type AppTerminal = Terminal<CrosstermBackend<io::Stdout>>;

pub fn setup_terminal() -> TuiResult<AppTerminal> {
    crossterm::terminal::enable_raw_mode()?;
    crossterm::execute!(io::stdout(), EnableMouseCapture)?;
    let stdout = io::stdout();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;
    Ok(terminal)
}

pub fn restore_terminal(terminal: &mut AppTerminal) -> TuiResult<()> {
    crossterm::execute!(io::stdout(), DisableMouseCapture)?;
    crossterm::terminal::disable_raw_mode()?;
    terminal.show_cursor()?;
    Ok(())
}
