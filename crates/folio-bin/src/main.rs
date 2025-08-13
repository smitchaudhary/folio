use clap::Parser;
use folio_bin::cli::Cli;

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Some(command) => {
            println!("Command: {:?}", command);
        }
        None => {
            println!("Running default TUI mode");
        }
    }
}
