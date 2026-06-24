<!--
SPDX-License-Identifier: CC-BY-SA-4.0
Copyright (c) Jonathan D.A. Jewell <j.d.a.jewell@open.ac.uk>
-->
# TEST-NEEDS.md — polysafe-gitfixer

## CRG Grade: C — ACHIEVED 2026-04-04

> Generated 2026-03-29 by punishing audit.

## Current State

| Category     | Count | Notes |
|-------------|-------|-------|
| Unit tests   | 0     | No inline tests, no test files |
| Integration  | 0     | None |
| E2E          | 0     | None |
| Benchmarks   | 0     | None |

**Source modules:** 6 Rust source files across 4 crates: capability (audit_log.rs, dir_capability.rs, lib.rs), fs_ops (lib.rs), git_ops (lib.rs), polysafe_nifs (lib.rs).

## What's Missing

### P2P (Property-Based) Tests
- [ ] Dir capability: property tests for capability creation/verification invariants
- [ ] Audit log: property tests for log entry integrity
- [ ] fs_ops: property tests for filesystem operation safety (no escaping sandbox)
- [ ] git_ops: property tests for git operation correctness

### E2E Tests
- [ ] Full fix cycle: detect issue -> create capability -> apply fix -> audit -> verify
- [ ] Git operation: clone -> modify -> commit -> verify integrity
- [ ] Capability lifecycle: create -> use -> revoke -> verify revoked

### Aspect Tests
- **Security:** A git fixing tool with capabilities and audit logging has ZERO security tests. Capability bypass, audit log tampering, path traversal in fs_ops, git injection — ALL untested
- **Performance:** No benchmarks for fix throughput
- **Concurrency:** No tests for concurrent fix operations, capability contention
- **Error handling:** No tests for git operation failure, filesystem permission denied, corrupted audit log

### Build & Execution
- [ ] `cargo test` across all 4 crates

### Benchmarks Needed
- [ ] Git operation speed
- [ ] Capability validation overhead
- [ ] Audit logging throughput

### Self-Tests
- [ ] Fix its own repository as smoke test
- [ ] Capability system self-test
- [ ] Audit log integrity verification

## Priority

**CRITICAL.** 6 source files, ZERO tests of any kind. A capability-based security tool with an audit log that has never been tested. The capability and audit_log modules are security-critical and completely unverified. This is one of the worst test situations in the entire scan.

## FAKE-FUZZ ALERT

- `tests/fuzz/placeholder.txt` is a scorecard placeholder inherited from rsr-template-repo — it does NOT provide real fuzz testing
- Replace with an actual fuzz harness (see rsr-template-repo/tests/fuzz/README.adoc) or remove the file
- Priority: P2 — creates false impression of fuzz coverage
