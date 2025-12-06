# polysafe-gitfixer

A polyglot implementation of a git backup merger tool, where each component is written in the language that provides the strongest safety guarantees for that component's concerns.

## Overview

This tool:
1. Scans a directory tree for git repositories
2. Finds backup directories (`*-backup`, `*.backup-*`)
3. Matches backups to their corresponding repos
4. Diffs backup vs repo contents
5. Offers interactive merge/replace/delete options
6. Maintains append-only audit log
7. Handles failures gracefully (supervision)

## Architecture

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                              COMPONENT MAP                                   │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│   ┌─────────────────┐         ┌─────────────────┐                          │
│   │     Haskell     │         │     Nickel      │                          │
│   │    TUI/CLI      │◄────────│     Config      │                          │
│   │     (Brick)     │         │    (schemas)    │                          │
│   └────────┬────────┘         └─────────────────┘                          │
│            │                                                                │
│            ▼                                                                │
│   ┌─────────────────────────────────────────────────────────────┐          │
│   │                    Elixir/OTP                                │          │
│   │              Orchestration & Supervision                     │          │
│   └───┬─────────────────┬─────────────────┬─────────────────┬───┘          │
│       │                 │                 │                 │              │
│       ▼                 ▼                 ▼                 ▼              │
│  ┌─────────┐      ┌──────────┐     ┌──────────┐      ┌──────────┐         │
│  │  Idris  │      │ Haskell  │     │   Rust   │      │   Rust   │         │
│  │Workflow │      │   Diff   │     │   Git    │      │   F/S    │         │
│  │  State  │      │  Engine  │     │   Ops    │      │   Ops    │         │
│  └─────────┘      └──────────┘     └──────────┘      └──────────┘         │
│                                                                            │
│                    ┌──────────────────────────────┐                        │
│                    │           Rust               │                        │
│                    │   Capability & Audit Layer   │                        │
│                    └──────────────────────────────┘                        │
└─────────────────────────────────────────────────────────────────────────────┘
```

## Components

| Component | Language | Safety Guarantee |
|-----------|----------|------------------|
| Configuration | Nickel | Schema validation, typed defaults |
| Capability & Audit | Rust | Path traversal prevention, tamper-evident logging |
| Filesystem Ops | Rust | RAII, atomic transactions, rollback on failure |
| Git Operations | Rust | Error handling, effect tracking |
| Diff Engine | Haskell | Totality, streaming for large files |
| Workflow State | Idris 2 | Typestate (can't call operations in wrong order) |
| Orchestration | Elixir/OTP | Fault isolation, supervision trees |
| TUI/CLI | Haskell (Brick) | Elm Architecture, exhaustive event handling |

## Building

### Prerequisites

- Rust (1.75+)
- Haskell (GHC 9.4+, Cabal 3.8+)
- Elixir (1.15+, OTP 26+)
- Nickel (1.4+)
- Idris 2 (0.6+) - optional, Haskell fallback available

### Build All

```bash
make all
```

### Build Individual Components

```bash
# Rust crates
make rust

# Haskell components
make haskell

# Elixir orchestrator
make elixir
```

## Project Structure

```
polysafe-gitfixer/
├── config/           # Nickel configuration schemas
├── crates/           # Rust components
│   ├── capability/   # Path safety & audit logging
│   ├── fs_ops/       # Transactional filesystem operations
│   ├── git_ops/      # Git repository operations
│   └── polysafe_nifs/# Rustler NIFs for Elixir
├── haskell/          # Haskell components
│   ├── diff-engine/  # Tree/file diffing
│   └── tui/          # Terminal UI
├── idris/            # Idris 2 workflow state machine
├── elixir/           # Elixir orchestrator
└── test/             # Integration tests
```

## License

MIT OR Apache-2.0
