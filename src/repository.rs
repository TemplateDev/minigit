
use std::fs;
use std::io;
use std::path::Path;
use crate::{models::commit::Commit, objects};

pub fn init() -> io::Result<()> {
    let current_dir = std::env::current_dir()?;
    let minigit_dir = current_dir.join(".minigit");

    if minigit_dir.exists() {
        println!("MiniGit repository already exists.");
        return Ok(());
    }

    // Create object storage.
    fs::create_dir_all(minigit_dir.join("objects"))?;

    // Create branch references directory.
    fs::create_dir_all(minigit_dir.join("refs").join("heads"))?;

    // Point HEAD to the default branch.
    fs::write(
        minigit_dir.join("HEAD"),
        "ref: refs/heads/main\n",
    )?;

    // Initialize an empty staging area.
    fs::write(
        minigit_dir.join("index"),
        "{}\n",
    )?;

    println!("Initialized empty MiniGit repository.");

    Ok(())
}

pub fn load_head(repo: &Path) -> std::io::Result<Option<Commit>> {
    let head_path = repo.join(".minigit").join("HEAD");
    let head_contents = std::fs::read_to_string(head_path)?;
    
    let reference = head_contents
        .trim()
        .strip_prefix("ref: ")
        .ok_or_else(|| {
            std::io::Error::other("Invalid HEAD reference")
        })?;

    let branch_path = repo.join(".minigit").join(reference);

    // A branch without a commit doesn't have a reference file yet.
    if !branch_path.exists() {
        return Ok(None);
    }

    let hash = std::fs::read_to_string(branch_path)?;
    let hash = hash.trim();

    if hash.is_empty() {
        return Ok(None);
    }

    let commit = objects::load_commit(repo, hash)?;

    Ok(Some(commit))
}
