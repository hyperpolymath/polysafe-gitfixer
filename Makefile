# SPDX-License-Identifier: MIT AND Palimpsest-0.8
# SPDX-FileCopyrightText: 2024-2025 The polysafe-gitfixer Contributors

.PHONY: all rust haskell idris elixir clean test check fmt shell

# Default target: build everything
all: rust haskell idris elixir

# Rust components
rust:
	cd crates && cargo build --release

rust-debug:
	cd crates && cargo build

rust-test:
	cd crates && cargo test

rust-check:
	cd crates && cargo check

rust-fmt:
	cd crates && cargo fmt

rust-clippy:
	cd crates && cargo clippy -- -D warnings

# Haskell components
haskell: haskell-diff haskell-tui

haskell-diff:
	cd haskell/diff-engine && cabal build

haskell-tui:
	cd haskell/tui && cabal build

haskell-test:
	cd haskell/diff-engine && cabal test
	cd haskell/tui && cabal test

haskell-clean:
	cd haskell/diff-engine && cabal clean
	cd haskell/tui && cabal clean

# Idris 2 workflow (optional - fallback to Haskell if not available)
idris:
	@if command -v idris2 >/dev/null 2>&1; then \
		cd idris/workflow && idris2 --build workflow.ipkg; \
	else \
		echo "Idris 2 not found, skipping workflow build (Haskell fallback will be used)"; \
	fi

# Elixir orchestrator (depends on Rust NIFs)
elixir: rust
	cd elixir/polysafe_gitfixer && mix deps.get && mix compile

elixir-test:
	cd elixir/polysafe_gitfixer && mix test

# Config validation
config-check:
	@if command -v nickel >/dev/null 2>&1; then \
		nickel typecheck config/schema.ncl; \
	else \
		echo "Nickel not found, skipping config validation"; \
	fi

# Run all tests
test: rust-test haskell-test elixir-test

# Check all (no tests, just compilation/type checking)
check: rust-check config-check
	cd haskell/diff-engine && cabal build --only-dependencies
	cd haskell/tui && cabal build --only-dependencies

# Format all code
fmt: rust-fmt
	@echo "Note: Add Haskell formatters (ormolu/fourmolu) as needed"

# Clean all build artifacts
clean:
	cd crates && cargo clean
	-cd haskell/diff-engine && cabal clean
	-cd haskell/tui && cabal clean
	-cd idris/workflow && rm -rf build/
	-cd elixir/polysafe_gitfixer && mix clean

# Development: watch and rebuild on changes
watch-rust:
	cd crates && cargo watch -x check

# Nix development shell
shell:
	nix develop

# Help
help:
	@echo "polysafe-gitfixer build system"
	@echo ""
	@echo "Targets:"
	@echo "  all          - Build all components"
	@echo "  rust         - Build Rust crates (release)"
	@echo "  rust-debug   - Build Rust crates (debug)"
	@echo "  haskell      - Build Haskell components"
	@echo "  idris        - Build Idris workflow (if available)"
	@echo "  elixir       - Build Elixir orchestrator"
	@echo "  test         - Run all tests"
	@echo "  check        - Type check without building"
	@echo "  fmt          - Format all code"
	@echo "  clean        - Remove build artifacts"
	@echo "  shell        - Enter Nix development shell"
	@echo "  help         - Show this help"
