// SPDX-License-Identifier: MIT AND Palimpsest-0.8

//! Capability-Based Security — Polysafe Primitives.
//!
//! This crate implements the core security model for the gitfixer ecosystem. 
//! It replaces traditional ACL-based checks with "Unforgeable Tokens" that 
//! grant explicit permission to perform specific actions.
//!
//! SECURITY PRIMITIVES:
//! 1. **DirCapability**: A path-restricted token. Once created, it can 
//!    only resolve paths within its designated sandbox, providing 
//!    compile-time and runtime protection against CWE-22 (Path Traversal).
//! 2. **AuditLog**: A cryptographic ledger. Every entry is chained to 
//!    the previous hash, ensuring that any tampering with the system 
//!    history is detectable via formal verification.

#![forbid(unsafe_code)]
mod dir_capability;
mod audit_log;

pub use dir_capability::{DirCapability, Permissions, CapabilityError};
pub use audit_log::{AuditLog, LogEntry, IntegrityError};
