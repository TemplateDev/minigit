use anyhow::{bail, Context, Result};
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};

use crate::{index, objects, repository};

pub fn run(branch: &str) -> Result<()> {
    let repo = std::env::current_dir()?;
    let minigit_dir = repo.join(".minigit");

    validate_branch_name(branch)?;

    // 1. Verify the target branch exists
    let branch_path = minigit_dir
        .join("refs")
        .join("heads")
        .join(branch);

    if !branch_path.is_file() {
        bail!("Branch '{branch}' does not exist");
    }

    // 2. Refuse to switch if there is uncommitted work
    ensure_clean_working_tree(&repo)?;

    // 3. Check if already on this branch
    let current_ref = repository::head_ref(&repo)?;

    if current_ref == format!("refs/heads/{branch}") {
        println!("Already on branch '{branch}'");
        return Ok(());
    }

    // 4. Read and load the target commit
    let target_hash = fs::read_to_string(&branch_path)?;
    let target_hash = target_hash.trim();

    if target_hash.is_empty() {
        bail!("Branch '{branch}' has no commits");
    }

    let target_commit = objects::load_commit(&repo, target_hash)?;
    let target_files = target_commit.files;

    // 5. Load the index
    let current_index = index::load(&repo)?;

    // 6. Restore the working directory
    restore_working_tree(
        &repo,
        &current_index.entries,
        &target_files,
    )?;

    // 7. Make the index match the target commit
    let new_index = index::Index {
        entries: target_files,
    };

    index::save(&repo, &new_index)?;

    // 8. Update HEAD last
    fs::write(
        minigit_dir.join("HEAD"),
        format!("ref: refs/heads/{branch}\n"),
    )?;

    println!("Switched to branch '{branch}'");

    Ok(())
}

fn validate_branch_name(name: &str) -> Result<()> {
    if name.is_empty()
        || name.starts_with('-')
        || name.contains('/')
        || name.contains('\\')
        || name.contains("..")
        || name.chars().any(char::is_whitespace)
    {
        bail!("Invalid branch name: {name}");
    }

    Ok(())
}

fn ensure_clean_working_tree(repo: &Path) -> Result<()> {
    let head_files = repository::load_head(repo)?
        .map(|commit| commit.files)
        .unwrap_or_default();

    let current_index = index::load(repo)?;

    if current_index.entries != head_files {
        bail!("Cannot checkout: you have staged changes");
    }

    let working_files = read_working_tree(repo)?;

    if working_files != current_index.entries {
        bail!(
            "Cannot checkout: working tree has modifications or untracked files"
        );
    }

    Ok(())
}

fn read_working_tree(repo: &Path) -> Result<BTreeMap<String, String>> {
    let mut files = BTreeMap::new();

    let walker = ignore::WalkBuilder::new(repo)
        .hidden(false)
        .standard_filters(true)
        .filter_entry(|entry| {
            let name = entry.file_name();

            name != ".minigit" && name != ".git"
        })
        .build();

    for entry in walker {
        let entry = entry?;
        let path = entry.path();

        if !entry.file_type().is_some_and(|kind| kind.is_file()) {
            continue;
        }

        let relative_path = path.strip_prefix(repo)?;
        let relative_path = relative_path.to_string_lossy().replace('\\', "/");

        let contents = fs::read(path)?;
        let hash = objects::to_hash(&contents)?;

        files.insert(relative_path, hash);
    }

    Ok(files)
}

fn restore_working_tree(
    repo: &Path,
    current_files: &BTreeMap<String, String>,
    target_files: &BTreeMap<String, String>,
) -> Result<()> {
    let objects_dir = repo.join(".minigit").join("objects");
    
    // Remove files that are no longer present in the target snapshot
    for path in current_files.keys() {
        if !target_files.contains_key(path) {
            let file_path = safe_repo_path(repo, path)?;

            if file_path.is_file() {
                fs::remove_file(&file_path)?;
                remove_empty_parents(repo, &file_path)?;
            }
        }
    }

    // Restore every file in the target snapshot
    for (path, hash) in target_files {
        let file_path = safe_repo_path(repo, path)?;

        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent)?;
        }
        
        if file_path.is_dir() {
            fs::remove_dir_all(&file_path)?;
        }

        let contents = fs::read(objects_dir.join(hash))
            .with_context(|| format!("Could not load blob for '{path}'"))?;

        fs::write(&file_path, contents)?;
    }

    Ok(())
}

fn remove_empty_parents(repo: &Path, file_path: &Path) -> Result<()> {
    let mut current = file_path.parent();

    while let Some(directory) = current {
        if directory == repo {
            break;
        }

        match fs::remove_dir(directory) {
            Ok(()) => {}
            Err(error) if error.kind() == std::io::ErrorKind::DirectoryNotEmpty => {
                break;
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                break;
            }
            Err(error) => return Err(error.into()),
        }

        current = directory.parent();
    }

    Ok(())
}

fn safe_repo_path(repo: &Path, relative_path: &str) -> Result<PathBuf> {
    let path = Path::new(relative_path);

    if path.as_os_str().is_empty()
        || path.components().any(|component| {
            !matches!(component, std::path::Component::Normal(_))
        })
    {
        bail!("Invalid path in commit: {relative_path}");
    }

    Ok(repo.join(path))
}