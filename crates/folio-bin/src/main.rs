use clap::Parser;
use folio_bin::cli::{Cli, Commands, ConfigSubcommands};
use folio_bin::error::{CliError, print_error};
use folio_core::{ItemType, Kind, OverflowStrategy, Status, View};

use std::str::FromStr;

fn main() {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async {
            if let Err(e) = run().await {
                print_error(&e);
                std::process::exit(1);
            }
        });
}

async fn run() -> Result<(), CliError> {
    let cli = Cli::parse();

    let client = folio_client::Client::from_env();

    match &cli.command {
        Some(command) => match command {
            Commands::Add {
                name,
                r#type,
                author,
                link,
                note,
                kind,
            } => {
                handle_add_command(&client, name, r#type, author, link, note, kind).await?;
            }
            Commands::List {
                status,
                r#type,
                archive,
                all,
            } => {
                let view = if *archive {
                    View::Archive
                } else if *all {
                    View::All
                } else {
                    View::Inbox
                };
                handle_list_command(&client, view, status.as_deref(), r#type.as_deref())?;
            }
            Commands::SetStatus {
                id,
                status,
                archive,
            } => {
                handle_set_status_command(&client, *id, status, *archive)?;
            }
            Commands::Edit { id, archive } => {
                handle_edit_command(&client, *id, *archive)?;
            }
            Commands::Archive { id } => {
                handle_archive_command(&client, *id)?;
            }
            Commands::Delete { id, archive } => {
                handle_delete_command(&client, *id, *archive)?;
            }
            Commands::MarkRef { id, archive } => {
                handle_mark_ref_command(&client, *id, *archive)?;
            }
            Commands::Config { subcommand } => {
                handle_config_command(&client, subcommand)?;
            }
        },
        None => {
            folio_tui::run_tui_default().await?;
        }
    }

    Ok(())
}

fn resolve_item_index(id: usize, len: usize) -> Result<usize, CliError> {
    let index = id.checked_sub(1).ok_or(CliError::ItemNotFound { id })?;
    if index < len {
        Ok(index)
    } else {
        Err(CliError::ItemNotFound { id })
    }
}

fn handle_list_command(
    client: &folio_client::Client,
    view: View,
    status_filters: Option<&[String]>,
    type_filters: Option<&[String]>,
) -> Result<(), CliError> {
    let items = client.list_items(view)?;

    let view_is_empty = items.is_empty();

    let filtered: Vec<_> = items
        .iter()
        .enumerate()
        .filter(|(_, item)| matches_filters(item, status_filters, type_filters))
        .map(|(pos, item)| (pos + 1, item))
        .collect();

    if filtered.is_empty() {
        if view_is_empty {
            match view {
                View::Inbox => println!("No items in inbox."),
                View::Archive => println!("No archived items."),
                View::All => println!("No items found."),
            }
        } else {
            println!("No items match the filter.");
        }
        return Ok(());
    }

    println!("{}", folio_core::Item::format_list_header());
    println!("{}", folio_core::Item::format_list_separator());

    for (pos, item) in &filtered {
        println!("{}", item.format_for_list(*pos));
    }
    Ok(())
}

fn matches_filters(
    item: &folio_core::Item,
    status_filters: Option<&[String]>,
    type_filters: Option<&[String]>,
) -> bool {
    if let Some(status_filters) = status_filters
        && !status_filters.is_empty()
    {
        let matches = status_filters
            .iter()
            .any(|s| Status::from_str(&s.to_lowercase()).is_ok_and(|status| status == item.status));
        if !matches {
            return false;
        }
    }

    if let Some(type_filters) = type_filters
        && !type_filters.is_empty()
    {
        let matches = type_filters.iter().any(|t| {
            ItemType::from_str(&t.to_lowercase()).is_ok_and(|item_type| item_type == item.item_type)
        });
        if !matches {
            return false;
        }
    }

    true
}

fn handle_set_status_command(
    client: &folio_client::Client,
    id: usize,
    status_str: &str,
    archive: bool,
) -> Result<(), CliError> {
    let new_status = Status::from_str(status_str).map_err(|_| CliError::InvalidStatus {
        status: status_str.to_string(),
    })?;

    let view = if archive { View::Archive } else { View::Inbox };
    let items = client
        .list_items(view)
        .map_err(|e| map_client_error(id, e))?;
    let index = resolve_item_index(id, items.len())?;
    let stable_id = items[index].id.clone();

    let result = client
        .change_status(&stable_id, new_status)
        .map_err(|e| map_client_error(id, e))?;

    if result.moved_to_archive {
        println!(
            "Item #{} status updated to '{}' and moved to archive",
            id, status_str
        );
    } else if result.moved_to_inbox {
        if !result.overflow.is_empty() {
            println!(
                "Item #{} status updated to '{}' and moved to inbox. {} item(s) were archived due to overflow:",
                id,
                status_str,
                result.overflow.len()
            );
            for item in &result.overflow {
                println!("  - {} ({})", item.name, item.status.as_string());
            }
        } else {
            println!(
                "Item #{} status updated to '{}' and moved to inbox",
                id, status_str
            );
        }
    } else {
        println!("Item #{} status updated to '{}'", id, status_str);
    }

    Ok(())
}

fn handle_edit_command(
    client: &folio_client::Client,
    id: usize,
    archive: bool,
) -> Result<(), CliError> {
    let view = if archive { View::Archive } else { View::Inbox };
    let items = client
        .list_items(view)
        .map_err(|e| map_client_error(id, e))?;
    let index = resolve_item_index(id, items.len())?;
    let item = &items[index];

    let original_name = item.name.clone();
    let original_type = format!("{:?}", item.item_type);
    let original_author = item.author.clone();
    let original_link = item.link.clone();
    let original_note = item.note.clone();

    println!("Editing item #{}. Leave blank to keep current value.", id);
    println!("Current name: {}", original_name);
    let name_input = prompt_for_input("Name")?;

    println!("Current type: {}", original_type);
    let type_input = prompt_for_input("Type")?;

    println!("Current author: {}", original_author);
    let author_input = prompt_for_input("Author")?;

    println!("Current link: {}", original_link);
    let link_input = prompt_for_input("Link")?;

    println!("Current note: {}", original_note);
    let note_input = prompt_for_input("Note")?;

    let mut request = folio_client::EditItemRequest::default();
    if !name_input.is_empty() {
        request.name = Some(name_input);
    }
    if !type_input.is_empty() {
        ItemType::from_str(&type_input).map_err(|_| CliError::InvalidItemType {
            item_type: type_input.clone(),
        })?;
        request.item_type = Some(type_input);
    }
    if !author_input.is_empty() {
        request.author = Some(author_input);
    }
    if !link_input.is_empty() {
        request.link = Some(link_input);
    }
    if !note_input.is_empty() {
        request.note = Some(note_input);
    }

    client
        .edit_item(&item.id, request)
        .map_err(|e| map_client_error(id, e))?;

    println!("Item #{} updated successfully", id);
    Ok(())
}

fn handle_archive_command(client: &folio_client::Client, id: usize) -> Result<(), CliError> {
    let inbox_items = client
        .list_items(View::Inbox)
        .map_err(|e| map_client_error(id, e))?;
    match resolve_item_index(id, inbox_items.len()) {
        Ok(index) => {
            let stable_id = inbox_items[index].id.clone();
            client
                .change_status(&stable_id, Status::Done)
                .map_err(|e| map_client_error(id, e))?;
            println!("Item #{} marked as done and archived successfully", id);
            Ok(())
        }
        Err(_) => {
            let archive_items = client
                .list_items(View::Archive)
                .map_err(|e| map_client_error(id, e))?;
            if resolve_item_index(id, archive_items.len()).is_ok() {
                println!("Item #{} is already in archive", id);
                Ok(())
            } else {
                Err(CliError::ItemNotFound { id })
            }
        }
    }
}

fn handle_delete_command(
    client: &folio_client::Client,
    id: usize,
    archive: bool,
) -> Result<(), CliError> {
    let view = if archive { View::Archive } else { View::Inbox };
    let items = client
        .list_items(view)
        .map_err(|e| map_client_error(id, e))?;
    let index = resolve_item_index(id, items.len())?;
    let item = &items[index];

    println!("Item to delete:");
    println!("  Name: {}", item.name);
    println!("  Type: {:?}", item.item_type);
    println!("  Status: {:?}", item.status);
    println!("  Author: {}", item.author);
    println!("  Link: {}", item.link);

    print!("Delete this item permanently? (y/N): ");
    use std::io::{Write, stdin, stdout};
    stdout().flush()?;

    let mut input = String::new();
    stdin().read_line(&mut input)?;
    let confirmation = input.trim().to_lowercase();

    if confirmation != "y" && confirmation != "yes" {
        println!("Delete operation cancelled");
        return Ok(());
    }

    client
        .delete_item(&item.id)
        .map_err(|e| map_client_error(id, e))?;

    println!("Item #{} deleted successfully", id);
    Ok(())
}

fn handle_mark_ref_command(
    client: &folio_client::Client,
    id: usize,
    archive: bool,
) -> Result<(), CliError> {
    if !archive {
        let inbox_items = client
            .list_items(View::Inbox)
            .map_err(|e| map_client_error(id, e))?;
        if resolve_item_index(id, inbox_items.len()).is_ok() {
            println!(
                "Cannot mark reference for items in inbox. Only archived (done) items can be marked as references. Use 'folio mark-ref --archive <id>'."
            );
            return Ok(());
        }
        return Err(CliError::ItemNotFound { id });
    }

    let archive_items = client
        .list_items(View::Archive)
        .map_err(|e| map_client_error(id, e))?;
    let index = resolve_item_index(id, archive_items.len())?;
    let item = client
        .toggle_kind(&archive_items[index].id)
        .map_err(|e| map_client_error(id, e))?;
    match item.kind {
        folio_core::Kind::Reference => println!("Item #{} marked as reference", id),
        folio_core::Kind::Normal => println!("Item #{} unmarked as reference", id),
    }

    Ok(())
}

fn prompt_for_input(field_name: &str) -> Result<String, CliError> {
    use std::io::{Write, stdin, stdout};

    print!("{}: ", field_name);
    stdout().flush()?;

    let mut input = String::new();
    stdin().read_line(&mut input)?;
    Ok(input.trim().to_string())
}

async fn handle_add_command(
    client: &folio_client::Client,
    name: &Option<String>,
    item_type: &Option<String>,
    author: &Option<String>,
    link: &Option<String>,
    note: &Option<String>,
    kind: &Option<String>,
) -> Result<(), CliError> {
    if name.is_none() {
        folio_tui::run_tui_add_form().await?;
        return Ok(());
    }

    let mut request =
        folio_client::AddItemRequest::new(name.clone().unwrap_or_else(|| "Untitled".to_string()));
    request.item_type = item_type.clone();
    request.author = author.clone();
    request.link = link.clone();
    request.note = note.clone();
    request.kind = kind.clone();

    match client.add_item(request) {
        Ok((item, to_archive)) => {
            if matches!(item.kind, Kind::Reference) {
                println!("Added reference item as done and archived");
            } else if !to_archive.is_empty() {
                println!(
                    "Item added successfully. The following item(s) were automatically archived due to overflow:"
                );
                for item in to_archive {
                    println!("  - {} ({})", item.name, item.status.as_string());
                }
            } else {
                println!("Item added successfully");
            }
            Ok(())
        }
        Err(folio_client::ClientError::InboxFull) => {
            println!("Inbox limit reached.");
            println!();
            println!("Choose an action:");
            println!("  [D]elete an existing item (use 'folio delete <id>')");
            println!("  [A]rchive an item (change status to 'done' or use 'folio archive <id>')");
            println!("  [I]ncrease inbox size: `folio config set max_items N`");
            println!(
                "  [C]hange overflow strategy: `folio config set archive_on_overflow [todo|any]`"
            );
            println!();
            println!("Would you like to see the current inbox items? (y/N): ");

            use std::io::{Write, stdin, stdout};
            stdout().flush()?;
            let mut input = String::new();
            tokio::task::block_in_place(|| stdin().read_line(&mut input))?;
            let response = input.trim().to_lowercase();

            if (response == "y" || response == "yes")
                && let Ok(inbox_items) = client.list_items(folio_core::View::Inbox)
                && !inbox_items.is_empty()
            {
                println!();
                println!("{}", folio_core::Item::format_list_header());
                println!("{}", folio_core::Item::format_list_separator());

                for (index, item) in inbox_items.iter().enumerate() {
                    println!("{}", item.format_for_list(index + 1));
                }
            }

            std::process::exit(1);
        }
        Err(e) => Err(CliError::Client(e)),
    }
}

fn handle_config_command(
    client: &folio_client::Client,
    subcommand: &ConfigSubcommands,
) -> Result<(), CliError> {
    match subcommand {
        ConfigSubcommands::List => {
            let config = client.get_config()?;
            let json = serde_json::to_string_pretty(&config)?;
            println!("{}", json);
        }
        ConfigSubcommands::Get { key } => {
            let config = client.get_config()?;
            match key.as_str() {
                "max_items" => println!("{}", config.max_items),
                "archive_on_overflow" => println!(
                    "{}",
                    match config.archive_on_overflow {
                        OverflowStrategy::Abort => "abort",
                        OverflowStrategy::Todo => "todo",
                        OverflowStrategy::Any => "any",
                    }
                ),
                _ => {
                    return Err(CliError::UnknownConfigKey { key: key.clone() });
                }
            }
        }
        ConfigSubcommands::Set { key, value } => {
            let mut config = client.get_config()?;
            match key.as_str() {
                "max_items" => {
                    let val = value
                        .parse::<u32>()
                        .map_err(|_| CliError::InvalidMaxItems {
                            value: value.clone(),
                        })?;
                    config.max_items = val;
                }
                "archive_on_overflow" => {
                    config.archive_on_overflow = match value.as_str() {
                        "abort" => OverflowStrategy::Abort,
                        "todo" => OverflowStrategy::Todo,
                        "any" => OverflowStrategy::Any,
                        _ => {
                            return Err(CliError::InvalidOverflowStrategy {
                                value: value.to_string(),
                            });
                        }
                    };
                }
                _ => {
                    return Err(CliError::UnknownConfigKey { key: key.clone() });
                }
            }
            client.set_config(&config)?;
            println!("Config updated successfully");
        }
        ConfigSubcommands::Reset => {
            client.set_config(&folio_core::Config::default())?;
            println!("Config reset to default values");
        }
    }

    Ok(())
}

fn map_client_error(id: usize, e: folio_client::ClientError) -> CliError {
    match e {
        folio_client::ClientError::Server { code, .. } if code == "not_found" => {
            CliError::ItemNotFound { id }
        }
        e => CliError::Client(e),
    }
}
