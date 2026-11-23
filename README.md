# MiniGit

> [!WARNING]
> This project is archived. It was created as a learning exercise and is not actively maintained.

MiniGit is a small Git-inspired version control implementation written in Rust. It was built to explore the concepts behind repositories, staged files, commits, branches, and checkouts.

This is an educational project, not a replacement for Git and not intended for production use.

## Implemented commands

| Command | Description |
| --- | --- |
| `minigit init` | Initialize a `.minigit` repository in the current directory |
| `minigit add <patterns...>` | Add matching files to the staging area |
| `minigit status` | Show staged, unstaged, and untracked changes |
| `minigit commit --message <message>` | Create a commit from the staged files |
| `minigit log` | Display commits from the current HEAD |
| `minigit branch` | List branches |
| `minigit branch create <name>` | Create a branch at the current commit |
| `minigit branch rename <old> <new>` | Rename a branch |
| `minigit branch delete <name>` | Delete a branch |
| `minigit checkout <branch-or-commit>` | Switch branches or check out a commit in detached HEAD mode |

## Build and run

Requires Rust and Cargo.

```bash
cargo build
```

Run the executable directly:

```bash
cargo run -- init
```

Or build and use the binary:

```bash
cargo build --release
.\target\release\minigit.exe init
```

## Example

From a directory you want to track:

```powershell
minigit.exe init
minigit.exe add .
minigit.exe status
minigit.exe commit --message "Initial commit"
minigit.exe log
```

Create and switch branches:

```powershell
minigit.exe branch create feature
minigit.exe checkout feature
```

Rename or delete branches:

```powershell
minigit.exe branch rename feature work
minigit.exe branch delete work
```

To inspect an earlier commit, pass its 40-character SHA-1 hash to `checkout`:

```powershell
minigit.exe checkout <full-commit-hash>
```

## How it works

MiniGit stores repository data in `.minigit`:

- `HEAD` points to a branch or contains a detached commit hash
- `index` stores the staged file paths and their blob hashes
- `objects/` stores SHA-1-addressed file blobs and JSON commit objects
- `refs/heads/` stores branch commit hashes

A commit records a message, timestamp, parent commit hash, and a snapshot of staged file paths and blob hashes.

## Behavior and limitations

- Checkout refuses to run if staged changes, working-tree modifications, or untracked files are present.
- Checkout restores the working tree and staging index before updating `HEAD`.
- Branch names cannot contain `/`, `..`, whitespace, backslashes, or start with `-`.
- Branches require at least one existing commit.
- The current branch cannot be deleted.
- `add` supports glob patterns and respects `.gitignore`-style ignore rules.
- Commit history follows parent links and supports a linear history from the current HEAD.
- There are no merge, rebase, remote, push, pull, clone, stash, or reset commands.
- Repository data is not compatible with Git repositories.

## Project structure

```text
src/
  main.rs          CLI definitions and command dispatch
  repository.rs    Repository initialization and HEAD resolution
  index.rs         Staging-area loading and saving
  objects.rs       Blob and commit object storage
  models/          Commit and HEAD data structures
  commands/        Init, add, status, commit, log, branch, and checkout logic
```
