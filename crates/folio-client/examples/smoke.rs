use folio_client::{AddItemRequest, Client, EditItemRequest};
use folio_core::{OverflowStrategy, Status, View};

fn main() {
    let base_url = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "http://127.0.0.1:8787".to_string());
    let client = Client::new(&base_url);

    let config = client.get_config().expect("get_config");
    println!(
        "config: {} / {:?}",
        config.max_items, config.archive_on_overflow
    );
    let mut new_config = config.clone();
    new_config.archive_on_overflow = OverflowStrategy::Any;
    let saved = client.set_config(&new_config).expect("set_config");
    println!(
        "set config: {} / {:?}",
        saved.max_items, saved.archive_on_overflow
    );

    let mut request = AddItemRequest::new("Smoke Item");
    request.author = Some("Tester".to_string());
    let (item, overflow) = client.add_item(request).expect("add_item");
    println!("added: {} ({} overflow)", item.name, overflow.len());
    let id = item.id.clone();

    let result = client.change_status(&id, Status::Doing).expect("doing");
    println!("doing: moved_to_archive={}", result.moved_to_archive);
    let result = client.change_status(&id, Status::Done).expect("done");
    println!("done: moved_to_archive={}", result.moved_to_archive);

    let item = client.toggle_kind(&id).expect("toggle_kind");
    println!("kind: {}", item.kind.display_emoji());

    let edited = client
        .edit_item(
            &id,
            EditItemRequest {
                author: Some("Edited".to_string()),
                ..Default::default()
            },
        )
        .expect("edit_item");
    println!("edited author: {}", edited.author);

    client.delete_item(&id).expect("delete_item");
    println!("deleted");

    let items = client.list_items(View::All).expect("list_items");
    println!("final: {} items total", items.len());
}
