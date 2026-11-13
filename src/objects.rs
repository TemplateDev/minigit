use sha1::{Sha1, Digest};
use std::fs;
use std::path::Path;

pub fn write_blob(repo: &Path, contents: &[u8]) -> std::io::Result<String> {
    let hash = Sha1::digest(contents)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<Vec<_>>()
        .join("");

    let object_path = repo
        .join(".minigit")
        .join("objects")
        .join(&hash);

    if !object_path.exists() {
        fs::write(object_path, contents)?;
    }

    Ok(hash)
}
