---
id: iterator-execution
depends_on: [ expectation-execution ]
sources:
  - assertr/src/assertions/core/iter/exact_size.rs
  - assertr/src/assertions/core/iter/iterator.rs
  - assertr/src/assertions/iterator/*.rs
---

# Iterator execution

[Architecture overview](README.md)

Iterator assertions scan the input once and stop when they can decide the result. They cannot restart the scan to
explain a failure, so they retain the values and observations needed for diagnostics. By comparison,
[collection assertions](collection-semantics.md#capability-model) can traverse the same collection more than once.

## Entry and continuation

| Family | Input and result |
|---|---|
| Terminal `IteratorAssertions` | Require an owned iterator. Consume the needed input, drop the rest, and return a chain over `()`. Borrowed traversal passes a borrowing iterator, such as `assert_that_owned!(values.iter())`. |
| `ExactSizeIterator` length checks | Read `len()` without consuming items. |

## Consumption

| Check | How far it scans |
|---|---|
| Single-element membership | Stop at the first match or forbidden item, or when the input ends. |
| `contains_all` | Stop when all expected values have matched or the input ends. An empty expected list consumes nothing. |
| `is_exhausted`, `is_not_exhausted` | Call `next()` once. |
| `has_count` | Use an exact `size_hint` only to reject a mismatch without consuming. Otherwise, including when the hint agrees, consume at most the expected count plus one. Report an exact count if the input ends, or a lower bound otherwise. |
| Empty prefix, suffix, or contiguous pattern | Consume nothing. |
| Prefix and positional exact equality | Compare each visited pair once and stop at the first mismatch. Exact matching reads at most the expected length plus one. |
| Nonempty suffix | Read to the end, then compare every required position. Distinguish input that is too short from unequal values. |
| Unordered exact equality | Reject a mismatching exact `size_hint` without consuming. Otherwise, buffer at most the expected length plus one and evaluate the collection's `ContainsExactlyInAnyOrder` on the buffer, adopting its report. A full buffer adds `Consumed elements`. |
| Unordered exact matchers | Reject a mismatching exact `size_hint` without consuming. Otherwise, buffer at most the expected length plus one, then use collection assignment and its diagnostics, even when the buffered lengths differ. |
| Contiguous pattern | Try overlapping windows, evaluating each window once, and stop at the first successful window. |

An exact `size_hint` can establish some failures without consuming items. Such reports show `Reported length` and
`Expected length` without an actual value or consumption count. A hint never establishes success. An assertion on
infinite input can finish only if it can decide the result without reaching the end.

## Retention and diagnostics

| Purpose | What is retained |
|---|---|
| Equality preview | The last 16 consumed items. Unordered equality instead shows every buffered item. |
| Suffix or contiguous equality window | Up to `max(pattern length, 16)` items, regardless of the diagnostic budget. |
| Matcher window | As many items as the pattern has positions. |
| Prefix, exact, or suffix equality failure | Indexed child failures, subject to the item budget. Successful pairs add none. The actual-value preview has a separate budget. |
| Matcher membership | Evidence from the first rejected candidates within the budget. Later rejections are only counted as omitted. |
| Contiguous matcher failure | One "does not match in this window" group with a `Window start` fact per rejected window, for the first rejected windows within the budget. Each group takes one slot. Later windows are still evaluated once and counted as omitted. A position can appear once per window that rejected it. |

Windows hold the items needed to perform a check, so diagnostic limits do not reduce them. Storage grows as input
arrives without reserving the pattern length upfront. Equality previews trim existing storage. Unordered equality
explains a rejection while its buffered items are alive and keeps only the owned report. See the
[scan implementation](../assertr/src/assertions/iterator/mod.rs) for queue details.

Matcher reports state how much input was consumed in a `Consumed elements` fact and locate child failures with typed
paths. They do not include equality-preview metadata. When a scan rejects empty input, the report describes the unmet
constraint. Omitting evidence from a nonempty scan does not trigger that fallback. Missing positional matchers are
described without evaluating them or running callbacks. Budgets may further truncate the evidence.

Iterator diagnostics identify items by the order in which they were yielded. Locations use
[typed paths](failure-processing.md#paths).

A zero item budget omits child failures for prefix, exact, and suffix mismatches. The assertion still fails and records
how many children were omitted. Streaming `contains_all` retains expected data and borrows missing operands only when
displaying them.

## Observation lifetime

The [execution adapter](../assertr/src/assertions/iterator/mod.rs) keeps one iterator alive throughout the scan and its
diagnostics. Neither the iterator nor its items need `Clone`. The private `consume` helper of `IteratorAssertions`
creates each scan inside the adapter, after tracking.

1. Track the assertion, then access the expected list before scanning. Borrow operands as comparisons reach them.
2. Scan through `&mut I` and retain the result. Explanation must not call `next` or `size_hint` again.
3. On rejection, keep the iterator and evidence alive until `Scan::explain` renders owned diagnostic values.
4. Drop the iterator before raising the failure, including on early exits and when the iterator owns a guard. Successful
   scans drop it before returning.

Explanation may [read expected data again](comparison-operands.md#reading-expected-data-again) under the usual
consistency requirements. Matcher and callback lists follow the [matcher-list rules](matcher-composition.md#matcher-lists).

[Streaming regressions](../assertr/src/assertions/iterator/tests.rs):

| Test | What it verifies |
|---|---|
| `direct::scans_keep_resources_until_rendering_and_preserve_stopping_points` | Equality and matcher scans stop at the required point and do not repeat observations. |
| `borrowed::membership_and_unordered_adapters_retain_the_owning_iterator` | Borrowed scans keep the iterator alive through rendering. |
| `borrowed::contains_all_stops_on_success_or_exhaustion` | Borrowed membership stops on success or at the end of input. |
| `borrowed::cardinality_keeps_resources_through_explanation_without_repeating_observations` | Length checks retain resources through explanation without observing them again. |
| `release::panic_routing_releases_the_iterator_before_presentation_without_poisoning` | Iterators release guards before panic presentation. |
| `tracking::sequence_views_are_accessed_after_tracking` | Expected views are accessed only after the assertion is tracked. |
| `reporting::failure_preview_is_capped_and_retains_the_decisive_item` | Equality previews stay bounded and keep the item that decided the failure. |
