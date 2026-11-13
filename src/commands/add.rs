use std::{fs};

use globset::{Glob, GlobSetBuilder};
use ignore::WalkBuilder;

use crate::{index, objects};

pub fn run(patterns: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    let repo = std::env::current_dir()?;

    // Patterns into a matcher.
    let mut builder = GlobSetBuilder::new();

    for pattern in patterns {
        let pattern = if pattern == "." {
            "**/*"
        } else {
            pattern.as_str()
        };

        builder.add(Glob::new(pattern)?);
    }

    let matcher = builder.build()?;

    // Walk the repository, respecting .gitignore.
    let walker = WalkBuilder::new(&repo)
        .hidden(false)
        .git_ignore(true)
        .git_exclude(false)
        .parents(false)
        .ignore(false)
        .build();

    let mut staging_area = index::load(&repo)?;
    let mut added = 0;

    for entry in walker {
        let entry = entry?;

        // Ignore MiniGit's internal files.
        if entry.depth() == 0 || entry.path().starts_with(repo.join(".minigit")) {
            continue;
        }

        // Ignore Git's internal files.
        if entry.depth() == 0 || entry.path().starts_with(repo.join(".git")) {
            continue;
        }

        if !entry.file_type().is_some_and(|kind| kind.is_file()) {
            continue;
        }

        let relative_path = entry.path().strip_prefix(&repo)?;

        if !matcher.is_match(relative_path) {
            continue;
        }

        let contents = fs::read(entry.path())?;
        let hash = objects::write_blob(&repo, &contents)?;

        let path_string = relative_path.to_string_lossy()
            .replace('\\', "/");

        staging_area.entries.insert(path_string.clone(), hash);

        println!("Added {path_string}");
        added += 1;
    }

    if added == 0 {
        println!("No matching files found.");
    }

    index::save(&repo, &staging_area)?;

    Ok(())
}