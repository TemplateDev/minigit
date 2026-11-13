use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Serialize, Deserialize)]
pub struct Commit {
    pub message: String,
    pub parent: Option<String>,
    pub files: BTreeMap<String, String>,
    pub timestamp: u64,
}