# AGENTS.md

This file provides guidance to AI coding agents (Claude Code, Codex, and others) when working with code in this
repository.

## Build & Test Commands

This is a Cargo workspace with three crates: `assertr` (main library), `assertr-derive` (proc macro), and
`assertr-no-std-tests` (internal, unpublished test crate verifying `no_std` behavior).

```bash
# Build all
cargo build --all

# Test supported configurations independently
cargo test -p assertr --no-default-features
cargo test -p assertr
cargo test -p assertr --all-features
cargo test -p assertr-derive
cargo test -p assertr-no-std-tests

# Test a single test
cargo test -p assertr --all-features <test_name>

# Lint & format
cargo fmt
cargo clippy -p assertr --all-targets --all-features -- -W clippy::pedantic

# Full tidy (update deps, sort, fmt, check, clippy pedantic, test, doc)
just tidy
```

## Required Change Workflow

Use `## [Unreleased]` in `CHANGELOG.md` as the default landing place for change entries while work is in progress:

1. Record the net release-notable effect of each change under the appropriate section in `## [Unreleased]`.
   Before adding a new item, inspect existing `Unreleased` entries for the same feature, fix, or design area.
   Update, merge, or remove related entries so the changelog describes the final current behavior, not intermediary
   iterations.
2. Re-evaluate the SemVer impact of the resulting changelog entry when you add or revise it.
3. If a change is SemVer-breaking, start its markdown list item with `- **Breaking:** ...`.
4. Do not bump crate versions or README dependency snippets for ordinary in-progress changes.

Adding a new assertion method to an existing public `*Assertions` trait is explicitly considered non-breaking in this
crate. Those traits are public for method discovery and are not supported as downstream implementation interfaces;
users should define separate assertion traits for custom types. Removing a method or incompatibly changing an existing
method remains breaking.

Right before publishing:

1. Inspect the `## [Unreleased]` section and determine the next version from the accumulated changes.
2. Treat any `- **Breaking:** ...` entry as the indicator that the release requires a breaking version bump.
3. Move the accumulated unreleased entries into a new `## [x.y.z] - YYYY-MM-DD` section.
4. Bump the affected crate version(s) accordingly. `assertr` declares an exact `=` requirement on `assertr-derive`
   (generated code relies on `assertr::__private`), so bump that requirement together with the derive crate. Because
   already-published `assertr` versions used caret requirements, the first `assertr-derive` release after `0.2.5` must
   be at least `0.3.0`; otherwise Cargo could resolve the new derive against an old `assertr` and the generated code
   would not compile.
5. Update every README installation example to reference the new crate version.
6. Extend the comparison link list at the end of `CHANGELOG.md` for the new version and update `[Unreleased]` to compare
   from the new tag.
7. Run `just tidy` as the final verification step.

## Architecture

**assertr** is a fluent assertion library for Rust with `no_std` support. The core API:

```rust
assert_that!(value).is_equal_to(expected).has_length(5);
assert_that!(&value).starts_with("hello");
```

### Core types (in `assertr/src/`)

- **`assert_that!` macro** (`assert_that_macro.rs`) - Uses autoref specialization to handle both borrowed
  (`&T → Actual::Borrowed`) and owned (`T → Actual::Owned`) values transparently.
- **`AssertThat<'t, T, M>`** (`lib.rs`) - The central struct. `T` is the value type, `M` is the mode (`Panic` or
  `Capture`). All assertion methods are implemented as trait impls on this type. Methods return `Self` for chaining.
- **`Actual<'t, T>`** (`actual.rs`) - Enum holding either `Borrowed(&'t T)` or `Owned(T)`.
- **`Mode` trait** (`mode.rs`) - `Panic` (fail immediately) vs `Capture` (collect failures for batch inspection).
- **`Failure`/`Fallible` traits** (`failure.rs`) - Failure message construction with location tracking, subject names,
  and detail messages.
- **`Condition<T>` trait** (`condition.rs`) - Reusable predicates used with `.is()` and `.has()`.
- **`AssertrPartialEq<Rhs>`** (`lib.rs`) - Custom equality trait with `EqContext` for field-by-field difference
  reporting.
- **Tracking** (`tracking.rs`) - Counts assertions per chain; panics if `AssertThat` is dropped with zero assertions
  (catches forgotten assertions).

### Assertion modules (`assertr/src/assertions/`)

Assertions are organized by type category, each in its own module with a trait:

- `core/` - Fundamental types: `PartialEq`, `PartialOrd`, `bool`, `char`, `str`, `Option`, `Result`, `Iterator`,
  `Array`, `Slice`, `Tuple`
- `alloc/` - Heap types: `String`, `Vec`, `Box`, `PanicValue`
- `std/` - Std types: `Path`, `Command`, `HashMap`, `Mutex`, `Type`
- `num/` - Numeric: `is_zero`, `is_positive`, `is_close_to`, `is_nan`, etc.
- `program.rs`, `http/`, `jiff/`, `tokio/`, `reqwest/` - Feature-gated external crate integrations

### Derive macro (`assertr-derive/`)

`#[derive(AssertrEq)]` generates a `{StructName}AssertrEq` companion struct with `Eq<FieldType>` fields for field-level
partial equality. Supports `#[assertr_eq(map_type = "...")]` and `#[assertr_eq(compare_with = "...")]` attributes. Only
processes public fields. Uses `darling` for attribute parsing.

Compile-fail tests use `trybuild` in `assertr-derive/tests/`.

### Key patterns for adding assertions

New assertion traits follow this pattern:

1. Define a trait (e.g., `FooAssertions`) with methods returning `Self`
2. Annotate the trait with `#[cfg_attr(feature = "fluent", assertr_derive::fluent_aliases)]` to auto-generate fluent
   aliases (e.g., `is_true` → `be_true`, `has_length` → `have_length`)
3. Implement it for `AssertThat<'_, Foo, M>` where `M: Mode`
4. Use `self.track_assertion()` at the start of each method
5. On failure, call `self.fail(...)` with a formatted message
6. Annotate all assertion methods with `#[track_caller]`

## Features

- **default**: `["std", "num"]`
- **full**: All features (`derive`, `fluent`, `http`, `jiff`, `libm`, `num`, `program`, `reqwest`, `serde`, `std`,
  `tokio`)
- **fluent**: Enables `IntoAssertContext` trait on all types, providing `.must()` (Panic mode) and `.verify()` (Capture
  mode) entry points, plus fluent aliases like `be_equal_to`, `have_length`
- Library supports `no_std` when `std` feature is disabled
- MSRV: `assertr` 1.89.0, `assertr-derive` 1.89.0

## Linting

Both crates forbid `unsafe_code` and deny `clippy::unwrap_used`.
