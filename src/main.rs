use clap::{Parser, Subcommand};

mod repository;
mod objects;
mod index;
mod commands;
mod models;

#[derive(Parser)]
#[command(name = "minigit", about = "A miniature Git implementation")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(clap::Subcommand)]
pub enum BranchAction {
    Create {
        name: String,
    },
    Delete {
        name: String,
    },
    Rename {
        old: String,
        new: String,
    },
}

#[derive(Subcommand)]
enum Commands {
    Init,
    Add { patterns: Vec<String> },
    Status,
    Commit {
        #[arg(short, long)]
        message: String
    },
    Log,
    Branch {
        #[command(subcommand)]
        action: Option<BranchAction>
    },
    Checkout {
        branch: String,
    }
}

fn main() {
    let cli = Cli::parse();

    let result: Result<(), Box<dyn std::error::Error>> = match cli.command {
        Commands::Init => repository::init().map_err(|error| Box::new(error) as Box<dyn std::error::Error>),
        Commands::Add { patterns } => commands::add::run(&patterns),
        Commands::Status => commands::status::run(),
        Commands::Commit { message } => commands::commit::run(&message),
        Commands::Log => commands::log::run(),
        Commands::Branch { action } => commands::branch::run(action).map_err(|error| {
            Box::new(std::io::Error::new(std::io::ErrorKind::Other, error.to_string()))
                as Box<dyn std::error::Error>
        }),
        Commands::Checkout { branch } => commands::checkout::run(&branch).map_err(|error| {
            Box::new(std::io::Error::new(std::io::ErrorKind::Other, error.to_string()))
                as Box<dyn std::error::Error>
        })
    };

    if let Err(error) = result {
        eprintln!("Error: {}", error);
        std::process::exit(1);
    }
}
