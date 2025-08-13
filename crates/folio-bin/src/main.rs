use chrono::Utc;
use clap::Parser;
use folio_bin::cli::{Cli, Commands};
use folio_core::add_with_cap;
use folio_core::{Item, ItemType, Kind, Status};
use folio_storage::{
    append_to_archive, get_inbox_path, load_config, load_items_from_file, save_inbox,
};
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
