---
id: reference-identity
depends_on: [ assertion-lifecycle, collection-semantics ]
sources:
  - assertr/src/assertions/core/identity.rs
  - assertr/src/assertions/collection/identity.rs
  - assertr/src/entry/mod.rs
---

# Reference identity

[Architecture overview](README.md)

Same-instance assertions use `core::ptr::eq`. They establish pointer equality, not allocation identity or an
application-level object ID.

## Which address is compared

[Scalar identity](../assertr/src/assertions/core/identity.rs) compares the current subject's address. Borrowing entry
normalizes one reference layer for sized pointees, so entering with a value or `&value` normally compares the value's
address. If owning entry, projection, or unsized entry leaves a reference-valued subject, scalar identity compares
storage for that reference. It does not dereference again.

[Collection identity](../assertr/src/assertions/collection/identity.rs) compares each item's declared `Borrow<U>` target
with the caller's `&U`. It requires neither `PartialEq` nor `ValueRenderer<U>`. Diagnostics use budgeted pointer text
and type information. Positional exact checks additionally need `ValueRenderer<usize>` for length evidence.

## Order, multiplicity, and pointer limits

Ordered exact identity requires `StableOrder`. Unordered exact identity uses
[one-to-one assignment](matcher-composition.md#exact-unordered-assignment) to preserve duplicate occurrences. Missing
and unexpected addresses remain separate evidence. Diagnostic retention is budgeted, but unordered assignment still
buffers all actual targets needed to decide truth.

Fat-pointer equality includes metadata. Equal data addresses with different slice lengths or trait-object metadata can
compare unequal. Collection diagnostics identify these metadata mismatches when observed. Distinct zero-sized values can
share an address and compare as the same instance.
