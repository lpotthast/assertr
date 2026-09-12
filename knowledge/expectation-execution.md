---
id: expectation-execution
depends_on: [ failure-processing ]
sources:
  - assertr/src/expectation/mod.rs
  - assertr/src/expectation/context.rs
  - assertr/src/assert_that/execution.rs
  - assertr/src/assert_that/capture.rs
  - assertr/src/assertions/matcher.rs
  - assertr/src/assertions/condition.rs
  - assertr/src/assertions/collection/elements_are.rs
  - assertr/src/expectation/satisfying.rs
  - assertr/tests/custom_assertions.rs
---

# Expectation execution

[Architecture overview](README.md)

Ordinary assertions and composition execute the same `Expectation` and `ExpectationDiagnostics` contracts. The chain
owns tracking, failure handling, and continuation. Definitions live beside their assertion family. Generic composition
lives in `expectation`. `matchers` catalogs public definitions and constructors, with subject namespaces for colliding
names. A matcher is an expectation used in composition.

## Evaluation and explanation

The [traits](../assertr/src/expectation/mod.rs) define these implementor obligations:

| Contract                                | Responsibility                                                                                                                                                                                |
|-----------------------------------------|-----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| `Expectation<T, R>::evaluate`           | Return `Result<Success<'a>, Rejection<'a>>` for the supplied subject. Do not add an assertion count or raise a failure on the enclosing chain.                                                |
| `Success<'a>`                           | Retain a successful observation needed for continuation, such as a borrowed payload or acquired guard. Use `()` when no witness is needed.                                                    |
| `Rejection<'a>`                         | Retain what explanation needs from the failed observation. It may borrow the subject or definition.                                                                                           |
| `ExpectationDiagnostics<T, R>::explain` | Populate the supplied `FailureBuilder`. Explain an original rejection or describe an unmet expectation with no subject. Do not repeat the check or user conversions already performed for it. |
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
Rejection resources remain available through explanation and are released before routing the completed failure. Those
[observation boundaries](observation-boundaries.md) do not make consuming or asynchronous operations reusable public
matchers.

## Counts and evaluation scope

An assertion count measures tracked attempts, not evaluations or diagnostic nodes. In capture mode:

| Operation                                         | Count on enclosing chain  | Evaluation and failures                                                                                       |
|---------------------------------------------------|---------------------------|---------------------------------------------------------------------------------------------------------------|
| One `matches(all_of(...))` call                   | 1                         | Several branches evaluate. Rejection raises one enclosing failure with bounded children.                      |
| One iterable `are(condition)` call                | 1                         | Each element is tested. Each rejected element raises its own failure.                                         |
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
