---
id: reference-identity
refines:
  - assertr
depends_on:
  - diagnostic-rendering
  - matcher-composition
related_to:
  - collection-semantics
  - fluent-entry
sources:
  - assertr/src/assertions/core/identity.rs
  - assertr/src/assertions/collection/identity.rs
  - assertr/src/entry/mod.rs
  - assertr/src/util/matching.rs
---

# Reference identity

[Architecture overview](README.md)

Same-instance assertions compare pointers with `core::ptr::eq`. Their result does not establish allocation identity or an application-level object ID.

## Which address is compared

Borrowing entry normalizes one reference layer for sized pointees. Entering with a value or `&value` therefore normally compares the value's address. If the subject is itself reference-valued after owning entry, projection, or unsized entry, scalar identity compares storage for that reference. It does not implicitly dereference again. See [entry and ownership](assertion-lifecycle.md#entry-subject-ownership-and-mode).

Collection identity borrows each stored item through its declared `Borrow<U>` target and compares that target with the caller's `&U`. It requires neither `PartialEq` nor a renderer for `U`. Diagnostics contain budgeted pointer text and type information. Ordered exact checks additionally require `ValueRenderer<usize>` for length evidence.

## Order, multiplicity, and pointer limits

Ordered exact identity requires `StableOrder` and pairs references positionally. Unordered exact identity uses [one-to-one assignment](matcher-composition.md#exact-unordered-assignment) to preserve duplicate occurrences. One address occurrence cannot satisfy several expected slots. Missing and unexpected addresses remain separate evidence.

Fat-pointer equality includes metadata. Pointers with the same data address can differ in slice length or trait-object metadata and fail identity. Diagnostics mention this when equal data addresses reveal the distinction. Distinct zero-sized values may share an address and compare as the same instance.

## Sources

[Scalar identity](../assertr/src/assertions/core/identity.rs) and [collection identity](../assertr/src/assertions/collection/identity.rs) implement pointer comparisons. [Entry macros](../assertr/src/entry/mod.rs) normalize references. [Collection capabilities](collection-semantics.md#capability-model) determine which ordered comparisons are available.
