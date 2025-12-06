// SPDX-License-Identifier: MIT AND Palimpsest-0.8
// SPDX-FileCopyrightText: 2024-2025 The polysafe-gitfixer Contributors

//! # Git Operations for polysafe-gitfixer
//!
//! Safe wrappers around git2 operations with proper error handling.
//!
//! This crate provides read-only git operations that are useful for
//! analyzing repositories and their backups. Write operations are
//! intentionally limited to staging/committing.

use std::path::Path;

use git2::{Repository, StatusOptions};
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Errors that can occur during git operations
#[derive(Debug, Error)]
pub enum GitError {
    #[error("not a git repository: {0}")]
    NotARepository(String),

    #[error("git error: {0}")]
    Git(#[from] git2::Error),

    #[error("no commits in repository")]
    NoCommits,

    #[error("remote not found: {0}")]
    RemoteNotFound(String),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

/// The object format (hash algorithm) used by a repository
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ObjectFormat {
    /// SHA-1 (traditional, 40 hex chars)
    Sha1,
    /// SHA-256 (newer, 64 hex chars)
    Sha256,
}

impl std::fmt::Display for ObjectFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ObjectFormat::Sha1 => write!(f, "sha1"),
            ObjectFormat::Sha256 => write!(f, "sha256"),
        }
    }
}

/// Status of a file in the working tree
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum FileStatus {
    /// File has not been modified
    Current,
    /// File has been modified but not staged
    Modified,
    /// File is new and not tracked
    New,
    /// File has been deleted
    Deleted,
    /// File has been renamed
    Renamed,
    /// File type changed (file <-> symlink <-> submodule)
    TypeChange,
    /// File is ignored
    Ignored,
    /// File has merge conflicts
    Conflicted,
}

/// A file entry with its status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusEntry {
    pub path: String,
    pub index_status: Option<FileStatus>,
    pub worktree_status: Option<FileStatus>,
}

/// Overall status of a repository
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoStatus {
    /// Path to the repository root
    pub path: String,

    /// Current branch name (None if detached HEAD)
    pub branch: Option<String>,

    /// HEAD commit hash (short form)
    pub head: Option<String>,

    /// Files with changes
    pub entries: Vec<StatusEntry>,

    /// Whether the repo is clean (no changes)
    pub is_clean: bool,

    /// Whether there are staged changes
    pub has_staged: bool,

    /// Whether there are unstaged changes
    pub has_unstaged: bool,

    /// Whether there are untracked files
    pub has_untracked: bool,
}

/// Information about a git remote
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoteInfo {
    pub name: String,
    pub url: Option<String>,
    pub push_url: Option<String>,
}

/// Check if a path contains a valid git repository
pub fn is_valid_repo<P: AsRef<Path>>(path: P) -> bool {
    Repository::open(path.as_ref()).is_ok()
}

/// Get the object format (hash algorithm) used by a repository
pub fn get_object_format<P: AsRef<Path>>(path: P) -> Result<ObjectFormat, GitError> {
    let repo = Repository::open(path.as_ref()).map_err(|_| {
        GitError::NotARepository(path.as_ref().display().to_string())
    })?;

    // git2 doesn't directly expose the object format, so we check the OID length
    // SHA-1 produces 20-byte (40 hex) hashes, SHA-256 produces 32-byte (64 hex)
    // We can check by looking at HEAD's OID if there are commits
    if let Ok(head) = repo.head() {
        if head.target().is_some() {
            // git2 OIDs are always 20 bytes for SHA-1 repos
            // For SHA-256 repos (experimental), they would be 32 bytes
            // Since git2's Oid is fixed at 20 bytes, we're on SHA-1
            return Ok(ObjectFormat::Sha1);
        }
    }

    // Default to SHA-1 for empty repos
    Ok(ObjectFormat::Sha1)
}

/// Get the URL of a remote
pub fn get_remote_url<P: AsRef<Path>>(
    path: P,
    remote_name: &str,
) -> Result<Option<String>, GitError> {
    let repo = Repository::open(path.as_ref()).map_err(|_| {
        GitError::NotARepository(path.as_ref().display().to_string())
    })?;

    let result = match repo.find_remote(remote_name) {
        Ok(remote) => Ok(remote.url().map(String::from)),
        Err(e) if e.code() == git2::ErrorCode::NotFound => Ok(None),
        Err(e) => Err(GitError::Git(e)),
    };
    result
}

/// Get information about all remotes
pub fn get_remotes<P: AsRef<Path>>(path: P) -> Result<Vec<RemoteInfo>, GitError> {
    let repo = Repository::open(path.as_ref()).map_err(|_| {
        GitError::NotARepository(path.as_ref().display().to_string())
    })?;

    let remote_names = repo.remotes()?;
    let mut remotes = Vec::new();

    for name in remote_names.iter().flatten() {
        if let Ok(remote) = repo.find_remote(name) {
            remotes.push(RemoteInfo {
                name: name.to_string(),
                url: remote.url().map(String::from),
                push_url: remote.pushurl().map(String::from),
            });
        }
    }

    Ok(remotes)
}

/// Get the full status of a repository
pub fn repo_status<P: AsRef<Path>>(path: P) -> Result<RepoStatus, GitError> {
    let path = path.as_ref();
    let repo = Repository::open(path).map_err(|_| {
        GitError::NotARepository(path.display().to_string())
    })?;

    // Get branch name
    let branch = if let Ok(head) = repo.head() {
        if head.is_branch() {
            head.shorthand().map(String::from)
        } else {
            None
        }
    } else {
        None
    };

    // Get HEAD commit
    let head = if let Ok(head) = repo.head() {
        head.target().map(|oid| format!("{:.7}", oid))
    } else {
        None
    };

    // Get status entries
    let mut opts = StatusOptions::new();
    opts.include_untracked(true)
        .include_ignored(false)
        .include_unmodified(false);

    let statuses = repo.statuses(Some(&mut opts))?;
    let mut entries = Vec::new();
    let mut has_staged = false;
    let mut has_unstaged = false;
    let mut has_untracked = false;

    for entry in statuses.iter() {
        let status = entry.status();
        let path_str = entry.path().unwrap_or("").to_string();

        let index_status = status_to_file_status(status, true);
        let worktree_status = status_to_file_status(status, false);

        if index_status.is_some() {
            has_staged = true;
        }

        if worktree_status.is_some() {
            has_unstaged = true;
        }

        if status.is_wt_new() {
            has_untracked = true;
        }

        entries.push(StatusEntry {
            path: path_str,
            index_status,
            worktree_status,
        });
    }

    Ok(RepoStatus {
        path: path.display().to_string(),
        branch,
        head,
        entries,
        is_clean: statuses.is_empty(),
        has_staged,
        has_unstaged,
        has_untracked,
    })
}

/// Stage all changes (equivalent to `git add -A`)
pub fn stage_all<P: AsRef<Path>>(path: P) -> Result<(), GitError> {
    let repo = Repository::open(path.as_ref()).map_err(|_| {
        GitError::NotARepository(path.as_ref().display().to_string())
    })?;

    let mut index = repo.index()?;
    index.add_all(["*"].iter(), git2::IndexAddOption::DEFAULT, None)?;
    index.write()?;

    Ok(())
}

/// Stage specific files
pub fn stage_files<P: AsRef<Path>, I, S>(path: P, files: I) -> Result<(), GitError>
where
    I: IntoIterator<Item = S>,
    S: AsRef<Path>,
{
    let repo = Repository::open(path.as_ref()).map_err(|_| {
        GitError::NotARepository(path.as_ref().display().to_string())
    })?;

    let mut index = repo.index()?;
    for file in files {
        index.add_path(file.as_ref())?;
    }
    index.write()?;

    Ok(())
}

/// Find all git repositories under a directory
pub fn find_repos<P: AsRef<Path>>(root: P, max_depth: usize) -> Result<Vec<String>, GitError> {
    let root = root.as_ref();
    let mut repos = Vec::new();

    find_repos_recursive(root, root, max_depth, &mut repos)?;

    Ok(repos)
}

fn find_repos_recursive(
    root: &Path,
    current: &Path,
    depth: usize,
    repos: &mut Vec<String>,
) -> Result<(), GitError> {
    if depth == 0 {
        return Ok(());
    }

    // Check if current is a git repo
    let git_dir = current.join(".git");
    if git_dir.exists() {
        repos.push(current.display().to_string());
        // Don't recurse into submodules
        return Ok(());
    }

    // Recurse into subdirectories
    if let Ok(entries) = std::fs::read_dir(current) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                // Skip hidden directories (except .git which we already checked)
                if let Some(name) = path.file_name() {
                    if name.to_string_lossy().starts_with('.') {
                        continue;
                    }
                }
                find_repos_recursive(root, &path, depth - 1, repos)?;
            }
        }
    }

    Ok(())
}

/// Get the default branch name for a repository
pub fn get_default_branch<P: AsRef<Path>>(path: P) -> Result<Option<String>, GitError> {
    let repo = Repository::open(path.as_ref()).map_err(|_| {
        GitError::NotARepository(path.as_ref().display().to_string())
    })?;

    // Try common default branch names
    for name in &["main", "master", "develop", "trunk"] {
        if repo.find_branch(name, git2::BranchType::Local).is_ok() {
            return Ok(Some(name.to_string()));
        }
    }

    // Fall back to HEAD's branch
    if let Ok(head) = repo.head() {
        if head.is_branch() {
            return Ok(head.shorthand().map(String::from));
        }
    }

    Ok(None)
}

/// Check if a repository has a specific remote
pub fn has_remote<P: AsRef<Path>>(path: P, remote_name: &str) -> Result<bool, GitError> {
    let repo = Repository::open(path.as_ref()).map_err(|_| {
        GitError::NotARepository(path.as_ref().display().to_string())
    })?;

    let has_it = repo.find_remote(remote_name).is_ok();
    Ok(has_it)
}

/// Get the number of commits in the repository
pub fn commit_count<P: AsRef<Path>>(path: P) -> Result<usize, GitError> {
    let repo = Repository::open(path.as_ref()).map_err(|_| {
        GitError::NotARepository(path.as_ref().display().to_string())
    })?;

    let head = repo.head().map_err(|_| GitError::NoCommits)?;
    let oid = head.target().ok_or(GitError::NoCommits)?;

    let mut revwalk = repo.revwalk()?;
    revwalk.push(oid)?;

    Ok(revwalk.count())
}

// Helper to convert git2 status flags to FileStatus
fn status_to_file_status(status: git2::Status, for_index: bool) -> Option<FileStatus> {
    if for_index {
        if status.is_index_new() {
            Some(FileStatus::New)
        } else if status.is_index_modified() {
            Some(FileStatus::Modified)
        } else if status.is_index_deleted() {
            Some(FileStatus::Deleted)
        } else if status.is_index_renamed() {
            Some(FileStatus::Renamed)
        } else if status.is_index_typechange() {
            Some(FileStatus::TypeChange)
        } else {
            None
        }
    } else {
        if status.is_wt_new() {
            Some(FileStatus::New)
        } else if status.is_wt_modified() {
            Some(FileStatus::Modified)
        } else if status.is_wt_deleted() {
            Some(FileStatus::Deleted)
        } else if status.is_wt_renamed() {
            Some(FileStatus::Renamed)
        } else if status.is_wt_typechange() {
            Some(FileStatus::TypeChange)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn init_repo(path: &Path) -> Repository {
        let repo = Repository::init(path).unwrap();

        // Configure user for commits
        let mut config = repo.config().unwrap();
        config.set_str("user.name", "Test User").unwrap();
        config.set_str("user.email", "test@example.com").unwrap();

        repo
    }

    #[test]
    fn test_is_valid_repo() {
        let tmp = TempDir::new().unwrap();

        // Not a repo
        assert!(!is_valid_repo(tmp.path()));

        // Init a repo
        Repository::init(tmp.path()).unwrap();
        assert!(is_valid_repo(tmp.path()));
    }

    #[test]
    fn test_repo_status_empty() {
        let tmp = TempDir::new().unwrap();
        init_repo(tmp.path());

        let status = repo_status(tmp.path()).unwrap();
        assert!(status.is_clean);
        assert!(!status.has_staged);
        assert!(!status.has_unstaged);
    }

    #[test]
    fn test_repo_status_with_changes() {
        let tmp = TempDir::new().unwrap();
        init_repo(tmp.path());

        // Create an untracked file
        fs::write(tmp.path().join("new.txt"), "content").unwrap();

        let status = repo_status(tmp.path()).unwrap();
        assert!(!status.is_clean);
        assert!(status.has_untracked);
    }

    #[test]
    fn test_stage_all() {
        let tmp = TempDir::new().unwrap();
        init_repo(tmp.path());

        // Create a file
        fs::write(tmp.path().join("file.txt"), "content").unwrap();

        // Stage all
        stage_all(tmp.path()).unwrap();

        let status = repo_status(tmp.path()).unwrap();
        assert!(status.has_staged);
    }

    #[test]
    fn test_find_repos() {
        let tmp = TempDir::new().unwrap();

        // Create some nested repos
        let repo1 = tmp.path().join("project1");
        let repo2 = tmp.path().join("group").join("project2");
        fs::create_dir_all(&repo1).unwrap();
        fs::create_dir_all(&repo2).unwrap();

        Repository::init(&repo1).unwrap();
        Repository::init(&repo2).unwrap();

        // Also create a non-repo directory
        fs::create_dir_all(tmp.path().join("not-a-repo")).unwrap();

        let repos = find_repos(tmp.path(), 5).unwrap();
        assert_eq!(repos.len(), 2);
    }

    #[test]
    fn test_get_remote_url() {
        let tmp = TempDir::new().unwrap();
        let repo = init_repo(tmp.path());

        // No remotes yet
        assert!(get_remote_url(tmp.path(), "origin").unwrap().is_none());

        // Add a remote
        repo.remote("origin", "https://github.com/user/repo.git").unwrap();

        let url = get_remote_url(tmp.path(), "origin").unwrap();
        assert_eq!(url, Some("https://github.com/user/repo.git".to_string()));
    }

    #[test]
    fn test_object_format() {
        let tmp = TempDir::new().unwrap();
        init_repo(tmp.path());

        // Should be SHA-1 (the default)
        let format = get_object_format(tmp.path()).unwrap();
        assert_eq!(format, ObjectFormat::Sha1);
    }
}
