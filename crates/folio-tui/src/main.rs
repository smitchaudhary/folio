use folio_tui;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();

    if args.len() > 1 && args[1] == "--add" {
        folio_tui::run_tui_add_form().await
    } else {
        folio_tui::run_tui_default().await
    }
}
