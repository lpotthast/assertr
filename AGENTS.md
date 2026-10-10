# AGENTS.md

Run `just --list` to discover repository workflows. Read the manifests, source, and rustdoc for current structure,
features, and API details.

Start with the [architecture overview](knowledge/README.md) for chain state, assertion capabilities, failure processing,
and rendering. Keep the relevant knowledge documents current when these contracts change.

Consult the [glossary](knowledge/glossary.md) before introducing terminology. Reuse existing names when their meanings
fit.

## Working contract

- Preserve the checkout exactly. Inspect `HEAD`, the index, and the worktree separately when relevant. Do not stage,
  unstage, reset, commit, publish, or alter unrelated changes without explicit instruction.
- Prefer sentences to em dashes and semicolons.

## Module layout

- Put all `mod` declarations, `use` and `pub use` items, and `extern crate` items at the top of a module, before any
  other item. This includes inline modules such as `mod tests { .. }`. Declare all `mod`s first, then group `use` and
  `pub use` items by origin: `alloc`, `core`, and `std` first, then external crates, then `crate`, `super`, and `self`.
  Never add an import next to the code that needs it.
- The only exception is a `macro_rules!` re-export (`pub(crate) use name;`), which must follow its definition. Child
  modules import such macros by path (`use super::name;`) instead of relying on textual macro scope, so `mod`
  declarations never have to follow a macro definition.

## Testing

- Write test assertions using assertr.
- Put unit tests besides the tested production code.
- Group tests belonging together into focused submodules.

## Changelog and releases

- Record only release-notable changes. Use `## [Unreleased]` by default. If the latest dated version section has not
  been published, merge changes into that section instead.
- Describe the net difference from the exact immediately preceding release. Never describe an intermediate committed or
  uncommitted design. Consolidate related entries so the final behavior is stated once.
- Reassess SemVer whenever an entry changes. Prefix breaking items with `- **Breaking:**`. Adding a method to an
  existing public `*Assertions` trait is explicitly non-breaking because these traits are for method discovery, not
  downstream implementation. Removing or incompatibly changing a method remains breaking.
- Every public export follows normal SemVer rules unless documented otherwise. Macro-only plumbing belongs in the
  unsupported `__private` module. Other internals stay `pub(crate)` in private modules. Do not publish an empty module
  merely to hold `pub(crate)` items.
- Do not bump versions or README dependency examples during ordinary development. For a release, derive the version from
  the changelog, move and date the entries, bump affected crates, update README versions in the landing-page rustdoc in
  `assertr/src/lib.rs`, regenerate with `just readme`, and update changelog comparison links, then run the release
  workflows. Keep `assertr`'s exact `assertr-macros` requirement synchronized with the macro crate because generated
  code depends on `assertr::__private`.

## Design boundaries

- Extend capability-based assertion families before creating type-specific traits. Order-free element operations use
  `CollectionAssertions`. Positional operations require `StableOrder`. Constant-time indexing requires
  `RandomAccess`. Set relations require `SetLookup`. Map iteration uses `Map`, while key queries require `MapLookup`.
  Strings use `StrAssertions` and lengths use `HasLength`. Per-type traits are only for genuinely type-specific
  behavior. Iteration-only impls (`Collection`, `Map`, `HasLength`) carry no lookup bounds such as `BuildHasher`,
  `Hash`, or `Ord`. Those belong on `SetLookup` and `MapLookup`.
- Presentation never grants behavior. `CollectionPresentation` and `RenderingOrder` control diagnostics only and remain
  independent of `StableOrder`, `RandomAccess`, and `SetLookup`.
- Custom `ValueRenderer`s render leaves. Assertr owns structural syntax. Render every diagnostic value through
  `context.render()` in expectations or `self.render()` in execution adapters, so the active renderer and
  `RenderingBudget` apply. Never format subjects directly with `Debug`.
- Keep `BTreeSet` and `BTreeMap` support available with `alloc`. Only hash collection implementations belong behind
  `std`. A feature wrapping a std-only dependency must enable `std` itself.

## Adding assertions

- Put behavior, exact diagnostic tests, and built-in adapter tests beside the generic family that owns them. Keep
  downstream-implementor and `no_std` coverage in existing integration fixtures instead of duplicating every assertion
  across every adapter. Pin each failure shape's exact report once, not again per mode or per delegating method.
- Prefer natural assertion names. Type-changing assertions do not use a `get_` prefix. Keep checking and extracting
  behavior distinguishable, for example `is_some` checks and `some` extracts, and `is_of_type` checks and `has_type`
  extracts.
- Shape public `*Assertions` traits as `<'t, subject parameters, M: Mode, R = DebugRenderer>`, declaring each parameter
  only when a signature uses it.
- New assertion traits use `#[cfg_attr(feature = "fluent", assertr_macros::fluent_aliases)]`. Follow
  `assertr-macros/src/fluent_aliases/naming.rs`. Use an explicit alias only when no rule applies, and
  `#[no_fluent_alias]` for deprecated names.
- Keep trait implementations independent of renderer capabilities. Put renderer and `Clone` bounds on individual methods
  in both the trait and impl. Put renderer bounds of an `Expectation` on its impl. Preserve the active renderer in
  projections and extractions. Add a `NoRenderer` compile-time regression for a new assertion trait or capability
  boundary.
- Mark assertion methods `#[track_caller]`. Delegate reusable checks to `matches` or `test_assertion`, which
  track once. Methods delegating to tracked assertions must not track again. Execution adapters track explicitly before
  the operation they own and preserve the caller location. Built-in adapters then use the private executor entry points
  that skip tracking. See [observation boundaries](knowledge/observation-boundaries.md) for async timing.
- Implement reusable leaf checks in `Expectation::evaluate` and their diagnostics in `Expectation::explain`. Prefer
  `property_expectation!` for unit property checks and `FailureBuilder::relations` for the common relation pair. Keep
  an explicit `KIND` on built-in expectations. Neither hook tracks or raises. Evaluation retains the original
  observation. Explanation populates and returns the supplied structured `FailureBuilder` without repeating
  observations. Bulk expected lists and operand views may be accessed again when they describe the same logical list
  and comparison values. Access counts are unspecified. The chain executor raises the completed failure. Child contexts
  instead build and retain evidence for the enclosing assertion.
- Use `.actual(..)`, `.relation(..)`, `.expected(..)` or `.unexpected(..)`, `.fact(Fact::labelled(..))` or
  `.fact(Fact::note(..))` (or `.facts(..)` for a group), and nested `.children(..)` for diagnostics. An execution adapter
  that constructs a failure directly starts it with `self.failure(FailureKind::..)` and passes it to `self.raise(..)`.
  Never format a failure body by hand. The common report grammar renders the structured fields. Relations are lowercase
  sentences without trailing periods and never embed values.
- Add explicit negative assertions only when commonly useful and not already represented by an existing assertion.
  Hand-write diagnostics that name the negation and preserve its evidence. There is no generic `.not()`. Allow at most
  one antonym synonym per positive assertion.
- Give every assertion method its own test submodule. Its first test is `caller_location_is_as_expected`, using
  `assert_caller_location!` and one failing call. The macro compares the exact caller location without fixed line
  numbers. When `fluent` applies, the test module of the file has one `fluent_aliases::are_as_expected` test with one
  passing call per alias. A pure delegating synonym gets these pins and does not duplicate behavior tests.

## Documentation and dependencies

- Document public API items.
- `README.md` is generated by cargo-rdme from the literal crate-level rustdoc in `assertr/src/lib.rs`. Edit that source,
  then run `just readme` after formatting. Do not edit the generated readme directly. `just check-readme` checks
  freshness without modifying files and runs in `just verify` and documentation CI.
- Use the narrowest dependency and feature set that works. Prefer small local code over a dependency used for one
  function.
- An MSRV bump updates `rust-version` in both crate manifests, the MSRV text and badge in the landing-page rustdoc
  (README), history table and the pinned CI version.
