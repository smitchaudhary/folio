use folio_client::Client;

fn main() {
    let base_url = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "http://127.0.0.1:8787".to_string());

    let client = Client::new(&base_url);
    match client.health() {
        Ok(()) => println!("health OK at {base_url}"),
        Err(e) => {
            eprintln!("health check failed: {e:?}");
            std::process::exit(1);
        }
    }
}
