use clap::Parser;
use folio_bin::cli::{Cli, Commands, ConfigSubcommands, ListView};
use folio_bin::error::{CliError, print_error};
use folio_core::{ItemType, Kind, OverflowStrategy, Status};
use folio_storage::{
    ConfigManager, append_to_archive, get_archive_path, get_inbox_path, load_items_from_file,
    save_inbox,
};

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

    let config_manager = ConfigManager::new().map_err(|e| CliError::ConfigError {
        message: e.to_string(),
    })?;

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
                handle_add_command(&config_manager, name, r#type, author, link, note, kind).await?;
            }
            Commands::List {
                status,
                r#type,
                archive,
                all,
            } => {
                let view = if *archive {
                    ListView::Archive
                } else if *all {
                    ListView::All
                } else {
                    ListView::Inbox
                };
                handle_list_command(view, status.as_deref(), r#type.as_deref()).await?;
            }
            Commands::SetStatus {
                id,
                status,
                archive,
            } => {
                handle_set_status_command(&config_manager, *id, status, *archive).await?;
            }
            Commands::Edit { id, archive } => {
                handle_edit_command(*id, *archive).await?;
            }
            Commands::Archive { id } => {
                handle_archive_command(*id).await?;
            }
            Commands::Delete { id, archive } => {
                handle_delete_command(*id, *archive).await?;
            }
            Commands::MarkRef { id, archive } => {
                handle_mark_ref_command(*id, *archive).await?;
            }
            Commands::Config { subcommand } => {
                handle_config_command(subcommand).await?;
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

async fn handle_list_command(
    view: ListView,
    status_filters: Option<&[String]>,
    type_filters: Option<&[String]>,
) -> Result<(), CliError> {
    let items: Vec<folio_core::Item> = match view {
        ListView::Inbox => load_items_from_file(get_inbox_path()?)?,
        ListView::Archive => load_items_from_file(get_archive_path()?)?,
        ListView::All => {
            let mut items = load_items_from_file(get_inbox_path()?)?;
            items.extend(load_items_from_file(get_archive_path()?)?);
            items
        }
    };

    let view_is_empty = items.is_empty();

    let filtered_items: Vec<_> = items
        .iter()
        .filter(|item| matches_filters(item, status_filters, type_filters))
        .collect();

    if filtered_items.is_empty() {
        if view_is_empty {
            match view {
                ListView::Inbox => println!("No items in inbox."),
                ListView::Archive => println!("No archived items."),
                ListView::All => println!("No items found."),
            }
        } else {
            println!("No items match the filter.");
        }
        return Ok(());
    }

    println!("{}", folio_core::Item::format_list_header());
    println!("{}", folio_core::Item::format_list_separator());

    for (pos, item) in items.iter().enumerate() {
        if matches_filters(item, status_filters, type_filters) {
            println!("{}", item.format_for_list(pos + 1));
        }
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
        let matches = status_filters.iter().any(|s| {
            Status::from_str(&s.to_lowercase()).is_ok_and(|status| status == item.status)
        });
        if !matches {
            return false;
        }
    }

    if let Some(type_filters) = type_filters
        && !type_filters.is_empty()
    {
        let matches = type_filters.iter().any(|t| {
            ItemType::from_str(&t.to_lowercase())
                .is_ok_and(|item_type| item_type == item.item_type)
        });
        if !matches {
            return false;
        }
    }

    true
}

async fn handle_set_status_command(
    config_manager: &ConfigManager,
    id: usize,
    status_str: &str,
    archive: bool,
) -> Result<(), CliError> {
    let inbox_path = get_inbox_path()?;
    let archive_path = get_archive_path()?;

    let inbox_items = load_items_from_file(&inbox_path)?;
    let archive_items = load_items_from_file(&archive_path)?;

    let new_status = Status::from_str(status_str).map_err(|_| CliError::InvalidStatus {
        status: status_str.to_string(),
    })?;

    let view_items = if archive { &archive_items } else { &inbox_items };
    let index = resolve_item_index(id, view_items.len())?;
    let stable_id = view_items[index].id();

    let config = config_manager.get();
    let result = folio_core::update_item_status(stable_id, new_status, inbox_items, archive_items, config)?;

    if !result.item_found {
        return Err(CliError::ItemNotFound { id });
    }

    if !result.moved_to_archive.is_empty() {
        println!(
            "Item #{} status updated to '{}' and moved to archive",
            id, status_str
        );
    } else if result.moved_to_inbox {
        if !result.overflow_items.is_empty() {
            println!(
                "Item #{} status updated to '{}' and moved to inbox. {} item(s) were archived due to overflow:",
                id,
                status_str,
                result.overflow_items.len()
            );
            for item in &result.overflow_items {
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

    save_inbox(&result.inbox_items)?;
    folio_storage::save_archive(&result.archive_items)?;

    Ok(())
}

async fn handle_edit_command(id: usize, archive: bool) -> Result<(), CliError> {
    let inbox_path = get_inbox_path()?;
    let archive_path = get_archive_path()?;

    let mut inbox_items = load_items_from_file(&inbox_path)?;
    let mut archive_items = load_items_from_file(&archive_path)?;

    let view_items: &mut Vec<folio_core::Item> = if archive {
        &mut archive_items
    } else {
        &mut inbox_items
    };
    let index = resolve_item_index(id, view_items.len())?;
    let item = &mut view_items[index];

    let original_name = item.name.clone();
    let original_type = format!("{:?}", item.item_type);
    let original_author = item.author.clone();
    let original_link = item.link.clone();
    let original_note = item.note.clone();

    println!("Editing item #{}. Leave blank to keep current value.", id);
    println!("Current name: {}", original_name);
    let name_input = prompt_for_input("Name").await?;
    if !name_input.is_empty() {
        item.name = name_input;
    }

    println!("Current type: {}", original_type);
    let type_input = prompt_for_input("Type").await?;
    if !type_input.is_empty() {
        item.item_type =
            ItemType::from_str(&type_input).map_err(|_| CliError::InvalidItemType {
                item_type: type_input,
            })?;
    }

    println!("Current author: {}", original_author);
    let author_input = prompt_for_input("Author").await?;
    if !author_input.is_empty() {
        item.author = author_input;
    }

    println!("Current link: {}", original_link);
    let link_input = prompt_for_input("Link").await?;
    if !link_input.is_empty() {
        item.link = link_input;
    }

    println!("Current note: {}", original_note);
    let note_input = prompt_for_input("Note").await?;
    if !note_input.is_empty() {
        item.note = note_input;
    }

    item.validate()?;

    if archive {
        folio_storage::save_archive(&archive_items)?;
    } else {
        save_inbox(&inbox_items)?;
    }

    println!("Item #{} updated successfully", id);
    Ok(())
}

async fn handle_archive_command(id: usize) -> Result<(), CliError> {
    let inbox_path = get_inbox_path()?;
    let archive_path = get_archive_path()?;
    let mut inbox_items = load_items_from_file(&inbox_path)?;
    let archive_items = load_items_from_file(&archive_path)?;

    let index = match resolve_item_index(id, inbox_items.len()) {
        Ok(index) => index,
        Err(_) => {
            if resolve_item_index(id, archive_items.len()).is_ok() {
                println!("Item #{} is already in archive", id);
                return Ok(());
            }
            return Err(CliError::ItemNotFound { id });
        }
    };

    let mut item = inbox_items.remove(index);

    item.status = Status::Done;
    folio_core::update_timestamps(&mut item);

    append_to_archive(&item)?;

    save_inbox(&inbox_items)?;

    println!("Item #{} marked as done and archived successfully", id);
    Ok(())
}

async fn handle_delete_command(id: usize, archive: bool) -> Result<(), CliError> {
    let inbox_path = get_inbox_path()?;
    let archive_path = get_archive_path()?;
    let mut inbox_items = load_items_from_file(&inbox_path)?;
    let mut archive_items = load_items_from_file(&archive_path)?;

    let view_items: &mut Vec<folio_core::Item> = if archive {
        &mut archive_items
    } else {
        &mut inbox_items
    };
    let index = resolve_item_index(id, view_items.len())?;
    let item = view_items[index].clone();

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

    view_items.remove(index);

    if archive {
        folio_storage::save_archive(&archive_items)?;
    } else {
        save_inbox(&inbox_items)?;
    }

    println!("Item #{} deleted successfully", id);
    Ok(())
}

async fn handle_mark_ref_command(id: usize, archive: bool) -> Result<(), CliError> {
    let inbox_path = get_inbox_path()?;
    let archive_path = get_archive_path()?;
    let inbox_items = load_items_from_file(&inbox_path)?;
    let mut archive_items = load_items_from_file(&archive_path)?;

    if !archive {
        if resolve_item_index(id, inbox_items.len()).is_ok() {
            println!(
                "Cannot mark reference for items in inbox. Only archived (done) items can be marked as references. Use 'folio mark-ref --archive <id>'."
            );
            return Ok(());
        }
        return Err(CliError::ItemNotFound { id });
    }

    let index = resolve_item_index(id, archive_items.len())?;
    let item = &mut archive_items[index];
    match item.kind {
        folio_core::Kind::Normal => {
            item.kind = folio_core::Kind::Reference;
            folio_storage::save_archive(&archive_items)?;
            println!("Item #{} marked as reference", id);
        }
        folio_core::Kind::Reference => {
            item.kind = folio_core::Kind::Normal;
            folio_storage::save_archive(&archive_items)?;
            println!("Item #{} unmarked as reference", id);
        }
    }

    Ok(())
}

async fn prompt_for_input(field_name: &str) -> Result<String, CliError> {
    use std::io::{Write, stdin, stdout};

    print!("{}: ", field_name);
    stdout().flush()?;

    let mut input = String::new();
    stdin().read_line(&mut input)?;
    Ok(input.trim().to_string())
}

async fn handle_add_command(
    config_manager: &ConfigManager,
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

    let config = config_manager.get();

    let inbox_path = get_inbox_path()?;
    let inbox_items = load_items_from_file(&inbox_path)?;

    let new_item = folio_core::create_item(
        name.clone().unwrap_or_else(|| "Untitled".to_string()),
        item_type.clone(),
        author.clone(),
        link.clone(),
        note.clone(),
        kind.clone(),
    )?;

    if matches!(new_item.kind, Kind::Reference) {
        let mut ref_item = new_item;
        ref_item.status = Status::Done;
        folio_core::update_timestamps(&mut ref_item);
        append_to_archive(&ref_item)?;
        println!("Added reference item as done and archived");
        return Ok(());
    }

    match folio_core::add_item_to_inbox(inbox_items, new_item, config) {
        Ok((new_inbox, to_archive)) => {
            let has_archived_items = !to_archive.is_empty();

            save_inbox(&new_inbox)?;

            for item in &to_archive {
                append_to_archive(item)?;
            }

            if has_archived_items {
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
        Err(_) => {
            match &config.archive_on_overflow {
                OverflowStrategy::Abort => {
                    println!("Inbox limit ({}) reached.", config.max_items);
                    println!();
                    println!("Choose an action:");
                    println!("  [D]elete an existing item (use 'folio delete <id>')");
                    println!(
                        "  [A]rchive an item (change status to 'done' or use 'folio archive <id>')"
                    );
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

                    if response == "y" || response == "yes" {
                        let inbox_items = load_items_from_file(&inbox_path)?;
                        if !inbox_items.is_empty() {
                            println!();
                            println!("{}", folio_core::Item::format_list_header());
                            println!("{}", folio_core::Item::format_list_separator());

                            for (index, item) in inbox_items.iter().enumerate() {
                                println!("{}", item.format_for_list(index + 1));
                            }
                        }
                    }

                    std::process::exit(1);
                }
                _ => {
                    // This shouldn't happen since we only return Err for Abort strategy
                    // But just in case, handle it gracefully
                    Err(CliError::InboxFull {
                        limit: config.max_items,
                        suggestions: "This should not happen with current overflow strategies."
                            .to_string(),
                    })
                }
            }
        }
    }
}

async fn handle_config_command(subcommand: &ConfigSubcommands) -> Result<(), CliError> {
    match subcommand {
        ConfigSubcommands::List => {
            let config_manager = ConfigManager::new().map_err(|e| CliError::ConfigError {
                message: e.to_string(),
            })?;
            let json = serde_json::to_string_pretty(config_manager.get())?;
            println!("{}", json);
        }
        ConfigSubcommands::Get { key } => {
            let config_manager = ConfigManager::new().map_err(|e| CliError::ConfigError {
                message: e.to_string(),
            })?;
            let config_value = serde_json::to_value(config_manager.get())?;

            match key.as_str() {
                "max_items" => println!("{}", config_value["max_items"]),
                "archive_on_overflow" => println!("{}", config_value["archive_on_overflow"]),
                "version" | "_v" => println!("{}", config_value["_v"]),
                _ => {
                    return Err(CliError::UnknownConfigKey { key: key.clone() });
                }
            }
        }
        ConfigSubcommands::Set { key, value } => {
            let mut config_manager = ConfigManager::new().map_err(|e| CliError::ConfigError {
                message: e.to_string(),
            })?;

            config_manager
                .update(|config| match key.as_str() {
                    "max_items" => {
                        if let Ok(val) = value.parse::<u32>() {
                            config.max_items = val;
                        }
                    }
                    "archive_on_overflow" => match value.as_str() {
                        "abort" => config.archive_on_overflow = OverflowStrategy::Abort,
                        "todo" => config.archive_on_overflow = OverflowStrategy::Todo,
                        "any" => config.archive_on_overflow = OverflowStrategy::Any,
                        _ => {}
                    },
                    _ => {}
                })
                .map_err(|e| CliError::ConfigError {
                    message: e.to_string(),
                })?;

            match key.as_str() {
                "max_items" => {
                    if value.parse::<u32>().is_err() {
                        return Err(CliError::InvalidMaxItems {
                            value: value.clone(),
                        });
                    }
                }
                "archive_on_overflow" => {
                    if !matches!(value.as_str(), "abort" | "todo" | "any") {
                        return Err(CliError::InvalidOverflowStrategy {
                            value: value.to_string(),
                        });
                    }
                }
                _ => {
                    return Err(CliError::UnknownConfigKey { key: key.clone() });
                }
            }

            println!("Config updated successfully");
        }
        ConfigSubcommands::Reset => {
            let mut config_manager = ConfigManager::new().map_err(|e| CliError::ConfigError {
                message: e.to_string(),
            })?;

            config_manager
                .update(|config| {
                    *config = folio_core::Config::default();
                })
                .map_err(|e| CliError::ConfigError {
                    message: e.to_string(),
                })?;

            println!("Config reset to default values");
        }
    }

    Ok(())
}
