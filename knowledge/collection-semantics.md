---
id: collection-semantics
depends_on: [ ]
sources:
  - assertr/src/assertions/has_length.rs
  - assertr/src/assertions/collection/mod.rs
  - assertr/src/assertions/collection/value.rs
  - assertr/src/assertions/set/mod.rs
  - assertr/src/assertions/map/mod.rs
  - assertr/src/assertions/map/imp.rs
  - assertr/src/assertions/map/entries_are.rs
  - assertr/src/assertions/core/iter/exact_size.rs
  - assertr/src/assertions/core/iter/iterator.rs
  - assertr/src/assertions/core/iter/into_iterator.rs
  - assertr/src/assertions/iterator/mod.rs
---

# Collections, maps, and iterators

[Architecture overview](README.md)

Rust uses subject capability bounds to select operations. Repeatable traversal, semantic positions, indexed access, and
native lookup are separate contracts. Diagnostic presentation supplies none of them.

## Capability model

Bounds enforce trait availability. Repeatability, meaningful positions, lookup consistency, and complexity are
implementor obligations that Rust cannot verify. The following table states those obligations.

| Capability                  | Required contract                                                                                             | Enables                                      |
|-----------------------------|---------------------------------------------------------------------------------------------------------------|----------------------------------------------|
| `HasLength`                 | Finite native length through `length()`. Strings count bytes. Integer range counts must fit `usize` or panic. | Length and emptiness checks.                 |
| `Collection: HasLength`     | `elements()` repeatedly yields references to the same elements in the same order.                             | Order-free element checks.                   |
| `StableOrder: Collection`   | Iteration positions are part of the value's semantics.                                                        | Positional checks and stable index evidence. |
| `RandomAccess: StableOrder` | `element_at` takes constant time and returns `None` out of bounds.                                            | Indexed extraction.                          |
| `SetLookup: Collection`     | Elements are unique. Native membership uses the equivalence relation enforcing that uniqueness.               | Subset, superset, and disjointness checks.   |
| `Map: HasLength`            | Repeatable `entries()` traversal of stored key/value references.                                              | Iteration-based map checks.                  |
| `MapLookup<Q>: Map`         | Native borrowed-key lookup returning the stored key and value.                                                | Key and keyed-entry queries.                 |

The [collection](../assertr/src/assertions/collection/mod.rs), [set](../assertr/src/assertions/set/mod.rs), and
[map](../assertr/src/assertions/map/mod.rs) rustdoc defines implementor contracts. A linked list has stable positions
without random access. A sorted set has deterministic traversal without semantic positions. Heaps also support
order-free collection checks. Strings use `StrAssertions`, not element-collection semantics.

Tree sets and maps work with `alloc`. Hash collections require `std`. Their
[presentation settings](diagnostic-rendering.md#capabilities-and-structure) affect syntax and evidence ordering only.

Expected elements and map values follow the [borrowed-view contract](expectation-execution.md#comparison-operands).
Bulk value lists use [repeatable expected data](expectation-execution.md#repeatable-bulk-expected-data).
List storage stays independent of element storage, so borrowed non-`Copy` elements need no clones.
Empty generic lists may require an explicit element type.

## Exact comparisons and keyed maps

Exact element comparisons preserve occurrence counts. In any-order comparisons, each actual occurrence must match a
distinct expected slot. The [shared assignment algorithm](matcher-composition.md#exact-unordered-assignment) handles
values, identity, matchers, and assertion callbacks, including overlapping expectations.

Exact keyed map checks instead use native lookup, length, and stored-key identity. `Map::entries` and
`MapLookup::get_key_value` must return references to the same stored keys and values. Checks remember visited entries,
so repeating a query cannot hide a missing distinct entry. A present key whose value is rejected remains visited and is
not also reported as unexpected. Each expected key occurrence performs one native lookup, retaining its result for
explanation. Bulk expected data may be borrowed again without repeating lookup.

`BorrowFor<K>` selects a bulk operand's `View` using the stored key type as context. Its `Borrow<View>`
implementation supplies the query. `MapLookup<View>` separately grants native lookup, carrying only that map's bounds.
Selection grants no universal `Hash`, `Ord`, or `PartialEq` requirement and never enables an equality-scan fallback.
Single-key methods still accept native `&Q` queries without operand registration.

String keys accept `str` views, and `Vec<u8>` keys accept `&[u8]` operands. Array operands currently select array views,
which do not supply native vector-key lookup. Pass an explicit slice. Fixed-view `AsRef` and collection capabilities
keep their existing contracts. Bulk list `AsRef` access follows the repeatable expected-data contract.

Custom bulk operands migrate from `MapKeyQuery`'s `Query` and `as_query()` to `BorrowFor<K>::View` and `Borrow<View>`.
The `Borrow` equality, ordering, and hashing obligations apply where those capabilities exist. Arbitrary field
projections should use an accessor or a dedicated query operand. References to custom wrappers need separate
implementations or explicit views when passed as individual operands. Borrowing the whole list uses the stored
wrapper type and needs no reference implementation. Foreign key and query types may require a local operand wrapper under orphan rules.
See [MapAssertions](../assertr/src/assertions/map/assertions.rs) for checked examples and the native lookup boundary.

## Borrowed traversal versus terminal streams

[`into_iter_*` assertions](../assertr/src/assertions/core/iter/into_iterator.rs) create one fresh borrowed iterator per
call and return the original chain. Terminal [`IteratorAssertions`](../assertr/src/assertions/core/iter/iterator.rs)
require ownership, consume the needed prefix, drop the remainder, and return a chain over `()`. Non-consuming
`ExactSizeIterator` length checks use `len()` separately.

Membership stops at the first match or forbidden item. Empty prefixes, suffixes, and contiguous criteria consume
nothing. Non-empty suffix checks must exhaust the iterator. Positional exact checks read at most the expected length
plus one, and may stop at the first mismatch. Unordered exact checks buffer at most that many elements. Exact
`size_hint` metadata can decide some failures without consumption. Infinite streams work only when a decision is
reachable without exhaustion.

[Equality previews](../assertr/src/assertions/iterator/mod.rs) retain the last 16 consumed items. Matcher scans retain
selected owned `Evidence`. Rendering budgets can further truncate either. A missing positional matcher is described
without evaluating it or running its callback. Direct iterator diagnostics may report yield positions. Borrowed
traversals do not report offsets as stable collection indexes.

The [streaming execution adapter](observation-boundaries.md#traversal) owns the scan and retained observations. Those
mechanics are independent of the capabilities that select collection operations.
