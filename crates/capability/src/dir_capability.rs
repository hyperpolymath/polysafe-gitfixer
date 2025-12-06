//! Directory capability - unforgeable tokens for safe path resolution

use std::path::{Path, PathBuf};
use thiserror::Error;
use serde::{Deserialize, Serialize};

/// Errors that can occur during capability operations
#[derive(Debug, Error)]
pub enum CapabilityError {
    #[error("path traversal attempt detected: {attempted_path:?} escapes root {root:?}")]
    PathTraversal {
        root: PathBuf,
        attempted_path: PathBuf,
    },

    #[error("path does not exist: {0}")]
    PathNotFound(PathBuf),

    #[error("permission denied: {operation} not allowed (have: {have:?})")]
    PermissionDenied {
        operation: &'static str,
        have: Permissions,
    },

    #[error("invalid root directory: {0}")]
    InvalidRoot(PathBuf),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

/// Permissions that can be granted to a capability
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Permissions {
    pub read: bool,
    pub write: bool,
    pub delete: bool,
}

impl Permissions {
    /// Full permissions (read, write, delete)
    pub const fn full() -> Self {
        Self {
            read: true,
            write: true,
            delete: true,
        }
    }

    /// Read-only permissions
    pub const fn read_only() -> Self {
        Self {
            read: true,
            write: false,
            delete: false,
        }
    }

    /// Read and write, but no delete
    pub const fn read_write() -> Self {
        Self {
            read: true,
            write: true,
            delete: false,
        }
    }
}

impl Default for Permissions {
    fn default() -> Self {
        Self::read_only()
    }
}

/// An unforgeable capability token that restricts access to a directory tree.
///
/// The capability can only be created from an existing, canonical path. Once
/// created, it can resolve relative paths within its tree, but will reject
/// any path that would escape the root (e.g., via `..` components).
///
/// # Security Model
///
/// - The root path is canonicalized at creation time
/// - All resolved paths are canonicalized and checked against the root
/// - Path traversal via `..` or symlinks outside the root is detected and rejected
/// - Permissions are checked at capability level, not filesystem level
///
/// # Example
///
/// ```no_run
/// use capability::{DirCapability, Permissions};
/// use std::path::Path;
///
/// let cap = DirCapability::new("/home/user/repos", Permissions::read_only())?;
///
/// // This works - path is within the root
/// let path = cap.resolve(Path::new("project/src/main.rs"))?;
///
/// // This fails - path escapes the root
/// let bad = cap.resolve(Path::new("../../../etc/passwd"));
/// assert!(bad.is_err());
/// # Ok::<(), capability::CapabilityError>(())
/// ```
#[derive(Debug, Clone)]
pub struct DirCapability {
    /// The canonical root path (absolute, no symlinks resolved)
    root: PathBuf,
    /// Permissions granted by this capability
    permissions: Permissions,
}

impl DirCapability {
    /// Create a new capability for the given root directory.
    ///
    /// The root must exist and be a directory. It will be canonicalized.
    pub fn new<P: AsRef<Path>>(root: P, permissions: Permissions) -> Result<Self, CapabilityError> {
        let root = root.as_ref();

        // Canonicalize to get absolute path with symlinks resolved
        let canonical = root.canonicalize().map_err(|_| {
            CapabilityError::InvalidRoot(root.to_path_buf())
        })?;

        // Verify it's a directory
        if !canonical.is_dir() {
            return Err(CapabilityError::InvalidRoot(root.to_path_buf()));
        }

        Ok(Self {
            root: canonical,
            permissions,
        })
    }

    /// Get the root path of this capability
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Get the permissions of this capability
    pub fn permissions(&self) -> Permissions {
        self.permissions
    }

    /// Resolve a relative path within this capability's root.
    ///
    /// Returns the canonical absolute path if it's within the root,
    /// or an error if the path would escape the root.
    pub fn resolve(&self, relative: &Path) -> Result<PathBuf, CapabilityError> {
        // Don't allow absolute paths - they must be relative to root
        if relative.is_absolute() {
            return Err(CapabilityError::PathTraversal {
                root: self.root.clone(),
                attempted_path: relative.to_path_buf(),
            });
        }

        // Join with root
        let joined = self.root.join(relative);

        // Canonicalize to resolve any .. or symlinks
        let canonical = joined.canonicalize().map_err(|_| {
            CapabilityError::PathNotFound(joined.clone())
        })?;

        // Verify the canonical path starts with our root
        if !canonical.starts_with(&self.root) {
            return Err(CapabilityError::PathTraversal {
                root: self.root.clone(),
                attempted_path: relative.to_path_buf(),
            });
        }

        Ok(canonical)
    }

    /// Resolve a path, allowing it to not exist yet (for creation).
    ///
    /// This resolves the parent directory and appends the final component.
    /// The parent must exist and be within the root.
    pub fn resolve_for_creation(&self, relative: &Path) -> Result<PathBuf, CapabilityError> {
        if relative.is_absolute() {
            return Err(CapabilityError::PathTraversal {
                root: self.root.clone(),
                attempted_path: relative.to_path_buf(),
            });
        }

        let joined = self.root.join(relative);

        // Get parent and filename
        let parent = joined.parent().ok_or_else(|| {
            CapabilityError::PathNotFound(joined.clone())
        })?;

        let filename = joined.file_name().ok_or_else(|| {
            CapabilityError::PathNotFound(joined.clone())
        })?;

        // Canonicalize the parent (must exist)
        let canonical_parent = parent.canonicalize().map_err(|_| {
            CapabilityError::PathNotFound(parent.to_path_buf())
        })?;

        // Verify parent is within root
        if !canonical_parent.starts_with(&self.root) {
            return Err(CapabilityError::PathTraversal {
                root: self.root.clone(),
                attempted_path: relative.to_path_buf(),
            });
        }

        Ok(canonical_parent.join(filename))
    }

    /// Check if this capability allows reading
    pub fn can_read(&self) -> bool {
        self.permissions.read
    }

    /// Check if this capability allows writing
    pub fn can_write(&self) -> bool {
        self.permissions.write
    }

    /// Check if this capability allows deletion
    pub fn can_delete(&self) -> bool {
        self.permissions.delete
    }

    /// Assert read permission, returning an error if not allowed
    pub fn require_read(&self) -> Result<(), CapabilityError> {
        if !self.permissions.read {
            return Err(CapabilityError::PermissionDenied {
                operation: "read",
                have: self.permissions,
            });
        }
        Ok(())
    }

    /// Assert write permission, returning an error if not allowed
    pub fn require_write(&self) -> Result<(), CapabilityError> {
        if !self.permissions.write {
            return Err(CapabilityError::PermissionDenied {
                operation: "write",
                have: self.permissions,
            });
        }
        Ok(())
    }

    /// Assert delete permission, returning an error if not allowed
    pub fn require_delete(&self) -> Result<(), CapabilityError> {
        if !self.permissions.delete {
            return Err(CapabilityError::PermissionDenied {
                operation: "delete",
                have: self.permissions,
            });
        }
        Ok(())
    }

    /// Create a sub-capability for a directory within this capability's root.
    ///
    /// The sub-capability cannot have more permissions than the parent.
    pub fn subcapability<P: AsRef<Path>>(
        &self,
        subdir: P,
        permissions: Permissions,
    ) -> Result<DirCapability, CapabilityError> {
        let resolved = self.resolve(subdir.as_ref())?;

        if !resolved.is_dir() {
            return Err(CapabilityError::InvalidRoot(resolved));
        }

        // Restrict permissions to what parent has
        let restricted = Permissions {
            read: permissions.read && self.permissions.read,
            write: permissions.write && self.permissions.write,
            delete: permissions.delete && self.permissions.delete,
        };

        Ok(DirCapability {
            root: resolved,
            permissions: restricted,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs;

    #[test]
    fn test_basic_resolution() {
        let tmp = TempDir::new().unwrap();
        let subdir = tmp.path().join("project");
        fs::create_dir(&subdir).unwrap();
        let file = subdir.join("main.rs");
        fs::write(&file, "fn main() {}").unwrap();

        let cap = DirCapability::new(tmp.path(), Permissions::read_only()).unwrap();
        let resolved = cap.resolve(Path::new("project/main.rs")).unwrap();
        assert_eq!(resolved, file.canonicalize().unwrap());
    }

    #[test]
    fn test_path_traversal_rejected() {
        let tmp = TempDir::new().unwrap();
        let cap = DirCapability::new(tmp.path(), Permissions::read_only()).unwrap();

        // Attempt to escape via ..
        let result = cap.resolve(Path::new("../../../etc/passwd"));
        assert!(matches!(result, Err(CapabilityError::PathTraversal { .. })));
    }

    #[test]
    fn test_absolute_path_rejected() {
        let tmp = TempDir::new().unwrap();
        let cap = DirCapability::new(tmp.path(), Permissions::read_only()).unwrap();

        let result = cap.resolve(Path::new("/etc/passwd"));
        assert!(matches!(result, Err(CapabilityError::PathTraversal { .. })));
    }

    #[test]
    fn test_symlink_escape_rejected() {
        let tmp = TempDir::new().unwrap();
        let outer_file = tmp.path().join("outside.txt");
        fs::write(&outer_file, "secret").unwrap();

        let inner_dir = tmp.path().join("inner");
        fs::create_dir(&inner_dir).unwrap();

        // Create symlink that points outside
        #[cfg(unix)]
        {
            use std::os::unix::fs::symlink;
            symlink(&outer_file, inner_dir.join("escape")).unwrap();
        }

        let cap = DirCapability::new(&inner_dir, Permissions::read_only()).unwrap();

        #[cfg(unix)]
        {
            // The symlink should resolve to outside the capability
            let result = cap.resolve(Path::new("escape"));
            assert!(matches!(result, Err(CapabilityError::PathTraversal { .. })));
        }
    }

    #[test]
    fn test_permissions() {
        let tmp = TempDir::new().unwrap();

        let cap = DirCapability::new(tmp.path(), Permissions::read_only()).unwrap();
        assert!(cap.can_read());
        assert!(!cap.can_write());
        assert!(!cap.can_delete());

        let cap = DirCapability::new(tmp.path(), Permissions::full()).unwrap();
        assert!(cap.can_read());
        assert!(cap.can_write());
        assert!(cap.can_delete());
    }

    #[test]
    fn test_subcapability() {
        let tmp = TempDir::new().unwrap();
        let subdir = tmp.path().join("project");
        fs::create_dir(&subdir).unwrap();

        let parent = DirCapability::new(tmp.path(), Permissions::read_write()).unwrap();
        let child = parent.subcapability("project", Permissions::full()).unwrap();

        // Child can't have more permissions than parent
        assert!(!child.can_delete());
        assert!(child.can_read());
        assert!(child.can_write());
    }

    #[test]
    fn test_resolve_for_creation() {
        let tmp = TempDir::new().unwrap();
        let cap = DirCapability::new(tmp.path(), Permissions::full()).unwrap();

        // Should succeed even though file doesn't exist
        let result = cap.resolve_for_creation(Path::new("new_file.txt")).unwrap();
        assert!(result.starts_with(tmp.path()));
        assert_eq!(result.file_name().unwrap(), "new_file.txt");
    }
}
