pub mod app;
pub mod data;
pub mod event;
pub mod forms;
pub mod state;
pub mod terminal;
pub mod widgets;

use crate::app::App;

pub async fn run_tui(add_form: bool) -> Result<(), Box<dyn std::error::Error>> {
    let mut app = if add_form {
        App::new_with_add_form()
    } else {
        App::new()
    };

    app.run().await
}

pub async fn run_tui_default() -> Result<(), Box<dyn std::error::Error>> {
    run_tui(false).await
}

pub async fn run_tui_add_form() -> Result<(), Box<dyn std::error::Error>> {
    run_tui(true).await
}
