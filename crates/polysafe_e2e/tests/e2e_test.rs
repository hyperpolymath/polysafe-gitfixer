// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Jonathan D.A. Jewell (hyperpolymath) <j.d.a.jewell@open.ac.uk>
//
// E2E tests for polysafe-gitfixer — exercises combined fs_ops + git_ops
// workflows that mirror real-world usage patterns.
//
// Each test creates an isolated temporary environment so there is no
// inter-test state contamination.

use std::fs;
use std::process::Command;
use capability::{DirCapability, Permissions};
use fs_ops::FsTransaction;
use git_ops::{find_repos, repo_status};

// ─── Helpers ────────────────────────────────────────────────────────────────

fn scratch() -> tempfile::TempDir {
    tempfile::tempdir().expect("create temp dir")
}

/// Initialise a git repo via the system git binary.
fn init_git_repo(path: &std::path::Path) {
    Command::new("git")
        .args(["init", "--quiet", path.to_str().unwrap()])
        .status()
        .expect("git init")
        .success()
        .then_some(())
        .expect("git init must succeed");

    for (k, v) in [("user.email", "test@test.local"), ("user.name", "Test")] {
        Command::new("git")
            .args(["-C", path.to_str().unwrap(), "config", k, v])
            .status()
            .expect("git config");
    }
}

// ─── E2E: capability + fs_ops ────────────────────────────────────────────────

/// Full flow: create a capability, use a transaction to write a file inside
/// the sandbox, then verify the file exists at the resolved path.
#[test]
fn e2e_capability_guards_transactional_write() {
    let tmp = scratch();
    let cap = DirCapability::new(tmp.path(), Permissions::all())
        .expect("create capability");

    // Stage a write via the transaction.
    let target = tmp.path().join("output.txt");
    let mut tx = FsTransaction::new();
    tx.write_file(target.clone(), b"hello polysafe".to_vec())
        .expect("enqueue write");
    tx.commit().expect("commit transaction");

    // The file must now exist and contain the expected content.
    let content = fs::read_to_string(&target).expect("read written file");
    assert_eq!(content, "hello polysafe");

    // Verify that the target is within the capability sandbox.
    let resolved = cap.resolve(std::path::Path::new("output.txt"))
        .expect("capability resolve after write");
    assert_eq!(resolved, target.canonicalize().unwrap());
}

/// Capability rejects writes that attempt to escape the sandbox.
#[test]
fn e2e_capability_blocks_path_traversal() {
    let tmp = scratch();
    let sub = tmp.path().join("sandbox");
    fs::create_dir(&sub).expect("create sandbox subdir");
    let cap = DirCapability::new(&sub, Permissions::all())
        .expect("create capability");

    // Attempt to escape the sandbox via `../`.
    let result = cap.resolve(std::path::Path::new("../escape.txt"));
    assert!(result.is_err(), "path traversal must be blocked");
}

/// Transaction rollback: a dropped-without-commit transaction leaves no files.
#[test]
fn e2e_transaction_rollback_leaves_no_files() {
    let tmp = scratch();
    let target = tmp.path().join("should_not_exist.txt");

    {
        let mut tx = FsTransaction::new();
        tx.write_file(target.clone(), b"ghost".to_vec())
            .expect("enqueue write");
        // tx dropped here without commit → RAII rollback
    }

    // File must not have been committed.
    assert!(!target.exists(), "rolled-back write must not persist");
}

// ─── E2E: git_ops discovery + status ────────────────────────────────────────

/// Full flow: create a repo, write a file, then verify status reflects it.
#[test]
fn e2e_git_status_after_adding_file() {
    let tmp = scratch();
    let repo_dir = tmp.path().join("repo");
    fs::create_dir(&repo_dir).expect("create repo dir");
    init_git_repo(&repo_dir);

    // Add an untracked file.
    fs::write(repo_dir.join("new_file.txt"), b"content").expect("write file");

    let status = repo_status(&repo_dir).expect("repo_status");
    assert!(!status.entries.is_empty(), "untracked file must appear in status");
    assert!(!status.is_clean, "repo must not be clean after adding a file");
}

/// find_repos must discover nested repos up to the specified depth.
#[test]
fn e2e_find_repos_discovers_nested_structure() {
    let tmp = scratch();
    // Create two repos at different nesting depths.
    let repo_a = tmp.path().join("a");
    let repo_b = tmp.path().join("projects").join("b");
    fs::create_dir_all(&repo_a).expect("create repo_a");
    fs::create_dir_all(&repo_b).expect("create repo_b");
    init_git_repo(&repo_a);
    init_git_repo(&repo_b);

    let found = find_repos(tmp.path(), 3).expect("find_repos");
    assert_eq!(found.len(), 2, "should discover exactly 2 repos; got: {:?}", found);
}

/// Combining find_repos + repo_status: discovered repos must all return valid status.
#[test]
fn e2e_find_repos_then_status_all_valid() {
    let tmp = scratch();
    for name in ["alpha", "beta"] {
        let dir = tmp.path().join(name);
        fs::create_dir(&dir).expect("create dir");
        init_git_repo(&dir);
    }

    let repos = find_repos(tmp.path(), 2).expect("find_repos");
    assert_eq!(repos.len(), 2);

    for repo_path in &repos {
        let status = repo_status(repo_path).expect("repo_status on discovered repo");
        assert_eq!(&status.path, repo_path);
    }
}
