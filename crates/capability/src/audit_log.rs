// SPDX-License-Identifier: MIT AND Palimpsest-0.8
// Copyright (c) 2026 Jonathan D.A. Jewell (hyperpolymath) <j.d.a.jewell@open.ac.uk>
//
//! Audit Log — Hash-Chained Tamper Evidence.
//!
//! This module implements a persistent, append-only ledger for system actions.
//! It uses cryptographic chaining to ensure that any modification to
//! historical entries is detectable via formal verification.
//!
//! INTEGRITY MODEL:
//! Each `LogEntry` carries the SHA-256 hash of the preceding entry.
//! The chain starts with a fixed `GENESIS_HASH`.
//!
//! AUDITED OPERATIONS:
//! - **Filesystem**: Reads, Writes, Moves, Deletes.
//! - **Capabilities**: Creation and path resolution events.
//! - **Git**: Repository status checks and commit actions.

use std::fs::{File, OpenOptions};
use std::io::{self, BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use ring::digest::{Context, SHA256};
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// The sentinel hash used as the `prev_hash` of the very first log entry.
const GENESIS_HASH: &str = "0000000000000000000000000000000000000000000000000000000000000000";

/// Categories of operations recorded in the audit log.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Operation {
    /// A file was read within a capability sandbox.
    FileRead { path: PathBuf },
    /// A file was written (created or overwritten).
    FileWrite { path: PathBuf },
    /// A file was moved from one path to another.
    FileMove { from: PathBuf, to: PathBuf },
    /// A file was deleted.
    FileDelete { path: PathBuf },
    /// A `DirCapability` token was created.
    CapabilityCreated { root: PathBuf },
    /// A path was resolved via a `DirCapability` token.
    CapabilityResolved { relative: PathBuf, canonical: PathBuf },
    /// A git repository status check was performed.
    GitStatusChecked { repo_path: PathBuf },
    /// A git commit was created.
    GitCommitCreated { repo_path: PathBuf, message: String },
}

/// A single record in the hash-chained audit log.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    /// Wall-clock timestamp of the operation (UTC).
    pub timestamp: DateTime<Utc>,
    /// SHA-256 hex digest of the preceding entry (or `GENESIS_HASH` for entry 0).
    pub prev_hash: String,
    /// The operation that was performed.
    pub operation: Operation,
}

impl LogEntry {
    /// Compute the SHA-256 hash of this entry's serialised form.
    pub fn hash(&self) -> String {
        let serialised = serde_json::to_string(self)
            .expect("LogEntry must be serialisable to compute hash");
        let mut ctx = Context::new(&SHA256);
        ctx.update(serialised.as_bytes());
        let digest = ctx.finish();
        hex::encode(digest.as_ref())
    }
}

/// Error returned when the audit log's hash chain is broken.
#[derive(Debug, thiserror::Error)]
pub enum IntegrityError {
    /// The hash stored in entry `index` does not match the hash of entry `index - 1`.
    #[error("hash chain broken at entry {index}: expected {expected}, found {found}")]
    ChainBroken { index: usize, expected: String, found: String },

    /// The log file could not be read.
    #[error("failed to read audit log: {0}")]
    Io(#[from] io::Error),

    /// A log entry could not be deserialised.
    #[error("failed to deserialise log entry at line {line}: {cause}")]
    Deserialisation { line: usize, cause: String },
}

/// An append-only, hash-chained audit log backed by a flat NDJSON file.
pub struct AuditLog {
    /// Open file handle (append mode).
    file: File,
    /// Hash of the most recently appended entry (or GENESIS_HASH if empty).
    last_hash: String,
}

impl AuditLog {
    /// Open (or create) an audit log at `path`.
    ///
    /// If the file already contains entries the last hash is reconstructed
    /// by reading the file from the beginning.
    pub fn open<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        // Derive last_hash by scanning existing entries.
        let last_hash = if path.as_ref().exists() {
            Self::last_hash_from_file(path.as_ref())?
        } else {
            GENESIS_HASH.to_owned()
        };

        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)?;

        Ok(Self { file, last_hash })
    }

    /// Read the file and return the hash of the last valid entry.
    fn last_hash_from_file(path: &Path) -> io::Result<String> {
        let f = File::open(path)?;
        let reader = BufReader::new(f);
        let mut last_hash = GENESIS_HASH.to_owned();
        for line in reader.lines() {
            let line = line?;
            if line.trim().is_empty() { continue; }
            if let Ok(entry) = serde_json::from_str::<LogEntry>(&line) {
                last_hash = entry.hash();
            }
        }
        Ok(last_hash)
    }

    /// Append `operation` to the log, chaining it to the previous entry.
    ///
    /// Each entry is written as a single JSON line and then `fsync`-ed to
    /// ensure physical persistence.
    pub fn append(&mut self, operation: Operation) -> io::Result<()> {
        let entry = LogEntry {
            timestamp: Utc::now(),
            prev_hash: self.last_hash.clone(),
            operation,
        };
        let new_hash = entry.hash();
        let mut line = serde_json::to_string(&entry)
            .expect("LogEntry must serialise");
        line.push('\n');
        self.file.write_all(line.as_bytes())?;
        self.file.sync_all()?;
        self.last_hash = new_hash;
        Ok(())
    }

    /// Verify the entire log at `path` by re-computing the hash chain.
    ///
    /// Returns the number of entries verified if the chain is unbroken.
    pub fn verify<P: AsRef<Path>>(path: P) -> Result<usize, IntegrityError> {
        let f = File::open(path.as_ref())?;
        let reader = BufReader::new(f);
        let mut prev_hash = GENESIS_HASH.to_owned();
        let mut count = 0usize;

        for (line_idx, line) in reader.lines().enumerate() {
            let line = line?;
            if line.trim().is_empty() { continue; }

            let entry: LogEntry = serde_json::from_str(&line)
                .map_err(|e| IntegrityError::Deserialisation { line: line_idx + 1, cause: e.to_string() })?;

            // Every entry except the first must chain to its predecessor.
            if count > 0 && entry.prev_hash != prev_hash {
                return Err(IntegrityError::ChainBroken {
                    index: count,
                    expected: prev_hash,
                    found: entry.prev_hash.clone(),
                });
            }

            prev_hash = entry.hash();
            count += 1;
        }

        Ok(count)
    }
}

// ─── hex helper (avoid pulling in the hex crate) ─────────────────────────────

mod hex {
    /// Encode a byte slice as lowercase hexadecimal.
    pub fn encode(bytes: &[u8]) -> String {
        bytes.iter().map(|b| format!("{:02x}", b)).collect()
    }
}
