# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/),
and this project adheres to [Semantic Versioning](https://semver.org/).

## [Unreleased]

### Added

- `AssertionFailure::subject_type_name` records the Rust type of the subject that raised the failure.
- `RenderingBudget` limits rendered leaf values and repeated diagnostic items by default. `RenderingBudget::unlimited()`
  restores complete output.
- `RenderingContext::value` returns a `renderer::Typed` adapter. `Typed::with_type_hint` selects the
  `renderer::TypeHint` (`Full`, `Short`, or `Label`) derived from the value's Rust type, and `Typed::show_type_hint`
  controls whether text output shows it. Hints are hidden by default.
- `BinaryHeap` implements `HasLength` and `Collection`, so heaps support length and order-free collection assertions.
  Its diagnostic presentation is sorted and explicitly marked as such.
- `StableOrderExtractAssertions` provides `get_first`, `get_last`, and `get_single`.
  `RandomAccessExtractAssertions` provides `get_at`. These panic-mode projections borrow the assertion chain and the
  selected element.
- `AssertionFailure` exposes every part of a failure as data: the rendered `actual` and `expected` values, the
  `relation` between them, the `unexpected` value of a negated assertion, `facts` (a list of `Fact { label, value }`),
  nested `children`, and a `FailureKind` tag in `kind`. `AssertionFailure::description()` renders the
  assertion-specific text from those fields. `Fact` and `FailureKind` are re-exported at the crate root. Nested
  failures raised by `_satisfying` assertions, and the elements rejected by positional or `_matching` assertions, are
  `children` located by a `Fact::INDEX` or `Fact::KEY` fact instead of pre-rendered detail strings.

### Changed

- **Breaking:** Custom assertions now render diagnostics through `AssertThat::render().value(...)`, `.values(...)`, or
  `.borrowed_values(...)`.
  `AssertThat::render_value`, `AssertThat::render_values`, `Renderable`, and `RenderableValues` were removed. The
  equivalent `EqContext` methods remain and return the same `renderer::Typed` and `renderer::RenderedValues` adapters.
  `RenderingContext::values` accepts any `Collection` and infers its item type. `borrowed_values` explicitly selects a
  different type borrowed by each item. Collection and map rendering adapters retain their sources by reference and
  obtain elements or entries only when formatted, avoiding temporary collections of references.
- **Breaking:** Container capabilities and diagnostic presentation are now independent. `Collection` inherits
  `HasLength` and replaces `STYLE`, `TYPE_NAME`, `DETERMINISTIC_ITERATION`, `length()`, and the separate `Sequence`
  marker with a required presentation-only `PRESENTATION: renderer::CollectionPresentation`. `StableOrder` is the
  explicit capability for meaningful ordinal positions, and `RandomAccess: StableOrder` adds constant-time
  `element_at`; `LinkedList` has only stable order, while slices, arrays, `Vec`, and `VecDeque` have both.
  `StableOrderAssertions` replaces `SequenceAssertions` and owns every positional finite-collection assertion:
  `starts_with`, `ends_with`, `contains_contiguous`, and `contains_exactly`, including their `_matching` and
  `_satisfying` variants. The equivalent positional `into_iter_*` methods were removed. Explicitly asserted iterators
  retain positional yield-stream assertions, while the remaining borrowed-iteration API is order-free. The
  native-membership `Set` trait was renamed to `SetLookup` and now directly declares the set capability. `Map` likewise
  inherits `HasLength`, replaces
  `TYPE_NAME`, `DETERMINISTIC_ITERATION`, and `length()` with presentation-only
  `RENDERING_ORDER: renderer::RenderingOrder`, and keeps lookup separate in `MapLookup`. `CollectionStyle` was replaced
  by `renderer::GroupStyle` for explicitly styled ad-hoc groups passed to `RenderingContext::values`,
  `RenderingContext::borrowed_values`, or `EqContext::render_values`.
- **Breaking:** The `AssertionFailure::description` and `AssertionFailure::details` fields were removed. The text of
  a failure is derived from its fields: call `description()` for the assertion-specific text, format the failure
  with `Display` for the complete report, or read `facts` for the evidence that `details` carried, each `Fact`
  formatting as `label: value`.
- **Breaking:** `AssertThat::fail`, `AssertThat::fail_with_details`, and the `failure::Failure` trait were removed.
  Custom leaf assertions raise failures the way built-in ones do: `AssertThat::failure(kind)` returns a
  `failure::FailureBuilder` that takes the rendered `actual`, the `relation`, the `expected` or `unexpected` value,
  labeled facts, notes, and nested children, and `raise()` records or panics. `FailureBuilder::detached` builds a
  child failure, and `AssertionFailure::located_at` with `Fact::index` or `Fact::key` locates it in the parent's
  subject.
- Every built-in failure is rendered from its fields by one grammar: `Actual:`, the relation sentence, `Expected:`
  (or `Unexpected:` for a negated assertion), then `Messages:` (chain messages), `Details:` (one bullet per fact as
  `label: value`), and `Nested failures:` (children indented one level and introduced by `At index N:` or
  `At key K:`). A failure without a relation is a direct comparison and keeps the aligned `Expected:` / `Actual:` pair.
  Relations are lowercase sentences without trailing periods and never embed a value. Text matched by downstream tests
  changes accordingly: `does not contain expected: 4` is now `does not contain` followed by `Expected: 4`,
  `contains unexpected: 2` is now `contains` followed by `Unexpected: 2`, `was expected to be empty, but it is not!`
  is now `is not empty`, `does not have the correct length` with an aligned length pair is now
  `does not have the expected length`, `Expected: 2`, and an `Actual length` fact, `is not of expected variant:
  Option::Some` is now `is not the expected variant` followed by `Expected: Option::Some`, `Values were expected to be
  different.` is now `is equal to` followed by `Unexpected:`, and nested per-element failures move from `Details:` to
  `Nested failures:`.
- Facts and children listing the elements of a set or map without a deterministic iteration order are sorted by
  rendered text, and a collection assertion no longer renders the failures of a rejected candidate element to text
  while it is still looking for a satisfying one.
- Positional assertions (`starts_with`, `ends_with`, `contains_exactly`, and their `_matching` variants, on
  collections and iterators) report the first rejected element as a nested failure at its index instead of describing
  the position in prose. Map assertions report a value that differs, fails its predicate, or fails its assertions as a
  nested failure at its key. Explicit iterator diagnostics carry their scan state as labeled facts (`Consumed
  elements`, `Reported length`, `Exhausted at index`, `Extra element at index`).
- `TokioWatchReceiverAssertions::has_changed` and `has_not_changed` no longer require renderer bounds, because their
  failures render no value.
- **Breaking:** `HasLength` is implemented for `str` and `[T]` directly and forwarded through blanket `&T` and
  `&mut T` implementations; the separate reference implementations were removed. Downstream types that implement it
  for both `T` and `&T` must drop the reference implementation. The trait remains dyn-compatible and contains only
  length and emptiness methods.
- Length diagnostics show the subject's short Rust type name, such as `Vec` or `[String]`, instead of its complete
  `core::any::type_name`.
- `renderer::CollectionPresentation` configures list or set syntax, type-hint visibility, and
  `renderer::RenderingOrder::{PreserveIteration, SortByRenderedText}` independently of behavioral capabilities.
  Positional collection diagnostics always preserve iteration order so displayed indexes cannot disagree with
  displayed elements.
- Order-free collection, borrowed-iteration, and iterable-condition diagnostics no longer describe traversal offsets
  as element indexes. Explicit iterator assertions may still report positions in the iterator's yield stream.
- Set relation diagnostics determine whether two sets have different underlying types from their canonical Rust type
  names rather than their presentation hints. Transparent reference forwarding does not make otherwise identical sets
  cross-type.

## [0.7.1] - 2026-09-02

### Added

- Assertion failures capture the asserted expression from `assert_that!`, `assert_that_owned!`, and
  `assert_that_type` and display it in a backticked `Expression:` field. The `fluent` feature provides
  `#[assertr::fluent_expressions]` for scoped expression capture on `must`, `must_owned`, `verify`, and `verify_owned`
  calls, including through a renamed `assertr` dependency, without changing ordinary method resolution for same-named
  user methods. It handles calls written directly in the annotated syntax, including macro invocations used as the
  receiver, but not fluent calls produced by later macro expansion. Derived child assertions start without the root
  expression because they represent a new diagnostic subject.

### Changed

- `assertr-derive`: Bumped to 0.4.1.

## [0.7.0] - 2026-08-30

### Added

- `BTreeSet`, `BTreeMap`, and `LinkedList` subjects, including `BTreeSet` and `BTreeMap` assertions in `no_std` builds.
- `contains_all` for every collection and `into_iter_contains_all` for streaming iterators.
- Sets gained `contains_matching`, `contains_satisfying`, `does_not_contain_matching`, `does_not_contain_satisfying`,
  and the whole `contains_exactly_in_any_order{,_matching,_satisfying}` family.
- Maps gained `contains_entry_satisfying`, `contains_exactly_entries_matching`, and
  `contains_exactly_entries_satisfying`.
- Structured captured failures through the public `AssertionFailure` type.
- `assert_that_owned!` for consuming assertions; `assert_that!` now borrows its subject.
- Reusable borrowed conditions and the fluent `be(condition)` alias.
- Fluent aliases for iterator, async function, program, command, and `reqwest` response assertions.
- `AssertThat::track_assertion` and `AssertThat::fail_with_details` for custom assertions.
- Complete public API documentation and dual-license files in both packages.
- Floating-point `is_normal()` and `is_subnormal()` assertions.
- `TokioMutexAssertions::has_value_satisfying`.
- `reqwest` status-class, header, text-body, and JSON-body assertions and projections.
- Separate `serde-json` and `serde-toml` features; `serde` still enables both.

### Changed

- **Breaking:** Generic container traits replace the per-type traits, and map assertions use native borrowed lookup.
  Prelude calls are mostly unchanged; sets use `contains_exactly_in_any_order`, while custom implementors may need the
  new `Collection`, `Sequence`, `Set`, `Map`, `MapLookup`, or `MapKeyQuery` traits.
- **Breaking:** Capture mode is closure-scoped and returns `Vec<AssertionFailure>`. `Mode` is sealed, mid-chain mode
  conversions are gone, and unused chains use `#[must_use]` warnings instead of panicking destructors.
- **Breaking:** Variant checks such as `is_some`, `is_ok`, and `is_ready` no longer extract. Use `get_some`, `get_ok`,
  `get_err`, `get_ready`, or `get_ascii` before asserting on the contained value.
- **Breaking:** Borrowing is now the default for `assert_that!`, projections, and satisfying callbacks. Consuming
  assertions and owned mappers use `assert_that_owned!`, `must_owned`, `verify_owned`, `satisfies_owned`,
  `satisfy_owned`, or `derive_owned`.
- **Breaking:** `Condition` was renamed to `AssertrCondition`; negated fluent aliases put `not` first; and `need_drop`
  now requires `fluent` (otherwise use `needs_drop`).
- **Breaking:** Generic `StrAssertions` replaces `StringAssertions` and `StrSliceAssertions`. Prelude users are
  unaffected.
- **Breaking:** The `reqwest` and `rootcause` integrations now use version 0.13. `assertr` also stops enabling
  `reqwest` defaults, `tokio/full`, and `serde/derive` for downstream builds.
- **Breaking:** With default features disabled, `std` and `libm` no longer imply `num`; enable `num` explicitly when
  needed. The default feature set is unchanged.
- **Breaking:** `AssertionRenderer<T>` was renamed to `ValueRenderer<T>`, and renderer bounds moved to individual
  assertion methods. Direct trait implementors and generic callers may need new renderer bounds or associated types.
- Collection, string, and path assertions preserve subject names and require fewer renderer capabilities.
- Negative and condition failures now provide clearer structured details, including failing element indexes.
- Dependency selection is narrower: the default build drops from 24 dependencies to 2, and the all-features build from
  141 to 99. Async panic assertions no longer require `UnwindSafe` futures.
- Documentation was reorganized around the rustdoc assertion index and API-owned guides.
- `assertr-derive`: Bumped to 0.4.0.

### Removed

- **Breaking:** `AssertThat::with_capture`, `AssertThat::capture_failures`, `AssertThat::take_failures`,
  `AssertThat::without_capture`, and capture-mode `unwrap_inner`.
- **Breaking:** The free functions `assert_that(&value)` and `assert_that_owned(value)`, deprecated since 0.4.4, were
  removed. Use the `assert_that!` / `assert_that_owned!` macros or the fluent `must()` / `must_owned()` entry points.
- **Breaking:** The empty public modules `details`, `tracking`, and `util` are private now.
- **Breaking:** The hidden, unused `EqContext::add_field_difference_rendered_with` method was removed.

### Fixed

- References to arrays now implement `HasLength`.
- Custom assertion descriptions no longer run into the closing failure banner.
- `is_in_range`, `is_not_in_range`, `Command::has_arg`, `reqwest::Response::has_status_code`, the `http::HeaderValue`
  assertions, Tokio watch assertions, and async panic assertions now report the caller's location.
- Fixed standalone `jiff`, `program`, `reqwest`, `serde*`, and `tokio` feature builds.
- Corrected unnamed time-zone diagnostics and absent path-component checks.
- Exact collection, iterator, and map comparisons now handle multiplicity, duplicate keys, and overlapping custom
  equality relations correctly.
- Failure details no longer leak into later failures in the chain.
- Corrected `Result` variant spelling, erased `Box<dyn Any>` type names, and fluent alias generation edge cases.
- Async panic assertions now catch panics raised while invoking the function, before it returns a future.

## [0.6.2] - 2026-08-20

### Added

- Iterator assertions now provide equality, predicate (`_matching`), and nested assertion (`_satisfying`) variants for
  membership, negative membership, prefixes, suffixes, contiguous subsequences, positional exact matches, and
  unordered exact matches. The same API is available as chainable `into_iter_*` assertions over fresh borrowed
  iteration, together with `into_iter_is_empty`, `into_iter_is_not_empty`, and `into_iter_has_length`.
- Added chainable, non-consuming `ExactSizeIteratorAssertions`: `has_remaining_count`,
  `has_no_remaining_elements`, and `has_remaining_elements`, including their fluent `have_remaining_*` aliases.

### Deprecated

- `into_iter_iterator_is_empty` was renamed to `into_iter_is_empty`. The old name remains as a forwarding alias.

### Fixed

- Iterator membership, prefix, contiguous-subsequence, and exact-match assertions now stream and short-circuit instead
  of eagerly collecting arbitrary iterators. Exact assertions consume or buffer at most `expected.len() + 1` elements,
  diagnostics retain a bounded 16-element tail preview, and unavoidable nontermination for never-deciding potentially
  unbounded iterators is documented.
- Chainable `into_iter_*` assertions now create exactly one borrowed iterator per assertion, including for diagnostics,
  and `into_iter_contains_exactly` no longer requires the redundant `T: PartialEq<E>` bound.
- In capture mode, diagnostic detail messages generated by a failed assertion no longer reappear in the failure
  messages of later assertions on the same chain. This affected every assertion attaching diagnostics such as the
  `Differences: ...` of `is_equal_to`, the element diagnostics of the collection assertions, and the `HashMap`, path,
  and numeric assertions. Assertion internals now pass such diagnostics directly into the raised failure instead of
  storing them on the assertion context, making the leak impossible by construction. Messages added via
  `with_detail_message` / `add_detail_message` are unaffected and still apply to every subsequent failure.

## [0.6.1] - 2026-08-20

### Added

- Added renderer-aware Rust pattern assertions via `is_matching(pattern!(...))` and
  `is_not_matching(pattern!(...))`, including pattern guards and diagnostics that show both the pattern source and the
  rendered actual value.
- Added `is_poisoned()` and `is_not_poisoned()` to `MutexAssertions` for `std::sync::Mutex`.
- Generic named-struct support in `#[derive(AssertrEq)]`, including lifetimes, type parameters, const generics, and
  where-clauses. Generic matchers render field values in their `Debug` output whenever `DebugRenderer` supports the
  field type, mirroring the bounds `#[derive(Debug)]` would require.
- Predicate-based variants for all element-collection assertions (slices, arrays, `Vec`, `VecDeque`):
  `contains_matching(predicate)` asserts that at least one element matches, `does_not_contain_matching(predicate)`
  asserts that no element matches, and `contains_exactly_matching(predicates)` asserts a positional, length-exact
  match. They complement the already existing order-independent `contains_exactly_in_any_order_matching(predicates)`.
- Assertion-based `_satisfying` variants for all element-collection assertions (slices, arrays, `Vec`, `VecDeque`):
  `contains_satisfying`, `does_not_contain_satisfying`, `contains_exactly_satisfying`, and
  `contains_exactly_in_any_order_satisfying`. Instead of a boolean predicate, each closure receives a capture-mode
  `AssertThat` borrowing an element, so all assertions implemented for the element type are applicable. An element
  matches when the closure raises no assertion failure. The captured failures of unsatisfied elements are embedded
  in the final assertion error.
- `Capture` and `Panic` are now exported from the prelude, making it easy to type-annotate `_satisfying` closures
  where inference needs help, e.g. `|it: AssertThat<i32, Capture>|` in closure arrays.
- `satisfies_borrowed(mapper, assertions)` on `AssertThat`, completing the `satisfies_*` family: like
  `satisfies_ref` it projects without cloning, but the closure receives a value-typed `AssertThat<U>` internally
  holding the borrow (as `assert_that!(&value)` produces), so every assertion implemented for `U` is applicable.
  Prefer it over `satisfies_ref` for all sized projection types. Fluent aliases `satisfy_borrowed` and `satisfy_ref`
  were added for parity with the existing `satisfy`.
- Documentation for the `derive`/`satisfies_*` family explaining why each variant exists, how they differ, and when
  to use which, including a comparison table on `satisfies`.
- Crate-level documentation presenting the mental model behind the API (ownership hidden inside `AssertThat<T>`,
  derived assertions, the `satisfies_*` split, capture mode) and documentation for the fluent entry points,
  explaining why `must()` / `must_owned()` and `verify()` / `verify_owned()` are separate functions and why the
  borrowing variants carry the shorter names.
- Added `just` recipes for installing maintenance tools and running checks, Clippy, tests, or the full non-mutating
  validation suite. `just tidy` now only updates dependencies, sorts manifests, and formats code.
- Continuous integration coverage for formatting, Clippy, tests, documentation, hosted and embedded `no_std`, and the
  declared minimum supported Rust version, all respecting the lockfile.

### Deprecated

- `contains_exactly_matching_in_any_order` was renamed to `contains_exactly_in_any_order_matching` so that the
  `_matching` suffix is applied consistently across all predicate-based collection assertions. The old method name and
  its fluent alias remain available but are deprecated.

### Fixed

- Unordered exact collection assertions now compare multiplicities one-to-one. Predicate variants use maximum
  bipartite matching so overlapping predicates are handled correctly and unmatched predicates are reported.
- Inclusive numeric ranges now report a length of one for a singleton and range length calculations avoid signed
  arithmetic overflow, with a clear panic when the mathematical length cannot fit in `usize`.
- `is_close_to()` no longer overflows at integer boundaries, treats positive-infinite deviation as unbounded for
  comparable non-NaN values, preserves equal infinities, and reports negative or NaN allowed deviations as assertion
  failures.
- Dropping an unused or uncaptured assertion during an existing panic no longer starts a second panic and aborts the
  process. Without `std`, panic-on-drop completion checks are disabled because unwind state is unavailable.
- `std::sync::Mutex` lock-state assertions now treat a poisoned but available mutex as unlocked rather than locked.
- Embedded `no_std` builds no longer pull in the standard-library-only `futures` dependency.
- Builds enabling the `num` feature without `std` or `libm` now compile. The float-classification assertions
  (`is_nan()`, `is_finite()`, `is_infinite()`) remain gated behind `std` or `libm`, as documented.
- `#[derive(AssertrEq)]` now accumulates all public-field differences, emits a normal diagnostic for unsupported tuple
  structs, resolves renamed `assertr` dependencies, omits private-field-only generic dependencies, and avoids generated
  helper-name collisions.
- Deriving an assertion from an already derived assertion no longer resets the parent's internal mode state. In capture
  mode, this previously raised a spurious "dropped without capturing" panic from the intermediate assertion even though
  every failure propagated to the root and was captured there.

### Changed

- Aligned `assertr-derive`'s declared MSRV with `assertr` at Rust 1.89 and updated its parser stack to `darling` 0.24
  and `syn` 3.
- Updated stale README version examples and clarified renderer, exact collection, numeric tolerance, mutex, and derive
  behavior.
- All element-collection assertions (slices, arrays, `Vec` and `VecDeque`) now share one internal implementation, so
  every collection type is guaranteed to produce identical failure messages.
- Code generated by `#[derive(AssertrEq)]` now uses runtime support types from the hidden `assertr::__private` module
  instead of emitting them per field, shrinking expansions. `assertr` and `assertr-derive` must be used in the versions
  released together. `assertr` now enforces this with an exact `=` version requirement on `assertr-derive`.
- `assertr-derive`: Bumped to 0.3.0.

## [0.6.0] - 2026-04-30

### Added

- `#[assertr_eq(compare_bounds = "...")]` for `AssertrEq` fields using custom `compare_with` functions, letting the
  derive macro stay agnostic about the comparison's trait bounds.

### Changed

- **Breaking:** Assertion traits now route diagnostics through an `AssertionRenderer<T>` (new type state on
  `AssertThat`, defaulting to `DebugRenderer`) instead of requiring `T: Debug`. Use `.with_renderer(...)` or
  `.with_debug_format(...)` to render non-`Debug` values. Some traits gained additional generics, notably
  `HashSetAssertions` and `HashMapAssertions` now carry the hasher type`S`.
- **Breaking:** Generated `AssertrEq` matcher structs now use a custom `Debug` implementation that prints
  `<unrendered>` for fields whose type does not implement `Debug`.

### Fixed

- `AssertrEq` no longer emits renderer bounds for private fields, matching how those fields are excluded from
  comparison.
- Collection comparisons (`AssertrPartialEq` for slices and maps) now use the active assertion renderer instead of
  always falling back to `DebugRenderer`.

## [0.5.7] - 2026-04-25

### Added

- `VecDeque<T>` assertions for membership, negative membership, ordered exact contents, unordered exact contents,
  predicate matching, and length checks, matching existing `Vec<T` assertions.

### Changed

- Switched remaining places to use `self.fail(|w: &mut String| { writedoc! {w, r"..."} })` over `format_args!` style
  assertion violation reporting, improving readability and maintainability of user-facing messages.

## [0.5.6] - 2026-04-17

### Fixed

- Restored `--no-default-features` / no-std compatibility by using `core`/`alloc` paths in core assertions.
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

[Unreleased]: https://github.com/lpotthast/assertr/compare/v0.7.1...HEAD

[0.7.1]: https://github.com/lpotthast/assertr/compare/v0.7.0...v0.7.1

[0.7.0]: https://github.com/lpotthast/assertr/compare/v0.6.2...v0.7.0

[0.6.2]: https://github.com/lpotthast/assertr/compare/v0.6.1...v0.6.2

[0.6.1]: https://github.com/lpotthast/assertr/compare/v0.6.0...v0.6.1

[0.6.0]: https://github.com/lpotthast/assertr/compare/v0.5.7...v0.6.0

[0.5.7]: https://github.com/lpotthast/assertr/compare/v0.5.6...v0.5.7

[0.5.6]: https://github.com/lpotthast/assertr/compare/v0.5.5...v0.5.6

[0.5.5]: https://github.com/lpotthast/assertr/compare/v0.5.4...v0.5.5

[0.5.4]: https://github.com/lpotthast/assertr/compare/v0.5.3...v0.5.4

[0.5.3]: https://github.com/lpotthast/assertr/compare/v0.5.2...v0.5.3

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
