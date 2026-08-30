# Every feature of the `assertr` crate, checked one at a time by `check-each-feature` and
# friends. Cargo unifies features across a single invocation, so an integration that silently
# depends on another feature only shows up when it is the only feature enabled.
features := "derive fluent http jiff libm num program reqwest rootcause serde serde-json serde-toml std tokio"

# Lists all available commands.
list:
  just --list

# Install tools required by other recipes.
install-tools:
    cargo +stable install cargo-msrv --locked
    cargo +stable install cargo-sort --locked

# Find the minimum supported rust version.
msrv:
    cargo msrv find --path assertr
    cargo msrv find --path assertr-derive

# Check the code.
check:
    # Check each package independently so Cargo cannot make a configuration pass through
    # workspace feature unification.
    cargo check -p assertr --all-targets --no-default-features
    cargo check -p assertr --all-targets --no-default-features --features std
    cargo check -p assertr --all-targets --no-default-features --features num
    cargo check -p assertr --all-targets
    cargo check -p assertr --all-targets --all-features
    cargo check -p assertr-derive --all-targets
    cargo check -p assertr-no-std-tests --all-targets

# Check every feature on its own, so no feature can hide behind another's dependencies.
check-each-feature:
    #!/usr/bin/env bash
    set -euo pipefail
    for feature in {{ features }}; do
        echo "==> --no-default-features --features $feature"
        cargo check -p assertr --all-targets --no-default-features --features "$feature"
    done

# Check the two `no_std` configurations: hosted (with `alloc`) and embedded.
check-no-std:
    cargo test -p assertr-no-std-tests
    cargo check -p assertr --lib --no-default-features --target thumbv8m.main-none-eabihf
    cargo check -p assertr --lib --no-default-features --features num,libm --target thumbv8m.main-none-eabihf

# Lint the code.
clippy:
    cargo clippy -p assertr --all-targets --no-default-features -- -D warnings -W clippy::pedantic
    cargo clippy -p assertr --all-targets --no-default-features --features std -- -D warnings -W clippy::pedantic
    cargo clippy -p assertr --all-targets --no-default-features --features num -- -D warnings -W clippy::pedantic
    cargo clippy -p assertr --all-targets -- -D warnings -W clippy::pedantic
    cargo clippy -p assertr --all-targets --all-features -- -D warnings -W clippy::pedantic
    cargo clippy -p assertr-derive --all-targets -- -D warnings -W clippy::pedantic
    cargo clippy -p assertr-no-std-tests --all-targets -- -D warnings -W clippy::pedantic

# Run all tests.
test:
    cargo test -p assertr --no-default-features
    cargo test -p assertr --no-default-features --features std
    cargo test -p assertr --no-default-features --features num
    cargo test -p assertr
    cargo test -p assertr --all-features
    cargo test -p assertr-derive
    cargo test -p assertr-no-std-tests

build-docs:
    RUSTDOCFLAGS="-D warnings" cargo doc --workspace --exclude assertr-no-std-tests --no-deps --all-features

open-docs:
    cargo doc -p assertr --all-features --no-deps --open

# Update all deps; sort all Cargo.toml deps; format all code.
tidy:
    cargo update --workspace
    cargo sort --workspace
    cargo fmt --all

# Run the full non-mutating validation suite.
verify:
    cargo fmt --all -- --check
    just check
    just check-each-feature
    just check-no-std
    just clippy
    just test
    just build-docs
