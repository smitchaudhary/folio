use folio_client::Client;

fn main() {
    let client = match std::env::args().nth(1) {
        Some(url) => Client::new(&url),
        None => Client::from_env(),
    };
    match client.health() {
        Ok(()) => println!("health OK"),
        Err(e) => {
            eprintln!("health check failed: {e:?}");
            std::process::exit(1);
        }
    }
}
