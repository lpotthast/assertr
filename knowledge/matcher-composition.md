---
id: matcher-composition
refines:
  - assertr
depends_on:
  - diagnostic-rendering
  - failure-processing
related_to:
  - collection-semantics
  - extension-contract
sources:
  - assertr/src/matchers/mod.rs
  - assertr/src/matchers/context.rs
  - assertr/src/matchers/not.rs
  - assertr/src/matchers/satisfying.rs
  - assertr/src/condition.rs
  - assertr/src/assertions/condition.rs
  - assertr/src/matchers/condition.rs
  - assertr/src/matchers/elements_are_in_any_order.rs
  - assertr/src/util/matching.rs
  - assertr-macros/src/partial/mod.rs
  - assertr-macros/tests/partial/*.stderr
---

# Matcher composition and structural expectations

[Architecture overview](README.md)

Matchers describe reusable expectations and evaluate them against actual values. They compose through collections, maps,
fields, predicates, typed conditions, and assertion callbacks.

## Truth, polarity, and evidence

An `AssertrMatcher` describes a constraint and evaluates it with a `MatchContext`. It returns truth and records local
evidence. The enclosing assertion raises the root failure.

The context carries polarity, renderer, budget, relative path, and evidence. Positive polarity explains why a value did
not match. Negative polarity explains why an unwanted match succeeded. `not` reverses truth and diagnostic polarity
without replaying the inner matcher:

```rust
use assertr::{
    matchers::{ge, not},
    prelude::*,
};

let too_small = assert_that!(2).capture(|it| it.matches(ge(3)));
let unwanted_match = assert_that!(4).capture(|it| it.matches(not(ge(3))));

assert_eq!(too_small.len(), 1);
assert_eq!(unwanted_match.len(), 1);
```

The first failure explains that `2` is not greater than or equal to `3`. The second explains that `4` is greater than or
equal to `3`, which violates the negated expectation. Descriptions exist independently of failures so successful
constraints can supply this evidence.

`all_of` evaluates every branch. An empty conjunction is true. `any_of` stops at its first successful branch. An empty
disjunction is false. Matchers own or borrow expectations according to their concrete type. Predicates run once per
evaluation requested by the enclosing operation.

A probe suppresses built-in evidence retention. It cannot undo side effects or work already performed by user callbacks.

## Assertion callbacks

`satisfying` adapts ordinary assertions into a matcher using a capture-mode child with the current renderer, budget, and
location setting. It matches when every captured assertion passes. The callback must implement `Fn`, the renderer must
be cloneable, and the callback must perform at least one assertion. Empty callbacks panic as misuse. User panics
propagate.

Captured failures become matcher evidence when needed. The callback still runs during a probe and may construct failures
before the probe discards them.

## Typed reusable conditions

`AssertrCondition<T>` is a domain predicate whose `test` returns `Result<(), Error>`. It carries a typed error and can
be reused by reference. Direct `is` and `has` assertions render that error as one unlabeled note. Iterable `are` and
`have` assertions retain one failure per offending element in capture mode. Panic mode stops at the first failure.
Traversal offsets are not reported as stable indexes.

These assertions require `ValueRenderer<Error>` without requiring a renderer for the subject. The matcher `condition`
adapter retains the typed error as nested evidence. A probe tests the condition without rendering its error. A zero-item
budget can omit the child without invoking the error renderer.

## Exact unordered assignment

Exact unordered element comparisons pair each actual item with a distinct expectation, preserving multiplicity. Greedy
matching can reject a valid input when an early item fits several constraints and a later item fits only one. The shared
utility finds a maximum bipartite matching with iterative augmenting paths and reports matched pairs and unmatched items
on both sides.

Search marks from a failed assignment remain valid until the matching changes. Reusing them avoids repeated dead
searches for surplus duplicates. An explicit stack keeps the search off the call stack.

Structural matchers cache each visited actual/expectation pair, including its evidence, so rearranging assignments does
not replay user predicates. The cache is sparse for easy matches but can approach the Cartesian product when constraints
overlap. Diagnostic space can therefore be quadratic.

[Exact keyed map checks](collection-semantics.md#exact-comparisons-and-keyed-maps) use native lookup and stored-entry
identity instead of this assignment algorithm.

## `partial!`

The `matchers` feature enables `partial!`. Runtime matchers and declarative matcher macros are available without it.

`partial!` selects named, tuple, unit, or explicitly annotated variant shapes. Named fields may end with `..`. Tuple `_`
positions are wildcards, and tuple `..` must be final to preserve selected indices. Generated Rust patterns retain
constructor resolution, visibility, field types, and exhaustiveness checks. Cargo metadata resolves the runtime crate,
including renamed dependencies.

Expected expressions are normalized once in source order within one enclosing expression. Borrowed temporaries therefore
live through the caller's statement. Runtime projections borrow selected fields and attach `Field`, `TupleIndex`, and
optional `Variant` paths. Plain values use heterogeneous equality. Matcher expressions use the matcher protocol.
Ambiguous expressions require `equal_to` or `as_matcher`.

The macro rejects duplicate fields, misplaced or repeated tuple rest, and trailing tokens. Unknown or private fields,
incompatible types, non-exhaustive patterns, and outlived borrows are ordinary Rust compile errors.

## Sources

The [matcher guide](../assertr/src/matchers/mod.rs) documents selection and
composition. [MatchContext](../assertr/src/matchers/context.rs), [not](../assertr/src/matchers/not.rs),
and [satisfying](../assertr/src/matchers/satisfying.rs) implement evaluation. [Conditions](../assertr/src/condition.rs),
their [assertions](../assertr/src/assertions/condition.rs), and
their [matcher adapter](../assertr/src/matchers/condition.rs) handle typed
errors. [Unordered matchers](../assertr/src/matchers/elements_are_in_any_order.rs) use
the [assignment utility](../assertr/src/util/matching.rs). The [partial macro](../assertr-macros/src/partial/mod.rs)
has [compile-fail fixtures](../assertr-macros/tests/partial/).
