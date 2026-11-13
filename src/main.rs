
use std::path::PathBuf;

use clap::{Parser, Subcommand};

mod repository;
mod objects;
mod index;
mod commands;

#[derive(Parser)]
#[command(name = "minigit", about = "A miniature Git implementation")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Init,
    Add { path: PathBuf }
}

fn main() {
    let cli = Cli::parse();

    let result: Result<(), Box<dyn std::error::Error>> = match cli.command {
        Commands::Init => repository::init().map_err(|error| Box::new(error) as Box<dyn std::error::Error>),
        Commands::Add { path } => commands::add::run(&path),
    };

    if let Err(error) = result {
        eprintln!("Error: {}", error);
        std::process::exit(1);
    }
}
