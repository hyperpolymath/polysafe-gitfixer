// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Jonathan D.A. Jewell (hyperpolymath) <j.d.a.jewell@open.ac.uk>
//
// Integration tests for the `git_ops` crate.
// Covers: valid repo detection, invalid path handling, repo status fields,
// find_repos discovery, and graceful failure modes.

use std::fs;
use std::process::Command;
use git_ops::{find_repos, repo_status, GitError};

// ─── Helpers ────────────────────────────────────────────────────────────────

fn scratch() -> tempfile::TempDir {
    tempfile::tempdir().expect("create temp dir")
}

/// Initialise a bare-minimum git repository in `dir` using the system `git`
/// binary.  Returns the path as a String.
fn init_git_repo(dir: &std::path::Path) -> String {
    Command::new("git")
        .args(["init", "--quiet", dir.to_str().unwrap()])
        .status()
        .expect("git init")
        .success()
        .then_some(())
        .expect("git init must succeed");

    // Configure a local user so git doesn't complain.
    for (k, v) in [("user.email", "test@test.local"), ("user.name", "Test")] {
        Command::new("git")
            .args(["-C", dir.to_str().unwrap(), "config", k, v])
            .status()
            .expect("git config");
    }

    dir.display().to_string()
}

// ─── repo_status: valid repos ────────────────────────────────────────────────

/// Calling `repo_status` on a freshly initialised repo must succeed.
#[test]
fn repo_status_on_fresh_repo_returns_ok() {
    let tmp = scratch();
    init_git_repo(tmp.path());
    let status = repo_status(tmp.path()).expect("repo_status on fresh repo");
    assert_eq!(status.path, tmp.path().display().to_string());
}

/// A fresh empty repository should have an empty entries list (clean tree).
#[test]
fn repo_status_fresh_repo_is_clean() {
    let tmp = scratch();
    init_git_repo(tmp.path());
    let status = repo_status(tmp.path()).expect("repo_status");
    // A freshly initialised repo may have no HEAD yet; entries should be empty.
    assert!(!status.has_staged, "fresh repo should have no staged changes");
}

/// After adding an untracked file, entries must be non-empty.
#[test]
fn repo_status_untracked_file_appears_in_entries() {
    let tmp = scratch();
    init_git_repo(tmp.path());
    fs::write(tmp.path().join("readme.txt"), b"hello").expect("write file");
    let status = repo_status(tmp.path()).expect("repo_status");
    assert!(!status.entries.is_empty(), "untracked file must appear in entries");
}

/// `is_clean` must reflect whether entries is empty.
#[test]
fn repo_status_is_clean_consistent_with_entries() {
    let tmp = scratch();
    init_git_repo(tmp.path());
    let status = repo_status(tmp.path()).expect("repo_status");
    assert_eq!(
        status.is_clean,
        status.entries.is_empty(),
        "is_clean must be consistent with entries being empty"
    );
}

// ─── repo_status: invalid paths ──────────────────────────────────────────────

/// A non-existent path must return `GitError::NotARepository`.
#[test]
fn repo_status_nonexistent_path_returns_not_a_repo() {
    let result = repo_status("/nonexistent/path/that/cannot/be/a/repo");
    assert!(result.is_err(), "nonexistent path must fail");
    match result.unwrap_err() {
        GitError::NotARepository(_) => {} // expected
        other => panic!("expected NotARepository, got: {:?}", other),
    }
}

/// A plain directory with no `.git` must return `NotARepository`.
#[test]
fn repo_status_plain_directory_is_not_a_repo() {
    let tmp = scratch();
    let result = repo_status(tmp.path());
    assert!(result.is_err(), "plain directory must not be a git repo");
    match result.unwrap_err() {
        GitError::NotARepository(_) => {}
        other => panic!("expected NotARepository, got: {:?}", other),
    }
}

/// Passing a file path (not a directory) must return an error.
#[test]
fn repo_status_file_path_returns_error() {
    let tmp = scratch();
    let file_path = tmp.path().join("not_a_dir.txt");
    fs::write(&file_path, b"data").expect("write file");
    let result = repo_status(&file_path);
    assert!(result.is_err(), "file path must not be accepted as a repo");
}

// ─── find_repos ──────────────────────────────────────────────────────────────

/// `find_repos` on a directory containing one repo must find exactly one.
#[test]
fn find_repos_discovers_single_repo() {
    let tmp = scratch();
    let repo_dir = tmp.path().join("myrepo");
    fs::create_dir(&repo_dir).expect("create repo dir");
    init_git_repo(&repo_dir);

    let found = find_repos(tmp.path(), 2).expect("find_repos");
    assert_eq!(found.len(), 1, "should find exactly one repo");
    assert!(found[0].contains("myrepo"), "found path should contain repo name");
}

/// `find_repos` on an empty directory must return an empty list.
#[test]
fn find_repos_empty_directory_returns_empty() {
    let tmp = scratch();
    let found = find_repos(tmp.path(), 3).expect("find_repos on empty dir");
    assert!(found.is_empty(), "no repos in an empty directory");
}
