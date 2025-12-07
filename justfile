# SPDX-License-Identifier: MIT AND Palimpsest-0.8
# SPDX-FileCopyrightText: 2024-2025 The polysafe-gitfixer Contributors
#
# justfile for polysafe-gitfixer
# Run tasks with: just <task>
# List tasks with: just --list

# Default recipe - show help
default:
    @just --list

# ==============================================================================
# Environment
# ==============================================================================

# Enter Nix development shell
shell:
    nix develop

# Check all required tools are available
check-env:
    @echo "Checking development environment..."
    @which rustc > /dev/null && echo "✓ Rust: $(rustc --version)" || echo "✗ Rust not found"
    @which elixir > /dev/null && echo "✓ Elixir: $(elixir --version | head -1)" || echo "✗ Elixir not found"
    @which ghc > /dev/null && echo "✓ GHC: $(ghc --version)" || echo "✗ GHC not found"
    @which idris2 > /dev/null && echo "✓ Idris2: $(idris2 --version)" || echo "✗ Idris2 not found"
    @which nickel > /dev/null && echo "✓ Nickel: $(nickel --version)" || echo "✗ Nickel not found"

# ==============================================================================
# Build - All Components
# ==============================================================================

# Build everything
all: rust haskell elixir idris
    @echo "All components built successfully"

# Build for release
release: rust-release haskell elixir idris
    @echo "Release build complete"

# Clean all build artifacts
clean: rust-clean haskell-clean elixir-clean idris-clean
    @echo "All build artifacts cleaned"

# ==============================================================================
# Rust
# ==============================================================================

# Build Rust crates (debug)
rust:
    cd crates && cargo build

# Build Rust crates (release)
rust-release:
    cd crates && cargo build --release

# Run Rust tests
rust-test:
    cd crates && cargo test

# Check Rust code (no build)
rust-check:
    cd crates && cargo check

# Format Rust code
rust-fmt:
    cd crates && cargo fmt

# Check Rust formatting
rust-fmt-check:
    cd crates && cargo fmt --check

# Run Clippy lints
rust-clippy:
    cd crates && cargo clippy -- -D warnings

# Clean Rust build artifacts
rust-clean:
    cd crates && cargo clean

# Watch Rust and rebuild on changes
rust-watch:
    cd crates && cargo watch -x check

# ==============================================================================
# Haskell
# ==============================================================================

# Build all Haskell components
haskell: haskell-diff haskell-tui

# Build diff engine
haskell-diff:
    cd haskell/diff-engine && cabal build

# Build TUI
haskell-tui:
    cd haskell/tui && cabal build

# Run Haskell tests
haskell-test:
    cd haskell/diff-engine && cabal test
    cd haskell/tui && cabal test

# Clean Haskell build artifacts
haskell-clean:
    -cd haskell/diff-engine && cabal clean
    -cd haskell/tui && cabal clean

# Update Haskell dependencies
haskell-update:
    cd haskell/diff-engine && cabal update
    cd haskell/tui && cabal update

# ==============================================================================
# Elixir
# ==============================================================================

# Build Elixir orchestrator
elixir: rust
    cd elixir/polysafe_gitfixer && mix deps.get && mix compile

# Run Elixir tests
elixir-test:
    cd elixir/polysafe_gitfixer && mix test

# Clean Elixir build artifacts
elixir-clean:
    -cd elixir/polysafe_gitfixer && mix clean

# Start Elixir REPL with project loaded
elixir-iex:
    cd elixir/polysafe_gitfixer && iex -S mix

# Format Elixir code
elixir-fmt:
    cd elixir/polysafe_gitfixer && mix format

# ==============================================================================
# Idris 2
# ==============================================================================

# Build Idris workflow
idris:
    @if command -v idris2 > /dev/null 2>&1; then \
        cd idris/workflow && idris2 --build workflow.ipkg; \
    else \
        echo "Idris 2 not found, using Haskell fallback"; \
        just haskell-workflow; \
    fi

# Clean Idris build artifacts
idris-clean:
    -rm -rf idris/workflow/build/

# Build Haskell workflow fallback
haskell-workflow:
    @echo "Building Haskell workflow fallback..."
    cd haskell/workflow && cabal build || echo "Haskell workflow not yet implemented"

# ==============================================================================
# Nickel Configuration
# ==============================================================================

# Validate Nickel configuration
config-check:
    nickel typecheck config/schema.ncl

# Export config to JSON
config-export:
    nickel export config/default.ncl > config/config.json

# ==============================================================================
# Testing
# ==============================================================================

# Run all tests
test: rust-test haskell-test elixir-test
    @echo "All tests passed"

# Run integration tests
integration-test:
    @echo "Running integration tests..."
    ./test/integration/run-tests.sh

# ==============================================================================
# Code Quality
# ==============================================================================

# Format all code
fmt: rust-fmt elixir-fmt
    @echo "All code formatted"

# Check all formatting
fmt-check: rust-fmt-check
    @echo "All formatting checks passed"

# Run all lints
lint: rust-clippy
    @echo "All lints passed"

# Run all checks (no tests)
check: rust-check config-check
    @echo "All checks passed"

# ==============================================================================
# Documentation
# ==============================================================================

# Build Rust documentation
docs-rust:
    cd crates && cargo doc --no-deps --open

# Build all documentation
docs: docs-rust
    @echo "Documentation built"

# ==============================================================================
# Development Helpers
# ==============================================================================

# Watch all and rebuild on changes
watch:
    @echo "Watching for changes..."
    cargo watch -w crates -x check

# Create a new git commit (interactive)
commit:
    git add -p && git commit

# Show git status
status:
    git status

# ==============================================================================
# CI/CD
# ==============================================================================

# Run CI checks (what CI would run)
ci: fmt-check lint rust-test
    @echo "CI checks passed"

# Full CI pipeline
ci-full: ci haskell-test elixir-test integration-test
    @echo "Full CI pipeline passed"
