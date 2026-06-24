<!--
SPDX-License-Identifier: MPL-2.0
Copyright (c) Jonathan D.A. Jewell <j.d.a.jewell@open.ac.uk>
-->
# PROOF-NEEDS.md — polysafe-gitfixer

## Current State

- **src/abi/*.idr**: NO
- **Dangerous patterns**: 0
- **LOC**: ~1,400 (Rust + Elixir NIFs)
- **ABI layer**: Missing

## What Needs Proving

| Component | What | Why |
|-----------|------|-----|
| Capability system | Capability grants are minimal and non-escalating | Over-privileged operations can damage repositories |
| Git operations | Git modifications preserve repository integrity | Corrupting git repos is catastrophic |
| File system operations | FS ops respect capability boundaries | Escaping sandbox damages the host system |
| NIF safety | Elixir NIF bridge does not corrupt BEAM VM memory | NIF bugs crash the entire Erlang VM |

## Recommended Prover

**Idris2** — Create `src/abi/` with capability types (indexed by permission set). Git operation correctness proofs would ensure repo integrity is preserved.

## Priority

**MEDIUM** — Git repository fixer that modifies repos. The capability system is the most important proof target — it bounds what the tool can do. Small codebase makes full coverage achievable.
