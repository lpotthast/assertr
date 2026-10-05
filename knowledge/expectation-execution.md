---
id: expectation-execution
depends_on: [ failure-processing ]
sources:
  - assertr/src/expectation/mod.rs
  - assertr/src/expectation/context.rs
  - assertr/src/assert_that/execution.rs
  - assertr/src/tracking.rs
  - assertr/src/assertions/matcher.rs
  - assertr/src/assertions/collection/elements_are.rs
  - assertr/tests/custom_assertions.rs
---

# Expectation execution

[Architecture overview](README.md)

Both ordinary assertions and composed matchers use `Expectation` to check a value and `ExpectationDiagnostics` to explain
a failure. The executor tracks the assertion and handles the completed failure. Expected values follow the
[operand access rules](comparison-operands.md), while [matcher composition](matcher-composition.md) determines which
nested checks run.

## Evaluation and explanation

Neither `evaluate` nor `explain` tracks assertions or raises failures. Implement them as follows:

| Hook or associated item | Responsibility |
|---|---|
| `Expectation<T, R>::evaluate` | Return `Result<Success<'a>, Rejection<'a>>` for the supplied subject. |
| `Success<'a>` | Retain a successful observation for continuation, such as a payload or guard. Use `()` when unnecessary. |
| `Rejection<'a>` | Retain the failed observation needed for explanation. May borrow the subject or definition. |
| `ExpectationDiagnostics<T, R>::explain` | Populate and return the supplied `FailureBuilder<Target>`. |
| `KIND` | Use the same `FailureKind` for rejection and missing-subject diagnostics. |
| `FLATTEN` | Permit composition to merge children into its receiving context. Composition then shows only the children and their omission count, not the definition's own relation, operands, or facts. Probes never explain. Ordinary execution still retains the enclosing failure. |

Retain converted views, errors, counts, or guards whenever obtaining them again would repeat an observation. Explanation
must not repeat comparisons, searches, lookups, callbacks, consumption, or other observations. It may
[read bulk expected data again](comparison-operands.md#reading-expected-data-again), subject to the consistency rules.
Rust's lifetimes allow borrowing but cannot enforce these requirements. Both traits forward through references without
cloning. See [diagnostic rendering](diagnostic-rendering.md#capabilities-and-structure) for renderer bounds.

### Present rejection versus missing subject

| Explanation input | Meaning | Equality with 3 |
|---|---|---|
| `Some((actual, rejection))` | Explain the original rejection. | Actual 2, expected 3. |
| `None` | Describe an unmet expectation without inventing or evaluating a subject. This is not a successful result. | Expected 3 and its requirement. |

Descriptions can retain field and variant paths without running predicates or callbacks. A negative expectation must
provide its own relation and unexpected operands. A generic negation could not derive the correct evidence from the
positive check.

## Chain execution

The executor constructs `AssertionContext` from the chain's renderer, budget, and location policy.

| Entry | Behavior |
|---|---|
| `apply_assertion` | Track and evaluate once. On failure, explain and raise the rejection. On success, drop the success value. Return the original chain. `matches` delegates without another count or failure wrapper. |
| `test_assertion` | Run the same steps, but return `Some(success)` on success. Raise a rejection and return `None` in capture mode. The caller decides how long to retain the success value. |
| Private methods used after tracking | Accept an adapter's observation and caller location, or `FnOnce` hooks for observation and explanation. Handle the failure without tracking again. |

Chain methods preserve `#[track_caller]`. A method that only delegates to a tracked assertion must not track again.
Argument expressions run before method entry. Tracking comes before library-controlled conversions and evaluation,
even on success. Without tracking, capture appears empty. Tracking twice inflates the count.
[Execution adapters](observation-boundaries.md#execution-adapters) track before the operation they perform.

The executor supplies an [attached builder](failure-processing.md#builder-completion) and raises the failure after
explanation returns. An explanation retaining a guard must render its values and release it before returning.
The executor cannot enforce resource handling inside custom hooks. Regression:
[`rejection_renders_the_original_guard_then_releases_it_before_continuation`](../assertr/src/assert_that/execution.rs).

## Counts and evaluation scope

Counts measure tracked attempts, not candidate evaluations or diagnostic nodes.

| Operation | Enclosing count |
|---|---|
| One matcher call, including composite or `satisfying` | 1. Nested [callback capture](matcher-composition.md#assertion-callbacks) is isolated. |
| Wrapper delegating to ordinary assertions | Sum of delegated attempts. |
| Mapping or derivation alone | 0. |

A single matcher can be evaluated against several subjects, with one evaluation per executor call.
[Unordered assignment](matcher-composition.md#exact-unordered-assignment) specifies how often candidate pairs are checked.

## Child scopes and evidence

`AssertionContext` has no public constructor. Definitions receive it from the executor or an enclosing definition.
It borrows rendering settings and stores paths and child failures within the diagnostic budget.

| Operation | Behavior |
|---|---|
| `isolated` | Start a separate group of evidence with inherited settings, path, and item allowance. Do not clone the renderer. Dropping the scope discards its evidence. |
| `evaluate` | Evaluate the check and immediately explain a rejection if diagnostics are enabled and the budget permits it. Drop the observed success or rejection before returning the boolean result. |
| `scoped` | Add one relative path segment for the operation, then append that scope's evidence once. |
| `record` | Retain an existing failure under the current path and item allowance. |
| `outcome` | Record a boolean rejection, constructing its constraint description only if needed. |
| `into_evidence` | Return owned failures and omission counts without borrowing subjects, definitions, or guards. |

Built-in completion discards evidence on success. On rejection, it adds a lazy fallback only if neither retained nor
omitted evidence exists. Each matcher family decides how to traverse its input and describe that fallback. Empty evidence
does not imply success. Private recording and completion helpers implement these rules
in [context.rs](../assertr/src/expectation/context.rs).

[Paths](failure-processing.md#paths) participate in evidence ordering before truncation. `Evidence::explain` removes the
originating context's prefix once. Descendants remain relative to the enclosing subject. Flattening applies the receiving
prefix once, preserving repeated field names when they refer to distinct nested fields. Regression:
[`grouped_evidence_has_relative_paths_without_losing_repeated_field_names`](../assertr/src/expectation/context.rs).

Before evaluating the next sibling, a child check finishes its observation, explains or discards it, and retains only
owned evidence. That evidence describes what the child saw at the time. Assertr does not roll back side effects or
isolate checks from concurrent changes.

## Budgets and probes

Changing the diagnostic budget or enabling a probe must not change whether a check passes. Implementors must preserve
this rule by keeping the result separate from the evidence. [Rendering budgets](diagnostic-rendering.md#bounded-retention)
limit the values and groups retained for diagnostics.

| Setting | Effect on child evidence |
|---|---|
| Preserve iteration | Child scopes share the remaining slots. Once the group is full, later rejections add to the omission count without constructing optional evidence. |
| Sort by rendered text | A later candidate can replace retained evidence even after the group fills. Child scopes keep the group's item allowance. Sort complete child reports, including paths, and break ties by encounter order. |
| Nested ordering | A sorted scope passes its ordering to descendants. A nested subject that normally preserves iteration order still uses the inherited sorting. |
| Zero item budget | Omit repeated child failures, but still explain ordinary leaf failures at the root. |
| `probe` | Evaluate without retaining built-in diagnostics or omission counts. Do not explain the root failure. |

Use `is_diagnostic()` to decide whether to build optional child evidence. It can return false because the budget is zero
or full, as well as during a probe. [Assertion callbacks](matcher-composition.md#assertion-callbacks) run their own capture,
so a probe does not suppress their diagnostics or side effects.

Transferred child omissions, later truncation, and omissions inside retained failures are accounted for separately.
Regressions: `scoped_paths_participate_in_sorting_before_truncation` and
`retains_remaining_capacity_and_suppresses_rendering_in_probes` in [context.rs](../assertr/src/expectation/context.rs).
