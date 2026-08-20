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
    cargo check -p assertr --all-targets
    cargo check -p assertr --all-targets --all-features
    cargo check -p assertr-derive --all-targets
    cargo check -p assertr-no-std-tests --all-targets

# Lint the code.
clippy:
    cargo clippy -p assertr --all-targets --no-default-features -- -W clippy::pedantic
    cargo clippy -p assertr --all-targets -- -W clippy::pedantic
    cargo clippy -p assertr --all-targets --all-features -- -W clippy::pedantic
    cargo clippy -p assertr-derive --all-targets -- -W clippy::pedantic
    cargo clippy -p assertr-no-std-tests --all-targets -- -W clippy::pedantic

# Run all tests.
test:
    cargo test -p assertr --no-default-features
    cargo test -p assertr
    cargo test -p assertr --all-features
    cargo test -p assertr-derive
    cargo test -p assertr-no-std-tests

# Update all deps; sort all Cargo.toml deps; format all code.
tidy:
    cargo update --workspace
    cargo sort --workspace
    cargo fmt --all

# Run the full non-mutating validation suite.
verify:
    cargo fmt --all -- --check
    just check
    just clippy
    just test
    cargo doc --workspace --exclude assertr-no-std-tests --no-deps --all-features
