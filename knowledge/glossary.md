---
id: glossary
refines:
  - assertr
depends_on: [ ]
related_to:
  - assertion-lifecycle
  - collection-semantics
  - diagnostic-rendering
  - extension-contract
  - failure-processing
  - fluent-entry
  - integration-boundaries
  - matcher-composition
  - reference-identity
sources:
  - assertr/src/lib.rs
  - assertr/src/actual.rs
  - assertr/src/mode.rs
  - assertr/src/assert_that/projection.rs
  - assertr/src/entry/**
  - assertr/src/assertions/mod.rs
  - assertr/src/assertions/has_length.rs
  - assertr/src/assertions/collection/mod.rs
  - assertr/src/assertions/map/mod.rs
  - assertr/src/assertions/set/mod.rs
  - assertr/src/condition.rs
  - assertr/src/matchers/**
  - assertr/src/failure/**
  - assertr/src/renderer/**
  - assertr/src/tracking.rs
  - assertr/src/util/matching.rs
---

# Glossary

[Architecture overview](README.md)

Use these names when describing or extending Assertr. Reuse a term when its definition fits. Add a new term only
for a distinct concept, and explain how it differs.

Prefer **assertion chain** for `AssertThat` in prose. Keep existing API spellings such as `IntoAssertContext`.
Use **match context** or **rendering context** when either is meant. Private implementation types are marked.

Each term links to its architectural explanation. Definitions use existing Rust names.

| Preferred term | Meaning and existing names |
|---|---|
| [Adapter](failure-processing.md#presentation-and-fallback) | `Adapter<Input>` transforms borrowed input into an owned output or error, including report conversion and side effects. |
| [Assertion callback](matcher-composition.md#assertion-callbacks) | A closure that runs assertions on a supplied chain, adapted into a reusable matcher by `satisfying` and `Satisfying<F>`. |
| [Assertion chain](assertion-lifecycle.md#chain-representation) | `AssertThat<'t, T, M, R>` combines a subject with its mode and diagnostic state, also called an assertion context in existing APIs. |
| [Assertion failure](failure-processing.md#structured-construction-and-ownership) | `AssertionFailure` records one failed assertion and its structured diagnostic fields before presentation. |
| [Assertion family](extension-contract.md#choosing-an-extension) | A group of assertion methods for a shared capability or value type, exposed through public `*Assertions` traits. |
| [Attached builder](failure-processing.md#structured-construction-and-ownership) | `FailureBuilder<Attached<'_>>` targets an assertion chain and delivers its failure through `raise()`. |
| [Borrowed entry](assertion-lifecycle.md#entry-subject-ownership-and-mode) | Starting a chain by borrowing its subject, using `assert_that!`, `must`, or `verify`. |
| [Capability](collection-semantics.md#capability-model) | A trait contract required by an operation, such as semantic ordering or rendering a particular leaf type. |
| [Capture mode](assertion-lifecycle.md#entry-subject-ownership-and-mode) | `Capture` collects assertion failures for the completed chain to return as `AssertionFailures`. |
| [Chain state](assertion-lifecycle.md#chain-representation) | The private `ChainState` stores the parent link, metadata, renderer, rendering settings, assertion count, and captured failures. |
| [Child chain](assertion-lifecycle.md#chain-representation) | A derived `AssertThat` that borrows a parent and propagates assertion counts and captured failures toward the root. |
| [Collection](collection-semantics.md#capability-model) | `Collection` extends `HasLength` with repeatable inspection of the same elements in the same iteration order. |
| [Collection presentation](diagnostic-rendering.md#capabilities-and-structure) | `CollectionPresentation` selects diagnostic group syntax, type-hint visibility, and rendering order without granting behavioral capabilities. |
| [Condition](matcher-composition.md#typed-reusable-conditions) | An `AssertrCondition<T>` tests a reusable domain property and returns `Result<(), Error>`, carrying a typed reason for rejection. |
| [Constraint description](matcher-composition.md#truth-polarity-and-evidence) | `ConstraintDescription` stores a matcher's relation, rendered expectation, and nested constraints independently of whether a value matches. |
| [Derivation](assertion-lifecycle.md#projections-and-continuation) | Creating a child chain with `derive`, `derive_owned`, or `derive_async`. |
| [Detached builder](failure-processing.md#structured-construction-and-ownership) | `FailureBuilder<Detached>` returns a failure through `build()` for nesting or inspection without raising it on a chain. |
| [Diagnostic leaf](diagnostic-rendering.md#capabilities-and-structure) | One value formatted by `ValueRenderer<T>` within the structure assembled by Assertr. |
| [Evidence](failure-processing.md#structured-construction-and-ownership) | Values, facts, paths, and nested failures retained to explain an assertion or matcher outcome. |
| [Exact unordered assignment](matcher-composition.md#exact-unordered-assignment) | Maximum bipartite matching pairs each actual element with a distinct expectation, preserving multiplicity when constraints overlap. |
| [Expression capture](fluent-entry.md#scoped-expression-capture) | Recording the subject's source spelling in failure metadata through entry macros or the `fluent_expressions` attribute. |
| [Extraction](integration-boundaries.md#retaining-checks-versus-extraction) | A checked continuation on an inner or resolved subject, restricted to panic mode when failure cannot produce that subject. |
| [Fact](failure-processing.md#structured-construction-and-ownership) | `Fact` attaches a label and rendered evidence value to one failure, with an empty label representing a note. |
| [Failure aggregate](failure-processing.md#structured-construction-and-ownership) | `AssertionFailures` stores an ordered collection of failures and provides slice access and iteration. |
| [Failure builder](failure-processing.md#structured-construction-and-ownership) | `FailureBuilder` assembles the fields of one failure, with attached and detached targets determining how it is completed. |
| [Failure kind](failure-processing.md#structured-construction-and-ownership) | `FailureKind` classifies the assertion family for adapters, while the relation and evidence explain the specific failure. |
| [Fluent alias](fluent-entry.md#borrowing-and-ownership) | An alternate assertion-method spelling generated under the `fluent` feature, such as `be_equal_to` for `is_equal_to`. |
| [HasLength](collection-semantics.md#capability-model) | `HasLength` supplies a finite native length for length and emptiness assertions, such as byte length for strings. |
| [Leaf assertion](extension-contract.md#implementing-an-assertion) | An assertion that tracks its own check and raises any failure through `self.failure(...).raise()`. |
| [Map](collection-semantics.md#capability-model) | `Map` extends `HasLength` with repeatable traversal of stored key/value entries, with key queries requiring `MapLookup`. |
| [Map key query](collection-semantics.md#exact-comparisons-and-keyed-maps) | `MapKeyQuery<K>` adapts an expected bulk key to the query type accepted by the map's native lookup. |
| [Map lookup](collection-semantics.md#exact-comparisons-and-keyed-maps) | `MapLookup<Q>` queries a map by a borrowed key view and returns references to its stored key and value. |
| [Mapping](assertion-lifecycle.md#projections-and-continuation) | Replacing a chain's subject through `map`, `map_owned`, or `map_async` while moving its existing state into the continuation. |
| [Match context](matcher-composition.md#truth-polarity-and-evidence) | `MatchContext` carries the renderer, budget, polarity, relative path, and isolated evidence for matcher evaluation. |
| [Match result](matcher-composition.md#truth-polarity-and-evidence) | `MatchResult` reports whether the evaluated matcher matched, independently of retained evidence and rendering limits. |
| [Matcher](matcher-composition.md#truth-polarity-and-evidence) | An `AssertrMatcher` describes an expectation and evaluates it into a `MatchResult`, recording evidence through a `MatchContext`. |
| [Mode](assertion-lifecycle.md#chain-representation) | The sealed `Mode` trait selects failure behavior at compile time through its only implementations, `Panic` and `Capture`. |
| [Multiplicity](matcher-composition.md#exact-unordered-assignment) | The number of occurrences of each element, preserved by exact unordered element comparisons. |
| [Note](failure-processing.md#structured-construction-and-ownership) | A `Fact` with an empty label, constructed with `Fact::note` and displayed without a label. |
| [Owned entry](assertion-lifecycle.md#entry-subject-ownership-and-mode) | Starting a chain by taking its receiver value through `assert_that_owned!`, `must_owned`, or `verify_owned`, including when that value is a reference. |
| [Panic mode](assertion-lifecycle.md#entry-subject-ownership-and-mode) | `Panic` presents and raises the first assertion failure immediately. |
| [Panic presentation](failure-processing.md#presentation-and-fallback) | The adapter selected by `with_panic_presentation` to produce panic text, defaulting to `ToHumanReadableText`. |
| [Path](failure-processing.md#structured-construction-and-ownership) | A relative location within a subject represented by `PathSegment` values for fields, tuple positions, variants, indexes, or rendered keys. |
| [Polarity](matcher-composition.md#truth-polarity-and-evidence) | The requested matcher outcome, with positive evaluation explaining rejection and negative evaluation explaining an unwanted success. |
| [Predicate](matcher-composition.md#truth-polarity-and-evidence) | A boolean test, adapted into a matcher by `predicate` and `Predicate<F>` without a condition's typed error. |
| [Probe](matcher-composition.md#truth-polarity-and-evidence) | Matcher evaluation with built-in diagnostic retention suppressed, while user callbacks may still perform side effects or rendering. |
| [Projection](assertion-lifecycle.md#projections-and-continuation) | A borrowed part or computed view of the current subject, used by the `derive` and `satisfies` families. |
| [Random access](collection-semantics.md#capability-model) | `RandomAccess` extends `StableOrder` with constant-time indexed access. |
| [Reference identity](reference-identity.md#which-address-is-compared) | Same-instance comparison using `core::ptr::eq`, including pointer metadata rather than inferring identity from value equality. |
| [Reference normalization](assertion-lifecycle.md#entry-subject-ownership-and-mode) | Borrowing entry treats a sized value and one reference layer as the same subject type, while unsized targets remain reference-typed. |
| [Relation](failure-processing.md#structured-construction-and-ownership) | The lowercase sentence describing a comparison or constraint, stored separately from diagnostic values and written without a trailing period. |
| [Rendered value](diagnostic-rendering.md#capabilities-and-structure) | `Rendered` stores a diagnostic tree with a `RenderedBody` and type metadata after leaf formatting and structural assembly. |
| [Rendering budget](diagnostic-rendering.md#bounded-retention) | `RenderingBudget` limits retained items per diagnostic group and characters per leaf independently of assertion truth. |
| [Rendering context](diagnostic-rendering.md#capabilities-and-structure) | `RenderingContext` combines a renderer and budget and supplies structural rendering adapters through `AssertThat::render()`. |
| [Rendering order](diagnostic-rendering.md#capabilities-and-structure) | `RenderingOrder` chooses preserved iteration order or sorting by rendered text for diagnostics, independently of `StableOrder`. |
| [Root chain](assertion-lifecycle.md#chain-representation) | An assertion chain without a parent that receives descendant counts and captured failures. |
| [Set lookup](collection-semantics.md#capability-model) | `SetLookup` extends `Collection` with native membership for set relations. |
| [Stable order](collection-semantics.md#capability-model) | `StableOrder` makes iteration positions part of collection semantics, enabling positional assertions and indexed evidence. |
| [Subject](assertion-lifecycle.md#chain-representation) | The value being asserted on, stored as `Actual::Borrowed` or `Actual::Owned` and inspected through `AssertThat::actual()`. |
| [Value renderer](diagnostic-rendering.md#capabilities-and-structure) | `ValueRenderer<T>` formats diagnostic leaves, with `DebugRenderer` as the default and Assertr assembling the surrounding structure. |
