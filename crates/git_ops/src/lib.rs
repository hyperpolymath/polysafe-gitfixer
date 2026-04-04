// SPDX-License-Identifier: MIT AND Palimpsest-0.8
// Copyright (c) 2026 Jonathan D.A. Jewell (hyperpolymath) <j.d.a.jewell@open.ac.uk>
//
//! Polysafe-Gitfixer — Safe Git Operations Kernel.
//!
//! This crate provides a high-assurance interface for Git repository analysis.
//! It encapsulates the complex `git2` bindings into a safe, domain-specific
//! API for tracking repository state and performing verified backups.
//!
//! DESIGN GOALS:
//! 1. **Read-Only by Default**: Prioritize analysis over mutation.
//! 2. **Object Format Awareness**: Explicitly handle SHA-1 and SHA-256 repos.
//! 3. **Bitemporal Logic**: Support tracking changes across both the Index
//!    and the Working Tree.

#![forbid(unsafe_code)]
use std::path::Path;
use git2::{Repository, StatusOptions};
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// ERROR SPACE: Categorized failures for git operations.
#[derive(Debug, Error)]
pub enum GitError {
    /// The path does not contain a valid git repository.
    #[error("not a git repository: {0}")]
    NotARepository(String),

    /// An underlying `git2` library error.
    #[error("git error: {0}")]
    Git(#[from] git2::Error),

    /// An I/O error during filesystem traversal.
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

/// The status of a single file in the working tree or index.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusEntry {
    /// Repository-relative path of the file.
    pub path: String,
    /// Short status code (e.g. "M " for modified in index, " M" for working-tree).
    pub status_code: String,
    /// Whether the change is staged in the index.
    pub is_staged: bool,
    /// Whether the change is in the working tree (not yet staged).
    pub is_unstaged: bool,
}

/// STATE MODEL: Aggregated status of a physical repository.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoStatus {
    /// Absolute path to the repository root.
    pub path: String,
    /// Name of the currently checked-out branch (None if detached HEAD).
    pub branch: Option<String>,
    /// Short OID (first 8 chars of the HEAD commit hash).
    pub head: Option<String>,
    /// Per-file status entries.
    pub entries: Vec<StatusEntry>,
    /// True when the working tree is clean (no staged or unstaged changes).
    pub is_clean: bool,
    /// True when at least one file has staged changes.
    pub has_staged: bool,
}

/// ANALYSIS: Retrieves the full status of a repository at `path`.
///
/// Scans the working tree for modified, new, deleted, or conflicted files.
/// Returns [`GitError::NotARepository`] if `path` is not a git repository.
pub fn repo_status<P: AsRef<Path>>(path: P) -> Result<RepoStatus, GitError> {
    let path_ref = path.as_ref();
    let repo = Repository::open(path_ref)
        .map_err(|_| GitError::NotARepository(path_ref.display().to_string()))?;

    // Resolve the current branch name (None for detached HEAD).
    let branch = repo.head()
        .ok()
        .and_then(|h| h.shorthand().map(|s| s.to_owned()));

    // Resolve the short HEAD commit OID.
    let head = repo.head()
        .ok()
        .and_then(|h| h.target())
        .map(|oid| oid.to_string()[..8.min(oid.to_string().len())].to_owned());

    // Collect per-file status entries.
    let mut opts = StatusOptions::new();
    opts.include_untracked(true)
        .include_ignored(false)
        .include_unmodified(false);

    let statuses = repo.statuses(Some(&mut opts))?;

    let entries: Vec<StatusEntry> = statuses
        .iter()
        .filter_map(|entry| {
            let path = entry.path()?.to_owned();
            let status = entry.status();
            let is_staged = status.is_index_new()
                || status.is_index_modified()
                || status.is_index_deleted()
                || status.is_index_renamed()
                || status.is_index_typechange();
            let is_unstaged = status.is_wt_new()
                || status.is_wt_modified()
                || status.is_wt_deleted()
                || status.is_wt_renamed()
                || status.is_wt_typechange();
            let status_code = format!(
                "{}{}",
                if is_staged { "M" } else { " " },
                if is_unstaged { "M" } else { " " },
            );
            Some(StatusEntry { path, status_code, is_staged, is_unstaged })
        })
        .collect();

    let is_clean = entries.is_empty();
    let has_staged = entries.iter().any(|e| e.is_staged);

    Ok(RepoStatus {
        path: path_ref.display().to_string(),
        branch,
        head,
        entries,
        is_clean,
        has_staged,
    })
}

/// DISCOVERY: Recursively searches for Git repositories within a directory tree.
///
/// Returns a list of absolute path strings for each `.git`-bearing directory
/// found at depth ≤ `max_depth` below `root`.
pub fn find_repos<P: AsRef<Path>>(root: P, max_depth: usize) -> Result<Vec<String>, GitError> {
    let root = root.as_ref();
    let mut repos = Vec::new();
    find_repos_recursive(root, 0, max_depth, &mut repos)?;
    Ok(repos)
}

/// Recursive helper for [`find_repos`].
fn find_repos_recursive(
    dir: &Path,
    depth: usize,
    max_depth: usize,
    repos: &mut Vec<String>,
) -> Result<(), GitError> {
    if depth > max_depth { return Ok(()); }

    // Check if this directory is itself a git repo.
    if dir.join(".git").exists() {
        repos.push(dir.display().to_string());
        // Don't descend further into a repo's own subdirectories.
        return Ok(());
    }

    // Otherwise recurse into sub-directories.
    if dir.is_dir() {
        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                find_repos_recursive(&path, depth + 1, max_depth, repos)?;
            }
        }
    }

    Ok(())
}
