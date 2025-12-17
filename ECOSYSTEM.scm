;; SPDX-License-Identifier: AGPL-3.0-or-later
;; SPDX-FileCopyrightText: 2025 Jonathan D.A. Jewell
;; ECOSYSTEM.scm — polysafe-gitfixer

(ecosystem
  (version "1.0.0")
  (name "polysafe-gitfixer")
  (type "project")
  (purpose "Polyglot git backup merger with maximum safety guarantees - scans directory trees for git repos, finds backup directories, offers interactive merge/replace/delete operations with capability-based security and append-only audit logging.")

  (position-in-ecosystem
    "Part of hyperpolymath ecosystem. Follows RSR guidelines.")

  (related-projects
    (project (name "rhodium-standard-repositories")
             (url "https://github.com/hyperpolymath/rhodium-standard-repositories")
             (relationship "standard")))

  (what-this-is
    "A safety-first tool for reconciling git repositories with their backups, using Rust for core operations, Haskell for diffing, Elixir for orchestration, and Idris for verified workflows.")

  (what-this-is-not
    "- NOT exempt from RSR compliance
     - NOT a general-purpose backup tool
     - NOT a replacement for git itself"))
