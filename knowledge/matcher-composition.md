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
  - assertr/src/assertions/collection/assertions.rs
  - assertr/src/assertions/collection/each.rs
  - assertr/src/assertions/collection/elements_are.rs
  - assertr/src/assertions/collection/elements_are_in_any_order.rs
  - assertr/src/assertions/map/entry_matcher_list.rs
  - assertr/src/expectation/assignment.rs
  - assertr/src/__private/field.rs
  - assertr/src/__private/partial_match.rs
  - assertr/src/util/matching.rs
  - assertr-macros/src/partial/mod.rs
  - assertr-macros/tests/progress.rs
  - assertr-macros/tests/partial/30-qualified-variants.rs
---

# Matcher composition

[Architecture overview](README.md)

A matcher is an expectation used as part of another check. `assertr::matchers` provides public definitions in namespaces
such as `string`, `collection`, and `map`. The prelude exposes the module, not constructor names.
[Rustdoc](../assertr/src/matchers.rs) shows the syntax and usage examples.

Public expectation types implement `Debug` and `Clone`, bounded on their operands and nested matchers. They are also
`Copy` when they hold no data, only concrete copyable data, or only borrowed targets. Callback definitions (`Predicate`,
`Satisfying`, `Pattern`, `HasValueSatisfying`) are `Clone` when their callback is and omit it from `Debug`. Identity definitions show addresses and type selections such as `IsOfType` name the
type, so neither bounds its target. Lists built by `matchers!`, `elements_are!`, and `entries_are!` use private nodes
without these traits. Arrays, slices, vectors, and tuples of matchers keep them.

## Evaluation and failure evidence

Each composite matcher decides which checks run, in what order, and how to describe a failure when no child evidence is
available. It uses the shared [expectation executor](expectation-execution.md) for tracking, child scopes, paths, budgets,
and failure handling.

| Matcher family | Evaluation | Empty subject/list |
|---|---|---|
| `all_of` | Evaluate every branch. | Accept. |
| `any_of` | Stop at first success. If all fail, report one nested failure ("does not match any alternative") whose children carry zero-based `Branch` facts. | Reject. |
| `each` | Evaluate every element without positional meaning. | Accept. |
| `contains_matching` | Stop at first matching element. A rejection stays one nested failure ("does not contain a matching element"), also when composed. | Reject. |
| `does_not_contain_matching` | Probe every element and identify unexpected matches. | Accept. |
| Positional collection matchers | Evaluate all available positions in each candidate window. Contiguous search stops at the first successful window. A rejected search reports "does not contain these elements contiguously" with one "does not match in this window" group per retained window, each carrying a `Window start` fact. | Empty criteria accept except exact matching against nonempty input. |
| `predicate` | Call `Fn(&T) -> bool`. `described_as` supplies the relation. There is no typed rejection error. Evaluation needs no renderer. Explaining a rejection renders the subject, so diagnostics require `R: ValueRenderer<T>`. | Not applicable. |

Candidate windows share the enclosing [evidence allowance](expectation-execution.md#budgets-and-probes). Failures from
earlier windows are discarded if a later window succeeds. Missing positions are described without evaluation. Suffix
constraints align from the subject's end. Constraints that precede a shorter subject are described without an index
path. [Iterator matchers](iterator-execution.md#retention-and-diagnostics) retain evidence according to the scan.
Iterator contiguous searches report the same per-window groups.

## Matcher lists

| Trait | Supported lists |
|---|---|
| `MatcherList<A, R>` | Sealed. `matchers!` nodes and tuples of up to twelve can mix matcher types. Arrays, slices, and vectors hold one matcher type. References reuse existing lists. |
| `EntryMatcherList<MapType, R>` | Sealed. `entries_are!` can mix keyed matcher types. Arrays, slices, and vectors hold one `Entry` type. |

Lists store definitions without boxing them or adding a renderer type parameter to the storage. Each slot can be
evaluated on a subject or described without one. Keyed slots also identify the stored key when the value check fails,
so [exact map checks](collection-semantics.md#keyed-maps) can determine which entries were visited.

List `_satisfying` adapters borrow callback slices after tracking and wrap each callback as it is needed. They allocate
no vector of wrappers and do not clone callbacks. Describing a missing slot never invokes its callback.
This differs from [bulk expected values](comparison-operands.md#reading-expected-data-again), which supply comparison
operands rather than executable constraints.

## Assertion callbacks

`satisfying` adapts a reusable `Fn` receiving a borrowed capture-mode chain and returning `()`. Every performed assertion
must pass. An empty callback panics as misuse. User panics propagate.

Its separate capture inherits the renderer, budget, and location policy, requiring `R: Clone`. Captured failures become
owned child failures with the enclosing path. Callback checks still execute and render inside a probe because they run
through this separate capture.

Ordinary assertion callbacks may accept `FnOnce` and retain their method's mode.
[`AssertThat::satisfies`](assertion-lifecycle.md#projections-and-continuation) projects and continues immediately.
`satisfying` constructs an expectation for later evaluation.

## Exact unordered assignment

An exact unordered match pairs actual occurrences with expected slots so that every occurrence and every slot is used
exactly once, including duplicates. The algorithm finds the largest possible set of pairs. Taking the first available
match is insufficient when expectations overlap.
The [shared utility](../assertr/src/util/matching.rs) supports equality, identity, matchers, and callbacks. When several
maximum assignments exist, no particular one is guaranteed.
[Keyed map checks](collection-semantics.md#keyed-maps) use native lookup.

### Candidate evaluation

Structural assignment evaluates each pair of an actual occurrence and an expected matcher at most once. A matcher can
still run for many pairs. Rejection diagnostics use the original observation. If assignment fails and diagnostics are
enabled, evaluate any remaining pairs that involve an unmatched occurrence or slot. Successful assignments, probes, and
zero item allowances skip this extra work. Probes also reject unequal lengths without comparisons and stop at the first
occurrence that cannot be assigned.

Each missing slot is reported with its constraint and candidate failures. An extra occurrence that satisfies an already
matched expectation is surplus, not a rejected candidate. `At slot` identifies a zero-based expected position, never an
actual collection index.

### Bounded candidate evidence

- Retain a sample of up to `k` direct child failures per occurrence and per slot, where `k` is the inherited item
  allowance. Nested children stay attached to their failure and take no additional sample positions.
- Sorted scopes rank child-report text. Original occurrence, slot, and child ordinal break ties and define unsorted order.
- After assignment, combine the samples. Report each rejection under its missing slot if there is one. Otherwise,
  report it under the unmatched occurrence if applicable. Failures from the remaining pair evaluations go directly to
  these destinations.
- Selecting and combining samples must not change pair evaluation order, budgets, or whether a rejection is explained.
  Rendering a constraint description is separate from explaining a pair's rejection.
- Assignment can make sampled candidates ineligible after their possible replacements have been discarded. In that
  case, leave the sample partly empty and count the lost candidates as omissions. Never evaluate a pair again to fill
  the gaps. Unlimited budgets retain all evidence.

For `n` occurrences and `m` slots, samples hold at most `k × (n + m)` references to failures, plus evidence for the current
pair. Shared failures are stored once. Nested failure trees and a scalar cache that can grow to O(nm) require additional
memory. The sample limit therefore does not bound total memory or guarantee faster execution.

Sampling and omission accounting are implemented in [assignment.rs](../assertr/src/expectation/assignment.rs). Pair
evaluation and report assembly are in [elements_are_in_any_order.rs](../assertr/src/assertions/collection/elements_are_in_any_order.rs),
with regressions `supports_overlapping_constraints` and
`assignment_can_starve_an_unexpected_sample_without_replaying_discarded_pairs`.

## Structural macros

`partial!` emits Rust patterns and borrowed projections. Only selected fields need comparison and rendering capabilities.
Rust still checks constructor resolution, visibility, field types, and exhaustiveness. Named `..` omits fields. Tuple
`_` skips one position and tuple `..` must be final.

Expected expressions construct explicit matchers once, in source order, within one expression. This preserves temporary
borrows through the statement. Selected fields, matcher-list elements, and `entries_are!` values follow the same rules.
Equality requires `eq`/`equal_to`, even for expressions that also implement equality.
[Operand selection](comparison-operands.md#borrowed-views) determines how reference-valued fields and items are compared.

Projections attach `Field`, `TupleIndex`, and optional `Variant` paths, including missing-subject descriptions. Constructor
paths support Rust's qualified and absolute forms. `variant::Type` means the optional marker followed by an absolute path.
Use `r#variant::Type` for a module named `variant`. Bare constructors named `variant` remain ordinary constructors.

`assertr::partial!` forwards `$crate` to a hidden procedural macro, so it works through renamed dependencies and facade
re-exports without looking up dependency names. Macro plumbing uses unsupported `__private`, with
[feature and version compatibility](platform-compatibility.md). The
[implementation](../assertr-macros/src/partial/mod.rs) and [fixtures](../assertr-macros/tests/partial/) distinguish
macro syntax errors from Rust type, visibility, exhaustiveness, and borrow errors.
