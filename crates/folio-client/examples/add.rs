use folio_client::{AddItemRequest, Client};

fn main() {
    let base_url = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "http://127.0.0.1:8787".to_string());
    let name = std::env::args()
        .nth(2)
        .unwrap_or_else(|| "Example Item".to_string());

    let mut request = AddItemRequest::new(&name);
    if let Some(kind) = std::env::args().nth(3) {
        request.kind = Some(kind);
    }

    let client = Client::new(&base_url);
    match client.add_item(request) {
        Ok((item, overflow)) => {
            println!(
                "added: {} | {} | {}",
                item.id,
                item.name,
                item.status.as_string()
            );
            for o in &overflow {
                println!("overflow archived: {} | {}", o.id, o.name);
            }
        }
        Err(e) => {
            eprintln!("add failed: {e:?}");
            std::process::exit(1);
        }
    }
}
