// SPDX-License-Identifier: PMPL-1.0-or-later
// Copyright (c) 2026 Jonathan D.A. Jewell (hyperpolymath) <j.d.a.jewell@open.ac.uk>
//
// Criterion benchmarks for polysafe-gitfixer.
// Three benchmarks:
//   1. bench_capability_resolve     — hot-path cost of sandbox path resolution.
//   2. bench_fs_transaction_write   — cost of a single-file write transaction.
//   3. bench_git_find_repos         — cost of find_repos on a small directory tree.

use std::fs;
use std::process::Command;
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use capability::{DirCapability, Permissions};
use fs_ops::FsTransaction;
use git_ops::find_repos;

fn scratch() -> tempfile::TempDir {
    tempfile::tempdir().expect("bench: create temp dir")
}

fn init_git_repo(path: &std::path::Path) {
    Command::new("git")
        .args(["init", "--quiet", path.to_str().unwrap()])
        .status()
        .expect("git init");
    for (k, v) in [("user.email", "bench@test.local"), ("user.name", "Bench")] {
        Command::new("git")
            .args(["-C", path.to_str().unwrap(), "config", k, v])
            .status()
            .expect("git config");
    }
}

// ─── Benchmark 1: capability path resolution ────────────────────────────────

/// Measures the overhead of resolving a safe relative path through a
/// `DirCapability`.  This is the critical hot-path for every guarded I/O op.
fn bench_capability_resolve(c: &mut Criterion) {
    let tmp = scratch();
    // Create the target file so canonicalize succeeds.
    fs::write(tmp.path().join("target.txt"), b"data").expect("write target");
    let cap = DirCapability::new(tmp.path(), Permissions::all())
        .expect("create capability");

    c.bench_function("capability_resolve", |b| {
        b.iter(|| {
            cap.resolve(black_box(std::path::Path::new("target.txt")))
                .expect("resolve must succeed in benchmark");
        });
    });
}

// ─── Benchmark 2: transactional file write ───────────────────────────────────

/// Measures the cost of staging and committing a single-file write transaction.
fn bench_fs_transaction_write(c: &mut Criterion) {
    let tmp = scratch();
    let content = b"benchmark payload data".to_vec();

    let mut counter = 0u64;
    c.bench_function("fs_transaction_write", |b| {
        b.iter(|| {
            // Use a unique filename per iteration to avoid rename-to-existing overhead.
            counter += 1;
            let target = tmp.path().join(format!("file_{}.txt", counter));
            let mut tx = FsTransaction::new();
            tx.write_file(target, black_box(content.clone()))
                .expect("enqueue write");
            tx.commit().expect("commit transaction");
        });
    });
}

// ─── Benchmark 3: find_repos discovery ──────────────────────────────────────

/// Measures the cost of find_repos on a directory tree containing 3 repos.
fn bench_git_find_repos(c: &mut Criterion) {
    let tmp = scratch();
    for name in ["repo_a", "repo_b", "repo_c"] {
        let dir = tmp.path().join(name);
        fs::create_dir(&dir).expect("create repo dir");
        init_git_repo(&dir);
    }

    c.bench_function("git_find_repos", |b| {
        b.iter(|| {
            find_repos(black_box(tmp.path()), black_box(2))
                .expect("find_repos must not fail in benchmark");
        });
    });
}

criterion_group!(
    benches,
    bench_capability_resolve,
    bench_fs_transaction_write,
    bench_git_find_repos,
);
criterion_main!(benches);
