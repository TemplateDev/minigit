use sha1::{Sha1, Digest};
use std::fs;
use std::path::Path;
use crate::models::commit::Commit;

pub fn to_hash(contents: &[u8]) -> std::io::Result<String> {
    let hash = Sha1::digest(contents)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<Vec<_>>()
        .join("");
    Ok(hash)
}

pub fn write_blob(repo: &Path, contents: &[u8]) -> std::io::Result<String> {
    let hash = to_hash(contents)?;

    let object_path = repo
        .join(".minigit")
        .join("objects")
        .join(&hash);

    if !object_path.exists() {
        fs::write(object_path, contents)?;
    }

    Ok(hash)
}

// Load a commit using its hash.
pub fn load_commit(
    repo: &Path,
    hash: &str,
) -> std::io::Result<Commit> {
    let object_path = repo
        .join(".minigit")
        .join("objects")
        .join(hash);

    let contents = fs::read(object_path)?;

    serde_json::from_slice(&contents)
        .map_err(std::io::Error::other)
}
