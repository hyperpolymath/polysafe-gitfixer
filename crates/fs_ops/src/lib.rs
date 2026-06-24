// SPDX-License-Identifier: MPL-2.0
// Copyright (c) Jonathan D.A. Jewell <j.d.a.jewell@open.ac.uk>
//! Transactional Filesystem Operations — High-Assurance I/O.
//!
//! This crate provides a safe, transactional interface for modifying 
//! the physical filesystem. It is designed to prevent data corruption 
//! and ensure atomicity during complex multi-file updates.
//!
//! SAFETY GUARANTEES:
//! 1. **RAII Rollback**: If a `FsTransaction` is dropped before being 
//!    formally committed, all pending operations are undone.
//! 2. **Atomicity**: Files are written to temporary buffers and only 
//!    renamed to the target path upon a successful commit.
//! 3. **Isolation**: All paths are resolved through a `DirCapability`, 
//!    eliminating path traversal vulnerabilities at the capability layer.

#![forbid(unsafe_code)]
mod transaction;

pub use transaction::{FsTransaction, FsError, FsOp};
