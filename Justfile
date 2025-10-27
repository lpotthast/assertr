# Lists all available commands.
list:
  just --list

# Find the minimum supported rust version
msrv:
    cargo install cargo-msrv
    cargo msrv find --path assertr
    cargo msrv find --path assertr-derive

tidy:
    cargo update --workspace
    cargo sort --workspace
    cargo fmt
    cargo check --all
    cargo clippy --all
    cargo test --all
