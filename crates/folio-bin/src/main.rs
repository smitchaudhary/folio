use clap::Parser;
use folio_bin::cli::Cli;

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
            // For now, just print the command to verify it's working
            println!("Command: {:?}", command);
        }
        None => {
            // Default behavior when no subcommand is provided
            println!("Running default TUI mode");
        }
    }
    
    Ok(())
}
