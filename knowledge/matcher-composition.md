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
  - assertr/src/expectation/field.rs
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
that are `Debug`, `Clone`, and `Copy` when their matchers are, and print like arrays. Arrays, slices, and vectors of
matchers keep these traits too.

A dedicated `Success` or `Rejection` type of a public expectation is public in the same `matchers` namespace as the
expectation, such as `matchers::collection::MissingElementsRejection`, so a concrete composite can name it. Its fields
stay private unless reading them is part of the contract, as for the `CloseToRejection`, `HeaderRejection`, and
`ValueRejection` variants and the `LockObservation` state. It implements `Debug`, and identity rejections show
addresses.

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
| `predicate` | Call `Fn(&T) -> bool`. `described_as` supplies the relation. A rejection renders the subject with the `rejected_as` relation, or nests the description under "does not satisfy the constraint" without one. There is no typed rejection error. The expectation requires `R: ValueRenderer<T>`. | Not applicable. |
| `field` | Project one field with `Fn(&T) -> &F` and evaluate the inner matcher at a `.name` path. The rejection contributes the inner matcher's evidence under that path. `partial!` uses the same expectation with optional projections for enum variants. | Not applicable. |

Candidate windows share the enclosing [evidence allowance](expectation-execution.md#budgets-and-probes). Failures from
earlier windows are discarded if a later window succeeds. Missing positions are described without evaluation. Suffix
constraints align from the subject's end. Constraints that precede a shorter subject are described without an index
path. [Iterator matchers](iterator-execution.md#retention-and-diagnostics) retain evidence according to the scan.
Iterator contiguous searches report the same per-window groups.

## Matcher lists

Both list traits are authoring machinery and live in `assertr::expectation`, next to `Expectation` and
`AssertionContext`. The `matchers` catalog holds the constructors.

| Trait | Supported lists |
|---|---|
| `MatcherList<A, R>` | Sealed. `matchers!` nodes can mix matcher types. Arrays, slices, and vectors hold one matcher type, such as `[is_one, is_two].map(predicate)` for non-capturing closures. References reuse existing lists. |
| `EntryMatcherList<MapType, R>` | Sealed. `matchers![entry(..), ..]` can mix keyed matcher types. Arrays, slices, and vectors hold one `Entry` type. |

Lists store definitions without boxing them or adding a renderer type parameter to the storage. Each slot can be
evaluated on a subject or described without one. Keyed slots also identify the stored key when the value check fails,
so [exact map checks](collection-semantics.md#keyed-maps) can determine which entries were visited.

List `_satisfying` methods store their callback lists and borrow them only after tracking, wrapping each callback as it
is needed. They allocate no vector of wrappers and do not clone callbacks. Describing a missing slot never invokes its
callback. This differs from [bulk expected values](comparison-operands.md#reading-expected-data-again), which supply
comparison operands rather than executable constraints.

## Assertion callbacks

`satisfying` adapts a reusable `Fn` receiving a borrowed capture-mode chain and returning `()`. Every performed assertion
must pass. An empty callback panics as misuse. User panics propagate.

Its separate capture inherits the renderer, budget, and location policy, requiring `R: Clone`. Captured failures become
owned child failures with the enclosing path. Callback checks still execute and render inside a probe because they run
through this separate capture.

Ordinary assertion callbacks may accept `FnOnce` and retain their method's mode.
[`AssertThat::satisfies`](assertion-lifecycle.md#projections-and-continuation) projects and continues immediately.
`satisfying` constructs an expectation for later evaluation.

## Searches

`contains_matching`, `contains_value_matching`, `any_of`, and contiguous `elements_are` stop at the first candidate
that matches and would discard the evidence of earlier rejections. In a diagnostic context they therefore probe their
candidates first, so a passing search explains and renders nothing, even in a sorted scope whose child limit never
caps retention. Only a failing search evaluates its candidates again with diagnostics, so matchers and callbacks of
rejected candidates can run twice on the failure path. Inside a probe the search runs once. Streaming iterator scans
cannot restart and evaluate each candidate once, keeping evidence within the budget.

## Exact unordered assignment

An exact unordered match pairs actual occurrences with expected slots so that every occurrence and every slot is used
exactly once, including duplicates. The algorithm finds the largest possible set of pairs. Taking the first available
match is insufficient when expectations overlap.
The [shared utility](../assertr/src/util/matching.rs) supports equality, identity, matchers, and callbacks. When several
maximum assignments exist, no particular one is guaranteed.
[Keyed map checks](collection-semantics.md#keyed-maps) use native lookup.

### Candidate evaluation

The assignment search evaluates pairs of an actual occurrence and an expected matcher only as probes, without
diagnostics. A sparse cache ensures the search evaluates each pair at most once. A matcher can still run for many
pairs. Probes also reject unequal lengths without comparisons and stop at the first occurrence that cannot be assigned.

When assignment fails and diagnostics are enabled, the report is built from the search result:

- Each missing slot is recorded as one group with its constraint. Inside it, every occurrence whose pair did not match
  during the search is evaluated again with diagnostics. Rejections against a missing slot belong only to that slot.
- Unexpected occurrences are recorded in one group. An occurrence that satisfies an already occupied slot is surplus.
  It is described through those slots, without rendering the element. Otherwise, its pairs with the occupied slots are
  evaluated again with diagnostics, so unexpected occurrences keep their evidence.
- Groups beyond the [evidence allowance](expectation-execution.md#budgets-and-probes) are only counted, and their pairs
  are not evaluated. Successful assignments, probes, and zero item allowances skip this extra work.

On the failure path, a pair can therefore be evaluated twice: once as a probe and once with diagnostics. Matchers and
callbacks of rejected pairs can run twice. Evidence follows ordinary context ordering, either iteration order or sorted
rendered text. `At slot` identifies a zero-based expected position, never an actual collection index. Memory beyond the
retained evidence is bounded by the sparse pair cache.

Pair evaluation and report assembly are in
[elements_are_in_any_order.rs](../assertr/src/assertions/collection/elements_are_in_any_order.rs), with regressions
`supports_overlapping_constraints`, `explains_both_unmatched_sides_by_evaluating_their_pairs_again`, and
`the_search_probes_each_pair_once_and_diagnostics_follow_it`.

## Structural macros

`partial!` emits Rust patterns and borrowed projections. Only selected fields need comparison and rendering capabilities.
Rust still checks constructor resolution, visibility, field types, and exhaustiveness. Named `..` omits fields. Tuple
`_` skips one position and tuple `..` must be final.

Expected expressions construct explicit matchers once, in source order, within one expression. This preserves temporary
borrows through the statement. Selected fields, matcher-list elements, and `entries_are!` values follow the same rules.
Equality requires `eq`, even for expressions that also implement equality.
[Operand selection](comparison-operands.md#borrowed-views) determines how reference-valued fields and items are compared.

Projections attach `Field`, `TupleIndex`, and optional `Variant` paths, including missing-subject descriptions. Constructor
paths support Rust's qualified and absolute forms. `variant::Type` means the optional marker followed by an absolute path.
Use `r#variant::Type` for a module named `variant`. Bare constructors named `variant` remain ordinary constructors.

`assertr::partial!` forwards `$crate` to a hidden procedural macro, so it works through renamed dependencies and facade
re-exports without looking up dependency names. Macro plumbing uses unsupported `__private`, with
[feature and version compatibility](platform-compatibility.md). The
[implementation](../assertr-macros/src/partial/mod.rs) and [fixtures](../assertr-macros/tests/partial/) distinguish
macro syntax errors from Rust type, visibility, exhaustiveness, and borrow errors.
