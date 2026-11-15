use std::time::{Duration, UNIX_EPOCH};

use crate::{objects, repository};

use chrono::{DateTime, Local};

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    let repo = std::env::current_dir()?;

    let Some(mut hash) = repository::head_hash(&repo)? else {
        println!("No commits yet.");
        return Ok(());
    };

    loop {
        let commit = objects::load_commit(&repo, &hash)?;

        let datetime: DateTime<Local> =
            (UNIX_EPOCH + Duration::from_secs(commit.timestamp)).into();

        println!("commit {}", hash);
        println!("Date:   {}", datetime.format("%Y-%m-%d %I:%M:%S %p"));
        println!();
        println!("    {}", commit.message);
        println!();

        match commit.parent {
            Some(parent) => hash = parent,
            None => break,
        }
    }

    Ok(())
}
