# SPDX-License-Identifier: MIT AND Palimpsest-0.8
# SPDX-FileCopyrightText: 2024-2025 The polysafe-gitfixer Contributors
#
# Nix flake for polysafe-gitfixer development environment
#
# Usage:
#   nix develop          # Enter development shell
#   nix build            # Build all components
#   nix flake check      # Run checks
#
# With direnv:
#   echo "use flake" > .envrc
#   direnv allow

{
  description = "polysafe-gitfixer - Polyglot git backup merger with maximum safety guarantees";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-24.05";
    nixpkgs-unstable.url = "github:NixOS/nixpkgs/nixos-unstable";

    flake-utils.url = "github:numtide/flake-utils";

    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    # Idris 2 from the official flake
    idris2 = {
      url = "github:idris-lang/Idris2";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, nixpkgs-unstable, flake-utils, rust-overlay, idris2 }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };
        pkgs-unstable = import nixpkgs-unstable {
          inherit system;
        };

        # Rust toolchain - stable with additional components
        rustToolchain = pkgs.rust-bin.stable.latest.default.override {
          extensions = [ "rust-src" "rust-analyzer" "clippy" "rustfmt" ];
        };

        # Haskell packages
        haskellPackages = pkgs.haskell.packages.ghc96;

        # Common build inputs for all shells
        commonBuildInputs = with pkgs; [
          # Version control
          git

          # Build tools
          gnumake
          just

          # Nix tools
          nil  # Nix LSP
          nixpkgs-fmt
        ];

        # Rust development
        rustInputs = with pkgs; [
          rustToolchain
          pkg-config
          openssl
          openssl.dev
          libgit2
          libssh2
          zlib

          # For Rustler (Elixir NIFs)
          erlang

          # Rust tools
          cargo-watch
          cargo-edit
          cargo-audit
        ];

        # Elixir development
        elixirInputs = with pkgs; [
          # Erlang/OTP 26+
          erlang_26

          # Elixir 1.15+
          elixir_1_16

          # Elixir tools
          elixir-ls

          # For NIFs
          gcc
          gnumake
        ];

        # Haskell development
        haskellInputs = with pkgs; [
          # GHC 9.6
          haskellPackages.ghc

          # Cabal
          cabal-install

          # Haskell tools
          haskellPackages.haskell-language-server
          haskellPackages.ormolu
          haskellPackages.hlint

          # For Brick TUI
          ncurses

          # Common Haskell libraries (for faster builds)
          haskellPackages.aeson
          haskellPackages.brick
          haskellPackages.bytestring
          haskellPackages.conduit
          haskellPackages.directory
          haskellPackages.filepath
          haskellPackages.optparse-applicative
          haskellPackages.streaming
          haskellPackages.text
          haskellPackages.vty
        ];

        # Idris 2 development
        idrisInputs = [
          idris2.packages.${system}.idris2
        ];

        # Nickel for configuration
        nickelInputs = with pkgs-unstable; [
          nickel
        ];

        # All development inputs combined
        allInputs = commonBuildInputs
          ++ rustInputs
          ++ elixirInputs
          ++ haskellInputs
          ++ idrisInputs
          ++ nickelInputs;

      in {
        # Development shell with all tools
        devShells = {
          default = pkgs.mkShell {
            buildInputs = allInputs;

            shellHook = ''
              echo "🔧 polysafe-gitfixer development environment"
              echo ""
              echo "Available tools:"
              echo "  Rust:    $(rustc --version)"
              echo "  Elixir:  $(elixir --version | head -1)"
              echo "  GHC:     $(ghc --version)"
              echo "  Idris2:  $(idris2 --version)"
              echo "  Nickel:  $(nickel --version)"
              echo ""
              echo "Commands:"
              echo "  make all     - Build all components"
              echo "  make test    - Run all tests"
              echo "  make check   - Type check without building"
              echo ""

              # Set up Rust environment
              export RUST_SRC_PATH="${rustToolchain}/lib/rustlib/src/rust/library"

              # Set up library paths for native dependencies
              export PKG_CONFIG_PATH="${pkgs.openssl.dev}/lib/pkgconfig:${pkgs.libgit2}/lib/pkgconfig"
              export LIBGIT2_SYS_USE_PKG_CONFIG=1
              export LIBSSH2_SYS_USE_PKG_CONFIG=1

              # Elixir/Erlang setup
              export ERL_AFLAGS="-kernel shell_history enabled"
              export MIX_HOME="$PWD/.nix-mix"
              export HEX_HOME="$PWD/.nix-hex"
              mkdir -p "$MIX_HOME" "$HEX_HOME"
              export PATH="$MIX_HOME/bin:$HEX_HOME/bin:$PATH"

              # Haskell setup
              export CABAL_DIR="$PWD/.cabal"
            '';

            # Environment variables
            LOCALE_ARCHIVE = "${pkgs.glibcLocales}/lib/locale/locale-archive";
          };

          # Minimal shell with just Rust (for CI or quick builds)
          rust = pkgs.mkShell {
            buildInputs = commonBuildInputs ++ rustInputs;

            shellHook = ''
              export RUST_SRC_PATH="${rustToolchain}/lib/rustlib/src/rust/library"
              export PKG_CONFIG_PATH="${pkgs.openssl.dev}/lib/pkgconfig:${pkgs.libgit2}/lib/pkgconfig"
              export LIBGIT2_SYS_USE_PKG_CONFIG=1
            '';
          };

          # Haskell-only shell
          haskell = pkgs.mkShell {
            buildInputs = commonBuildInputs ++ haskellInputs;
          };

          # Elixir-only shell
          elixir = pkgs.mkShell {
            buildInputs = commonBuildInputs ++ elixirInputs ++ rustInputs;

            shellHook = ''
              export ERL_AFLAGS="-kernel shell_history enabled"
              export MIX_HOME="$PWD/.nix-mix"
              export HEX_HOME="$PWD/.nix-hex"
              mkdir -p "$MIX_HOME" "$HEX_HOME"
              export PATH="$MIX_HOME/bin:$HEX_HOME/bin:$PATH"
            '';
          };
        };

        # Checks
        checks = {
          # Rust formatting and linting
          rust-fmt = pkgs.runCommand "rust-fmt-check" {
            buildInputs = [ rustToolchain ];
          } ''
            cd ${self}/crates
            cargo fmt --check
            touch $out
          '';

          rust-clippy = pkgs.runCommand "rust-clippy-check" {
            buildInputs = rustInputs;
          } ''
            cd ${self}/crates
            cargo clippy -- -D warnings
            touch $out
          '';

          # Nickel config validation
          nickel-check = pkgs.runCommand "nickel-check" {
            buildInputs = nickelInputs;
          } ''
            nickel typecheck ${self}/config/schema.ncl
            touch $out
          '';
        };

        # Packages (when we have buildable outputs)
        packages = {
          # Rust crates
          rust-crates = pkgs.rustPlatform.buildRustPackage {
            pname = "polysafe-gitfixer-crates";
            version = "0.1.0";
            src = ./crates;
            cargoLock.lockFile = ./crates/Cargo.lock;

            nativeBuildInputs = with pkgs; [ pkg-config ];
            buildInputs = with pkgs; [ openssl libgit2 libssh2 zlib ];

            # Skip tests in Nix build (run separately)
            doCheck = false;
          };
        };
      }
    );
}
