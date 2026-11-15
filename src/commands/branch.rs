use anyhow::{Context, Result, bail};
use std::fs;
use std::path::Path;

use crate::{BranchAction, repository};

pub fn run(action: Option<BranchAction>) -> Result<()> {
    let repo = std::env::current_dir()?;
    let heads_dir = repo.join(".minigit").join("refs").join("heads");

    match action {
        None => list_branches(&repo, &heads_dir),
        Some(action) => match action {
            BranchAction::Create { name } => create_branch(&repo, &heads_dir, &name),
            BranchAction::Rename { old, new } => rename_branch(&repo, &old, &new),
            BranchAction::Delete { name } => delete_branch(&repo, &name)
        }
    }
}

fn list_branches(repo: &Path, heads_dir: &Path) -> Result<()> {
    let current_ref = repository::head_ref(&repo)?;
    let current_branch = current_ref
        .strip_prefix("refs/heads/")
        .unwrap_or(&current_ref);

    let mut branches = Vec::new();

    for entry in fs::read_dir(heads_dir)? {
        let entry = entry?;
        if entry.file_type()?.is_file() {
            branches.push(entry.file_name().to_string_lossy().to_string());
        }
    }

    branches.sort();

    for branch in branches {
        if branch == current_branch {
            println!("* {branch}");
        } else {
            println!("  {branch}");
        }
    }

    Ok(())
}

fn create_branch(repo: &Path, heads_dir: &Path, name: &str) -> Result<()> {
    if name.is_empty()
        || name.starts_with('-')
        || name.contains('/')
        || name.contains("..")
        || name.chars().any(char::is_whitespace)
    {
        bail!("Invalid branch name: {name}");
    }

    let current_hash = repository::head_hash(repo)?
        .ok_or_else(|| anyhow::anyhow!("Cannot create a branch before the first commit"))?;

    let branch_path = heads_dir.join(name);

    if branch_path.exists() {
        bail!("Branch '{name}' already exists");
    }

    fs::write(branch_path, format!("{current_hash}\n"))?;

    println!("Created branch '{name}'");

    Ok(())
}

fn validate_branch_name(name: &str) -> Result<()> {
    if name.is_empty()
        || name.starts_with('-')
        || name.contains('/')
        || name.contains("..")
        || name.chars().any(char::is_whitespace)
    {
        bail!("Invalid branch name: {name}");
    }

    Ok(())
}

pub fn delete_branch(repo: &Path, name: &str) -> Result<()> {
    validate_branch_name(name)?;

    let current_ref = repository::head_ref(repo)?;
    let current_branch = current_ref
        .strip_prefix("refs/heads/")
        .unwrap_or(&current_ref);

    if name == current_branch {
        bail!("Cannot delete the currently checked-out branch");
    }

    let branch_path = repo
        .join(".minigit")
        .join("refs")
        .join("heads")
        .join(name);

    if !branch_path.is_file() {
        bail!("Branch '{name}' does not exist");
    }

    fs::remove_file(&branch_path)
        .with_context(|| format!("Failed to delete branch '{name}'"))?;

    println!("Deleted branch '{name}'");

    Ok(())
}

pub fn rename_branch(
    repo: &Path,
    old: &str,
    new: &str,
) -> Result<()> {
    validate_branch_name(old)?;
    validate_branch_name(new)?;

    let heads_dir = repo
        .join(".minigit")
        .join("refs")
        .join("heads");

    let old_path = heads_dir.join(old);
    let new_path = heads_dir.join(new);

    if !old_path.is_file() {
        bail!("Branch '{old}' does not exist");
    }

    if new_path.exists() {
        bail!("Branch '{new}' already exists");
    }

    fs::rename(&old_path, &new_path)?;

    // If this is the currently checked-out branch,
    // update HEAD to point to the new name.
    let current_ref = repository::head_ref(repo)?;

    if current_ref == format!("refs/heads/{old}") {
        fs::write(
            repo.join(".minigit").join("HEAD"),
            format!("ref: refs/heads/{new}\n"),
        )?;
    }

    println!("Renamed branch '{old}' to '{new}'");

    Ok(())
}

