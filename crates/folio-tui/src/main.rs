use folio_tui::app::App;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();

    let mut app = if args.len() > 1 && args[1] == "--add" {
        App::new_with_add_form()
    } else {
        App::new()
    };

    app.run().await?;
    Ok(())
}
