---
id: expectation-execution
depends_on: [ failure-processing ]
sources:
  - assertr/Cargo.toml
  - assertr/src/lib.rs
  - assertr/src/assertions/core/partial_eq.rs
  - assertr/src/assertions/core/partial_ord.rs
  - assertr/src/assertions/core/range.rs
  - assertr/src/assertions/num/mod.rs
  - assertr/src/assertions/collection/value.rs
  - assertr/src/assertions/map/imp.rs
  - assertr/src/expectation/mod.rs
  - assertr/src/expectation/context.rs
  - assertr/src/assert_that/execution.rs
  - assertr/src/assert_that/capture.rs
  - assertr/src/assertions/matcher.rs
  - assertr/src/assertions/collection/elements_are.rs
  - assertr/src/assertions/collection/each.rs
  - assertr/src/expectation/satisfying.rs
  - assertr/tests/custom_assertions.rs
  - assertr/tests/bulk_allocations.rs
---

# Expectation execution

[Architecture overview](README.md)

Ordinary assertions and composition execute the same `Expectation` and `ExpectationDiagnostics` contracts. The chain
owns tracking, failure handling, and continuation. Definitions live beside their assertion family. Generic composition
lives in `expectation`. `matchers` catalogs public definitions and constructors, with subject namespaces for colliding
names. A matcher is an expectation used in composition.

## Evaluation and explanation

The [traits](../assertr/src/expectation/mod.rs) define these implementor obligations:

Neither hook tracks or raises. Explanation returns the supplied structured builder. The chain executor raises it,
while a child context builds and retains evidence for the enclosing assertion.

| Contract                                | Responsibility                                                                                                                                                                                |
|-----------------------------------------|-----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| `Expectation<T, R>::evaluate`           | Return `Result<Success<'a>, Rejection<'a>>` for the supplied subject. Do not add an assertion count or raise a failure on the enclosing chain.                                                |
| `Success<'a>`                           | Retain a successful observation needed for continuation, such as a borrowed payload or acquired guard. Use `()` when no witness is needed.                                                    |
| `Rejection<'a>`                         | Retain what explanation needs from the failed observation. It may borrow the subject or definition.                                                                                           |
| `ExpectationDiagnostics<T, R>::explain` | Populate the supplied `FailureBuilder`. Explain an original rejection or describe an unmet expectation with no subject. Do not repeat observations. Repeatable expected data may be accessed again. |
| `KIND`                                  | Classify both forms of explanation with the same `FailureKind`.                                                                                                                               |
| `FLATTEN`                               | Declare that composition can merge child failures directly, applying the current context's path to their relative paths. Ordinary execution still retains the enclosing failure.              |

Rust permits observations to borrow the subject or definition for the declared lifetime. It does not enforce the
semantic obligation to retain the original observation. Keep converted views, errors, counts, or guards whenever
reconstructing them would repeat work or observe a different state. Both traits forward through references without
cloning. Negative definitions own their checks, relations, and unexpected operands.

Leaf diagnostic bounds belong on `ExpectationDiagnostics`. Composites that explain children during evaluation also need
their children's diagnostic capabilities. Runtime evidence suppression cannot remove those compile-time bounds.

### Present rejection versus missing subject

The same equality expectation for 3 has two explanation inputs:

| Situation                                     | Execution                                                       | Explanation input           | Evidence                                                 |
|-----------------------------------------------|-----------------------------------------------------------------|-----------------------------|----------------------------------------------------------|
| A present element is 2                        | Evaluate it and retain the rejection.                           | `Some((actual, rejection))` | Actual 2 and expected 3.                                 |
| An expected element has no actual counterpart | Describe the expectation without evaluating a fabricated value. | `None`                      | Expected 3 with the relation describing the requirement. |

`None` never means successful evaluation. A missing-subject description can preserve field or variant paths without
running a predicate or callback. The structural regression
[
`preserves_sequence_length_metadata_and_budget_in_nested_failures`](../assertr/src/assertions/collection/elements_are.rs)
pins length-mismatch descriptions, paths, and bounded evidence.

## Comparison operands

Ordinary value assertions and their reusable definitions use `E: BorrowFor<A>` to select the expected value's
borrowed type, `E::View`. `A` is the declared subject, collection or iterator item, map key or value, or range bound type.
There is no separately inferred comparison target. A matcher can select different views for different subject types.
The independent `borrow-for` crate owns standard selections, custom-wrapper opt-in, and the relationship to `Borrow`.
Assertr [re-exports the dependency](../assertr/src/lib.rs) and enables its `alloc` feature. Standard selections include
`T` and `&T` expectations for `T`, literals for `String`, and array/vector views.
The public [borrowed equality guide](../assertr/src/crate_docs.md#borrowed-equality) demonstrates reusable borrowed
operands and a custom wrapper. The regressions
[`borrows_once_after_tracking_and_retains_failure_evidence`](../assertr/src/assertions/core/partial_eq.rs) and
[`unsized_view_is_borrowed_once_after_tracking_and_rendered_on_both_rejections`](../assertr/src/assertions/core/partial_eq.rs)
pin tracking before conversion and reuse of the selected view during explanation.

Equality requires `A: PartialEq<E::View>`. Negation negates `PartialEq::eq`, even when `ne` is overridden.
Ordering requires `A: PartialOrd<E::View>` and rejects incomparable results. Range containment retains symmetric
ordering requirements. Selection does not grant these capabilities. In particular, string ordering may require
explicit `str` views, as documented in [PartialOrdAssertions](../assertr/src/assertions/core/partial_ord.rs).
Numeric tolerance, signed-duration tolerance, and time-zone checks require `View = A`. Expected values and deviations
borrow independently. Signed-duration tolerance borrows expected value before deviation and retains both references
with a flag that is true for an invalid negative deviation. Its generic operands can require explicit types for
previously inferred `.into()` and `Default::default()` targets.

Standard ranges with borrowed, sized bounds have inherent containment methods selecting the native `RangeBounds<B>`
implementation for the pointee type. This avoids inferring between `RangeBounds<B>` and `RangeBounds<&B>` when the
expected element is also borrowed. Unbounded `..` selects the operand's own type. These methods and their fluent
aliases delegate to `RangeBoundAssertions` with an explicit bound type, preserving its tracking and expectation
execution. Custom ranges still use the generic `RangeBounds` extension directly. Fully qualified trait calls can
select `B` explicitly.

Reusable `ContainsElement<B, E>` and `DoesNotContainElement<B, E>` definitions offer two constructors. `new` fixes
`B` to the operand's type, so borrowed endpoints and unbounded ranges infer without annotations, including in
composition. Passing a reference selects that reference type and requires its renderer. `borrowing` selects a bound
type explicitly, as in `ContainsElement::<String>::borrowing(&value)`, and retains the supplied operand for evaluation
through `BorrowFor<B>`. This also permits pointee-only renderers for borrowed bounds. Direct containment methods
execute these borrowing definitions with their selected `B`. Both constructors store without borrowing or cloning,
and every direct call and matcher uses the same expectation evaluation and diagnostics.

Definitions store supplied values without cloning or accessing their views. Scalar evaluation borrows each operand
once after assertion tracking and retains the selected view on rejection. Renderers support the actual type, selected
view, and structural leaves such as keys or indexes. Wrappers need no renderer.

Scalar definitions support unsized subjects such as `str` and slices when evaluated directly. Assertion-chain storage
and entry normalization are unchanged. Reference-valued subjects, fields, and items keep their declared types. Use
`dereferenced` to match pointees. Other comparison policies can use explicit views, predicates, or custom expectations.
Empty generic expected lists may require an explicit element type. Single native key queries, set relations, identity,
string/path views, and `RangeBounds` operands keep their own contracts.

### Repeatable bulk expected data

Bulk value, key, and entry lists use finite, slice-backed `AsRef` storage. Arrays, slices, vectors, and compatible
wrappers reuse their storage. Collect generators explicitly before the assertion, such as
`.contains_all(generator.collect::<Vec<_>>())`. Borrowed lists use the stored element type's `BorrowFor` selection,
so a list of custom wrappers needs no additional selection for references to those wrappers.

During evaluation and explanation, repeated slice access must return the same logical list, and repeated operand
borrowing must describe the same comparison value. Access counts and interleaving with comparisons are unspecified.
Library-controlled access occurs after assertion tracking. Stateful preparation belongs before the assertion, or in a
custom expectation that retains its observation. Scalar borrowing and matcher, callback, guard, and identity contracts
remain unchanged.

Bulk rejections retain only failed observations, such as mismatches, missing values, lengths, and lookup results.
Explanation reads expected data from the stored definition through budgeted rendering adapters. It never repeats
comparisons, searches, callbacks, or iterator consumption. Prefix, suffix, ordered exact, collection membership, and
map key checks allocate no expected-view buffer. Successful prefix checks over a million elements allocate zero bytes.
Other algorithms retain their required actual-element, membership, window, or assignment storage.
The separate [allocation integration executable](../assertr/tests/bulk_allocations.rs) measures these boundaries
with a thread-local counting allocator, including optimized million-element checks and budgeted failure rendering.

Bulk map keys and keyed matchers use the stored key type as the `BorrowFor` context, separately requiring
[`MapLookup<View>`](collection-semantics.md#exact-comparisons-and-keyed-maps). `contains_keys` retains only missing
queries. Exact-entry equality resolves each key followed by its expected value, performs native lookup, and compares
when present. Each expected occurrence performs one lookup. Missing queries and unequal values retain their selected
views, while stored-key coverage preserves duplicate-query handling. Explanation never repeats lookup or comparison.
`Entry` resolves its query once for lookup and path construction. Its rejection keeps stored-key identity and owned
child evidence, which explanation attaches directly without borrowing the query again.

A later evaluation of a reusable definition resolves its operands anew. Missing-subject descriptions access only
expected data and perform no lookup, comparison, or callback execution. Expected-data access counts are unspecified,
and rendering budgets can limit which expected operands are borrowed for diagnostics. Observation counts and truth
remain independent of rendering budgets, except for documented probe suppression of diagnostic assignment.

## Chain execution

The [executor](../assertr/src/assert_that/execution.rs) constructs an `AssertionContext` from the chain's renderer,
budget, and location policy. These are executor guarantees for each call:

- `apply_assertion` tracks once and calls the definition's `evaluate` once. It explains and raises a rejection, or drops
  a success, then returns the original chain. `matches` delegates without an extra count or failure wrapper.
- `test_assertion` tracks and executes the same check, returning `Some(success)` for a projection or callback. Rejection
  raises through the active mode and returns `None` in capture mode. The caller controls a returned success's lifetime.

Delegating methods preserve `#[track_caller]` and do not track again. Execution adapters that invoke user conversions,
consume iterators, or await I/O own those steps and their tracking boundary. They pass retained observations and the
original caller location to the shared executor. Its private `FnOnce` entry point also accepts consuming observation
steps directly and shares context construction, failure construction, and mode routing with reusable expectations.
The executor transfers each rejection to explanation. The hook must retain resources until their diagnostic values have
been rendered and release temporary guards before returning the builder. The executor then raises that builder. It
does not control an implementor's resource handling inside the hook. The regressions
[`ordinary_and_matcher_execution_retain_borrowed_rejection_without_retesting`](../assertr/src/assert_that/execution.rs)
and [`rejection_renders_the_original_guard_then_releases_it_before_continuation`](../assertr/src/assert_that/execution.rs)
pin observation reuse and guard release on these paths. Those [observation boundaries](observation-boundaries.md) do
not make consuming or asynchronous operations reusable public matchers.

## Counts and evaluation scope

An assertion count measures tracked attempts, not evaluations or diagnostic nodes. In capture mode:

| Operation                                         | Count on enclosing chain  | Evaluation and failures                                                                                       |
|---------------------------------------------------|---------------------------|---------------------------------------------------------------------------------------------------------------|
| One `matches(all_of(...))` call                   | 1                         | Several branches evaluate. Rejection raises one enclosing failure with bounded children.                      |
| One `matches(each(...))` call                    | 1                         | Every collection element is evaluated. Rejection raises one enclosing failure with bounded children.          |
| One `matches(satisfying(...))` call               | 1                         | The callback tracks its checks on an isolated capture root. Their failures become enclosing matcher evidence. |
| A method composed entirely of ordinary assertions | Sum of delegated attempts | The wrapper does not track again.                                                                             |
| Mapping or derivation alone                       | 0                         | Creates a continuation or child without asserting.                                                            |

The executor's once-per-call guarantee does not mean once per matcher value or once per entire collection. Composition
can evaluate a reusable definition against several subjects. Unordered structural matching separately guarantees
[at most one evaluation per candidate pair](matcher-composition.md#exact-unordered-assignment).

## Child scopes and evidence

`AssertionContext` has no public constructor. Definitions execute through a chain or a context supplied to another
definition. The context borrows rendering settings and owns paths and child evidence. `isolated` creates a child scope
without cloning the renderer.

`AssertionContext::evaluate` consumes or drops a child's observation before returning its boolean result. Built-in
composites call it before evaluating a sibling and retain owned `Evidence`: child failures and omission counts, with no
borrowed subjects or guards. This prevents a retained guard from remaining acquired during a sibling's evaluation. It
does not undo user effects or isolate the subject from concurrent changes. Full paths participate in evidence ordering
and truncation. `Evidence::explain` removes the originating context's path prefix when attaching children to an
enclosing failure. Descendants retain their own relative paths. Flattening applies the receiving context's prefix once
when merging children. This also preserves genuine repeated field names. The regression
[`grouped_evidence_has_relative_paths_without_losing_repeated_field_names`](../assertr/src/expectation/context.rs)
checks both an enclosing group and a nested field with the same name.

### Guarded rejection trace

| Stage                            | Retained state                                                                                           |
|----------------------------------|----------------------------------------------------------------------------------------------------------|
| Evaluate a borrowed cell or lock | Rejection owns the acquired guard and the original observation.                                          |
| Explain the rejection            | Render the guarded value into an owned `AssertionFailure`. Release the guard before explanation returns. |
| Retain child evidence            | Keep the rendered failure, without the guard or original subject borrow.                                 |
| Evaluate the next sibling        | Acquire independently. Earlier evidence still describes the earlier observation.                         |

The downstream regression
[`diagnostics_use_the_retained_observation_and_release_its_guard`](../assertr/tests/custom_assertions.rs)
changes the cell after failure construction and verifies that evidence still contains the earlier value.
[`scoped_paths_participate_in_sorting_before_truncation`](../assertr/src/expectation/context.rs)
pins path ordering before budget retention.

## Budgets and probes

Implementors must derive pass/fail from the check, independently of the evidence budget or probe flag. This is a trait
obligation. Rust cannot prevent a downstream definition from branching incorrectly on diagnostic settings. The context
records truth separately from evidence, so omitting children cannot turn a recorded rejection into success.

`probe` executes with evidence retention disabled. Built-in diagnostics are suppressed, but predicates and assertion
callbacks can still render or mutate state. A zero item budget limits repeated groups, including child evidence. It is
not a probe and does not suppress an ordinary root leaf's explanation. The regression
[`zero_budget_preserves_truth_without_rendering_leaves`](../assertr/src/assertions/collection/elements_are.rs)
pins this distinction for composition. [Rendering budgets](diagnostic-rendering.md#bounded-retention) define retention
limits, and [assertion callbacks](matcher-composition.md#assertion-callbacks) define isolated capture behavior.
