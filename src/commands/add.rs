use std::{fs, path::Path};

use crate::{index, objects};

pub fn run(path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    if path.is_absolute() || path.starts_with("..") {
        return Err("Please provide a relative file path".into());
    }

    if !path.is_file() {
        return Err(format!("File not found: {}", path.display()).into());
    }

    let repo = std::env::current_dir()?;
    let contents = fs::read(path)?;

    let hash = objects::write_blob(&repo, &contents)?;

    let mut staging_area = index::load(&repo)?;
    staging_area.entries.insert(
        path.to_string_lossy().to_string(),
        hash,
    );

    index::save(&repo, &staging_area)?;

    println!("Added {}", path.display());

    Ok(())
}