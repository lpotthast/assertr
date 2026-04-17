# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/),
and this project adheres to [Semantic Versioning](https://semver.org/).

## [Unreleased]

### Fixed

- Restored `--no-default-features` / no-std compatibility by using `core`/`alloc` paths in core assertions and keeping.
- Aligned the README assertion table with the actual prelude exports, including feature gates, panic-only extract
  assertions, and recently added string, path, HTTP header, program, async function, and rootcause entries.
- Reworked the README installation and usage guidance to document the `fluent` feature, clarify `no_std` setup,
  clean up examples, and improve the reference sections for conditions, derived assertions, testing, and MSRV.

## [0.5.5] - 2026-04-15

### Fixed

- `PartialOrd` comparison assertions now fail for unordered values such as `NaN` instead of accidentally passing when
  `partial_cmp()` returns `None`.

## [0.5.4] - 2026-04-14

### Added

- `HashSet<T>` assertions for membership, bulk membership, subset, superset, and disjoint-set checks.
- Negative collection membership assertions via `does_not_contain()` for `&[T]`, `[T; N]`, `Vec<T>`, and iterators.
- Expanded `HashMap<K, V>` assertions with `does_not_contain_value()`, `does_not_contain_entry()`,
  `contains_keys()`, and `contains_exactly_entries()`.
- String blankness parity via `is_not_blank()` for both `&str` and `String`.
- ASCII case-insensitive equality assertions for `&str` and `String`.

## [0.5.3] - 2026-04-14

### Added

- `Vec<T>::contains_exactly_in_any_order` for direct parity with slice assertions.
- Array assertions for `contains` and `contains_exactly_matching_in_any_order`.

### Changed

- Aligned `[T; N]`, `&[T]`, and `Vec<T>` collection assertion coverage in the README.
- Broadened array `contains_exactly` to support comparable expected element types via `AssertrPartialEq`, matching
  slice and `Vec` behavior.
- Expanded `Vec<T>` and `[T; N]` collection assertion tests.

## [0.5.2] - 2026-04-13

### Added

- Capture-mode compatible `OptionAssertions::is_some_satisfying` and `PollAssertions::is_ready_satisfying` assertions.

### Changed

- Reworked `Option` and `Poll` assertion failure formatting to use the newer `writedoc!` builder style.

## [0.5.1] - 2026-04-13

### Added

- Optional `rootcause` feature with `ReportCollection`/`ReportAttachments` length support, report context/count
  assertions, and dynamic current-context extraction.
- Support for deriving fluent alternative functions for functions with generic parameters.

### Changed

- `assertr-derive`: Bumped to 0.2.5.

## [0.5.0] - 2026-03-23

### Added

- New `fluent` feature gate and `IntoAssertContext` trait, providing `.must()` / `.must_owned()` (Panic mode) and
  `.verify()` / `.verify_owned()` (Capture mode) entry points on all types.
- Fluent assertion aliases auto-generated via the `#[fluent_aliases]` proc-macro attribute (e.g., `is_true` → `be_true`,
  `has_length` → `have_length`, `starts_with` → `start_with`).
- `assert_that_owned()` function for entering an assertion context with an owned value.
- `AssertThat::and()` no-op method for readability in fluent chains.
- `AssertThat::satisfy()` fluent alias for `satisfies()`.
- `AssertThat::new_capturing()` constructor for direct Capture mode entry (behind `fluent` feature).
- `Type::new()` constructor.
- CLAUDE.md, LLM instructions.
- CHANGELOG.md.
- `assertr-derive`: `#[fluent_aliases]` proc-macro attribute for auto-generating fluent assertion aliases.

### Changed

- **Breaking:** `assert_that()` function now takes `&T` (borrowed) instead of `T` (owned). Use `assert_that_owned()`
  for the previous owned-value behavior, or prefer the `assert_that!()` macro which handles both transparently.
- **Breaking:** Updated `map_async` signature to use explicit `Fut` generic and simplified lifetime bounds.
- Renamed internal constructor `AssertThat::new()` to `AssertThat::new_panicking()`.
- Updated dependencies.
- Fix all pedantic clippy lints.
- `assertr-derive`: Bumped to 0.2.4.

### Removed

- **Breaking:** `assert_that_ref()` - The (still deprecated) `assert_that()` now takes its input by reference.
- **Breaking:** `AssertingThat` and `AssertingThatRef` traits.

### Fixed

- `RefCellAssertions::is_not_mutably_borrowed()` had inverted logic. It incorrectly failed when the `RefCell` had no
  borrows and incorrectly passed when the `RefCell` was mutably borrowed.
- `SignedDurationAssertions::is_positive()` error message incorrectly said "to be negative" instead of "to be positive".
- Redundant duplicate condition check in `SliceAssertions::contains_exactly_matching_in_any_order()`.

## [0.4.4] - 2026-03-22

### Added

- `assert_that!` macro as the primary entrypoint into an assertion context, handling both owned and borrowed values via
  autoref specialization.

### Changed

- Deprecated `assert_that()` and `assert_that_ref()` functions in favor of the new `assert_that!` macro.
- Fix all pedantic clippy lints.

## [0.4.3] - 2025-12-17

### Fixed

- Add missing `#[track_caller]` annotations to assertion methods, ensuring correct panic locations in test output.

## [0.4.2] - 2025-10-27

### Added

- Additional `From` conversions for the `Program` type.

## [0.4.1] - 2025-10-27

### Added

- `program` feature with assertions for the `Program` type.

## [0.4.0] - 2025-10-02

### Added

- Panic assertions for async functions/futures via `panics_async()`.
- `map_async` and `map_async_owned` methods mirroring synchronous `map` and `map_owned`.

### Changed

- **Breaking:** Bumped MSRV to 1.89.0.
- **Breaking:** Switched to Rust edition 2024.
- Updated dependencies.

## [0.3.9] - 2025-09-17

### Added

- Negative `&str` assertions: `does_not_contain`, `does_not_start_with`, `does_not_end_with`.

## [0.3.8] - 2025-09-12

### Added

- `PathAssertions::starts_with` and `PathAssertions::ends_with`.

## [0.3.7] - 2025-09-10

### Added

- `http` feature with `HttpHeaderValueAssertions`.

## [0.3.6] - 2025-09-10

### Fixed

- New lifetime clippy lints.
- Inverted ranges are now always reported as empty / having a length of zero, matching iterator behavior.

## [0.3.5] - 2025-06-25

### Changed

- Updated installation instructions.

## [0.3.4] - 2025-06-25

### Added

- `unwrap_inner` functions for extracting values from `Option` and `Result` assertion chains.

### Fixed

- Subject name not being written into assertion failure messages.

## [0.3.3] - 2025-06-25

### Fixed

- Range length calculations and assertions for edge cases.

## [0.3.2] - 2025-06-24

### Added

- `Default` derive on `Eq` (defaulting to `Any`).
- `Default` derive on generated `*AssertrEq` structs, enabling partial matches without specifying all unwanted fields
  as `any()`.

## [0.3.1] - 2025-05-14

### Added

- `HasLength` implementation for `HashSet`.

## [0.3.0] - 2025-05-13

### Added

- `has_debug_string` assertion for types implementing `Debug`.
- `num` as a default feature.

### Changed

- Moved numeric assertion module to the same depth as other library-related assertion modules.

## [0.2.0] - 2025-05-08

### Added

- `#[derive(AssertrEq)]` proc macro for partial struct equality.
- Tokio assertions: `Mutex`, `RwLock`, `watch::Receiver`.
- `map_owned` for mapping owned values in assertion chains.
- `String` / `&str` `has_length` assertion.
- `contains_exactly_matching_in_any_order` for slices and `Vec`.

### Changed

- **Breaking:** Bumped MSRV to 1.85.0.
- Moved existing assertions into `std` module to allow assertions for types from other crates to coexist.

## [0.1.0] - 2025-01-17

### Added

- Initial release.
- Fluent assertion API via `assert_that()` and `assert_that_ref()` functions.
- Core `AssertThat` struct with `Panic` and `Capture` modes.
- Assertions for: `PartialEq`, `PartialOrd`, `bool`, `char`, `&str`, `String`, `Option`, `Result`, `Iterator`,
  `Vec`, `Box`, `HashMap`, `Mutex`, `RefCell`, `Path`, `Command`, slices, arrays, and ranges.
- Numeric assertions via the `num` feature: `is_zero`, `is_positive`, `is_negative`, `is_close_to`, `is_nan`, etc.
- `Condition` trait for reusable predicates with `satisfies()`.
- `AssertrPartialEq` trait for field-by-field difference reporting.
- Assertion tracking (panics if `AssertThat` is dropped with zero assertions).

[Unreleased]: https://github.com/lpotthast/assertr/compare/v0.5.5...HEAD

[0.5.5]: https://github.com/lpotthast/assertr/compare/v0.5.4...v0.5.5

[0.5.4]: https://github.com/lpotthast/assertr/compare/v0.5.3...v0.5.4

[0.5.2]: https://github.com/lpotthast/assertr/compare/v0.5.1...v0.5.2

[0.5.1]: https://github.com/lpotthast/assertr/compare/v0.5.0...v0.5.1

[0.5.0]: https://github.com/lpotthast/assertr/compare/v0.4.4...v0.5.0

[0.4.4]: https://github.com/lpotthast/assertr/compare/v0.4.3...v0.4.4

[0.4.3]: https://github.com/lpotthast/assertr/compare/v0.4.2...v0.4.3

[0.4.2]: https://github.com/lpotthast/assertr/compare/v0.4.1...v0.4.2

[0.4.1]: https://github.com/lpotthast/assertr/compare/v0.4.0...v0.4.1

[0.4.0]: https://github.com/lpotthast/assertr/compare/v0.3.9...v0.4.0

[0.3.9]: https://github.com/lpotthast/assertr/compare/v0.3.8...v0.3.9

[0.3.8]: https://github.com/lpotthast/assertr/compare/v0.3.7...v0.3.8

[0.3.7]: https://github.com/lpotthast/assertr/compare/v0.3.6...v0.3.7

[0.3.6]: https://github.com/lpotthast/assertr/compare/v0.3.5...v0.3.6

[0.3.5]: https://github.com/lpotthast/assertr/compare/v0.3.4...v0.3.5

[0.3.4]: https://github.com/lpotthast/assertr/compare/v0.3.3...v0.3.4

[0.3.3]: https://github.com/lpotthast/assertr/compare/v0.3.2...v0.3.3

[0.3.2]: https://github.com/lpotthast/assertr/compare/v0.3.1...v0.3.2

[0.3.1]: https://github.com/lpotthast/assertr/compare/v0.3.0...v0.3.1

[0.3.0]: https://github.com/lpotthast/assertr/compare/v0.2.0...v0.3.0

[0.2.0]: https://github.com/lpotthast/assertr/compare/v0.1.0...v0.2.0

[0.1.0]: https://github.com/lpotthast/assertr/releases/tag/v0.1.0
