# AGENTS.md

Run `just --list` to discover repository workflows. Read the manifests, source, and rustdoc for current structure,
features, and API details.

## Working contract

- Preserve the checkout exactly. Inspect `HEAD`, the index, and the worktree separately when relevant. Do not stage,
  unstage, reset, commit, publish, or alter unrelated changes without explicit instruction.
- Prefer sentences over em dashes and semicolons.

## Changelog and releases

- Record only release-notable changes. Use `## [Unreleased]` by default. If the latest dated version section has not
  been published, merge changes into that section instead.
- Describe the net difference from the exact immediately preceding release. Never describe an intermediate committed
  or uncommitted design. Consolidate related entries so the final behavior is stated once.
- Reassess SemVer whenever an entry changes. Prefix breaking items with `- **Breaking:**`. Adding a method to an
  existing public `*Assertions` trait is explicitly non-breaking because these traits are for method discovery, not
  downstream implementation. Removing or incompatibly changing a method remains breaking.
- Every public export follows normal SemVer rules unless documented otherwise. Macro-only plumbing belongs in the
  unsupported `__private` module. Other internals stay `pub(crate)` in private modules. Do not publish an empty module
  merely to hold `pub(crate)` items.
- Do not bump versions or README dependency examples during ordinary development. For a release, derive the version
  from the changelog, move and date the entries, bump affected crates, update README versions and changelog comparison
  links, then run the release workflows. Keep `assertr`'s exact `assertr-derive` requirement synchronized with the
  derive crate because generated code depends on `assertr::__private`.

## Design boundaries

- Extend capability-based assertion families before creating type-specific traits. Order-free element operations use
  `CollectionAssertions`. Positional operations require `StableOrder`. Constant-time indexing requires
  `RandomAccess`. Set relations require `SetLookup`. Map iteration uses `Map`, while key queries require `MapLookup`.
  Strings use `StrAssertions` and lengths use `HasLength`. Per-type traits are only for genuinely type-specific
  behavior.
- Presentation never grants behavior. `CollectionPresentation` and `RenderingOrder` control diagnostics only and
  remain independent of `StableOrder`, `RandomAccess`, and `SetLookup`.
- Custom `ValueRenderer`s render leaves. Assertr owns structural syntax. Render every diagnostic value through
  `self.render()` and its adapters so the active renderer and `RenderingBudget` apply. Never format subjects directly
  with `Debug`.
- Keep `BTreeSet` and `BTreeMap` support available with `alloc`. Only hash collection implementations belong behind
  `std`. A feature wrapping a std-only dependency must enable `std` itself.

## Adding assertions

- Put behavior, exact diagnostic tests, and built-in adapter tests beside the generic family that owns them. Keep
  downstream-implementor and `no_std` coverage in existing integration fixtures instead of duplicating every
  assertion across every adapter.
- New assertion traits use `#[cfg_attr(feature = "fluent", assertr_derive::fluent_aliases)]`. Follow
  `assertr-derive/src/fluent_aliases/naming.rs`. Use an explicit alias only when no rule applies, and
  `#[no_fluent_alias]` for deprecated names.
- Keep trait implementations independent of renderer capabilities. Put renderer and `Clone` bounds on individual
  methods in both the trait and impl. Preserve the active renderer in projections and extractions. Add a `NoRenderer`
  compile-time regression for a new trait or capability boundary.
- Mark assertion methods `#[track_caller]` and call `self.track_assertion()` first. A composing method whose entire
  body delegates to tracked assertions must not track again.
- Every leaf assertion, built-in or downstream, raises its failure through `self.failure(FailureKind::..)` with
  `.actual(..)`, `.relation(..)`, `.expected(..)` or `.unexpected(..)`, labeled `.fact(..)`s, `.note(..)`s, and
  nested `.children(..)`, then `.raise()`. Never format a failure body by hand: `Display` renders every failure from
  its fields with one grammar. Relations are lowercase sentences without trailing periods and never embed values.
- Add explicit negative assertions only when commonly useful and not already represented by an existing assertion.
  Hand-write diagnostics that name the negation and preserve its evidence. There is no generic `.not()`. Allow at most
  one antonym synonym per positive assertion.
- Give every assertion method its own test submodule. When `fluent` applies, the first test is
  `fluent_alias_is_as_expected` with one passing fluent call. A pure delegating synonym gets only this alias pin and
  does not duplicate behavior tests.

## Documentation and dependencies

- `README.md` is included unconditionally as crate documentation. Every Rust block must compile as a top-level doctest
  with minimal features. Keep the README as a landing page and detailed guides in the owning API's rustdoc.
- Use the narrowest dependency and feature set that works. Prefer small local code over a dependency used for one
  function.
- An MSRV bump updates `rust-version` in both crate manifests, the README MSRV text, history, and badge, and the pinned
  CI version.
