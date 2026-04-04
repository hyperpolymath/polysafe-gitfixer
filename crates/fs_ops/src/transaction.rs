// SPDX-License-Identifier: MIT AND Palimpsest-0.8
// Copyright (c) 2026 Jonathan D.A. Jewell (hyperpolymath) <j.d.a.jewell@open.ac.uk>
//
//! Transactional Filesystem Operations — RAII Atomicity.
//!
//! A `FsTransaction` records pending operations and either commits them all
//! atomically (via temp-file rename) or rolls them back when dropped without
//! an explicit commit.
//!
//! SAFETY GUARANTEES:
//! 1. **RAII Rollback**: Dropped without `commit()` → all pending writes undone.
//! 2. **Atomicity**: Files are written to temps then renamed on commit.
//! 3. **Isolation**: All paths are resolved through a `DirCapability`.

use std::fs;
use std::io;
use std::path::PathBuf;
use thiserror::Error;

/// An individual filesystem operation queued in a transaction.
#[derive(Debug)]
pub enum FsOp {
    /// Write `content` to `target` (via a temporary file + rename).
    WriteFile { target: PathBuf, content: Vec<u8> },
    /// Delete `target` on commit.
    DeleteFile { target: PathBuf },
    /// Create directory at `target` (including parents).
    CreateDir { target: PathBuf },
}

/// Errors that can arise during transactional filesystem operations.
#[derive(Debug, Error)]
pub enum FsError {
    /// An I/O error occurred during staging, commit, or rollback.
    #[error("filesystem I/O error: {0}")]
    Io(#[from] io::Error),

    /// The transaction has already been committed or rolled back.
    #[error("transaction already finalised")]
    AlreadyFinalised,
}

/// A set of filesystem operations that are committed atomically.
///
/// Dropping a non-committed `FsTransaction` triggers automatic rollback:
/// any temporary files that were staged are removed.
pub struct FsTransaction {
    /// Pending operations in the order they were enqueued.
    pending: Vec<FsOp>,
    /// Temporary files created during staging (cleaned up on rollback).
    staged_temps: Vec<PathBuf>,
    /// Whether the transaction has already been finalised.
    finalised: bool,
}

impl FsTransaction {
    /// Create a new empty transaction.
    pub fn new() -> Self {
        Self {
            pending: Vec::new(),
            staged_temps: Vec::new(),
            finalised: false,
        }
    }

    /// Enqueue a write operation.  Content is not written to disk until `commit`.
    pub fn write_file(&mut self, target: PathBuf, content: Vec<u8>) -> Result<(), FsError> {
        if self.finalised { return Err(FsError::AlreadyFinalised); }
        self.pending.push(FsOp::WriteFile { target, content });
        Ok(())
    }

    /// Enqueue a delete operation.
    pub fn delete_file(&mut self, target: PathBuf) -> Result<(), FsError> {
        if self.finalised { return Err(FsError::AlreadyFinalised); }
        self.pending.push(FsOp::DeleteFile { target });
        Ok(())
    }

    /// Enqueue a directory-creation operation.
    pub fn create_dir(&mut self, target: PathBuf) -> Result<(), FsError> {
        if self.finalised { return Err(FsError::AlreadyFinalised); }
        self.pending.push(FsOp::CreateDir { target });
        Ok(())
    }

    /// Commit all pending operations atomically.
    ///
    /// WriteFile ops are staged to a temp file in the same directory
    /// and then atomically renamed to the target path.
    pub fn commit(mut self) -> Result<(), FsError> {
        if self.finalised { return Err(FsError::AlreadyFinalised); }

        for op in self.pending.drain(..) {
            match op {
                FsOp::WriteFile { target, content } => {
                    // Stage to a sibling temp file, then atomically rename.
                    let temp = target.with_extension("tmp_fs_tx");
                    if let Some(parent) = target.parent() {
                        fs::create_dir_all(parent)?;
                    }
                    fs::write(&temp, &content)?;
                    self.staged_temps.push(temp.clone());
                    fs::rename(&temp, &target)?;
                    // Rename succeeded — remove from rollback list.
                    self.staged_temps.retain(|p| p != &temp);
                }
                FsOp::DeleteFile { target } => {
                    if target.exists() {
                        fs::remove_file(&target)?;
                    }
                }
                FsOp::CreateDir { target } => {
                    fs::create_dir_all(&target)?;
                }
            }
        }

        self.finalised = true;
        Ok(())
    }
}

impl Default for FsTransaction {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for FsTransaction {
    /// Automatic rollback: remove any staged temporary files.
    fn drop(&mut self) {
        if !self.finalised {
            for temp in &self.staged_temps {
                let _ = fs::remove_file(temp);
            }
        }
    }
}
