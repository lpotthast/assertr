# Lists all available commands.
list:
  just --list

# Find the minimum supported rust version.
msrv:
    cargo install cargo-msrv
    cargo msrv find --path assertr
    cargo msrv find --path assertr-derive

# Update all deps; sort all Cargo.toml deps; format, check and lint all code; run all tests.
tidy:
    cargo update --workspace
    cargo sort --workspace
    cargo fmt
    cargo check --all
    cargo check --all --no-default-features
    cargo check --all --all-features
    cargo clippy --all -- -W clippy::pedantic
    cargo clippy --all --no-default-features -- -W clippy::pedantic
    cargo clippy --all --all-features -- -W clippy::pedantic
    cargo test --all
    cargo test --all --no-default-features
    cargo test --all --all-features
    cargo doc --no-deps --all-features
