
use std::fs;
use std::io;
use std::path::Path;
use crate::{models::commit::Commit, objects};
use crate::models::head::Head;
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


/// Loads the commit currently referenced by `HEAD`, if one exists.
///
/// Returns `Ok(None)` when the current branch has no commit yet.
pub fn load_head(repo: &Path) -> Result<Option<Commit>> {
    let Some(hash) = head_hash(repo)? else {
        return Ok(None);
    };

    Ok(Some(objects::load_commit(repo, &hash)?))
}

/// Returns the commit hash referenced by `HEAD`, if one exists.
///
/// Returns `Ok(None)` when the current branch has no commit yet.
pub fn head_hash(repo: &Path) -> Result<Option<String>> {
    match read_head(repo)? {
        Head::Symbolic(reference) => {
            let reference_path = repo.join(".minigit").join(reference);

            if !reference_path.exists() {
                return Ok(None);
            }

            let hash = fs::read_to_string(reference_path)?;
            let hash = hash.trim();

            if hash.is_empty() {
                Ok(None)
            } else {
                Ok(Some(hash.to_string()))
            }
        }
        Head::Detached(hash) => Ok(Some(hash)),
    }
}

pub fn head_ref(repo: &Path) -> Result<String> {
    match read_head(repo)? {
        Head::Symbolic(reference) => Ok(reference),
        Head::Detached(_) => {
            anyhow::bail!("HEAD is detached and does not point to a branch")
        }
    }
}

pub fn read_head(repo: &Path) -> Result<Head> {
    let head_path = repo.join(".minigit").join("HEAD");
    let contents = fs::read_to_string(head_path)?;
    let contents = contents.trim();

    if let Some(reference) = contents.strip_prefix("ref: ") {
        if !reference.starts_with("refs/heads/") {
            anyhow::bail!("Invalid HEAD reference");
        }

        return Ok(Head::Symbolic(reference.to_string()));
    }

    if contents.is_empty() {
        anyhow::bail!("HEAD is empty");
    }

    Ok(Head::Detached(contents.to_string()))
}