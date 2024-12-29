# Lists all available commands.
list:
  just --list

# Find the minimum supported rust version
msrv:
    cargo install cargo-msrv
    cargo msrv find --path assertr
