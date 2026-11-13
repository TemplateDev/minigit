
use clap::{Parser, Subcommand};

mod repository;

#[derive(Parser)]
#[command(name = "minigit", about = "A miniature Git implementation")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Init,
}

fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Init => repository::init(),
    };

    if let Err(error) = result {
        eprintln!("Error: {}", error);
        std::process::exit(1);
    }
}
