pub mod app;
pub mod data;
pub mod error;
pub mod event;
pub mod forms;
pub mod state;
pub mod terminal;
pub mod widgets;

use crate::app::App;
pub use error::{TuiError, TuiResult};

pub async fn run_tui(add_form: bool) -> TuiResult<()> {
    let mut app = if add_form {
        App::new_with_add_form()
    } else {
        App::new()
    };

    app.run().await
}

pub async fn run_tui_default() -> TuiResult<()> {
    run_tui(false).await
}

pub async fn run_tui_add_form() -> TuiResult<()> {
    run_tui(true).await
}
