# rocq-of-rust formal verification targets
#
# Prerequisites (see README section below / docs/builds.md in rocq-of-rust):
#   1. Rust nightly toolchain used by rocq-of-rust:
#        rustup toolchain install nightly-2025-12-07 --profile minimal --component rustc-dev
#   2. The rocq-of-rust repository cloned locally and its Rust translator installed:
#        git clone https://github.com/formal-land/rocq-of-rust.git ~/programming/rocq-of-rust
#        cd ~/programming/rocq-of-rust && cargo +nightly-2025-12-07 install --path lib/
#   3. opam switch with Rocq and the RocqOfRust base library compiled:
#        opam switch create rocq-of-rust ocaml.5.1.0
#        eval $(opam env --switch=rocq-of-rust)
#        opam repo add rocq-released https://rocq-prover.org/opam/released
#        cd ~/programming/rocq-of-rust/RocqOfRust && opam install --deps-only . -y && make
#
# Usage:
#   make rocq-transpile   # generate .v files from the Rust crate (cargo rocq-of-rust)
#   make rocq-of-rust     # compile every translated .v file against RocqOfRust

ROCQ_OF_RUST_DIR ?= $(HOME)/programming/rocq-of-rust
NIGHTLY ?= nightly-2025-12-07

.PHONY: rocq-transpile rocq-of-rust

# Translate the Rust crate to Rocq (generates .v files, git-ignored).
rocq-transpile:
	cargo +$(NIGHTLY) rocq-of-rust --axiomatize

# Type-check every translated .v file against the RocqOfRust base library.
rocq-of-rust: rocq-transpile
	./rocq_compile_all.sh
