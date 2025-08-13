use chrono::Utc;
use clap::Parser;
use folio_bin::cli::{Cli, Commands, ConfigSubcommands};
use folio_core::add_with_cap;
use folio_core::{Item, ItemType, Kind, OverflowStrategy, Status};
use folio_storage::{
    append_to_archive, get_archive_path, get_inbox_path, load_config, load_items_from_file,
    save_config, save_inbox,
};
use serde_json;
use std::process;
use std::str::FromStr;

fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match &cli.command {
        Some(command) => {
            match command {
                Commands::Add {
                    name,
                    r#type,
                    author,
                    link,
                    note,
                    kind,
                } => {
                    handle_add_command(name, r#type, author, link, note, kind)?;
                }
                Commands::List { status, r#type } => {
                    handle_list_command(status.as_deref(), r#type.as_deref())?;
                }
                Commands::SetStatus { id, status } => {
                    handle_set_status_command(*id, status)?;
                }
                Commands::Config { subcommand } => {
                    handle_config_command(subcommand)?;
                }
                _ => {
                    // For now, just print the command to verify it's working
                    println!("Command: {:?}", command);
                }
            }
        }
        None => {
            // Default behavior when no subcommand is provided
            println!("Running default TUI mode");
        }
    }

    Ok(())
}

fn handle_list_command(
    status_filters: Option<&[String]>,
    type_filters: Option<&[String]>,
) -> Result<(), Box<dyn std::error::Error>> {
    let inbox_path = get_inbox_path()?;
    let archive_path = get_archive_path()?;

    let inbox_items = load_items_from_file(&inbox_path)?;
    let archive_items = load_items_from_file(&archive_path)?;

    let mut all_items = Vec::new();
    all_items.extend(inbox_items);
    all_items.extend(archive_items);

    let filtered_items: Vec<_> = all_items
        .into_iter()
        .filter(|item| {
            if let Some(status_filters) = status_filters {
                if !status_filters.is_empty() {
                    let matches = status_filters.iter().any(|s| {
                        Status::from_str(&s.to_lowercase())
                            .map_or(false, |status| status == item.status)
                    });
                    if !matches {
                        return false;
                    }
                }
            }

            if let Some(type_filters) = type_filters {
                if !type_filters.is_empty() {
                    let matches = type_filters.iter().any(|t| {
                        ItemType::from_str(&t.to_lowercase())
                            .map_or(false, |item_type| item_type == item.item_type)
                    });
                    if !matches {
                        return false;
                    }
                }
            }

            true
        })
        .collect();

    if filtered_items.is_empty() {
        println!("No items found.");
        return Ok(());
    }
    println!(
        "{:<4} {:<6} {:<30} {:<10} {:<20} {:<15}",
        "ID", "Status", "Name", "Type", "Added", "Author"
    );
    println!("{}", "-".repeat(100));

    for (index, item) in filtered_items.iter().enumerate() {
        let status_char = match item.status {
            folio_core::Status::Todo => "T",
            folio_core::Status::Doing => "D",
            folio_core::Status::Done => "✓",
        };

        let type_abbr = match item.item_type {
            folio_core::ItemType::Article => "art.",
            folio_core::ItemType::Video => "vid.",
            folio_core::ItemType::Blog => "blog",
            folio_core::ItemType::Other => "other",
        };

        let added_date = item.added_at.format("%Y-%m-%d").to_string();

        let name_display = if item.name.len() > 28 {
            format!("{}..", &item.name[..26])
        } else {
            item.name.clone()
        };

        let author_display = if item.author.len() > 13 {
            format!("{}..", &item.author[..11])
        } else {
            item.author.clone()
        };

        println!(
            "{:<4} {:<6} {:<30} {:<10} {:<20} {:<15}",
            index + 1,
            status_char,
            name_display,
            type_abbr,
            added_date,
            author_display
        );
    }
    Ok(())
}

fn handle_set_status_command(
    id: usize,
    status_str: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let inbox_path = get_inbox_path()?;
    let archive_path = get_archive_path()?;

    let mut inbox_items = load_items_from_file(&inbox_path)?;
    let mut archive_items = load_items_from_file(&archive_path)?;

    let mut item_found = false;

    if id > 0 && id <= inbox_items.len() {
        let item_index = id - 1;
        let item = &mut inbox_items[item_index];

        let new_status =
            Status::from_str(status_str).map_err(|_| format!("Invalid status: {}", status_str))?;

        let item_status_changed_to_done = item.status != Status::Done && new_status == Status::Done;

        item.status = new_status;

        folio_core::update_timestamps(item);

        item_found = true;

        if item_status_changed_to_done {
            let done_item = inbox_items.remove(item_index);
            archive_items.push(done_item);
            println!(
                "Item #{} status updated to '{}' and moved to archive",
                id, status_str
            );
        } else {
            println!("Item #{} status updated to '{}'", id, status_str);
        }
    } else if id > inbox_items.len() && id <= inbox_items.len() + archive_items.len() {
        let item_index = id - inbox_items.len() - 1;
        let item = &mut archive_items[item_index];

        let new_status =
            Status::from_str(status_str).map_err(|_| format!("Invalid status: {}", status_str))?;

        item.status = new_status;

        folio_core::update_timestamps(item);

        item_found = true;
        println!("Item #{} status updated to '{}'", id, status_str);
    }

    if !item_found {
        return Err(format!("Item with ID {} not found", id).into());
    }

    save_inbox(&inbox_items)?;
    folio_storage::save_archive(&archive_items)?;

    Ok(())
}

fn handle_add_command(
    name: &Option<String>,
    item_type: &Option<String>,
    author: &Option<String>,
    link: &Option<String>,
    note: &Option<String>,
    kind: &Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let config = load_config()?;

    let inbox_path = get_inbox_path()?;
    let inbox_items = load_items_from_file(&inbox_path)?;

    let parsed_type = match item_type {
        Some(t) => ItemType::from_str(t).unwrap_or(ItemType::Article),
        None => ItemType::Article,
    };

    let parsed_kind = match kind {
        Some(k) => Kind::from_str(k).unwrap_or(Kind::Normal),
        None => Kind::Normal,
    };

    let new_item = Item {
        name: name.clone().unwrap_or_else(|| "Untitled".to_string()),
        item_type: parsed_type,
        status: Status::Todo,
        author: author.clone().unwrap_or_default(),
        link: link.clone().unwrap_or_default(),
        added_at: Utc::now(),
        started_at: None,
        finished_at: None,
        note: note.clone().unwrap_or_default(),
        kind: parsed_kind,
        version: 1,
    };

    new_item.validate()?;

    if matches!(new_item.kind, Kind::Reference) {
        append_to_archive(&new_item)?;
        println!("Added reference item directly to archive");
        return Ok(());
    }

    let (new_inbox, to_archive) = add_with_cap(
        inbox_items,
        new_item,
        config.max_items as usize,
        config.archive_on_overflow,
    )?;

    save_inbox(&new_inbox)?;

    for item in to_archive {
        append_to_archive(&item)?;
    }

    println!("Item added successfully");
    Ok(())
}

fn handle_config_command(subcommand: &ConfigSubcommands) -> Result<(), Box<dyn std::error::Error>> {
    match subcommand {
        ConfigSubcommands::List => {
            let config = load_config()?;
            let json = serde_json::to_string_pretty(&config)?;
            println!("{}", json);
        }
        ConfigSubcommands::Get { key } => {
            let config = load_config()?;
            let config_value = serde_json::to_value(&config)?;

            match key.as_str() {
                "max_items" => println!("{}", config_value["max_items"]),
                "archive_on_overflow" => println!("{}", config_value["archive_on_overflow"]),
                "auto_archive_done_days" => println!("{}", config_value["auto_archive_done_days"]),
                "version" | "_v" => println!("{}", config_value["_v"]),
                _ => {
                    eprintln!("Unknown config key: {}", key);
                    process::exit(1);
                }
            }
        }
        ConfigSubcommands::Set { key, value } => {
            let mut config = load_config()?;

            match key.as_str() {
                "max_items" => match value.parse::<u32>() {
                    Ok(val) => config.max_items = val,
                    Err(_) => {
                        eprintln!("Invalid value for max_items: {}", value);
                        process::exit(1);
                    }
                },
                "auto_archive_done_days" => match value.parse::<u32>() {
                    Ok(val) => config.auto_archive_done_days = val,
                    Err(_) => {
                        eprintln!("Invalid value for auto_archive_done_days: {}", value);
                        process::exit(1);
                    }
                },
                "archive_on_overflow" => match value.as_str() {
                    "abort" => config.archive_on_overflow = OverflowStrategy::Abort,
                    "todo" => config.archive_on_overflow = OverflowStrategy::Todo,
                    "done" => config.archive_on_overflow = OverflowStrategy::Done,
                    "any" => config.archive_on_overflow = OverflowStrategy::Any,
                    _ => {
                        eprintln!(
                            "Invalid value for archive_on_overflow: {}. Must be one of: abort, todo, done, any",
                            value
                        );
                        process::exit(1);
                    }
                },
                _ => {
                    eprintln!("Unknown config key: {}", key);
                    process::exit(1);
                }
            }

            save_config(&config)?;
            println!("Config updated successfully");
        }
        ConfigSubcommands::Reset => {
            let config = folio_core::Config::default();
            save_config(&config)?;
            println!("Config reset to default values");
        }
    }

    Ok(())
}
