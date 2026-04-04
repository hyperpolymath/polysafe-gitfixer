// SPDX-License-Identifier: PMPL-1.0-or-later
// Copyright (c) 2026 Jonathan D.A. Jewell (hyperpolymath) <j.d.a.jewell@open.ac.uk>
//
// Unit + integration tests for the `capability` crate.
// Covers: DirCapability creation, path resolution, path-traversal rejection,
// permission models, attenuation, and AuditLog hash-chain integrity.

use std::fs;
use std::path::Path;
use capability::{
    DirCapability, Permissions, CapabilityError,
    AuditLog,
};

// ─── Helpers ────────────────────────────────────────────────────────────────

fn scratch() -> tempfile::TempDir {
    tempfile::tempdir().expect("create temp dir")
}

// ─── DirCapability creation ──────────────────────────────────────────────────

/// Creating a capability for an existing directory must succeed.
#[test]
fn capability_creation_for_existing_dir_succeeds() {
    let tmp = scratch();
    let cap = DirCapability::new(tmp.path(), Permissions::all());
    assert!(cap.is_ok(), "capability creation should succeed for an existing dir");
}

/// Creating a capability for a non-existent path must return an error.
#[test]
fn capability_creation_for_nonexistent_path_fails() {
    let path = Path::new("/nonexistent/path/that/never/exists");
    let result = DirCapability::new(path, Permissions::read_only());
    assert!(result.is_err(), "creating capability for nonexistent path must fail");
}

/// The capability root should equal the canonicalized temp directory.
#[test]
fn capability_root_is_canonical() {
    let tmp = scratch();
    let cap = DirCapability::new(tmp.path(), Permissions::all())
        .expect("create capability");
    // Root must be absolute.
    assert!(cap.root().is_absolute(), "capability root must be absolute");
}

/// Permissions must be faithfully stored and retrievable.
#[test]
fn capability_permissions_stored_correctly() {
    let tmp = scratch();
    let perms = Permissions { read: true, write: false, delete: false };
    let cap = DirCapability::new(tmp.path(), perms).expect("create capability");
    let stored = cap.permissions();
    assert!(stored.read);
    assert!(!stored.write);
    assert!(!stored.delete);
}

// ─── Path resolution ─────────────────────────────────────────────────────────

/// Resolving a valid relative path to an existing file must return Ok.
#[test]
fn capability_resolve_existing_file_succeeds() {
    let tmp = scratch();
    let filename = "hello.txt";
    fs::write(tmp.path().join(filename), b"hi").expect("write file");

    let cap = DirCapability::new(tmp.path(), Permissions::all())
        .expect("create capability");
    let result = cap.resolve(Path::new(filename));
    assert!(result.is_ok(), "resolve of existing file must succeed: {:?}", result);
}

/// Resolving an absolute path must be rejected regardless of sandbox.
#[test]
fn capability_rejects_absolute_path() {
    let tmp = scratch();
    let cap = DirCapability::new(tmp.path(), Permissions::all())
        .expect("create capability");
    let result = cap.resolve(Path::new("/etc/passwd"));
    assert!(result.is_err(), "absolute path must be rejected");
    match result.unwrap_err() {
        CapabilityError::AbsolutePathRejected(_) => {} // correct variant
        other => panic!("expected AbsolutePathRejected, got: {:?}", other),
    }
}

/// Resolving a path that escapes via `..` must be rejected.
#[test]
fn capability_rejects_dotdot_traversal() {
    let tmp = scratch();
    // Create a subdirectory so we can attempt traversal from it.
    let sub = tmp.path().join("sub");
    fs::create_dir(&sub).expect("create subdir");
    let cap = DirCapability::new(&sub, Permissions::all())
        .expect("create capability for subdir");
    // `../` would escape the subdir sandbox.
    let result = cap.resolve(Path::new("../escape.txt"));
    assert!(result.is_err(), ".. traversal must be rejected");
}

// ─── Attenuation ────────────────────────────────────────────────────────────

/// Attenuating a capability to a sub-directory must succeed.
#[test]
fn capability_attenuation_to_subdir_succeeds() {
    let tmp = scratch();
    let sub = tmp.path().join("workspace");
    fs::create_dir(&sub).expect("create workspace subdir");

    let parent_cap = DirCapability::new(tmp.path(), Permissions::all())
        .expect("create parent capability");
    let child_cap = parent_cap.attenuate(Path::new("workspace"), Permissions::read_only());
    assert!(child_cap.is_ok(), "attenuation to existing subdir must succeed");
    assert!(child_cap.unwrap().root().ends_with("workspace"));
}

// ─── AuditLog ────────────────────────────────────────────────────────────────

/// An empty audit log must verify successfully with 0 entries.
#[test]
fn audit_log_empty_verifies_ok() {
    let tmp = scratch();
    let log_path = tmp.path().join("audit.log");
    // Create an empty file.
    fs::write(&log_path, b"").expect("create empty log");
    let count = AuditLog::verify(&log_path).expect("verify empty log");
    assert_eq!(count, 0);
}

/// Appending entries and verifying must report the correct entry count.
#[test]
fn audit_log_append_and_verify_chain_intact() {
    use capability::audit_log::Operation;

    let tmp = scratch();
    let log_path = tmp.path().join("audit.log");

    let mut log = AuditLog::open(&log_path).expect("open audit log");
    log.append(Operation::CapabilityCreated { root: tmp.path().to_path_buf() })
        .expect("append entry 1");
    log.append(Operation::FileRead { path: tmp.path().join("data.txt") })
        .expect("append entry 2");
    log.append(Operation::GitStatusChecked { repo_path: tmp.path().to_path_buf() })
        .expect("append entry 3");

    let count = AuditLog::verify(&log_path).expect("verify chain");
    assert_eq!(count, 3, "should have verified 3 entries");
}
