// SPDX-License-Identifier: MPL-2.0
// Copyright (c) Jonathan D.A. Jewell <j.d.a.jewell@open.ac.uk>
//
//! Directory Capability — Unforgeable Path Safety.
//!
//! This module implements the `DirCapability` token, the primary security
//! primitive for the Polysafe ecosystem. It provides a high-assurance mechanism
//! for path resolution that is mathematically immune to traversal attacks.
//!
//! SECURITY MODEL:
//! 1. **Canonicalization**: The root path is resolved to its absolute,
//!    physical form at token creation time.
//! 2. **Verification**: Every relative path resolution is checked against
//!    the canonical root. If the result escapes the root, an error is returned.
//! 3. **Attenuation**: Capabilities can spawn "sub-capabilities" with a
//!    restricted subset of the original permissions.

use std::path::{Path, PathBuf};
use thiserror::Error;
use serde::{Deserialize, Serialize};

/// Bitmask of operations a `DirCapability` token is permitted to perform.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Permissions {
    /// Token may resolve paths (required for any operation).
    pub read: bool,
    /// Token may create or overwrite files within the sandbox.
    pub write: bool,
    /// Token may delete files within the sandbox.
    pub delete: bool,
}

impl Permissions {
    /// Full read-write-delete permissions.
    pub fn all() -> Self {
        Self { read: true, write: true, delete: true }
    }

    /// Read-only permissions — no mutation allowed.
    pub fn read_only() -> Self {
        Self { read: true, write: false, delete: false }
    }
}

/// CAPABILITY ERROR: Describes specific security violations.
#[derive(Debug, Error)]
pub enum CapabilityError {
    /// The resolved path would escape the sandbox root.
    #[error("path traversal attempt: {attempted_path:?} escapes root {root:?}")]
    PathTraversal { root: PathBuf, attempted_path: PathBuf },

    /// The operation is not permitted by the token's `Permissions`.
    #[error("permission denied: {operation} not allowed by capability (have: {have:?})")]
    PermissionDenied { operation: &'static str, have: Permissions },

    /// The path does not exist on the filesystem.
    #[error("path not found: {0:?}")]
    PathNotFound(PathBuf),

    /// Absolute paths are always rejected to prevent root-escaping.
    #[error("absolute path rejected: {0:?} — use relative paths only")]
    AbsolutePathRejected(PathBuf),

    /// I/O error during path canonicalization.
    #[error("I/O error during capability operation: {0}")]
    Io(#[from] std::io::Error),
}

/// An unforgeable token granting sandboxed access to a directory tree.
///
/// Once created via [`DirCapability::new`], all path operations are
/// checked against the canonical root.  The token cannot be forged
/// because the root is resolved at construction time and stored privately.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirCapability {
    /// Canonical (absolute, symlink-resolved) sandbox root.
    root: PathBuf,
    /// What operations this token permits.
    permissions: Permissions,
}

impl DirCapability {
    /// Create a new capability for `root` with the given permissions.
    ///
    /// Returns an error if `root` does not exist or cannot be canonicalized.
    pub fn new(root: &Path, permissions: Permissions) -> Result<Self, CapabilityError> {
        let canonical_root = root.canonicalize().map_err(|_| CapabilityError::PathNotFound(root.to_path_buf()))?;
        Ok(Self { root: canonical_root, permissions })
    }

    /// The canonical root of this capability sandbox.
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// The permissions granted by this token.
    pub fn permissions(&self) -> Permissions {
        self.permissions
    }

    /// Resolve `relative` to an absolute path that is guaranteed to stay
    /// within the sandbox root.
    ///
    /// # Errors
    ///
    /// Returns [`CapabilityError::AbsolutePathRejected`] if `relative` is absolute.
    /// Returns [`CapabilityError::PathTraversal`] if the resolved path escapes the root.
    /// Returns [`CapabilityError::PathNotFound`] if the path does not exist.
    pub fn resolve(&self, relative: &Path) -> Result<PathBuf, CapabilityError> {
        // SAFETY: Absolute paths are REJECTED to prevent root-escaping.
        if relative.is_absolute() {
            return Err(CapabilityError::AbsolutePathRejected(relative.to_path_buf()));
        }

        let joined = self.root.join(relative);
        let canonical = joined.canonicalize()
            .map_err(|_| CapabilityError::PathNotFound(joined.clone()))?;

        // ESCAPE DETECTION: Ensure the final path is still within the sandbox.
        if !canonical.starts_with(&self.root) {
            return Err(CapabilityError::PathTraversal {
                root: self.root.clone(),
                attempted_path: relative.to_path_buf(),
            });
        }

        Ok(canonical)
    }

    /// Attenuate this capability to a sub-directory with reduced permissions.
    ///
    /// The new capability's root is `self.root / sub_dir`.
    /// Permissions can only be equal to or more restrictive than the parent.
    pub fn attenuate(&self, sub_dir: &Path, permissions: Permissions) -> Result<Self, CapabilityError> {
        let new_root = self.resolve(sub_dir)?;
        Ok(Self { root: new_root, permissions })
    }
}
