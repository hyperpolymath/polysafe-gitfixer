;; -*- mode: scheme -*-
;; SPDX-License-Identifier: MIT AND Palimpsest-0.8
;; STATE.scm - Project state checkpoint for polysafe-gitfixer
;; Download at session end, upload at session start to resume context

(define state
  '((metadata
     . ((format-version . "0.2.0")
        (project . "polysafe-gitfixer")
        (updated . "2025-12-08T00:00:00Z")
        (generator . "Claude/STATE-system")))

    (user-context
     . ((name . "Jonathan D.A. Jewell")
        (handle . "@Hyperpolymath")
        (roles . ("architect" "rust-developer" "security-engineer"))
        (languages . ("Rust" "Elixir" "Haskell" "Idris"))
        (tools . ("GitLab" "GitHub" "Podman" "Guix"))
        (values . ("FOSS" "formal-verification" "capability-security" "polyglot"))))

    (session-context
     . ((conversation-id . "01RrCvazrZbt5T3GxdtZfUrq")
        (branch . "claude/create-state-scm-01RrCvazrZbt5T3GxdtZfUrq")))

    ;; =========================================================================
    ;; CURRENT POSITION
    ;; =========================================================================
    (current-position
     . ((summary . "Rust foundation crates implemented with core security primitives")
        (phase . "foundation")
        (completion-percent . 25)

        (implemented
         . (("capability" . "DirCapability with path traversal prevention, Permissions model")
            ("capability" . "AuditLog with hash-chained tamper-evident logging")
            ("fs_ops" . "FsTransaction with atomic ops, journaling, RAII rollback")
            ("git_ops" . "Safe git2 wrappers: status, staging, repo discovery, remotes")
            ("polysafe_nifs" . "Placeholder module structure for Rustler NIFs")))

        (not-implemented
         . (("polysafe_nifs" . "Actual NIF function implementations (commented out)")
            ("orchestrator" . "Elixir supervision tree and workflow engine")
            ("diff-engine" . "Haskell semantic diff for polyglot merge")
            ("tui" . "Haskell brick/vty terminal UI")
            ("workflow" . "Idris verified merge strategies")
            ("cli" . "User-facing command-line interface")
            ("backup-detection" . "Auto-detect backup files and match to repos")))))

    ;; =========================================================================
    ;; ROUTE TO MVP v1
    ;; =========================================================================
    (mvp-v1-roadmap
     . ((goal . "CLI tool that safely merges git backup files into working repos")
        (target-features
         . ("Detect backup .git directories"
            "Match backups to source repositories"
            "Interactive diff review"
            "Safe merge with rollback"
            "Audit trail of all operations"))

        (phases
         . (((phase . 1)
             (name . "Complete Rust Foundation")
             (status . "in-progress")
             (completion . 85)
             (tasks
              . (("Implement actual NIF bindings" . "pending")
                 ("Add chrono to workspace deps" . "pending")
                 ("Add uuid to workspace deps" . "pending")
                 ("Add hex crate or keep inline" . "pending")
                 ("Integration tests across crates" . "pending"))))

            ((phase . 2)
             (name . "Elixir Orchestrator")
             (status . "pending")
             (completion . 0)
             (tasks
              . (("Initialize mix project" . "pending")
                 ("Design supervision tree" . "pending")
                 ("Integrate Rustler NIFs" . "pending")
                 ("Implement GenServer workers" . "pending")
                 ("Add telemetry/logging" . "pending"))))

            ((phase . 3)
             (name . "Backup Detection Engine")
             (status . "pending")
             (completion . 0)
             (tasks
              . (("Scan filesystem for .git dirs" . "pending")
                 ("Fingerprint repos by remote URLs" . "pending")
                 ("Match backups to working copies" . "pending")
                 ("Handle orphaned backups" . "pending"))))

            ((phase . 4)
             (name . "Diff & Merge Core")
             (status . "pending")
             (completion . 0)
             (tasks
              . (("Compare commit histories" . "pending")
                 ("Identify divergent branches" . "pending")
                 ("Generate merge candidates" . "pending")
                 ("3-way merge implementation" . "pending"))))

            ((phase . 5)
             (name . "CLI Interface")
             (status . "pending")
             (completion . 0)
             (tasks
              . (("Argument parsing (clap)" . "pending")
                 ("Interactive prompts" . "pending")
                 ("Progress reporting" . "pending")
                 ("JSON output mode" . "pending"))))))))

    ;; =========================================================================
    ;; KNOWN ISSUES & BLOCKERS
    ;; =========================================================================
    (issues
     . (((id . "ISS-001")
         (severity . "low")
         (title . "Missing workspace dependencies")
         (description . "chrono, uuid, hex used in crates but not declared in workspace Cargo.toml")
         (affected . ("capability/audit_log" "fs_ops/transaction")))

        ((id . "ISS-002")
         (severity . "medium")
         (title . "NIF bindings are placeholder only")
         (description . "polysafe_nifs has all NIF code commented out, awaiting Elixir side")
         (blocked-by . "Elixir orchestrator project must be created first"))

        ((id . "ISS-003")
         (severity . "low")
         (title . "SECURITY.md is GitHub template")
         (description . "Security policy file contains placeholder text, not actual policy"))

        ((id . "ISS-004")
         (severity . "low")
         (title . "No integration tests")
         (description . "Each crate has unit tests but no cross-crate integration tests"))

        ((id . "ISS-005")
         (severity . "info")
         (title . "Repository URL mismatch")
         (description . "Cargo.toml points to GitLab but code is also on GitHub"))))

    ;; =========================================================================
    ;; QUESTIONS FOR MAINTAINER
    ;; =========================================================================
    (questions
     . (((id . "Q-001")
         (topic . "Architecture")
         (question . "Should MVP be pure Rust CLI, or commit to Elixir orchestration?")
         (context . "Elixir adds complexity but enables better supervision/fault-tolerance")
         (options . ("Pure Rust with tokio" "Elixir + Rust NIFs" "Hybrid: Rust CLI + optional Elixir")))

        ((id . "Q-002")
         (topic . "Platform Priority")
         (question . "Which platforms should MVP target?")
         (options . ("Linux-only" "Linux + macOS" "Linux + macOS + Windows"))
         (notes . "Windows git paths and symlinks differ significantly"))

        ((id . "Q-003")
         (topic . "Backup Format Support")
         (question . "What backup formats should MVP support?")
         (context . "Different backup tools create different structures")
         (options . ("Raw .git directories only"
                     "Bare repos"
                     "Git bundles"
                     "tar/zip archives"
                     "All of the above")))

        ((id . "Q-004")
         (topic . "UI Priority")
         (question . "MVP interface priority?")
         (options . ("CLI-only" "CLI + TUI" "CLI + Web UI")))

        ((id . "Q-005")
         (topic . "Diff Engine")
         (question . "Start Haskell diff-engine now or defer?")
         (context . "Could use basic byte-diff for MVP, semantic diff for v2")
         (tradeoff . "Faster MVP vs better merge quality"))))

    ;; =========================================================================
    ;; LONG-TERM ROADMAP
    ;; =========================================================================
    (roadmap
     . ((v1-0
         . ((codename . "Archivist")
            (goal . "Basic backup merger CLI")
            (features . ("Backup detection"
                         "Repo matching"
                         "Interactive merge"
                         "Audit logging"
                         "Transactional safety"))))

        (v1-5
         . ((codename . "Curator")
            (goal . "Enhanced intelligence")
            (features . ("Semantic diff (Haskell)")
                        "Auto-resolve simple conflicts"
                        "Branch topology visualization"
                        "Backup scheduling recommendations"))))

        (v2-0
         . ((codename . "Librarian")
            (goal . "Full polyglot stack")
            (features . ("TUI with brick/vty"
                         "Verified merge strategies (Idris)"
                         "Plugin system for custom resolvers"
                         "Distributed backup coordination"))))

        (future
         . ((ideas . ("Cloud backup integration"
                      "Git LFS handling"
                      "Submodule-aware merging"
                      "Time-travel debugging for merges"
                      "AI-assisted conflict resolution"))))))

    ;; =========================================================================
    ;; PROJECT CATALOG (for STATE.scm standard compatibility)
    ;; =========================================================================
    (projects
     . (((name . "capability-crate")
         (status . "complete")
         (completion . 100)
         (category . "foundation")
         (next-steps . ()))

        ((name . "fs_ops-crate")
         (status . "complete")
         (completion . 100)
         (category . "foundation")
         (next-steps . ()))

        ((name . "git_ops-crate")
         (status . "complete")
         (completion . 100)
         (category . "foundation")
         (next-steps . ()))

        ((name . "polysafe_nifs-crate")
         (status . "blocked")
         (completion . 10)
         (category . "integration")
         (blockers . ("Elixir orchestrator not started"))
         (next-steps . ("Create Elixir mix project" "Implement NIF functions")))

        ((name . "elixir-orchestrator")
         (status . "pending")
         (completion . 0)
         (category . "orchestration")
         (next-steps . ("Initialize mix project" "Design supervision tree")))

        ((name . "haskell-diff-engine")
         (status . "pending")
         (completion . 0)
         (category . "algorithms")
         (next-steps . ("Evaluate: needed for MVP or v2?")))

        ((name . "cli-interface")
         (status . "pending")
         (completion . 0)
         (category . "user-interface")
         (next-steps . ("Decide: Rust clap or Elixir escript")))))

    ;; =========================================================================
    ;; VELOCITY HISTORY (for burndown tracking)
    ;; =========================================================================
    (history
     . (((date . "2024-12-01")
         (snapshot . (("capability-crate" . 100)
                      ("fs_ops-crate" . 100)
                      ("git_ops-crate" . 100)
                      ("polysafe_nifs-crate" . 10)
                      ("elixir-orchestrator" . 0))))

        ((date . "2025-12-08")
         (snapshot . (("capability-crate" . 100)
                      ("fs_ops-crate" . 100)
                      ("git_ops-crate" . 100)
                      ("polysafe_nifs-crate" . 10)
                      ("elixir-orchestrator" . 0)))
         (notes . "Initial STATE.scm creation"))))

    ;; =========================================================================
    ;; CRITICAL NEXT ACTIONS
    ;; =========================================================================
    (next-actions
     . (((priority . 1)
         (action . "Decide: Pure Rust vs Elixir orchestration for MVP")
         (owner . "@Hyperpolymath")
         (context . "Blocks all further architecture decisions"))

        ((priority . 2)
         (action . "Fix workspace Cargo.toml dependencies")
         (owner . "any")
         (context . "Add chrono, uuid, hex to workspace.dependencies"))

        ((priority . 3)
         (action . "Write integration tests for Rust crates")
         (owner . "any")
         (context . "Verify capability + fs_ops + git_ops work together"))

        ((priority . 4)
         (action . "Update SECURITY.md with real policy")
         (owner . "@Hyperpolymath")
         (context . "Replace GitHub template with actual security contact info"))))))

;; Query helpers (for minikanren-style queries if using Guile)
;; (define (project-status name state)
;;   (assoc-ref (assoc-ref state 'projects) name))
