use std::{fs};

use crate::{
    index,
    models::commit::Commit,
    objects,
    repository,
};

pub fn run(message: &str) -> Result<(), Box<dyn std::error::Error>> {
    let repo = std::env::current_dir()?;

    if message.trim().is_empty() {
        return Err("Commit message cannot be empty".into());
    }

    // Load the staging area
    let mut staging_area = index::load(&repo)?;
    let previous_commit = repository::load_head(&repo)?;

    let previous_files = previous_commit
        .as_ref()
        .map(|commit| &commit.files);

    // Don't create a commit if nothing has changed.
    if previous_files.is_some_and(|files| files == &staging_area.entries) {
        return Err("Nothing to commit".into());
    }

    if previous_files.is_none() && staging_area.entries.is_empty() {
        return Err("Nothing to commit".into());
    }

    let parent = repository::head_hash(&repo)?;
    let commit = Commit {
        message: message.to_string(),
        parent, // Point to parent or none
        files: std::mem::take(&mut staging_area.entries),
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_secs(),
    };

    let commit_hash = objects::write_commit(&repo, &commit)?;

    // Resolve the current branch.
    let head_contents = fs::read_to_string(
        repo.join(".minigit").join("HEAD")
    )?;

    let reference = head_contents
        .trim()
        .strip_prefix("ref: ")
        .ok_or("Invalid HEAD reference")?;

    let branch_path = repo.join(".minigit").join(reference);

    // Advance the branch pointer.
    fs::write(&branch_path, format!("{commit_hash}\n"))?;

    // Clear the staging area.
    index::save(&repo, &staging_area)?;

    println!("[{}] {}", &commit_hash[..7], message);

    Ok(())
}
