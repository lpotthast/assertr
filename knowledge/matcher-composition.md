---
id: matcher-composition
depends_on: [ expectation-execution, collection-semantics ]
sources:
  - assertr/src/matchers.rs
  - assertr/src/expectation/mod.rs
  - assertr/src/expectation/all_of.rs
  - assertr/src/expectation/any_of.rs
  - assertr/src/expectation/satisfying.rs
  - assertr/src/expectation/predicate.rs
  - assertr/src/expectation/lists.rs
  - assertr/src/condition.rs
  - assertr/src/assertions/condition.rs
  - assertr/src/assertions/collection/elements_are_in_any_order.rs
  - assertr/src/__private/field.rs
  - assertr/src/__private/partial_match.rs
  - assertr/src/util/matching.rs
  - assertr-macros/src/partial/mod.rs
  - assertr-macros/tests/progress.rs
---

# Matcher composition

[Architecture overview](README.md)

A matcher is a reusable expectation used in composition. It implements the
[same evaluation and explanation contracts](expectation-execution.md) as ordinary reusable checks. Import constructors
from `assertr::matchers` individually or with `matchers::*`. Every public expectation is available there, with subject
namespaces such as `string`, `collection`, and `map` for family-specific definitions and colliding names. The prelude
exposes the module, not its constructor names. Use unit values such as `IsSome` directly, and `new` constructors to
supply operands or select a type parameter. References reuse the same definition without cloning.
The [matcher rustdoc](../assertr/src/matchers.rs) owns syntax and usage examples.

## Truth and evidence

`matches` executes a definition directly through the chain. A leaf equality matcher produces an `Equality` failure
without a matcher wrapper. Composites define their enclosing relation and child evidence.

`all_of` evaluates every branch and accepts an empty list. `any_of` stops at the first success and rejects an empty
list. Rejected alternatives use isolated scopes and survive only if every branch fails. `predicate` wraps
`Fn(&T) -> bool`, needs no subject renderer, and has no typed rejection error. `described_as` supplies its relation.

Collection and map composition respects their [capabilities](collection-semantics.md#capability-model). Diagnostic order
cannot grant positional semantics. [Evidence budgets and probes](expectation-execution.md#child-scopes-and-evidence)
change retention, never pass/fail results. A composite can still require renderers for its own evidence, such as
`ValueRenderer<usize>` for `any_of` branch numbers.

## Assertion callbacks

[`satisfying`](../assertr/src/expectation/satisfying.rs) adapts a reusable `Fn` callback to an expectation. It receives
a borrowed capture-mode chain and returns `()`. Every performed assertion must pass. Its failures become owned child
evidence with the enclosing path. An empty callback panics as misuse, and user panics propagate.

Callback capture inherits the renderer, budget, and location policy and requires `R: Clone`. It still executes inside a
probe, so callback effects and rendering cannot be suppressed by the outer matcher. Ordinary assertion callbacks may
accept `FnOnce` and retain their method's failure mode. `AssertThat::satisfies` projects and continues a chain, whereas
`satisfying` constructs an expectation for later evaluation.

## Typed reusable conditions

[`AssertrCondition<T>`](../assertr/src/condition.rs) tests a domain property with `Result<(), Error>`.
`Condition` retains the original error and renders it as an unlabeled note without testing again. Direct `is` and `has`
and the `condition` matcher constructor share this definition. They require `ValueRenderer<Error>`, not a subject
renderer.

Iterable `are` and `have` track one assertion for the call. Capture raises one failure per offending element. Panic mode
stops at the first failure. Traversal offsets are not reported as stable indexes. See the
[condition family](../assertr/src/assertions/condition.rs) for these execution adapters.

## Exact unordered assignment

Exact unordered comparisons pair actual occurrences with distinct expected slots. Greedy assignment is insufficient when
expectations overlap. The shared [maximum bipartite matching utility](../assertr/src/util/matching.rs) revisits
assignments through iterative augmenting paths. Exactness requires no unmatched occurrence on either side. One actual
occurrence cannot satisfy two duplicate expectations.

[Unordered structural matching](../assertr/src/assertions/collection/elements_are_in_any_order.rs) evaluates each
actual/expectation pair at most once. The same definition can still run for many pairs. Rejections preserve missing
expectations, unexpected occurrences, and multiplicity. An extra occurrence matching an occupied expectation must be
explained as surplus, not as a value that failed that expectation. `at slot` identifies a zero-based expectation
position, never an actual collection index. Missing expectations retain their complete constraint and bounded candidate
failures as nested evidence, including equality candidates. No canonical assignment is promised when several maximum
assignments exist.

Rendering budgets do not bound comparison work or total memory. Current candidate evaluation and retained evidence can
approach the Cartesian product. The implementation's cache completion and evidence retention rules are documented beside
the [matching code](../assertr/src/assertions/collection/elements_are_in_any_order.rs).
[`supports_overlapping_constraints`](../assertr/src/assertions/collection/elements_are_in_any_order.rs)
regresses the case a greedy assignment would
reject. [Exact keyed map checks](collection-semantics.md#exact-comparisons-and-keyed-maps)
use native lookup instead.

## Structural macros

Runtime constructors and declarative matcher macros require no feature. `matchers!` creates heterogeneous list nodes.
The `matchers` feature enables procedural `partial!` for named, tuple, unit, and explicitly annotated variant shapes.
Only selected fields need comparison or rendering capabilities.

`partial!` emits Rust patterns and borrowed projections, preserving constructor resolution, visibility, field types, and
exhaustiveness checks. Named `..` omits remaining fields. Tuple `_` skips one position and tuple `..` must be final.
Projections attach `Field`, `TupleIndex`, and optional `Variant` paths, including in descriptions with no actual
subject.

Expected expressions construct explicit matchers once in source order within one enclosing expression, preserving
temporary borrows through the caller's statement. Use `eq`, an alias for `equal_to`, for heterogeneous equality. Every
selected `partial!` field, matcher-list element, and `entries_are!` value uses the shared expectation protocol. Map keys
remain native lookup operands. Expressions that also support equality are still used as matchers unless wrapped in `eq`
or `equal_to`. Macro-only projection/list plumbing lives in unsupported `__private`. Runtime crate resolution supports
renamed dependencies.

The [partial implementation](../assertr-macros/src/partial/mod.rs) and
[compile-fail fixtures](../assertr-macros/tests/partial/) distinguish macro syntax errors from ordinary Rust type,
visibility, exhaustiveness, and borrow errors.
