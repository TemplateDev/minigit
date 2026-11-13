use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, fs, path::Path};

#[derive(Serialize, Deserialize, Default)]
pub struct Index {
    #[serde(default)]
    pub entries: BTreeMap<String, String>,
}

pub fn load(repo: &Path) -> std::io::Result<Index> {
    let path = repo.join(".minigit").join("index");

    if !path.exists() {
        return Ok(Index::default());
    }

    let contents = fs::read_to_string(path)?;

    serde_json::from_str(&contents).map_err(std::io::Error::other)
}

pub fn save(repo: &Path, index: &Index) -> std::io::Result<()> {
    let path = repo.join(".minigit").join("index");
    let contents = serde_json::to_string_pretty(index).map_err(std::io::Error::other)?;

    fs::write(path, contents)
}
