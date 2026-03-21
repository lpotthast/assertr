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
    cargo clippy --all -- -W clippy::pedantic
    cargo test --all
