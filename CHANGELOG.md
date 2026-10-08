# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/),
and this project adheres to [Semantic Versioning](https://semver.org/).

## [Unreleased]

## [0.8.0] - 2026-10-08

### Added

- Reusable expectations. An `Expectation` evaluates a subject once and explains a rejection or describes an unmet
  expectation through the structured failure builder. Apply one to a chain with `matches` (fluent alias
  `match_expectation`), `apply_assertion`, or `test_assertion`, which also returns the successful observation. Every
  built-in assertion family is backed by public expectations that compose the same way as custom definitions.
- The `matchers` catalog re-exports every public expectation, with common checks at its root and subject namespaces for
  family-specific names. Combine them with `all_of`, `any_of`, `anything`, `predicate`, `predicate_list`, `satisfying`
  (assertion callbacks), `dereferenced` (any `Deref` subject), `each`, `elements_are!`, `elements_are_in_any_order!`,
  `entries_are!`, `matchers!` (heterogeneous matcher lists), and `pattern!`. Use `eq` (an alias for `equal_to`), `lt`,
  `gt`, `le`, and `ge` for comparisons. Ordering matchers reject incomparable values. `DoesNotMatchPattern` matches a
  pattern negatively. Matcher lists and keyed value expectations require explicit matchers. `matchers![..]` builds
  heterogeneous lists, including keyed `matchers![entry(..), ..]` lists. Arrays, slices, and vectors hold one matcher
  type. Passing a plain value where a matcher is expected produces a compile error that suggests `eq(value)`. All of
  this works without optional features or `std`.
- Domain checks need no trait implementation: `predicate(..).described_as(..).rejected_as(..)` names a boolean check
  and its rejection, and `matchers::field` applies a matcher to one field and reports evidence at that field. Return
  them from a function as `impl Expectation`. Full implementations can keep the default `Expectation::KIND`
  (`FailureKind::Predicate`) and explain the common case with `FailureBuilder::relations`.
- `partial!` matches selected struct or enum fields without derives or attributes on domain types and renders only the
  selected leaves. Each selected field requires an explicit matcher, such as `eq(value)` or a nested `partial!`.
  Qualified constructor paths work with the optional `variant` prefix. It also works through facade crates that
  re-export `assertr`. Enable the new `partial` feature, which supports `no_std` with `alloc`.
- Map assertions `contains_entry_matching` and `contains_value_matching` accept composed value matchers.
- Reference identity assertions `is_same_instance_as` and `is_not_same_instance_as`, plus collection membership and
  exact comparisons of borrowed targets that preserve duplicate counts, without equality or target renderer bounds.
- Borrowed panic-mode element projections `get_first`, `get_last`, and `get_single` for `StableOrder` collections, and
  `get_at` for `RandomAccess` collections.
- `BinaryHeap` supports length and order-free collection assertions.
- Collection, set, and map assertions work on `&mut` subjects, such as `assert_that_owned!(&mut vec)`.
- `Box<dyn Any + Send>` and `Box<dyn Any + Send + Sync>` support `is_of_type`, `has_type`, and `has_type_ref`, for
  example on payloads from `catch_unwind` or `JoinHandle::join`. Box and panic-payload `is_of_type` checks preserve
  the subject and work in panic and capture mode.
- `is_close_to` supports `core::num::Wrapping`. The `NumericDistance` documentation explains how to check foreign
  numeric types.
- `RenderingBudget` limits diagnostics to 256 items per group and 4,096 characters per rendered leaf by default.
  Configure it with `with_max_items` and `with_max_leaf_characters`, apply it with `with_rendering_budget`, or disable
  both limits with `RenderingBudget::unlimited()`. Reports state how many items or nested failures were omitted.
  Limits never change assertion outcomes.
- `with_panic_presentation` installs an owned `'static + RefUnwindSafe` closure producing the panic text from a failure,
  shared by derived assertions. With `std`, a panicking presentation falls back to the built-in report.
- `AssertionFailure` and `AssertionFailures` implement `core::error::Error` with readable `Display` and `Debug` reports.
  `Fact` and `renderer::Rendered` expose their diagnostic data as public fields.
- `RenderingContext` renders values, collections, maps, synthetic value and key/value lists, and one-field variants and
  structs, including inaccessible fields, into owned `Rendered` trees through the active leaf renderer and budget.
  Rendered values retain Rust type metadata. `compact()` renders leaves for inline use, and synthetic evidence selects
  its ordering through `RenderingOrder`.
- docs.rs shows the features required by feature-gated items.
- Public expectation types implement `Debug` and `Clone`, bounded on their operands and nested matchers, and `Copy`
  when they hold no data, only concrete copyable data, or only borrowed targets. Callback definitions such as
  `Predicate`, `Satisfying`, and `Pattern` are `Clone` when their callback is and omit it from `Debug`. Identity
  definitions show addresses. Rejection types such as `MissingElementsRejection` and `EntryRejection` implement
  `Debug`.
- `#[fluent_expressions(crate = <path>)]` names a re-exported runtime, so expression capture works in crates that
  reach `assertr` only through a facade crate:
  `#[my_facade::assertr::fluent_expressions(crate = ::my_facade::assertr)]`. Without the argument, the attribute still
  finds the possibly renamed `assertr` dependency.

### Changed

- **Breaking:** Removed `AssertrCondition`, `ConditionAssertions`, `IterableConditionAssertions`, and their `is`,
  `has`, `are`, `have`, and fluent `be` methods. Use `matches` with a matcher or `predicate`.
- **Breaking:** Equality, ordering, and collection, iterator, and map value comparisons use standard `PartialEq` and
  `PartialOrd` with owned or borrowed operands selected by `BorrowFor` from the `borrow-for` crate, re-exported as
  `assertr::borrow_for`. Ordinary values and references work automatically. String literals remain usable for equality
  with `String` subjects, elements, and map values without allocation. Arrays compare with vectors, and vectors compare
  with slice subjects. Custom operand wrappers opt in by declaring their borrowed view. Range containment accepts owned
  or borrowed operands with owned or borrowed range subjects. The unbounded range `..` needs an explicit bound type.
  This replaces `AssertrPartialEq` and the public `cmp` API, including `Eq`, `eq`, `any`, `EqContext`, and
  `Differences`. Replace the former root and prelude `eq(value)` with `matchers::eq(value)`, and `any()` with
  `matchers::anything()` or `..` in `partial!`. Other cross-type comparisons use an explicit borrowed view
  implementation, a predicate, or a custom expectation. `NumAssertions::is_close_to`,
  `ZonedAssertions::is_in_time_zone`, and `TokioWatchReceiverAssertions::has_current_value` take `BorrowFor` operands
  too, so untyped `.into()` arguments may need a type annotation.
- **Breaking:** Bulk value, key, and entry methods accept finite slice-backed `AsRef` lists instead of iterators. This
  affects `contains_all`, `into_iter_contains_all`, `contains_keys`, `contains_exactly_entries`, and
  `contains_exactly_entries_satisfying`, including their fluent aliases, which lose their iterator generic parameter.
  Collect generators explicitly with `.collect::<Vec<_>>()`.
- **Breaking:** Removed the published `MapKeyQuery` extension API. Bulk map-key operands and keyed matchers use
  `BorrowFor<K>::View` with the stored key as context, plus `Borrow<View>`. Custom renderers render the selected query
  view instead of the operand wrapper. Native `MapLookup` requirements and single-key `&Q` calls are preserved. Slice
  operands now work with vector keys.
- **Breaking:** Map entry methods take the key query first in turbofish order: `contains_entry::<Q, E>`,
  `does_not_contain_entry::<Q, E>`, and `contains_entry_satisfying::<Q, A>`. The value parameter `E` of
  `contains_entry` and `does_not_contain_entry` is now the operand type instead of the borrowed value target.
- **Breaking:** Removed `AssertrEq`, its generated companion types and helper attributes, and the `derive` feature.
  Use `matches(partial!(...))` with `features = ["partial"]`.
- **Breaking:** `assertr-macros` 0.5.0 replaces `assertr-derive` as the procedural macro crate. Direct users must
  update their dependency and replace `assertr_derive::` paths with `assertr_macros::`. Use `assertr::partial!` for
  structural matching.
- **Breaking:** `#[fluent_aliases]` no longer wraps generated aliases in `#[cfg(feature = "fluent")]`. Trait authors
  make aliases optional with `#[cfg_attr(feature = "fluent", assertr_macros::fluent_aliases)]`. `#[fluent_alias(..)]`
  reports compile errors for names that are not string literals or identifiers, keyword names become raw identifiers,
  and generated aliases link to their original method instead of repeating its documentation.
- **Breaking:** Collection, iterator, and map `*_matching` methods and fluent aliases accept matchers instead of bare
  predicates. Wrap closures with `matchers::predicate`, predicate arrays with `matchers::predicate_list`, and keyed
  matcher lists with `[entry(..)]`, `matchers![entry(..), ..]`, or `matchers::entry_matchers`. Predicate rejections
  render the rejected subject, so their diagnostics require a renderer for it.
- **Breaking:** `StableOrder` and `StableOrderAssertions` replace `Sequence` and `SequenceAssertions` and own positional
  prefix, suffix, contiguous, and exact comparisons. Removed `into_iter_starts_with`, `into_iter_ends_with`,
  `into_iter_contains_contiguous`, and `into_iter_contains_exactly`, including their `_matching` and `_satisfying`
  variants and fluent aliases. Call `starts_with`, `ends_with`, `contains_contiguous`, or `contains_exactly` on
  `StableOrder` collections, or assert an owned iterator with `assert_that_owned!(x.into_iter())`.
- **Breaking:** Custom `Collection` and `Map` implementations must move `length` to `HasLength`, replace collection
  `STYLE` and `TYPE_NAME` with `PRESENTATION: CollectionPresentation`, and replace map `TYPE_NAME` with
  `RENDERING_ORDER: RenderingOrder`, using the types in `renderer`. Custom set implementations and bounds must rename
  `Set` to `SetLookup`. `Collection`, `StableOrder`, `RandomAccess`, `SetLookup`, `Map`, and `MapLookup` gain blanket
  `&mut C` implementations, which conflict with downstream implementations for `&mut` types.
- **Breaking:** `&R` implements `ValueRenderer<T>` whenever `R` does, so `with_renderer(&renderer)` works with derived
  assertions for renderers that are not `Clone`. Renderers implementing `ValueRenderer` for both a type and its
  references must remove the reference implementations.
- **Breaking:** `HasLength` covers `str` and `[T]` directly and forwards through blanket `&T` and `&mut T`
  implementations. Downstream types implementing it for both a value and its references must remove their reference
  implementations. Length and element assertions on `HashSet` and `HashMap` no longer require `S: BuildHasher`.
- **Breaking:** Assertion traits share one generic shape: subject parameters, then the mode where a signature needs it,
  then a renderer `R` defaulting to `DebugRenderer`. Unused lifetimes are gone. This changes `OptionAssertions<T, M,
  R>`, `ResultAssertions<T, E, M, R>`, `PollAssertions<T, M, R>`, `BoxAssertions<R>`, `HttpHeaderValueAssertions<M, R>`,
  `RootcauseDynamicReportAssertions<M, R>`, and `ProgramAssertions<'a, R>`. `NumAssertions<T, R>`, `StrAssertions<R>`,
  `LengthAssertions<T, R>`, and `PathAssertions<P, R>` replace their associated `Subject` and `Renderer` types with
  parameters. Generic bounds naming these traits must be updated, for example to `NumAssertions<T, MyRenderer>`. String
  assertions render the subject's `str` view and require `ValueRenderer<str>` instead of a renderer for the subject
  type, so renderers supporting only `String` must also render `str`.
- **Breaking:** `capture`, `verify`, and `verify_owned` return `AssertionFailures` instead of a vector. Use
  `into_vec()` where a vector is required.
- **Breaking:** `AssertionFailure` replaces `description` and `details` with structured values, relations, facts,
  nested failures, typed child paths and constraints, type metadata, and `FailureKind` tags. Read the fields directly,
  or use `Display` for text.
- **Breaking:** Custom leaf assertions implement `Expectation` instead of using `fail`, `fail_with_details`, and
  `failure::Failure`. Execution adapters that cannot be expressed as an expectation start a failure with
  `self.failure(kind)`, add structured evidence, and pass it to `self.raise(..)`.
- **Breaking:** Custom diagnostic code must replace `render_value`, `render_values`, `Renderable`, and
  `RenderableValues` with the methods of `AssertThat::render()`, such as `value`, `collection`, and `borrowed_values`,
  which return owned `Rendered` values. Replace `CollectionStyle` with `renderer::GroupStyle`.
- **Breaking:** Diagnostic operands now use the active renderer, including strings, paths, numeric evidence, integration
  values, and original errors. Custom renderers must implement the method-level `ValueRenderer` bounds, and generic
  callers must supply `R` in formatting, exact-size iterator, and reqwest response assertion trait bounds.
- **Breaking:** `as_json()` and `as_toml()` return owned `Result` subjects that preserve serialization errors, and the
  `json()` and `toml()` adapters are removed. Replace string chains with `.as_json().get_ok()` in panic mode or
  `.as_json().is_ok_satisfying(...)` in capture mode, and likewise for TOML.
- **Breaking:** `has_type` and `has_type_ref` move from `BoxAssertions` and `PanicValueAssertions` to
  `BoxExtractAssertions`, which covers boxed `dyn Any` payloads and `PanicValue`. `BoxAssertions` keeps the
  non-extracting `is_of_type`, and `PanicValueAssertions` is removed. Import `BoxExtractAssertions` or use the prelude.
- **Breaking:** `RootcauseReportAssertions`, `RootcauseDynamicReportAssertions`, and
  `RootcauseDynamicReportExtractAssertions` cover both `Report` and `ReportRef` subjects. Replace imports of
  `RootcauseReportRefAssertions`, `RootcauseDynamicReportRefAssertions`, and
  `RootcauseDynamicReportRefExtractAssertions`.
- **Breaking:** `is_matching` and `is_not_matching` require `Fn` pattern guards, so guards consuming their captures no
  longer compile.
- **Breaking:** Replace `ProgramAssertionsRequiringPanicMode` and its `exists_and` method and fluent alias with
  `ProgramExtractAssertions::get_resolved_path`.
- **Breaking:** Range `contains_element` and `does_not_contain_element` consume and return their assertion chain.
  Chain successive checks or start a new chain instead of reusing a moved one.
- **Breaking:** `TokioMutexAssertions` takes the mode parameter (`TokioMutexAssertions<T, M, R>`). Its
  `has_value_satisfying` runs the callback in the chain's mode, like `is_some_satisfying`, so callback failures are
  reported directly instead of nested under a mutex failure, and the method no longer requires `ValueRenderer<T>`.
- **Breaking:** Tokio watch `has_changed` and `has_not_changed` move to `TokioWatchReceiverAssertions`, supporting panic
  and capture modes without renderer or `Clone` bounds. Replace imports of the removed
  `TokioWatchReceiverExtractAssertions`.
- **Breaking:** Memory assertions move from `assertions::std::mem` to `assertions::core::mem` and work without `std`.
  Import `MemAssertions` from the core or crate-wide prelude.
- **Breaking:** HTTP header values lose their own `is_empty` and `is_not_empty`. `HeaderValue` implements `HasLength`,
  so the same calls and `has_length` resolve to `LengthAssertions` through the prelude and count bytes. `is_sensitive`
  and `is_insensitive` require `ValueRenderer<HeaderValue>` instead of `ValueRenderer<bool> + Clone` and report the
  header value.
- Failing HTTP header checks show header values, including values marked sensitive, and escape non-ASCII bytes. This
  covers reqwest `has_header_value` and `does_not_have_header`, and `HeaderValue` checks such as `is_ascii`.
- **Breaking:** The `http` feature enables `std`. `serde-json`, `serde-toml`, and `serde` no longer enable `std` and
  work on embedded `no_std` targets with `alloc`.
- **Breaking:** `jiff` is built without its default features. Enable jiff's time-zone database features in your own
  dependency if your tests look up time zones by name.
- **Breaking:** `NumAssertions::is_close_to` uses rounded absolute floating-point distance through
  `assertions::distance::NumericDistance`, retaining overflow-safe integer comparisons without requiring `Clone`.
  Generic callers need this bound next to `Num`, and custom numeric types implement `zero_distance` and
  `checked_distance`. Numbers and `jiff::SignedDuration` share one `IsCloseTo` expectation, which needs no optional
  feature and renders its values compactly.
- **Breaking:** `SignedDurationAssertions::is_close_to` takes `BorrowFor<SignedDuration>` operands instead of concrete
  `SignedDuration` parameters, so untyped `.into()` or `Default::default()` arguments no longer infer. It compares exact
  inclusive nanosecond distances without arithmetic panics at extreme values and fails explicitly for a negative
  allowed deviation.
- **Breaking:** `AssertThat` inherits unwind-safety requirements from its subject and renderer. Callers using
  `catch_unwind` with non-unwind-safe state must review that state before explicitly using `AssertUnwindSafe`.
- **Breaking:** Debug and Display comparisons preserve quotes and escapes exactly, including rootcause current-context
  Debug comparisons. Use `has_debug_string("42")` for preformatted numeric expectations and include Debug's surrounding
  quotes when expecting string output.
- Failure reports use one layout for values, relations, messages, facts, and nested failures, with child paths such as
  `At [2]:`. An empty line separates nested failures when any of them spans several lines. Update diagnostic text
  snapshots. Notable changes: disjunctions and existential matchers such as
  `contains_matching` report one nested group ("does not match any alternative", "does not contain a matching element"),
  contiguous matcher searches over collections and iterators report "does not contain these elements contiguously" and
  group evidence per rejected window with a `Window start` fact, exhausted scans describe the first missing expectation
  and the required length, length rejections decided by an exact `size_hint` report the reported and expected lengths
  without consuming input, and length diagnostics use short Rust type names.
- Element, set, and map diagnostics for hash collections sort values and per-element evidence by rendered text before
  applying item limits and mark multi-item output as sorted. Positional diagnostics preserve iteration order, and
  order-free diagnostics omit traversal indexes.
- Unordered matching searches each actual/expected pair at most once and reports missing expectations, unexpected
  elements, and surplus occurrences of satisfied expectations. To explain a failure, rejected pairs are evaluated
  again, so their matchers and callbacks can run twice. Iterator `contains_exactly_in_any_order` reports like the
  collection assertion. Iterator `contains_matching` and `contains_contiguous_matching` retain the first rejected
  candidates within the rendering budget.
- Positive collection, stable-order, and iterator `*_satisfying` assertions no longer require element renderers. Map
  callback assertions require key renderers only.
- `with_conditional_detail_message` accepts `FnOnce` closures.
- Set relation failures no longer add a note when the sets have different Rust types.

### Fixed

- **Breaking:** Path existence checks pass only when filesystem inspection confirms the result. `exists` and
  `does_not_exist` treat a missing path, including one below a non-directory ancestor, as absent and fail with the
  original I/O error for any other inspection error. `is_a_file`, `is_a_directory`, and `is_a_symlink` no longer
  report inspection errors as a missing path. These methods require `ValueRenderer<std::io::Error>`, which the default
  `DebugRenderer` provides.
- `into_iter_has_length` no longer trusts an agreeing exact `size_hint`. It verifies it by reading at most one element
  beyond the expected length.
- Tokio `RwLock` state assertions retain acquired guards while rendering failures, preventing lock reacquisition races.
- Rootcause current-context type mismatches, range `is_outside_of_range`, and standard and Tokio lock `is_free` aliases
  report the caller's assertion location. Owned rootcause report assertions include the `Expression:` line and no
  longer require a `Clone` renderer.
- Range diagnostics preserve excluded lower bounds.
- `capture` on a derived chain lists local detail messages, including those added in the callback, before inherited
  messages.
- `fluent_aliases` supports async assertion methods by awaiting the delegated call.
- `fluent_expressions` no longer rewrites `verify` and `verify_owned` callback arguments, so user-defined methods with
  `Fn`, `FnMut`, `FnOnce`, function-pointer, or macro-generated callbacks compile unchanged. A user-defined
  `#[track_caller]` `verify` whose result contains an inner Assertr verification's failures attaches the outer receiver
  expression to them.

### Removed

- **Breaking:** Removed deprecated `contains_exactly_matching_in_any_order`, `contain_exactly_matching_in_any_order`,
  and `into_iter_iterator_is_empty`. Use `contains_exactly_in_any_order_matching`, its fluent alias, and
  `into_iter_is_empty` respectively.
- **Breaking:** Removed `Actual::map`. Match on the `Actual` variants instead.
- `indoc` is no longer a dependency of `assertr`.

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

[Unreleased]: https://github.com/lpotthast/assertr/compare/v0.8.0...HEAD

[0.8.0]: https://github.com/lpotthast/assertr/compare/v0.7.1...v0.8.0

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
