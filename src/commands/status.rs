use std::{
    collections::{BTreeMap, BTreeSet},
    fs
};

use ignore::WalkBuilder;

use crate::{index, objects, repository};

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    let repo = std::env::current_dir()?;

    // Load the three states.
    let head = repository::load_head(&repo)?;
    let index = index::load(&repo)?;

    let head_files = head
        .map(|commit| commit.files)
        .unwrap_or_default();

    let index_files = index.entries;

    // Scan the working directory without writing any objects.
    let mut working_files = BTreeMap::new();

    let walker = WalkBuilder::new(&repo)
        .hidden(false)
        .git_ignore(true)
        .git_exclude(false)
        .parents(false)
        .ignore(false)
        .build();

    for entry in walker {
        let entry = entry?;

        if entry.depth() == 0 {
            continue;
        }

        let path = entry.path();

        // No MiniGit's or Git's internal directories.
        if path.starts_with(repo.join(".minigit"))
            || path.starts_with(repo.join(".git"))
        {
            continue;
        }

        if !entry.file_type().is_some_and(|kind| kind.is_file()) {
            continue;
        }

        let relative_path = path.strip_prefix(&repo)?;

        let path_string = relative_path
            .to_string_lossy()
            .replace('\\', "/");

        let contents = fs::read(path)?;
        let hash = objects::to_hash(&contents)?;

        working_files.insert(path_string, hash);
    }

    // Compare HEAD against index.
    let mut staged = Vec::new();

    let mut paths = BTreeSet::new();
    paths.extend(head_files.keys().cloned());
    paths.extend(index_files.keys().cloned());

    for path in paths {
        let old = head_files.get(&path);
        let new = index_files.get(&path);

        if old == new {
            continue;
        }

        let description = match (old, new) {
            (None, Some(_)) => "new file",
            (Some(_), None) => "deleted",
            (Some(_), Some(_)) => "modified",
            (None, None) => continue,
        };

        staged.push((description, path));
    }

    // Compare index against working directory.
    let mut unstaged = Vec::new();

    for (path, index_hash) in &index_files {
        match working_files.get(path) {
            None => unstaged.push(("deleted", path.clone())),
            Some(working_hash) if working_hash != index_hash => {
                unstaged.push(("modified", path.clone()));
            }
            _ => {}
        }
    }

    // Files on disk that aren't in the index are untracked.
    let untracked: Vec<_> = working_files
        .keys()
        .filter(|path| !index_files.contains_key(*path))
        .cloned()
        .collect();

    // Display the results.
    if staged.is_empty() && unstaged.is_empty() && untracked.is_empty() {
        println!("Nothing to commit, working tree clean.");
        return Ok(());
    }

    if !staged.is_empty() {
        println!("Changes to be committed:");

        for (description, path) in staged {
            println!("  {description}: {path}");
        }

        println!();
    }

    if !unstaged.is_empty() {
        println!("Changes not staged for commit:");

        for (description, path) in unstaged {
            println!("  {description}: {path}");
        }

        println!();
    }

    if !untracked.is_empty() {
        println!("Untracked files:");

        for path in untracked {
            println!("  {path}");
        }

        println!();
    }

    Ok(())
}