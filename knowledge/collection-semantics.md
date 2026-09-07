---
id: collection-semantics
refines:
  - assertr
depends_on:
  - diagnostic-rendering
related_to:
  - matcher-composition
  - assertion-lifecycle
  - reference-identity
sources:
  - assertr/src/assertions/collection/**
  - assertr/src/assertions/set/**
  - assertr/src/assertions/map/**
  - assertr/src/assertions/core/iter/**
  - assertr/src/assertions/iterator/**
  - assertr/src/matchers/entries_are.rs
  - assertr/src/util/matching.rs
---

# Collections, maps, and streaming iterators

[Architecture overview](README.md)

Collection assertions are selected by behavioral capabilities. A custom type implements the capabilities its semantics
support.

## Capability model

| Capability     | Contract                                                                                                                       |
|----------------|--------------------------------------------------------------------------------------------------------------------------------|
| `HasLength`    | Supplies length.                                                                                                               |
| `Collection`   | Extends `HasLength` with repeatable inspection by reference. Every `elements` call yields the same elements in the same order. |
| `StableOrder`  | Makes iteration positions part of the value's semantics. Enables ordered assertions and index-bearing evidence.                |
| `RandomAccess` | Adds constant-time indexed access to stable order.                                                                             |
| `SetLookup`    | Supplies native membership for set relations.                                                                                  |
| `Map`          | Supplies repeatable traversal of stored key/value entries.                                                                     |
| `MapLookup<Q>` | Adds native lookup through a borrowed query type.                                                                              |

Sets and heaps support order-free collection operations. A sorted set still has no semantic positions. A linked list has
stable order but lacks random access.

`CollectionPresentation` and `RenderingOrder` determine diagnostic syntax and ordering. They grant no behavioral
capabilities. Tree maps and sets are available with `alloc`. Hash collections require `std`.

## Exact comparisons and keyed maps

Exact unordered comparisons of collection elements and iterator items preserve duplicates
through [maximum one-to-one assignment](matcher-composition.md#exact-unordered-assignment). This works for values,
matchers, and assertion callbacks, including overlapping expectations.

Exact keyed map checks use native lookup, length checks, and stored-key identity. `Map::entries` and
`MapLookup::get_key_value` must return references to the same stored keys and values. The checks remember which entries
expected keys reached, then report any unvisited entries as unexpected. Repeating a query cannot hide a missing distinct
entry.

`MapLookup<Q>` accepts borrowed views such as `str` for a `String` key. `MapKeyQuery` resolves bulk-query inference
without imposing universal `Hash` or `Ord` bounds. Each map implementation carries the bounds its native lookup needs.

## Borrowed traversal versus terminal streams

Borrowed `IntoIterator` assertions create one fresh iterator per assertion and return the original chain. Their names
use the `into_iter_` prefix. Direct `Iterator` assertions take ownership, consume the stream, drop any remainder, and
return an assertion over `()`.

Membership stops at the first match. Negative membership stops at the first forbidden item. Contiguous search stops when
a window succeeds. Empty prefix, suffix, and contiguous criteria consume nothing. A non-empty suffix check must exhaust
the iterator to know its ending.

Ordered exact comparison can stop at its first mismatch. An unordered exact check buffers at most the expected length
plus one to detect a surplus item. Exact length metadata from `size_hint` can let equality and cardinality checks fail
without consuming anything. An infinite iterator is usable only when the assertion can reach a decision without
exhaustion.

Equality diagnostics retain a tail of at most 16 consumed items. This tail includes the decisive item when a consumed
item ends the scan, but a smaller rendering budget can omit it from the displayed preview. Matcher scans retain selected
evidence instead of an equality preview. [Rendering budgets](diagnostic-rendering.md#bounded-retention) limit the final
evidence in either case.

Direct iterator assertions may report yield positions. Borrowed traversal offsets are not stable collection
indexes. [Reference identity](reference-identity.md) describes pointer comparisons and their ordering requirements.

## Sources

The [collection](../assertr/src/assertions/collection/mod.rs), [set](../assertr/src/assertions/set/mod.rs),
and [map](../assertr/src/assertions/map/mod.rs) module docs define implementor
contracts. [Map checks](../assertr/src/assertions/map/imp.rs)
and [keyed matchers](../assertr/src/matchers/entries_are.rs) track stored
entries. [Iterator](../assertr/src/assertions/core/iter/iterator.rs)
and [borrowed traversal](../assertr/src/assertions/core/iter/into_iterator.rs) traits use the
shared [streaming implementation](../assertr/src/assertions/iterator/mod.rs).
