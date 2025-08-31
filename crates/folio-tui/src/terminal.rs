use crate::error::TuiResult;
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use std::io;

pub type AppTerminal = Terminal<CrosstermBackend<io::Stdout>>;

pub fn setup_terminal() -> TuiResult<AppTerminal> {
    crossterm::terminal::enable_raw_mode()?;
    let stdout = io::stdout();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;
    Ok(terminal)
}

pub fn restore_terminal(terminal: &mut AppTerminal) -> TuiResult<()> {
    crossterm::terminal::disable_raw_mode()?;
    terminal.show_cursor()?;
    Ok(())
}
