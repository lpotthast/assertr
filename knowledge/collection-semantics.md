---
id: collection-semantics
depends_on: [ ]
sources:
  - assertr/src/assertions/has_length.rs
  - assertr/src/assertions/collection/mod.rs
  - assertr/src/assertions/collection/value.rs
  - assertr/src/assertions/set/mod.rs
  - assertr/src/assertions/map/mod.rs
  - assertr/src/assertions/map/assertions.rs
  - assertr/src/assertions/map/imp.rs
  - assertr/src/assertions/map/entry.rs
  - assertr/src/assertions/map/entries_are.rs
---

# Collection, set, and map semantics

[Architecture overview](README.md)

Collection traits determine which assertions a type supports. Traversal, meaningful positions, indexing, and native
lookup are separate capabilities. Diagnostic formatting does not grant any of them. Rust checks the trait bounds, but
implementors must provide the promised behavior and performance.

## Capability model

| Capability | What the implementation must provide | Enables |
|---|---|---|
| `HasLength` | A finite native `length()`. Strings count bytes. Integer range counts must fit `usize` or panic. | Length and emptiness checks. |
| `Collection: HasLength` | `elements()` repeatedly yields references to the same elements in the same order. | Order-free checks. |
| `StableOrder: Collection` | Element positions are part of the collection's meaning, as in a list. | Positional checks and indexes in failures. |
| `RandomAccess: StableOrder` | Constant-time `element_at`, `None` out of bounds. | Indexed extraction. |
| `SetLookup: Collection` | Unique elements and native membership using the same equivalence that makes elements unique. | Set relations. |
| `Map: HasLength` | Each traversal returns references to the same stored keys and values. | Map checks that iterate over entries. |
| `MapLookup<Q>: Map` | Native borrowed-key lookup returning the same stored entries as traversal. | Key and keyed-entry queries. |

A linked list has stable positions without random access. A sorted set always traverses in order, but that order does
not make its elements positional. Heaps support checks that do not depend on order. Strings use `StrAssertions`.
Shared and mutable reference subjects forward every collection, set, and map capability of their target.
Iteration-only implementations (`HasLength`, `Collection`, `Map`) carry no lookup bounds such as `BuildHasher`, `Hash`,
or `Ord`. Those belong on `SetLookup` and `MapLookup`.
See [platform compatibility](platform-compatibility.md#feature-support) for collection feature requirements.

The [collection](../assertr/src/assertions/collection/mod.rs), [set](../assertr/src/assertions/set/mod.rs), and
[map](../assertr/src/assertions/map/mod.rs) rustdoc gives the full trait requirements.
See [diagnostic rendering](diagnostic-rendering.md#capabilities-and-structure) for presentation rules.

## Membership and exactness

| Operation | Meaning |
|---|---|
| Element membership | Search by the [selected comparison view](comparison-operands.md#borrowed-views). Set subjects still use collection equality for these checks. |
| `contains_all` | Every expected value has a match. Duplicate expectations may reuse one actual occurrence. |
| Ordered exact | Match every position and require equal lengths. Needs `StableOrder`. |
| Unordered exact | Pair every actual occurrence and every expected slot exactly once, preserving duplicates. Uses [maximum assignment](matcher-composition.md#exact-unordered-assignment). |
| Set relations | Native `SetLookup` membership and its uniqueness equivalence. |

For example, `[1]` contains all of `[1, 1]`, but does not contain exactly those occurrences in any order.
[Matcher composition](matcher-composition.md#evaluation-and-failure-evidence) explains how matcher-based checks evaluate candidates.
[Iterator execution](iterator-execution.md) covers checks that scan input once.

Collection prefix and suffix equality retain the original mismatch and render it as a child failure only when needed.
A zero item budget records one omitted child. A rejection caused only by length has no mismatch child. Paths remain
relative to the collection subject. Suffix equality aligns both lists at their ends, so a shorter subject is compared
with the suffix's tail and reports its length plus only genuine mismatches.

## Keyed maps

Exact keyed map checks use native lookup, length, and stored-key identity:

- `Map::entries` and `MapLookup::get_key_value` must reference the same stored keys and values.
- Repeated queries cannot cover a missing distinct entry.
- A present key counts as visited even if its value fails the check. It is not also reported as unexpected.
- Duplicate queries that match every entry while lengths differ report a length failure.

### Query selection

| Input | Contract |
|---|---|
| Bulk key operand | `BorrowFor<K>::View`, with stored key type `K` as context. `Borrow<View>` supplies the query. |
| Native lookup | Separately requires `MapLookup<View>` with that map's own bounds. No universal `Hash`, `Ord`, or `PartialEq` requirement and no equality-scan fallback. |
| Single-key method | Accepts native `&Q` without operand registration. |
| String key | Accepts `str` views. |
| `Vec<u8>` key | Accepts explicit `&[u8]`. Array operands select array views and do not supply native vector-key lookup. |

Custom wrappers and borrowed lists follow the [operand access rules](comparison-operands.md#operand-access). Native
lookup also relies on `Borrow` preserving the applicable equality, ordering, and hashing behavior. Arbitrary field
projections need an accessor or a dedicated query operand. Rust's orphan rules may require a local wrapper when key
and query types come from another crate.
See [MapAssertions](../assertr/src/assertions/map/assertions.rs) for checked examples.

### Map observations

| Check | Evaluation and retained rejection |
|---|---|
| `contains_keys` | One lookup per expected occurrence. Retain missing queries only. |
| Exact-entry equality | Resolve key, then expected value, then perform one lookup per occurrence. Compare present values. Retain missing queries, unequal-value views, and stored-key coverage. One child failure per mismatched expected entry. |
| `Entry` | Resolve query once for lookup and path. Retain stored-key identity and owned child evidence. |

Explanation repeats neither lookup nor comparison. `Entry` also reuses its query observation without borrowing again.
Other bulk expected data may still be [read again](comparison-operands.md#reading-expected-data-again).
