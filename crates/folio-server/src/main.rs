use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "folio-server", about = "Folio server", version = "0.1.0")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Serve,
    Import,
}

fn main() {
    let cli = Cli::parse();
    match cli.command {
        Commands::Serve => println!("Not implemented yet"),
        Commands::Import => println!("Not implemented yet"),
    }
}
