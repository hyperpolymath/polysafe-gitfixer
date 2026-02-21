// SPDX-License-Identifier: MIT AND Palimpsest-0.8

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

/// CAPABILITY ERROR: Describes specific security violations.
#[derive(Debug, Error)]
pub enum CapabilityError {
    #[error("path traversal attempt: {attempted_path:?} escapes root {root:?}")]
    PathTraversal { root: PathBuf, attempted_path: PathBuf },
    #[error("permission denied: {operation} not allowed")]
    PermissionDenied { operation: &'static str, have: Permissions },
    // ... [other errors]
}

/// RESOLVER: Transforms a relative path into a safe, absolute PathBuf.
impl DirCapability {
    pub fn resolve(&self, relative: &Path) -> Result<PathBuf, CapabilityError> {
        // SAFETY: Absolute paths are REJECTED to prevent root-escaping.
        if relative.is_absolute() {
            return Err(CapabilityError::PathTraversal { root: self.root.clone(), attempted_path: relative.to_path_buf() });
        }

        let joined = self.root.join(relative);
        let canonical = joined.canonicalize().map_err(|_| CapabilityError::PathNotFound(joined.clone()))?;

        // ESCAPE DETECTION: Ensure the final path is still within the sandbox.
        if !canonical.starts_with(&self.root) {
            return Err(CapabilityError::PathTraversal { root: self.root.clone(), attempted_path: relative.to_path_buf() });
        }

        Ok(canonical)
    }
}
