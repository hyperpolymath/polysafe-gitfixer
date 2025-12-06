// SPDX-License-Identifier: MIT AND Palimpsest-0.8
// SPDX-FileCopyrightText: 2024-2025 The polysafe-gitfixer Contributors

//! # Transactional Filesystem Operations
//!
//! This crate provides atomic, transactional filesystem operations with automatic
//! rollback on failure. Operations are journaled and can be committed or rolled
//! back as a unit.
//!
//! ## Safety Guarantees
//!
//! - **RAII Cleanup**: If a transaction is dropped without being committed, all
//!   operations are automatically rolled back.
//!
//! - **Atomic Copies**: Files are first written to a `.tmp` suffix, then renamed,
//!   ensuring the target file is never in a partial state.
//!
//! - **Capability-based Access**: All operations go through a `DirCapability`,
//!   preventing path traversal attacks.
//!
//! ## Example
//!
//! ```no_run
//! use fs_ops::FsTransaction;
//! use capability::{DirCapability, Permissions};
//!
//! let cap = DirCapability::new("/path/to/dir", Permissions::full())?;
//! let mut tx = FsTransaction::new(&cap);
//!
//! tx.copy_file("src/file.txt", "dst/file.txt")?;
//! tx.create_dir("dst/subdir")?;
//!
//! // If we don't call commit(), dropping tx will roll back all changes
//! tx.commit()?;
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

mod transaction;

pub use transaction::{FsTransaction, FsError, FsOp};
