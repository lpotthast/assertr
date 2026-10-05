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

Same-instance assertions use `core::ptr::eq`. They compare pointers, which cannot establish whether two values belong to
the same allocation or share an application-level ID.

## Which address is compared

| Family | Compared address | Diagnostic bounds |
|---|---|---|
| [Scalar](../assertr/src/assertions/core/identity.rs) | The current subject's address, without further dereferencing. | No equality or renderer bounds. Reports use pointer text and type information. |
| [Collection](../assertr/src/assertions/collection/identity.rs) | Each item's declared `Borrow<U>` target against the caller's `&U`. | No `PartialEq` or `ValueRenderer<U>`. Positional exact checks also need `ValueRenderer<usize>` for lengths. |

[Entry normalization](assertion-lifecycle.md#entry-subject-ownership-and-mode) determines the subject type.
For `AssertThat<&T>`, scalar identity takes `&&T` and compares the reference's storage, not its pointee.

## Order, multiplicity, and pointer limits

- Ordered exact identity requires `StableOrder`.
- Unordered exact identity uses [one-to-one assignment](matcher-composition.md#exact-unordered-assignment), preserving
  duplicates and reporting missing and unexpected addresses separately. Deciding the result still requires buffering
  all actual targets.
- Pointer diagnostics are budgeted. Fat-pointer equality includes metadata. Equal data addresses can compare unequal
  because slice lengths or trait-object metadata differ. Collection diagnostics identify observed metadata mismatches.
- Distinct zero-sized values may share an address and compare as the same instance.
