use chrono::Utc;
use clap::Parser;
use folio_bin::cli::{Cli, Commands, ConfigSubcommands};
use folio_core::add_with_cap;
use folio_core::{Item, ItemType, Kind, OverflowStrategy, Status};
use folio_storage::{
    append_to_archive, get_inbox_path, load_config, load_items_from_file, save_config, save_inbox,
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
