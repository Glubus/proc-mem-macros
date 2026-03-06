set shell := ["bash", "-cu"]
set dotenv-load := false

# ── Aliases ───────────────────────────────────────────────────────────────────
alias c := check
alias t := test
alias d := doc
alias w := watch

# ── Default: full CI suite ────────────────────────────────────────────────────

# Run the complete check suite (fmt → clippy → test).
# This is what CI runs and what you run before every push.
[group("ci")]
default: check

# fmt-check + clippy + test in sequence — all must pass.
[group("ci")]
check: fmt-check clippy test

# ── Formatting ────────────────────────────────────────────────────────────────

# Apply rustfmt in-place (nightly for unstable_features in rustfmt.toml).
[group("fmt")]
fmt:
    cargo +nightly fmt --all

# Check formatting without modifying files — fails loudly if anything is off.
[group("fmt")]
fmt-check:
    cargo +nightly fmt --all -- --check

# ── Linting ───────────────────────────────────────────────────────────────────

# Clippy with the full pedantic battery.
# Every warning is a hard error. No exceptions.
[group("lint")]
clippy:
    cargo clippy --all-targets --all-features -- \
        -D warnings \
        -D clippy::all \
        -D clippy::pedantic \
        -D clippy::nursery \
        -A clippy::module_name_repetitions \
        -A clippy::missing_errors_doc

# Same as clippy but auto-fixes what it can.
[group("lint")]
clippy-fix:
    cargo clippy --fix --allow-dirty --all-targets --all-features -- \
        -D warnings \
        -D clippy::all \
        -D clippy::pedantic \
        -D clippy::nursery \
        -A clippy::module_name_repetitions \
        -A clippy::missing_errors_doc

# ── Testing ───────────────────────────────────────────────────────────────────

# Run all tests with full feature flags.
[group("test")]
test:
    cargo test --all-features

# Run tests and show stdout even on success.
[group("test")]
test-verbose:
    cargo test --all-features -- --nocapture

# ── Documentation ─────────────────────────────────────────────────────────────

# Build and open the crate docs (private items included).
[group("doc")]
doc:
    cargo doc --no-deps --document-private-items --open

# Build docs without opening — useful in CI to catch broken intra-doc links.
[group("doc")]
doc-check:
    RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --document-private-items

# ── Build ─────────────────────────────────────────────────────────────────────

# Debug build — only to confirm compilation; proc-macros have no binary output.
[group("build")]
build:
    cargo build --all-features

# Release build.
[group("build")]
build-release:
    cargo build --release --all-features

# ── Dev helpers ───────────────────────────────────────────────────────────────

# Watch mode: re-runs clippy on every file change.
# Requires `cargo-watch` (`cargo install cargo-watch`).
[group("dev")]
watch:
    cargo watch -x "clippy --all-targets --all-features -- -D warnings"

# Nuke all build artifacts.
[group("dev")]
clean:
    cargo clean

# Print the expanded output of the macro for debugging.
# Usage: just expand MyStruct
[group("dev")]
expand target="":
    cargo expand {{ target }}
