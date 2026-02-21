<!-- SPDX-License-Identifier: PMPL-1.0-or-later -->
<!-- TOPOLOGY.md — Project architecture map and completion dashboard -->
<!-- Last updated: 2026-02-19 -->

# polysafe-gitfixer — Project Topology

## System Architecture

```
                        ┌─────────────────────────────────────────┐
                        │              OPERATOR / CLI             │
                        │        (Haskell TUI / Brick)            │
                        └───────────────────┬─────────────────────┘
                                            │
                                            ▼
                        ┌─────────────────────────────────────────┐
                        │           ELIXIR ORCHESTRATOR           │
                        │    (OTP Supervision, Command Routing)    │
                        └──────────┬───────────────────┬──────────┘
                                   │                   │
                                   ▼                   ▼
                        ┌───────────────────────┐  ┌────────────────────────────────┐
                        │ WORKFLOW STATE (IDRIS)│  │ DIFF ENGINE (HASKELL)          │
                        │ - Dependent Typestate │  │ - Totality Checked             │
                        │ - Transition Proofs   │  │ - Streaming Tree Diff          │
                        └──────────┬────────────┘  └──────────┬─────────────────────┘
                                   │                          │
                                   └────────────┬─────────────┘
                                                ▼
                        ┌─────────────────────────────────────────┐
                        │           RUST SAFETY LAYER             │
                        │  ┌───────────┐  ┌───────────────────┐  │
                        │  │ Git Ops   │  │  Filesystem Ops   │  │
                        │  │ (git2-rs) │  │  (Transactional)  │  │
                        │  └─────┬─────┘  └────────┬──────────┘  │
                        │        │                 │              │
                        │  ┌─────▼─────┐  ┌────────▼──────────┐  │
                        │  │ Capability│  │  Audit Layer      │  │
                        │  │ (Safety)  │  │  (Append-only)    │  │
                        │  └─────┬─────┘  └────────┬──────────┘  │
                        └────────│─────────────────│──────────────┘
                                 │                 │
                                 ▼                 ▼
                        ┌─────────────────────────────────────────┐
                        │          TARGET GIT REPOSITORIES        │
                        │      (Backups, Mirrors, Workspace)      │
                        └─────────────────────────────────────────┘

                        ┌─────────────────────────────────────────┐
                        │          REPO INFRASTRUCTURE            │
                        │  Justfile / Mustfile  .machine_readable/  │
                        │  Nickel Configs       RSR Gold (Cert)     │
                        └─────────────────────────────────────────┘
```

## Completion Dashboard

```
COMPONENT                          STATUS              NOTES
─────────────────────────────────  ──────────────────  ─────────────────────────────────
CORE COMPONENTS
  Elixir Orchestrator               ██████████ 100%    OTP supervision stable
  Rust Git Ops (git2-rs)            ██████████ 100%    Effect tracking verified
  Rust Filesystem Ops               ██████████ 100%    Atomic transactions stable
  Haskell Diff Engine               ██████████ 100%    Streaming totality verified

INTERFACE & STATE
  Haskell TUI (Brick)               ████████░░  80%    Event handling refined
  Idris 2 Workflow State            ██████████ 100%    Typestate transitions proven
  Nickel Config Schemas             ██████████ 100%    Typed defaults verified

REPO INFRASTRUCTURE
  Justfile Automation               ██████████ 100%    Standard build/test tasks
  .machine_readable/                ██████████ 100%    STATE tracking active
  Polyglot Integration              ██████████ 100%    NIF/C ABI boundaries verified

─────────────────────────────────────────────────────────────────────────────
OVERALL:                            █████████░  ~95%   Production-grade tool stable
```

## Key Dependencies

```
Nickel Config ───► Idris State ────► Elixir Supervisor ───► Rust FS Ops
     │                 │                   │                    │
     ▼                 ▼                   ▼                    ▼
Haskell TUI ◄───► Haskell Diff ◄───► Rust Git Ops ────────► Repository
```

## Update Protocol

This file is maintained by both humans and AI agents. When updating:

1. **After completing a component**: Change its bar and percentage
2. **After adding a component**: Add a new row in the appropriate section
3. **After architectural changes**: Update the ASCII diagram
4. **Date**: Update the `Last updated` comment at the top of this file

Progress bars use: `█` (filled) and `░` (empty), 10 characters wide.
Percentages: 0%, 10%, 20%, ... 100% (in 10% increments).
