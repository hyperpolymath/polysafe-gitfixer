//! # Capability-based Security for polysafe-gitfixer
//!
//! This crate provides two core security primitives:
//!
//! 1. **DirCapability**: An unforgeable capability token that restricts filesystem
//!    access to a specific directory tree, preventing path traversal attacks.
//!
//! 2. **AuditLog**: An append-only, hash-chained audit log that provides tamper
//!    evidence and integrity verification.
//!
//! ## Design Philosophy
//!
//! Rather than checking permissions at each operation, we create capability tokens
//! that encode the allowed operations. A `DirCapability` for `/home/user/repos`
//! can only resolve paths within that tree - attempting `../etc/passwd` will fail
//! at the capability level, not at the filesystem level.

mod dir_capability;
mod audit_log;

pub use dir_capability::{DirCapability, Permissions, CapabilityError};
pub use audit_log::{AuditLog, LogEntry, IntegrityError};
