use anyhow::{bail, Context, Result};
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};

use crate::{index, objects, repository};
use crate::models::head::Head;

pub fn run(target: &str) -> Result<()> {
    let repo = std::env::current_dir()?;
    let minigit_dir = repo.join(".minigit");

    // 1. Resolve the target as either a branch or a commit hash
    let (target_hash, target_branch) = resolve_target(&repo, target)?;

    // 2. Load the commit
    let target_commit = objects::load_commit(&repo, &target_hash)?;
    let target_files = target_commit.files;

    // 3. Refuse to switch if there is uncommitted work
    ensure_clean_working_tree(&repo)?;

    // 4. Check whether we're already at the requested target
    let current_head = repository::read_head(&repo)?;

    match (&current_head, target_branch.as_deref()) {
        (Head::Symbolic(reference), Some(branch))
            if reference == &format!("refs/heads/{branch}") =>
        {
            println!("Already on branch '{branch}'");
            return Ok(());
        }

        (Head::Detached(current_hash), None)
            if current_hash == &target_hash =>
        {
            println!("Already at commit {}", &target_hash[..7]);
            return Ok(());
        }

        _ => {}
    }

    // 5. Load the current index
    let current_index = index::load(&repo)?;

    // 6. Restore the working directory to the target snapshot
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
    let new_head = match &target_branch {
        Some(branch) => format!("ref: refs/heads/{branch}\n"),
        None => format!("{target_hash}\n"),
    };

    fs::write(minigit_dir.join("HEAD"), new_head)?;

    // 9. Report the result
    match target_branch {
        Some(branch) => println!("Switched to branch '{branch}'"),
        None => println!(
            "HEAD is now at {} (detached)",
            &target_hash[..7]
        ),
    }

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
        .standard_filters(true)
        .hidden(false)
        .require_git(false)
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
            bail!(
                "Cannot checkout: directory '{}' conflicts with a tracked file",
                path
            );
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

fn resolve_target(
    repo: &Path,
    target: &str,
) -> Result<(String, Option<String>)> {
    // Check if target is a valid branch name
    if validate_branch_name(target).is_ok() {
        let branch_path = repo
            .join(".minigit")
            .join("refs")
            .join("heads")
            .join(target);

        if branch_path.is_file() {
            let hash = fs::read_to_string(&branch_path)?;
            let hash = hash.trim().to_string();

            if hash.is_empty() {
                bail!("Branch '{target}' has no commits");
            }

            return Ok((hash, Some(target.to_string())));
        }
    }

    // If it wasn't a branch, check whether it *looks* like a SHA-1 hash.
    if target.len() != 40
        || !target.bytes().all(|byte| byte.is_ascii_hexdigit())
    {
        bail!("'{target}' is not a valid branch name or commit hash");
    }

    // Verify that the commit actually exists and can be loaded.
    objects::load_commit(repo, target)
        .with_context(|| format!("Commit '{target}' not found"))?;

    Ok((target.to_string(), None))
}

#[cfg(test)]
mod tests {
    use super::read_working_tree;
    use std::{fs, time::SystemTime};

    #[test]
    fn read_working_tree_ignores_gitignored_files_outside_git_repositories() {
        let unique = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let repo = std::env::temp_dir().join(format!(
            "minigit-checkout-ignore-{}-{unique}",
            std::process::id()
        ));
        fs::create_dir(&repo).unwrap();
        fs::write(repo.join(".gitignore"), "ignored.txt\n").unwrap();
        fs::write(repo.join("ignored.txt"), "ignored").unwrap();

        let working_tree = read_working_tree(&repo);
        fs::remove_dir_all(&repo).unwrap();

        let working_tree = working_tree.unwrap();
        assert!(working_tree.contains_key(".gitignore"));
        assert!(!working_tree.contains_key("ignored.txt"));
    }
}