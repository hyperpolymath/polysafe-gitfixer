// SPDX-License-Identifier: MPL-2.0
// Copyright (c) Jonathan D.A. Jewell <j.d.a.jewell@open.ac.uk>
// SPDX-FileCopyrightText: 2024-2025 The polysafe-gitfixer Contributors

//! # Rustler NIF Bindings for polysafe-gitfixer
//!
//! This crate provides Elixir NIF bindings for the Rust components:
//! - capability (path safety, audit logging)
//! - git_ops (git repository operations)
//!
//! The NIFs are exposed to Elixir through the Rustler library.
//!
//! ## Architecture
//!
//! Each NIF function delegates to the corresponding Rust crate:
//! - `capability::DirCapability` for path-safe filesystem access
//! - `capability::AuditLog` for tamper-evident logging
//! - `git_ops` for git repository introspection and staging
//!
//! Handle management uses string-serialised canonical paths as opaque
//! identifiers, since Rustler NIFs are stateless between calls. The
//! Elixir side is responsible for tracking handle lifetimes via a
//! GenServer or ETS table.

#![forbid(unsafe_code)]
use rustler::{Encoder, Env, NifResult, Term};

mod atoms {
    rustler::atoms! {
        ok,
        error,
        not_found,
        permission_denied,
        path_traversal,
        invalid_repo,
    }
}

// ---------------------------------------------------------------------------
// Capability NIFs
// ---------------------------------------------------------------------------

/// Create a directory capability for the given path with the specified
/// permission string.
///
/// ## Parameters
/// - `path`        - Absolute path to the directory root
/// - `permissions` - One of "full", "read_only", "read_write"
///
/// ## Returns
/// `{:ok, canonical_root_path}` on success, `{:error, reason}` on failure.
#[rustler::nif]
fn create_capability<'a>(env: Env<'a>, path: String, permissions: String) -> NifResult<Term<'a>> {
    let perms = match permissions.as_str() {
        "full" => capability::Permissions::full(),
        "read_only" => capability::Permissions::read_only(),
        "read_write" => capability::Permissions::read_write(),
        other => {
            return Ok((
                atoms::error(),
                format!("unknown permission string: {other}; expected full|read_only|read_write"),
            )
                .encode(env));
        }
    };

    match capability::DirCapability::new(&path, perms) {
        Ok(cap) => {
            // Return the canonical root as the handle — the Elixir side
            // re-opens a DirCapability from this path on each subsequent call.
            let handle = cap.root().display().to_string();
            Ok((atoms::ok(), handle).encode(env))
        }
        Err(capability::CapabilityError::PathNotFound(p)) => {
            Ok((atoms::error(), atoms::not_found(), p.display().to_string()).encode(env))
        }
        Err(capability::CapabilityError::InvalidRoot(p)) => {
            Ok((atoms::error(), format!("invalid root: {}", p.display())).encode(env))
        }
        Err(e) => Ok((atoms::error(), e.to_string()).encode(env)),
    }
}

/// Resolve a relative path through an existing capability.
///
/// ## Parameters
/// - `cap_handle`    - Canonical root path returned by `create_capability/2`
/// - `relative_path` - Relative path to resolve within the capability root
///
/// ## Returns
/// `{:ok, absolute_path}` or `{:error, :path_traversal}` / `{:error, reason}`.
#[rustler::nif]
fn resolve_path<'a>(
    env: Env<'a>,
    cap_handle: String,
    relative_path: String,
) -> NifResult<Term<'a>> {
    // Re-open capability from the handle (canonical root path)
    let cap = match capability::DirCapability::new(&cap_handle, capability::Permissions::read_only())
    {
        Ok(c) => c,
        Err(e) => return Ok((atoms::error(), e.to_string()).encode(env)),
    };

    match cap.resolve(std::path::Path::new(&relative_path)) {
        Ok(resolved) => Ok((atoms::ok(), resolved.display().to_string()).encode(env)),
        Err(capability::CapabilityError::PathTraversal { .. }) => {
            Ok((atoms::error(), atoms::path_traversal()).encode(env))
        }
        Err(e) => Ok((atoms::error(), e.to_string()).encode(env)),
    }
}

// ---------------------------------------------------------------------------
// Audit Log NIFs
// ---------------------------------------------------------------------------

/// Open or create an append-only audit log at the given path.
///
/// ## Parameters
/// - `path` - Filesystem path for the audit log file
///
/// ## Returns
/// `{:ok, log_path}` on success (the path doubles as the handle),
/// `{:error, reason}` on failure.
#[rustler::nif]
fn open_audit_log<'a>(env: Env<'a>, path: String) -> NifResult<Term<'a>> {
    match capability::AuditLog::new(&path) {
        Ok(_log) => Ok((atoms::ok(), path).encode(env)),
        Err(e) => Ok((atoms::error(), e.to_string()).encode(env)),
    }
}

/// Append an entry to the audit log.
///
/// ## Parameters
/// - `log_handle` - Path returned by `open_audit_log/1`
/// - `operation`  - An Elixir map with keys `"type"` and `"details"` that is
///                  serialised to a `capability::audit_log::Operation::Custom`
///
/// ## Returns
/// `:ok` on success, `{:error, reason}` on failure.
///
/// Note: Currently records every entry as `Operation::Custom` with the
/// `type` and `details` fields extracted from the Elixir term's string
/// representation. A richer NIF-side decoder can be added later.
#[rustler::nif]
fn append_audit_entry<'a>(
    env: Env<'a>,
    log_handle: String,
    operation: Term<'a>,
) -> NifResult<Term<'a>> {
    let mut log = match capability::AuditLog::new(&log_handle) {
        Ok(l) => l,
        Err(e) => return Ok((atoms::error(), e.to_string()).encode(env)),
    };

    // Encode the Elixir term as a custom operation. A production NIF would
    // decode structured maps here, but for the initial bridge we capture
    // the term's debug representation as details.
    let op = capability::audit_log::Operation::Custom {
        kind: "nif_entry".to_string(),
        details: format!("{:?}", operation),
    };

    match log.append(op) {
        Ok(()) => Ok(atoms::ok().encode(env)),
        Err(e) => Ok((atoms::error(), e.to_string()).encode(env)),
    }
}

/// Verify the integrity of the hash chain in an audit log.
///
/// ## Parameters
/// - `log_handle` - Path returned by `open_audit_log/1`
///
/// ## Returns
/// `{:ok, entry_count}` if the chain is valid, `{:error, reason}` otherwise.
#[rustler::nif]
fn verify_audit_log<'a>(env: Env<'a>, log_handle: String) -> NifResult<Term<'a>> {
    match capability::AuditLog::verify(&log_handle) {
        Ok(count) => Ok((atoms::ok(), count as u64).encode(env)),
        Err(e) => Ok((atoms::error(), e.to_string()).encode(env)),
    }
}

// ---------------------------------------------------------------------------
// Filesystem Transaction NIFs
//
// NOTE: The `fs_ops` crate's `transaction` module is not yet implemented.
// These NIFs are wired to return descriptive errors until the transaction
// layer is complete. Each todo!() has been replaced with a graceful error
// return so that the NIF module loads and the Elixir side gets actionable
// error tuples instead of panics.
// ---------------------------------------------------------------------------

/// Begin a new filesystem transaction scoped to a capability root.
///
/// ## Parameters
/// - `cap_handle` - Canonical root path from `create_capability/2`
///
/// ## Returns
/// `{:ok, tx_id}` on success, `{:error, reason}` on failure.
///
/// Currently returns an error because the `fs_ops::FsTransaction` backend
/// is not yet implemented.
#[rustler::nif]
fn begin_transaction<'a>(env: Env<'a>, cap_handle: String) -> NifResult<Term<'a>> {
    // Validate that the capability root is still valid
    match capability::DirCapability::new(&cap_handle, capability::Permissions::full()) {
        Ok(_cap) => {
            // fs_ops::FsTransaction is declared but the transaction module is
            // missing — return a structured error so the Elixir caller knows
            // this feature is pending.
            Ok((
                atoms::error(),
                "fs_ops transaction layer not yet implemented — see crates/fs_ops/src/transaction.rs",
            )
                .encode(env))
        }
        Err(e) => Ok((atoms::error(), e.to_string()).encode(env)),
    }
}

/// Copy a file within a transaction.
///
/// Currently returns an error — transaction layer pending.
#[rustler::nif]
fn tx_copy_file<'a>(
    env: Env<'a>,
    _tx_handle: String,
    _from: String,
    _to: String,
) -> NifResult<Term<'a>> {
    Ok((
        atoms::error(),
        "fs_ops transaction layer not yet implemented — begin_transaction must succeed first",
    )
        .encode(env))
}

/// Commit a transaction, applying all buffered operations atomically.
///
/// Currently returns an error — transaction layer pending.
#[rustler::nif]
fn tx_commit<'a>(env: Env<'a>, _tx_handle: String) -> NifResult<Term<'a>> {
    Ok((
        atoms::error(),
        "fs_ops transaction layer not yet implemented — begin_transaction must succeed first",
    )
        .encode(env))
}

/// Roll back a transaction, undoing all buffered operations.
///
/// Currently returns an error — transaction layer pending.
#[rustler::nif]
fn tx_rollback<'a>(env: Env<'a>, _tx_handle: String) -> NifResult<Term<'a>> {
    Ok((
        atoms::error(),
        "fs_ops transaction layer not yet implemented — begin_transaction must succeed first",
    )
        .encode(env))
}

// ---------------------------------------------------------------------------
// Git Operations NIFs
// ---------------------------------------------------------------------------

/// Check whether the given path contains a valid git repository.
///
/// ## Parameters
/// - `path` - Filesystem path to check
///
/// ## Returns
/// `true` or `false`.
#[rustler::nif]
fn is_git_repo(path: String) -> bool {
    git_ops::is_valid_repo(&path)
}

/// Return the full status of a git repository as an Elixir term.
///
/// ## Parameters
/// - `path` - Root of the git repository
///
/// ## Returns
/// `{:ok, status_map}` where `status_map` is a map with keys:
/// - `"path"`, `"branch"`, `"head"`, `"is_clean"`,
///   `"has_staged"`, `"has_unstaged"`, `"has_untracked"`,
///   `"entries"` (list of `%{"path" => ..., "index" => ..., "worktree" => ...}`)
///
/// Returns `{:error, :invalid_repo}` if the path is not a git repository.
#[rustler::nif]
fn git_status<'a>(env: Env<'a>, path: String) -> NifResult<Term<'a>> {
    match git_ops::repo_status(&path) {
        Ok(status) => {
            // Build a map-like term from the RepoStatus struct.
            // Rustler's Encoder for tuples and vecs gives us a clean Elixir
            // representation without needing a full NifStruct derive.
            let entries: Vec<(String, String, String)> = status
                .entries
                .iter()
                .map(|e| {
                    (
                        e.path.clone(),
                        format!("{:?}", e.index_status),
                        format!("{:?}", e.worktree_status),
                    )
                })
                .collect();

            Ok((
                atoms::ok(),
                (
                    ("path", status.path),
                    ("branch", format!("{:?}", status.branch)),
                    ("head", format!("{:?}", status.head)),
                    ("is_clean", status.is_clean),
                    ("has_staged", status.has_staged),
                    ("has_unstaged", status.has_unstaged),
                    ("has_untracked", status.has_untracked),
                    ("entries", entries),
                ),
            )
                .encode(env))
        }
        Err(_) => Ok((atoms::error(), atoms::invalid_repo()).encode(env)),
    }
}

/// Stage all changes in the repository (equivalent to `git add -A`).
///
/// ## Parameters
/// - `path` - Root of the git repository
///
/// ## Returns
/// `:ok` on success, `{:error, :invalid_repo}` on failure.
#[rustler::nif]
fn git_stage_all<'a>(env: Env<'a>, path: String) -> NifResult<Term<'a>> {
    match git_ops::stage_all(&path) {
        Ok(()) => Ok(atoms::ok().encode(env)),
        Err(_) => Ok((atoms::error(), atoms::invalid_repo()).encode(env)),
    }
}

/// Find all git repositories under a root directory up to `max_depth`.
///
/// ## Parameters
/// - `root`      - Directory to search from
/// - `max_depth` - Maximum recursion depth
///
/// ## Returns
/// `{:ok, [path, ...]}` — a list of absolute paths to discovered repos.
#[rustler::nif]
fn find_git_repos<'a>(env: Env<'a>, root: String, max_depth: u32) -> NifResult<Term<'a>> {
    match git_ops::find_repos(&root, max_depth as usize) {
        Ok(repos) => Ok((atoms::ok(), repos).encode(env)),
        Err(e) => Ok((atoms::error(), e.to_string()).encode(env)),
    }
}

rustler::init!(
    "Elixir.PolysafeGitfixer.Native",
    [
        create_capability,
        resolve_path,
        open_audit_log,
        append_audit_entry,
        verify_audit_log,
        begin_transaction,
        tx_copy_file,
        tx_commit,
        tx_rollback,
        is_git_repo,
        git_status,
        git_stage_all,
        find_git_repos,
    ]
);
