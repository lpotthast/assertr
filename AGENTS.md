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
cargo test -p assertr --no-default-features --features std
cargo test -p assertr --no-default-features --features num
cargo test -p assertr
cargo test -p assertr --all-features
cargo test -p assertr-derive
cargo test -p assertr-no-std-tests

# Test a single test
cargo test -p assertr --all-features <test_name>

# Every feature on its own, so no feature can hide behind another's dependencies
just check-each-feature

# Hosted and embedded no_std
just check-no-std

# Lint & format
cargo fmt
cargo clippy -p assertr --all-targets --all-features -- -W clippy::pedantic

# Update deps, sort Cargo.toml dependencies, format
just tidy

# Full non-mutating validation suite: fmt check, check matrix, no_std, clippy pedantic, tests, doc
just verify
```

The pinned configuration matrix is: minimal (`--no-default-features`), default, every single feature alone,
`--all-features`, hosted `no_std` (`assertr-no-std-tests`), and embedded `no_std`
(`--target thumbv8m.main-none-eabihf`). `just verify` runs all of it.

## Required Change Workflow

Use `## [Unreleased]` in `CHANGELOG.md` as the default landing place for change entries while work is in progress.
Use the last "## [version] - date" section if this version was not yet released. It is the next planned upcoming release
and all changes must be appended to that section instead of falsely being added to "## [Unreleased]".

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

Every publicly exported item is public API and follows the usual SemVer rules unless its documentation explicitly
says otherwise. Anything that exists only because a macro expands to it belongs in the explicitly unsupported
`__private` module, and anything else stays `pub(crate)` inside a private module. Do not add a `pub mod` whose only
purpose is to hold `pub(crate)` items: that publishes an empty documentation page and nothing else.

`README.md` doubles as the crate front page: `assertr/src/lib.rs` includes it as crate documentation unconditionally,
so every `cargo test -p assertr <features> --doc` run, including `--no-default-features`, runs every README code block
as a doctest. README examples must therefore compile and pass as top-level snippets in the minimal configuration. They
import what they use, prefer core assertions over feature-gated ones (hidden `# #[cfg(feature = "...")]` lines work but
render as literal `#` lines on GitHub), and are not wrapped in `#[test]` functions, which doctests strip without
running them. The README is a landing page. Detailed guides belong in the rustdoc of the API item that owns the topic,
which the README links to.

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
7. Run `just tidy`, then `just verify` as the final verification step.

## Architecture

**assertr** is a fluent assertion library for Rust with `no_std` support. The core API:

```rust
assert_that!(value).is_equal_to(expected).has_length(5); // Borrows; `value` stays usable.
assert_that_owned!(fallible_closure).panics();           // Takes ownership (consuming assertions only).
```

### Core types (in `assertr/src/`)

- **`assert_that!` macro** (`__private/assert_that_macro.rs`) - Borrows its input (`Actual::Borrowed`): named
  values remain usable afterwards, temporaries live until the end of the enclosing statement. Expressions that are
  themselves references are unwrapped one level via autoref specialization, so `assert_that!(&value)` and
  `assert_that!(value)` both yield an `AssertThat<Value>`; references to unsized targets (`&str`, `&Path`) keep the
  reference as subject.
  `assert_that_owned!` takes ownership (`Actual::Owned`) for consuming assertions (`panics()`, terminal iterator
  assertions, `unwrap_inner()`).
- **`AssertThat<'t, T, M>`** (`lib.rs`) - The central struct. `T` is the value type, `M` is the mode (`Panic` or
  `Capture`). All assertion methods are implemented as trait impls on this type. Methods return `Self` for chaining.
- **`Actual<'t, T>`** (`actual.rs`) - Enum holding either `Borrowed(&'t T)` or `Owned(T)`.
- **`Mode` trait** (`mode.rs`) - Sealed; `Panic` (fail immediately) and `Capture` (collect failures) are stateless
  unit type-state markers. `Mode::CAPTURES` drives the failure behavior; derived assertions always share their
  root's mode. Capture mode is closure-scoped via `AssertThat::capture(assertions)` (fluent: `verify(assertions)`),
  which returns the collected failures.
- **`AssertionFailure`/`Failure`/`Fallible`** (`failure.rs`) - `AssertionFailure` is the public structured failure
  (location, subject name, description, per-failure details, chain-level messages), rendered only at the panic or
  display boundary via `Display`. `Failure` builds the description; `Fallible` routes failures to the root.
- **`AssertrCondition<T>` trait** (`condition.rs`) - Reusable predicates used with `.is()` / `.has()` and, per
  element of an iterable, `.are()` / `.have()`. Implemented for `&C` so instances are reusable; the `Assertr` prefix
  avoids exporting the collision-prone name `Condition` through the prelude.
- **`AssertrPartialEq<Rhs>`** (`lib.rs`) - Custom equality trait with `EqContext` for field-by-field difference
  reporting.
- **`Type<T>` / `assert_that_type<T>()`** (`lib.rs`) - A zero-sized subject and entry point for assertions about a
  type itself.
- **Tracking** (`tracking.rs`, private module) - Counts assertions per chain via the public inherent
  `AssertThat::track_assertion`. Nothing panics on drop: unused panic-mode chains are caught at compile time via
  `#[must_use]`, and `capture` panics (as a normal function) when its closure performed no assertions.
- **Private modules** - `tracking.rs`, `details.rs`, `util/`, and `conversion.rs` are private; the API they carry is
  reachable through inherent `impl` blocks on `AssertThat` or through the prelude, not through a module path.
- **`__private`** (`__private/`) - Everything the macros and the derive-generated code expand to:
  `__private::assert_that_macro::{Wrap, Fallback, owned}` for `assert_that!` / `assert_that_owned!`,
  `__private::new_pattern` for `pattern!`, and the derive support types. `#[doc(hidden)]`, exempt from semver, never
  named directly. All `#[macro_export]` macros route through `$crate::__private::...`.

### Assertion modules (`assertr/src/assertions/`)

Assertions are organized by type category, each in its own module with a trait:

- `collection/`, `set/`, `map/` - The generic container families. Four public extension traits, no per-type
  assertion traits, so implementations cannot drift apart in bounds or diagnostics:
  - `Collection` (`collection/mod.rs`) carries `Item`, `STYLE`, `TYPE_NAME`, `length()`, and `elements()`.
    `CollectionAssertions` (`collection/assertions.rs`) is blanket-implemented for every
    `AssertThat<C>` where `C: Collection` and holds the order-free assertions. Implemented for slices, arrays,
    `Vec`, `VecDeque`, `LinkedList`, `BTreeSet`, and `HashSet`.
  - `Sequence: Collection` is a marker meaning "the element order is meaningful". `SequenceAssertions`
    (`collection/sequence.rs`) holds the three order-sensitive assertions, so `contains_exactly` on a set is a
    compile error rather than a test that passes for the wrong reason. Sets are deliberately not sequences.
  - `Set: Collection` (`set/mod.rs`) adds native membership lookup; `SetAssertions` holds the subset, superset, and
    disjointness relations, each accepting *any* other `Set` so hashers and container types can differ.
  - `Map` (`map/mod.rs`) carries `Key`, `Value`, `TYPE_NAME`, `length()`, and `entries()`. Lookup by key
    is the separate `MapLookup<Q>: Map` trait (`get_key_value()`), whose bounds live on each implementation rather
    than on the trait, so a `HashMap` demands `Q: Hash + Eq` and a `BTreeMap` demands `Q: Ord`, and no key type has
    to satisfy both. `MapAssertions` is blanket-implemented over `Map`; every assertion that queries a key bounds
    `MapLookup<Q>` (bulk methods: `Map<Key = K> + MapLookup<K>`, since `MapLookup<Self::Key>` is a bounds cycle).
    Implemented for `HashMap` and `BTreeMap`.

  Container syntax is rendered by `assertr`; custom renderers only need implementations for element, key, and value
  leaf types, never for `Vec<T>`, slices, sets, maps, or borrowed intermediary collections. Every assertion body lives
  once in the family's private `imp.rs`. `set/` and `map/` sit outside `std/` on purpose:
  `BTreeSet`/`BTreeMap` are `alloc`, so the families reach `no_std` builds; only the `HashSet`/`HashMap` impls are
  `#[cfg(feature = "std")]`. `TYPE_NAME` independently puts the container's name in front of the rendered subject;
  `CollectionStyle` and `Collection::STYLE` select list (`[...]`) or set (`{...}`) delimiters. Built-in sequences use
  `List` with no name, while built-in sets use `Set` with their concrete type name. The four assertion traits are
  re-exported by the prelude for method lookup. The implementor-facing `Collection`, `CollectionStyle`, `Sequence`,
  `Set`, and `Map` items stay out of the prelude.
- `core/` - Fundamental types: `PartialEq`, `PartialOrd`, `bool`, `char`, `Option`, `Result`, `Iterator`, `Tuple`,
  and `StrAssertions` (`core/string.rs`), blanket-implemented for every `AsRef<str>` subject
- `alloc/` - Heap types: `Box`, `PanicValue`
- `std/` - Std types: `Path`, `Command`, `Mutex`, plus `MemAssertions` for `Type<T>`
- `num/` - Numeric: `is_zero`, `is_positive`, `is_close_to`, `is_nan`, etc.
- `program.rs`, `http/`, `jiff/`, `tokio/`, `reqwest/`, `rootcause/` - Feature-gated external crate integrations

### Derive macro (`assertr-derive/`)

`#[derive(AssertrEq)]` generates a `{StructName}AssertrEq` companion struct with `Eq<FieldType>` fields for field-level
partial equality. Supports `#[assertr_eq(map_type = "...")]` and `#[assertr_eq(compare_with = "...")]` attributes. Only
processes public fields. Uses `darling` for attribute parsing.

Compile-fail tests use `trybuild` in `assertr-derive/tests/`.

### Key patterns for adding assertions

First check whether the assertion belongs to an existing generic family instead of a new per-type trait: order-free
element assertions go into `CollectionAssertions` (implement `Collection` for the new type rather than writing a
trait), order-sensitive ones into `SequenceAssertions` via `Sequence`, set relations into `SetAssertions` via `Set`,
map assertions into `MapAssertions` via `Map`, string assertions into `StrAssertions`, length assertions into
`LengthAssertions` via `HasLength`. Per-type traits are for genuinely type-specific behavior only. Test each
assertion's behavior and exact diagnostics beside its single implementation in the family's `assertions.rs` or
`sequence.rs`. Test each built-in adapter's `length()`, iteration/lookup, style, type name, and reference
forwarding beside the extension-trait impls in the family's `mod.rs`. Keep downstream implementor and `no_std`
availability checks in the existing integration fixtures; do not duplicate every assertion across every adapter in
a conformance matrix.

Where a new trait is warranted, it follows this pattern:

1. Define a trait (e.g., `FooAssertions`) with methods returning `Self`
2. Annotate the trait with `#[cfg_attr(feature = "fluent", assertr_derive::fluent_aliases)]` to auto-generate fluent
   aliases (e.g., `is_true` → `be_true`, `has_length` → `have_length`). Negated methods put `not` first
   (`is_not_empty` → `not_be_empty`, `has_not_changed` → `not_have_changed`, `does_not_contain` → `not_contain`);
   the possessive `has_no_x` keeps its order (`have_no_x`); namespace prefixes such as `into_iter_` are kept in
   front of the derived alias. Name new methods so a rule applies (the rules live in
   `assertr-derive/src/fluent_aliases/naming.rs`); `get_*` methods pass through because their verb is already
   imperative; use `#[fluent_alias("...")]` only where none fits (`is` → `be`), and `#[no_fluent_alias]` on
   deprecated names.
3. Implement it for `AssertThat<'_, Foo, M, R>` where `M: Mode`, independently of the renderer's capabilities. Never
   put `R: ValueRenderer<_>`, `R: Clone`, `R: Debug`, or a concrete `DebugRenderer` on the assertion-trait impl:
   one method's needs must not hide the entire trait. Put the exact renderer and `Clone` bounds on each method that
   needs them, in both the trait declaration and its impl. If a trait does not already expose `R`, use an associated
   renderer type (and, when necessary, an associated subject type) or add a renderer parameter with
   `R = DebugRenderer` to preserve existing explicit trait paths. Every projection or extraction returns
   `AssertThat<NewSubject, M, R>` so the active renderer is preserved. Add a `NoRenderer` compile-time regression
   proving the trait remains implemented without any rendering support. See `ValueRenderer`'s *Capability bounds
   belong to methods* section for the rationale.
4. Use `self.track_assertion()` at the start of each method, unconditionally. Composing assertions (those whose whole
   body delegates through `satisfies` and friends) must not call it: the delegated assertions already track
5. On failure, call `self.fail(...)` with a formatted message, or `self.fail_with_details(details, ...)` when the
   failure carries evidence of its own. Both are public methods, pinned by `assertr/tests/custom_assertions.rs`
6. Annotate all assertion methods with `#[track_caller]`
7. Negatives are explicit paired methods (`is_x` / `is_not_x`, `contains` / `does_not_contain`); there is no generic
   `.not()`. Add a negative only when the negated fact is commonly asserted (absence, inequality, non-emptiness,
   not locked, does not exist, does not panic) and no existing assertion already expresses it (`is_none` is the
   negative of `is_some`, `is_err` of `is_ok`, `is_false` of `is_true`, `is_pending` of `is_ready`; there is no
   `is_not_some`). Everything else is negated through a computed `bool`, `satisfies`, or a condition. A negative is a
   hand-written assertion with its own diagnostics: its failure text names the negation ("unexpected",
   "not expected", "to not be ...") and renders the evidence a plain boolean flip would lose (the unexpected
   element, the matching elements, the violated range). Antonym synonyms (`is_free` for `is_not_locked`) are
   allowed, one per positive. Cover every pair's complementarity and negative-specific evidence in the negative
   method's colocated test module.
8. Give every assertion method its own test module (`mod tests { mod is_foo { .. } }`) whose first test is
   `#[test] #[cfg(feature = "fluent")] fn fluent_alias_is_as_expected()`, a single passing call of the fluent alias
   (`value.must().be_foo()`), so the expected fluent name is pinned visually and by the compiler. A pure synonym
   (a method delegating to another assertion, e.g. `is_free` → `is_not_locked`) gets a shallow module holding only
   that pin test; it must not re-test the behavior, which the aliased method's module already covers

## Features

- **default**: `["std", "num"]`
- **full**: All features (`derive`, `fluent`, `http`, `jiff`, `libm`, `num`, `program`, `reqwest`, `rootcause`,
  `serde`, `serde-json`, `serde-toml`, `std`, `tokio`)
- **fluent**: Enables the borrowing `IntoAssertContext` and consuming `IntoOwnedAssertContext` entry traits, providing
  `.must()` / `.verify()` and `.must_owned()` / `.verify_owned()`, plus fluent aliases like `be_equal_to`, `have_length`
- **serde**: A group enabling `serde-json` and `serde-toml`; each of those carries only the crate its conversion needs
- Library supports `no_std` when `std` feature is disabled
- MSRV: `assertr` 1.89.0, `assertr-derive` 1.89.0
- An MSRV bump touches `rust-version` in both `Cargo.toml` files, the `## MSRV` section and its history list in
  `README.md`, the version hard-coded in the README's MSRV badge, and the version pinned in the CI matrix

### Dependency policy

Every dependency is deliberate. Three rules follow from that:

- Depend on the narrowest crate that provides what is used (`num-traits`, not the `num` umbrella), and request the
  narrowest feature set from it (`tokio/sync`, `reqwest` without default features, `serde` without `derive`).
- Prefer a small amount of local code over a dependency when the dependency exists for one function.
- A feature that wraps a std-only crate must enable `std` itself, or `--no-default-features --features <it>` does not
  compile. `just check-each-feature` is what proves this.

## Linting

`assertr` and `assertr-derive` both forbid `unsafe_code`. Only `assertr-derive` denies `clippy::unwrap_used`
crate-wide.
