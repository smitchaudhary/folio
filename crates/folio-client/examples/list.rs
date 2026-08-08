use folio_client::Client;
use folio_core::View;

fn main() {
    let base_url = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "http://127.0.0.1:8787".to_string());
    let view = match std::env::args().nth(2).as_deref() {
        Some("archive") => View::Archive,
        Some("all") => View::All,
        _ => View::Inbox,
    };

    let client = Client::new(&base_url);
    match client.list_items(view) {
        Ok(items) => {
            println!("{} items in {:?}:", items.len(), view);
            for item in &items {
                println!(
                    "  {} | {} | {}",
                    item.id,
                    item.name,
                    item.status.as_string()
                );
            }
        }
        Err(e) => {
            eprintln!("list failed: {e:?}");
            std::process::exit(1);
        }
    }
}
