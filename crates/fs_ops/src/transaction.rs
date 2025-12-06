// SPDX-License-Identifier: MIT AND Palimpsest-0.8
// SPDX-FileCopyrightText: 2024-2025 The polysafe-gitfixer Contributors

//! Filesystem transaction with journaling and rollback

use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};

use capability::{CapabilityError, DirCapability};
use thiserror::Error;
use uuid::Uuid;

/// Errors that can occur during filesystem operations
#[derive(Debug, Error)]
pub enum FsError {
    #[error("capability error: {0}")]
    Capability(#[from] CapabilityError),

    #[error("io error: {0}")]
    Io(#[from] io::Error),

    #[error("transaction already committed")]
    AlreadyCommitted,

    #[error("path is not a file: {0}")]
    NotAFile(PathBuf),

    #[error("path is not a directory: {0}")]
    NotADirectory(PathBuf),

    #[error("path already exists: {0}")]
    AlreadyExists(PathBuf),

    #[error("rollback failed for operation {op}: {error}")]
    RollbackFailed { op: String, error: String },
}

/// A filesystem operation that can be rolled back
#[derive(Debug, Clone)]
pub enum FsOp {
    /// A file was created (should be deleted on rollback)
    Created(PathBuf),

    /// A directory was created (should be deleted on rollback)
    CreatedDir(PathBuf),

    /// A file rename is pending (should be reverted on rollback)
    PendingRename {
        from: PathBuf,
        to: PathBuf,
    },

    /// A file is pending deletion (backup was made)
    PendingDelete {
        path: PathBuf,
        backup: PathBuf,
    },

    /// A file was renamed (should be renamed back on rollback)
    Renamed {
        from: PathBuf,
        to: PathBuf,
    },
}

/// A transactional filesystem operation set
///
/// All operations are recorded in a journal. If the transaction is dropped
/// without being committed, all operations are rolled back in reverse order.
pub struct FsTransaction<'a> {
    /// The capability that restricts our access
    capability: &'a DirCapability,

    /// Journal of operations performed
    journal: Vec<FsOp>,

    /// Whether the transaction has been committed
    committed: bool,

    /// Unique ID for this transaction (used for temp files)
    id: Uuid,
}

impl<'a> FsTransaction<'a> {
    /// Create a new transaction using the given capability
    pub fn new(capability: &'a DirCapability) -> Self {
        Self {
            capability,
            journal: Vec::new(),
            committed: false,
            id: Uuid::new_v4(),
        }
    }

    /// Get the transaction ID
    pub fn id(&self) -> Uuid {
        self.id
    }

    /// Get the current journal
    pub fn journal(&self) -> &[FsOp] {
        &self.journal
    }

    /// Copy a file atomically within the capability's root
    ///
    /// The file is first written to a temporary location, then renamed to
    /// the final destination.
    pub fn copy_file<P: AsRef<Path>, Q: AsRef<Path>>(
        &mut self,
        from: P,
        to: Q,
    ) -> Result<(), FsError> {
        self.ensure_not_committed()?;
        self.capability.require_read()?;
        self.capability.require_write()?;

        let src = self.capability.resolve(from.as_ref())?;
        let dst = self.capability.resolve_for_creation(to.as_ref())?;

        if !src.is_file() {
            return Err(FsError::NotAFile(src));
        }

        // Create temp file path
        let tmp = self.temp_path(&dst);

        // Copy to temp location
        {
            let mut src_file = File::open(&src)?;
            let mut tmp_file = File::create(&tmp)?;
            let mut buffer = [0u8; 64 * 1024];
            loop {
                let bytes_read = src_file.read(&mut buffer)?;
                if bytes_read == 0 {
                    break;
                }
                tmp_file.write_all(&buffer[..bytes_read])?;
            }
            tmp_file.sync_all()?;
        }

        // Record temp file creation
        self.journal.push(FsOp::Created(tmp.clone()));

        // Rename temp to final
        fs::rename(&tmp, &dst)?;

        // Update journal: remove Created, add Renamed
        self.journal.pop();
        self.journal.push(FsOp::Created(dst));

        Ok(())
    }

    /// Move a file within the capability's root
    pub fn move_file<P: AsRef<Path>, Q: AsRef<Path>>(
        &mut self,
        from: P,
        to: Q,
    ) -> Result<(), FsError> {
        self.ensure_not_committed()?;
        self.capability.require_write()?;

        let src = self.capability.resolve(from.as_ref())?;
        let dst = self.capability.resolve_for_creation(to.as_ref())?;

        if !src.is_file() {
            return Err(FsError::NotAFile(src));
        }

        fs::rename(&src, &dst)?;
        self.journal.push(FsOp::Renamed {
            from: src,
            to: dst,
        });

        Ok(())
    }

    /// Create a directory within the capability's root
    pub fn create_dir<P: AsRef<Path>>(&mut self, path: P) -> Result<(), FsError> {
        self.ensure_not_committed()?;
        self.capability.require_write()?;

        let dir = self.capability.resolve_for_creation(path.as_ref())?;

        if dir.exists() {
            return Err(FsError::AlreadyExists(dir));
        }

        fs::create_dir(&dir)?;
        self.journal.push(FsOp::CreatedDir(dir));

        Ok(())
    }

    /// Create a directory and all parent directories within the capability's root
    pub fn create_dir_all<P: AsRef<Path>>(&mut self, path: P) -> Result<(), FsError> {
        self.ensure_not_committed()?;
        self.capability.require_write()?;

        let path = path.as_ref();
        let mut current = PathBuf::new();
        let mut created = Vec::new();

        for component in path.components() {
            current.push(component);

            // Check if this path exists
            let full = match self.capability.resolve(&current) {
                Ok(p) => p,
                Err(CapabilityError::PathNotFound(_)) => {
                    // Need to create it
                    let p = self.capability.resolve_for_creation(&current)?;
                    fs::create_dir(&p)?;
                    created.push(p.clone());
                    p
                }
                Err(e) => return Err(e.into()),
            };

            if !full.is_dir() {
                return Err(FsError::NotADirectory(full));
            }
        }

        // Record all created directories
        for dir in created {
            self.journal.push(FsOp::CreatedDir(dir));
        }

        Ok(())
    }

    /// Delete a file within the capability's root
    ///
    /// The file is first backed up, allowing rollback if needed.
    pub fn delete_file<P: AsRef<Path>>(&mut self, path: P) -> Result<(), FsError> {
        self.ensure_not_committed()?;
        self.capability.require_delete()?;

        let file = self.capability.resolve(path.as_ref())?;

        if !file.is_file() {
            return Err(FsError::NotAFile(file));
        }

        // Create backup
        let backup = self.temp_path(&file);
        fs::rename(&file, &backup)?;

        self.journal.push(FsOp::PendingDelete {
            path: file,
            backup,
        });

        Ok(())
    }

    /// Delete a directory and all its contents within the capability's root
    pub fn delete_dir_all<P: AsRef<Path>>(&mut self, path: P) -> Result<(), FsError> {
        self.ensure_not_committed()?;
        self.capability.require_delete()?;

        let dir = self.capability.resolve(path.as_ref())?;

        if !dir.is_dir() {
            return Err(FsError::NotADirectory(dir));
        }

        // Create backup location
        let backup = self.temp_path(&dir);
        fs::rename(&dir, &backup)?;

        self.journal.push(FsOp::PendingDelete {
            path: dir,
            backup,
        });

        Ok(())
    }

    /// Write content to a file atomically
    pub fn write_file<P: AsRef<Path>>(&mut self, path: P, content: &[u8]) -> Result<(), FsError> {
        self.ensure_not_committed()?;
        self.capability.require_write()?;

        let dst = self.capability.resolve_for_creation(path.as_ref())?;
        let tmp = self.temp_path(&dst);

        // Write to temp
        {
            let mut file = File::create(&tmp)?;
            file.write_all(content)?;
            file.sync_all()?;
        }

        self.journal.push(FsOp::Created(tmp.clone()));

        // Rename to final
        fs::rename(&tmp, &dst)?;

        // Update journal
        self.journal.pop();
        self.journal.push(FsOp::Created(dst));

        Ok(())
    }

    /// Commit the transaction
    ///
    /// This finalizes all operations. After commit, backups are deleted
    /// and the transaction cannot be rolled back.
    pub fn commit(mut self) -> Result<(), FsError> {
        self.ensure_not_committed()?;

        // Clean up any backups from deletions
        for op in &self.journal {
            if let FsOp::PendingDelete { backup, .. } = op {
                // Best effort cleanup - if backup deletion fails, that's fine
                let _ = if backup.is_dir() {
                    fs::remove_dir_all(backup)
                } else {
                    fs::remove_file(backup)
                };
            }
        }

        self.committed = true;
        Ok(())
    }

    /// Roll back all operations in reverse order
    fn rollback(&mut self) {
        // Process journal in reverse
        while let Some(op) = self.journal.pop() {
            let result = match &op {
                FsOp::Created(path) => {
                    if path.is_dir() {
                        fs::remove_dir_all(path)
                    } else {
                        fs::remove_file(path)
                    }
                }

                FsOp::CreatedDir(path) => fs::remove_dir(path),

                FsOp::Renamed { from, to } => fs::rename(to, from),

                FsOp::PendingDelete { path, backup } => fs::rename(backup, path),

                FsOp::PendingRename { from, to } => fs::rename(to, from),
            };

            if let Err(e) = result {
                // Log rollback failure but continue
                tracing::error!(
                    "Rollback failed for {:?}: {}",
                    op,
                    e
                );
            }
        }
    }

    /// Generate a temporary path for atomic operations
    fn temp_path(&self, original: &Path) -> PathBuf {
        let file_name = original
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "unknown".to_string());

        original.with_file_name(format!(".{}.{}.tmp", file_name, self.id))
    }

    /// Ensure the transaction hasn't been committed yet
    fn ensure_not_committed(&self) -> Result<(), FsError> {
        if self.committed {
            return Err(FsError::AlreadyCommitted);
        }
        Ok(())
    }
}

impl Drop for FsTransaction<'_> {
    fn drop(&mut self) {
        if !self.committed {
            self.rollback();
        }
    }
}

/// Copy a directory recursively within a transaction
pub fn copy_dir_recursive(
    tx: &mut FsTransaction<'_>,
    from: &Path,
    to: &Path,
) -> Result<(), FsError> {
    let cap = tx.capability;
    let src = cap.resolve(from)?;

    if !src.is_dir() {
        return Err(FsError::NotADirectory(src));
    }

    // Create destination directory
    tx.create_dir(to)?;

    // Iterate entries
    for entry in fs::read_dir(&src)? {
        let entry = entry?;
        let entry_name = entry.file_name();
        let entry_name_str = entry_name.to_string_lossy();

        let src_path = from.join(&*entry_name_str);
        let dst_path = to.join(&*entry_name_str);

        if entry.file_type()?.is_dir() {
            copy_dir_recursive(tx, &src_path, &dst_path)?;
        } else {
            tx.copy_file(&src_path, &dst_path)?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use capability::Permissions;
    use tempfile::TempDir;

    #[test]
    fn test_copy_file_atomic() {
        let tmp = TempDir::new().unwrap();
        let src = tmp.path().join("source.txt");
        fs::write(&src, "hello world").unwrap();

        let cap = DirCapability::new(tmp.path(), Permissions::full()).unwrap();
        let mut tx = FsTransaction::new(&cap);

        tx.copy_file("source.txt", "dest.txt").unwrap();
        tx.commit().unwrap();

        assert!(tmp.path().join("dest.txt").exists());
        assert_eq!(
            fs::read_to_string(tmp.path().join("dest.txt")).unwrap(),
            "hello world"
        );
    }

    #[test]
    fn test_rollback_on_drop() {
        let tmp = TempDir::new().unwrap();
        fs::write(tmp.path().join("original.txt"), "original").unwrap();

        let cap = DirCapability::new(tmp.path(), Permissions::full()).unwrap();

        {
            let mut tx = FsTransaction::new(&cap);
            tx.write_file("new.txt", b"new content").unwrap();
            tx.delete_file("original.txt").unwrap();

            // Verify changes are visible
            assert!(tmp.path().join("new.txt").exists());
            assert!(!tmp.path().join("original.txt").exists());

            // Drop without commit
        }

        // Changes should be rolled back
        assert!(!tmp.path().join("new.txt").exists());
        assert!(tmp.path().join("original.txt").exists());
        assert_eq!(
            fs::read_to_string(tmp.path().join("original.txt")).unwrap(),
            "original"
        );
    }

    #[test]
    fn test_create_dir() {
        let tmp = TempDir::new().unwrap();
        let cap = DirCapability::new(tmp.path(), Permissions::full()).unwrap();
        let mut tx = FsTransaction::new(&cap);

        tx.create_dir("subdir").unwrap();
        tx.commit().unwrap();

        assert!(tmp.path().join("subdir").is_dir());
    }

    #[test]
    fn test_create_dir_all() {
        let tmp = TempDir::new().unwrap();
        let cap = DirCapability::new(tmp.path(), Permissions::full()).unwrap();
        let mut tx = FsTransaction::new(&cap);

        tx.create_dir_all("a/b/c/d").unwrap();
        tx.commit().unwrap();

        assert!(tmp.path().join("a/b/c/d").is_dir());
    }

    #[test]
    fn test_move_file() {
        let tmp = TempDir::new().unwrap();
        fs::write(tmp.path().join("src.txt"), "content").unwrap();

        let cap = DirCapability::new(tmp.path(), Permissions::full()).unwrap();
        let mut tx = FsTransaction::new(&cap);

        tx.move_file("src.txt", "dst.txt").unwrap();
        tx.commit().unwrap();

        assert!(!tmp.path().join("src.txt").exists());
        assert!(tmp.path().join("dst.txt").exists());
    }

    #[test]
    fn test_move_file_rollback() {
        let tmp = TempDir::new().unwrap();
        fs::write(tmp.path().join("src.txt"), "content").unwrap();

        let cap = DirCapability::new(tmp.path(), Permissions::full()).unwrap();

        {
            let mut tx = FsTransaction::new(&cap);
            tx.move_file("src.txt", "dst.txt").unwrap();
            // Drop without commit
        }

        // Move should be rolled back
        assert!(tmp.path().join("src.txt").exists());
        assert!(!tmp.path().join("dst.txt").exists());
    }

    #[test]
    fn test_delete_file_rollback() {
        let tmp = TempDir::new().unwrap();
        fs::write(tmp.path().join("file.txt"), "important").unwrap();

        let cap = DirCapability::new(tmp.path(), Permissions::full()).unwrap();

        {
            let mut tx = FsTransaction::new(&cap);
            tx.delete_file("file.txt").unwrap();
            assert!(!tmp.path().join("file.txt").exists());
            // Drop without commit
        }

        // File should be restored
        assert!(tmp.path().join("file.txt").exists());
        assert_eq!(
            fs::read_to_string(tmp.path().join("file.txt")).unwrap(),
            "important"
        );
    }

    #[test]
    fn test_permission_check() {
        let tmp = TempDir::new().unwrap();
        fs::write(tmp.path().join("file.txt"), "content").unwrap();

        let cap = DirCapability::new(tmp.path(), Permissions::read_only()).unwrap();
        let mut tx = FsTransaction::new(&cap);

        // Should fail - no write permission
        let result = tx.write_file("new.txt", b"content");
        assert!(result.is_err());
    }

    #[test]
    fn test_copy_dir_recursive() {
        let tmp = TempDir::new().unwrap();

        // Create source structure
        fs::create_dir(tmp.path().join("src")).unwrap();
        fs::create_dir(tmp.path().join("src/sub")).unwrap();
        fs::write(tmp.path().join("src/file1.txt"), "file1").unwrap();
        fs::write(tmp.path().join("src/sub/file2.txt"), "file2").unwrap();

        let cap = DirCapability::new(tmp.path(), Permissions::full()).unwrap();
        let mut tx = FsTransaction::new(&cap);

        copy_dir_recursive(&mut tx, Path::new("src"), Path::new("dst")).unwrap();
        tx.commit().unwrap();

        assert!(tmp.path().join("dst").is_dir());
        assert!(tmp.path().join("dst/sub").is_dir());
        assert_eq!(
            fs::read_to_string(tmp.path().join("dst/file1.txt")).unwrap(),
            "file1"
        );
        assert_eq!(
            fs::read_to_string(tmp.path().join("dst/sub/file2.txt")).unwrap(),
            "file2"
        );
    }
}
