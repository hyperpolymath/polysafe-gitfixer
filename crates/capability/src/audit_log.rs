//! Append-only, hash-chained audit log for tamper evidence

use std::fs::{File, OpenOptions};
use std::io::{self, BufRead, BufReader, Write};
use std::path::Path;

use chrono::{DateTime, Utc};
use ring::digest::{Context, SHA256};
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Errors that can occur during audit log operations
#[derive(Debug, Error)]
pub enum IntegrityError {
    #[error("hash chain broken at entry {index}: expected {expected}, got {actual}")]
    ChainBroken {
        index: usize,
        expected: String,
        actual: String,
    },

    #[error("log entry {index} has invalid format")]
    InvalidFormat { index: usize },

    #[error("log file is empty but expected entries")]
    EmptyLog,

    #[error("io error: {0}")]
    Io(#[from] io::Error),

    #[error("serialization error: {0}")]
    Serialization(String),
}

/// Type of operation being logged
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Operation {
    /// Directory capability was created
    CapabilityCreated {
        root: String,
        permissions: String,
    },
    /// File was read
    FileRead { path: String },
    /// File was written
    FileWrite { path: String, size: u64 },
    /// File was deleted
    FileDelete { path: String },
    /// Directory was created
    DirCreate { path: String },
    /// Directory was deleted
    DirDelete { path: String },
    /// File was renamed/moved
    FileMove { from: String, to: String },
    /// Git operation was performed
    GitOperation { repo: String, operation: String },
    /// Backup was merged
    BackupMerge {
        backup_path: String,
        repo_path: String,
        files_merged: usize,
    },
    /// Transaction was committed
    TransactionCommit { id: String },
    /// Transaction was rolled back
    TransactionRollback { id: String, reason: String },
    /// Custom operation
    Custom { kind: String, details: String },
}

/// A single entry in the audit log
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    /// Timestamp of the operation
    pub timestamp: DateTime<Utc>,
    /// SHA-256 hash of the previous entry (hex-encoded)
    pub prev_hash: String,
    /// The operation that was performed
    pub operation: Operation,
    /// Optional additional context
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<String>,
}

impl LogEntry {
    /// Create a new log entry with the given operation
    pub fn new(operation: Operation, prev_hash: String) -> Self {
        Self {
            timestamp: Utc::now(),
            prev_hash,
            operation,
            context: None,
        }
    }

    /// Add optional context to the entry
    pub fn with_context(mut self, context: impl Into<String>) -> Self {
        self.context = Some(context.into());
        self
    }

    /// Compute the SHA-256 hash of this entry
    pub fn hash(&self) -> String {
        let json = serde_json::to_string(self).expect("LogEntry should serialize");
        let mut ctx = Context::new(&SHA256);
        ctx.update(json.as_bytes());
        let digest = ctx.finish();
        hex::encode(digest.as_ref())
    }
}

/// An append-only, hash-chained audit log
///
/// Each entry includes the hash of the previous entry, creating a tamper-evident
/// chain. If any entry is modified, the chain will break at that point.
///
/// The log uses JSON lines format for human readability and debuggability.
pub struct AuditLog {
    file: File,
    last_hash: String,
    entry_count: usize,
}

impl AuditLog {
    /// The genesis hash for the first entry in a new log
    const GENESIS_HASH: &'static str = "0000000000000000000000000000000000000000000000000000000000000000";

    /// Create a new audit log at the given path.
    ///
    /// If the file already exists, it will be opened for appending and the
    /// chain will be verified. If it doesn't exist, a new file will be created.
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self, IntegrityError> {
        let path = path.as_ref();

        if path.exists() {
            Self::open_existing(path)
        } else {
            Self::create_new(path)
        }
    }

    /// Create a new audit log file
    fn create_new(path: &Path) -> Result<Self, IntegrityError> {
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)?;

        Ok(Self {
            file,
            last_hash: Self::GENESIS_HASH.to_string(),
            entry_count: 0,
        })
    }

    /// Open an existing audit log and verify its integrity
    fn open_existing(path: &Path) -> Result<Self, IntegrityError> {
        // First, verify the chain
        let (last_hash, count) = Self::verify_file(path)?;

        // Then open for appending
        let file = OpenOptions::new()
            .append(true)
            .open(path)?;

        Ok(Self {
            file,
            last_hash,
            entry_count: count,
        })
    }

    /// Verify the integrity of a log file and return the last hash
    fn verify_file(path: &Path) -> Result<(String, usize), IntegrityError> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);

        let mut expected_prev_hash = Self::GENESIS_HASH.to_string();
        let mut count = 0;

        for (idx, line) in reader.lines().enumerate() {
            let line = line?;
            if line.trim().is_empty() {
                continue;
            }

            let entry: LogEntry = serde_json::from_str(&line).map_err(|e| {
                IntegrityError::Serialization(format!("entry {}: {}", idx, e))
            })?;

            if entry.prev_hash != expected_prev_hash {
                return Err(IntegrityError::ChainBroken {
                    index: idx,
                    expected: expected_prev_hash,
                    actual: entry.prev_hash,
                });
            }

            expected_prev_hash = entry.hash();
            count += 1;
        }

        Ok((expected_prev_hash, count))
    }

    /// Append a new entry to the audit log
    ///
    /// This will compute the hash chain, write the entry, and fsync to disk.
    pub fn append(&mut self, operation: Operation) -> io::Result<()> {
        self.append_with_context(operation, None)
    }

    /// Append a new entry with optional context
    pub fn append_with_context(
        &mut self,
        operation: Operation,
        context: Option<String>,
    ) -> io::Result<()> {
        let mut entry = LogEntry::new(operation, self.last_hash.clone());
        if let Some(ctx) = context {
            entry = entry.with_context(ctx);
        }

        // Compute hash for next entry
        let new_hash = entry.hash();

        // Serialize to JSON line
        let json = serde_json::to_string(&entry)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

        // Write with newline
        writeln!(self.file, "{}", json)?;

        // Sync to disk for durability
        self.file.sync_all()?;

        // Update state
        self.last_hash = new_hash;
        self.entry_count += 1;

        Ok(())
    }

    /// Verify the integrity of this log file
    ///
    /// Reads the entire file and verifies the hash chain is intact.
    pub fn verify<P: AsRef<Path>>(path: P) -> Result<usize, IntegrityError> {
        let (_, count) = Self::verify_file(path.as_ref())?;
        Ok(count)
    }

    /// Get the number of entries in the log
    pub fn entry_count(&self) -> usize {
        self.entry_count
    }

    /// Get the hash of the last entry (or genesis hash if empty)
    pub fn last_hash(&self) -> &str {
        &self.last_hash
    }

    /// Read all entries from a log file
    pub fn read_all<P: AsRef<Path>>(path: P) -> Result<Vec<LogEntry>, IntegrityError> {
        let file = File::open(path.as_ref())?;
        let reader = BufReader::new(file);
        let mut entries = Vec::new();

        for (idx, line) in reader.lines().enumerate() {
            let line = line?;
            if line.trim().is_empty() {
                continue;
            }

            let entry: LogEntry = serde_json::from_str(&line).map_err(|e| {
                IntegrityError::Serialization(format!("entry {}: {}", idx, e))
            })?;
            entries.push(entry);
        }

        Ok(entries)
    }
}

/// Helper module for hex encoding/decoding
mod hex {
    pub fn encode(bytes: &[u8]) -> String {
        bytes.iter().map(|b| format!("{:02x}", b)).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_new_log() {
        let tmp = TempDir::new().unwrap();
        let log_path = tmp.path().join("audit.log");

        let log = AuditLog::new(&log_path).unwrap();
        assert_eq!(log.entry_count(), 0);
        assert_eq!(log.last_hash(), AuditLog::GENESIS_HASH);
    }

    #[test]
    fn test_append_and_verify() {
        let tmp = TempDir::new().unwrap();
        let log_path = tmp.path().join("audit.log");

        {
            let mut log = AuditLog::new(&log_path).unwrap();
            log.append(Operation::FileRead {
                path: "/test/file.txt".into(),
            }).unwrap();
            log.append(Operation::FileWrite {
                path: "/test/file.txt".into(),
                size: 1024,
            }).unwrap();
            assert_eq!(log.entry_count(), 2);
        }

        // Verify the log
        let count = AuditLog::verify(&log_path).unwrap();
        assert_eq!(count, 2);

        // Reopen and append more
        {
            let mut log = AuditLog::new(&log_path).unwrap();
            assert_eq!(log.entry_count(), 2);
            log.append(Operation::FileDelete {
                path: "/test/file.txt".into(),
            }).unwrap();
            assert_eq!(log.entry_count(), 3);
        }

        let count = AuditLog::verify(&log_path).unwrap();
        assert_eq!(count, 3);
    }

    #[test]
    fn test_tamper_detection() {
        let tmp = TempDir::new().unwrap();
        let log_path = tmp.path().join("audit.log");

        // Create a log with entries
        {
            let mut log = AuditLog::new(&log_path).unwrap();
            log.append(Operation::FileRead {
                path: "/first".into(),
            }).unwrap();
            log.append(Operation::FileRead {
                path: "/second".into(),
            }).unwrap();
            log.append(Operation::FileRead {
                path: "/third".into(),
            }).unwrap();
        }

        // Tamper with the file by modifying the second entry
        {
            use std::io::Read;

            let mut content = String::new();
            File::open(&log_path).unwrap().read_to_string(&mut content).unwrap();

            // Replace "/second" with "/hacked"
            let tampered = content.replace("/second", "/hacked");

            let mut file = OpenOptions::new()
                .write(true)
                .truncate(true)
                .open(&log_path)
                .unwrap();
            file.write_all(tampered.as_bytes()).unwrap();
        }

        // Verification should fail
        let result = AuditLog::verify(&log_path);
        assert!(matches!(result, Err(IntegrityError::ChainBroken { .. })));
    }

    #[test]
    fn test_read_all_entries() {
        let tmp = TempDir::new().unwrap();
        let log_path = tmp.path().join("audit.log");

        {
            let mut log = AuditLog::new(&log_path).unwrap();
            log.append(Operation::GitOperation {
                repo: "/repos/project".into(),
                operation: "status".into(),
            }).unwrap();
            log.append_with_context(
                Operation::BackupMerge {
                    backup_path: "/backup".into(),
                    repo_path: "/repo".into(),
                    files_merged: 42,
                },
                Some("User confirmed merge".into()),
            ).unwrap();
        }

        let entries = AuditLog::read_all(&log_path).unwrap();
        assert_eq!(entries.len(), 2);

        match &entries[0].operation {
            Operation::GitOperation { repo, operation } => {
                assert_eq!(repo, "/repos/project");
                assert_eq!(operation, "status");
            }
            _ => panic!("Expected GitOperation"),
        }

        match &entries[1].operation {
            Operation::BackupMerge { files_merged, .. } => {
                assert_eq!(*files_merged, 42);
            }
            _ => panic!("Expected BackupMerge"),
        }
        assert_eq!(entries[1].context, Some("User confirmed merge".into()));
    }

    #[test]
    fn test_hash_chain_structure() {
        let tmp = TempDir::new().unwrap();
        let log_path = tmp.path().join("audit.log");

        {
            let mut log = AuditLog::new(&log_path).unwrap();
            log.append(Operation::FileRead { path: "/a".into() }).unwrap();
            log.append(Operation::FileRead { path: "/b".into() }).unwrap();
        }

        let entries = AuditLog::read_all(&log_path).unwrap();

        // First entry should have genesis hash
        assert_eq!(entries[0].prev_hash, AuditLog::GENESIS_HASH);

        // Second entry should have hash of first
        let first_hash = entries[0].hash();
        assert_eq!(entries[1].prev_hash, first_hash);
    }
}
