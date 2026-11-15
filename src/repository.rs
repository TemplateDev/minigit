
use std::fs;
use std::io;
use std::path::Path;
use crate::{models::commit::Commit, objects};
use anyhow::{Result};

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

/// Reads `HEAD` to find the current branch, then loads and returns its commit.
/// Returns `Ok(None)` when the branch has no commit yet; returns an error if
/// `HEAD`, the branch reference, or the referenced commit cannot be read.
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


/// Reads `HEAD` to find the current branch, then returns the hash stored in
/// that branch's reference file. Returns `Ok(None)` when the branch reference
/// does not exist yet or contains no hash. Unlike [`load_head`], this function
/// does not load or validate the referenced commit. Returns an error if `HEAD`
/// or an existing branch reference cannot be read, or if `HEAD` is invalid.
pub fn head_hash(repo: &Path) -> std::io::Result<Option<String>> {
    let head_contents = std::fs::read_to_string(
        repo.join(".minigit").join("HEAD")
    )?;

    let reference = head_contents
        .trim()
        .strip_prefix("ref: ")
        .ok_or_else(|| std::io::Error::other("Invalid HEAD reference"))?;

    let branch_path = repo.join(".minigit").join(reference);

    if !branch_path.exists() {
        return Ok(None);
    }

    let hash = std::fs::read_to_string(branch_path)?;
    let hash = hash.trim().to_string();

    if hash.is_empty() {
        Ok(None)
    } else {
        Ok(Some(hash))
    }
}

pub fn head_ref(repo: &Path) -> Result<String> {
    let head_path = repo.join(".minigit").join("HEAD");
    let contents = fs::read_to_string(head_path)?;

    let reference = contents
        .trim()
        .strip_prefix("ref: ")
        .ok_or_else(|| anyhow::anyhow!("Invalid HEAD reference"))?;

    if !reference.starts_with("refs/heads/") {
        anyhow::bail!("HEAD does not point to a branch");
    }

    Ok(reference.to_string())
}
