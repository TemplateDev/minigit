
use std::fs;
use std::io;

pub fn init() -> io::Result<()> {
    let current_dir = std::env::current_dir()?;
    let minigit_dir = current_dir.join(".minigit");

    if minigit_dir.exists() {
        println!("MiniGit repository already exists.");
        return Ok(());
    }

    // Create object storage.
    fs::create_dir_all(minigit_dir.join("objects"))?;

    // Create branch references directory.
    fs::create_dir_all(minigit_dir.join("refs").join("heads"))?;

    // Point HEAD to the default branch.
    fs::write(
        minigit_dir.join("HEAD"),
        "ref: refs/heads/main\n",
    )?;

    // Initialize an empty staging area.
    fs::write(
        minigit_dir.join("index"),
        "{}\n",
    )?;

    println!("Initialized empty MiniGit repository.");

    Ok(())
}
