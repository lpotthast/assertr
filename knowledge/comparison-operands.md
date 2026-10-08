---
id: comparison-operands
depends_on: [ expectation-execution ]
sources:
  - assertr/src/lib.rs
  - assertr/src/expectation/mod.rs
  - assertr/src/assertions/core/partial_eq.rs
  - assertr/src/assertions/core/partial_ord.rs
  - assertr/src/assertions/core/range.rs
  - assertr/src/assertions/num/mod.rs
  - assertr/src/assertions/jiff/signed_duration.rs
  - assertr/src/assertions/collection/value.rs
  - assertr/src/assertions/core/debug.rs
  - assertr/src/assertions/core/display.rs
  - assertr/src/assertions/rootcause/report.rs
  - assertr/src/crate_docs.md
  - assertr/tests/bulk_allocations.rs
---

# Comparison operands

[Architecture overview](README.md)

Comparisons borrow a view of each expected value. The declared subject type determines which view is selected.
Native map queries also follow the [map lookup rules](collection-semantics.md#query-selection). Same-instance checks use
[pointer comparisons](reference-identity.md).

## Borrowed views

`E: BorrowFor<A>` selects an expected operand's borrowed type, `E::View`, through `Borrow<View>`. `A` is the declared
subject, item, key, value, or range-bound type. Assertr does not infer a separate comparison target. The same matcher can
therefore select different views when used with different subject types.

The `borrow-for` crate defines the standard selections and lets custom wrappers provide their own. Assertr re-exports it as
`assertr::borrow_for`. The [borrowed equality guide](../assertr/src/crate_docs.md#borrowed-equality) covers supported views
and wrapper implementations.

| Check | Required comparison |
|---|---|
| Equality | `A: PartialEq<E::View>`. Negation negates `eq`, even if `ne` is overridden. |
| Ordering | `A: PartialOrd<E::View>`. Incomparable results reject. String ordering may need explicit `str` views. |
| Range containment | Symmetric ordering bounds and [bound selection](#range-bound-selection). |
| Numeric tolerance, signed-duration tolerance, time-zone checks | `View = A`. Tolerance operands borrow independently. |

Scalar expectations can evaluate unsized `str` and slices directly. Subjects, fields, and items that are references keep
their declared types. Use `dereferenced` to check the value behind a reference, box, `String`, or other `Deref` value.
Chain storage follows [entry normalization](assertion-lifecycle.md#entry-subject-ownership-and-mode). Generic `.into()`,
`Default::default()`, and empty expected lists may need explicit types.

## Operand access

Constructors store operands without cloning or accessing views. Library-controlled access begins after
[assertion tracking](expectation-execution.md#chain-execution).

| Input | Evaluation and explanation |
|---|---|
| Scalar operand | Borrow once per evaluation. Retain the selected view on rejection. Explanation reuses it. Later evaluations borrow anew. |
| Tolerance operands | Borrow expected value and deviation independently. Signed duration borrows in that order and retains both views plus a flag for negative deviation. |
| Bulk expected list | The list and its operand views may be read again under the consistency rules below. |
| Missing subject | Access expected data to describe the requirement, without comparison, lookup, or callback execution. |

### Reading expected data again

Bulk value, key, and entry lists use `AsRef` to expose a finite slice. Arrays, slices, vectors, and compatible wrappers
provide this without copying their elements. Collect generators beforehand. Borrowed lists select views from the stored
element type, so a borrowed list of wrappers needs no extra `BorrowFor` implementation for references to those wrappers.

If you implement a wrapper, every slice access must return the same logical list, and every borrow must represent the
same comparison value. Assertr does not specify how often it accesses this data or how accesses interleave with
comparisons. Prepare stateful inputs beforehand, or retain their observed values in a custom expectation. This rule
allows repeated access to bulk data only. It does not permit repeating scalar borrows, callbacks, matcher evaluations,
guard acquisition, or identity observations.

Bulk rejections retain the failed observations. Explanation may read expected data again through the rendering context
and borrow only the operands that fit the display budget. It never repeats comparisons, searches, lookups, or
consumption. [Map observations](collection-semantics.md#map-observations) specify retained query results.

Prefix, suffix, ordered exact, collection membership, and map key checks allocate no expected-view buffer. Algorithms
may still need actual-element, membership, window, or assignment storage. The
[allocation fixture](../assertr/tests/bulk_allocations.rs) covers million-element successes and budgeted failures.

## Range-bound selection

For standard ranges whose bounds are references to sized values, the inherent containment methods select `RangeBounds<B>` for
the referenced value. This avoids ambiguity with `RangeBounds<&B>`. The unbounded range `..` needs an explicit `B` or a
`ContainsElement` expectation.
Methods and aliases delegate to `RangeBoundAssertions` with an explicit `B`. Custom ranges use that generic trait, and
fully qualified calls can select `B` explicitly.

| Reusable containment constructor | Selection |
|---|---|
| `ContainsElement::new`, `DoesNotContainElement::new` | Fix `B` to operand type. A reference operand selects a reference type. |
| `ContainsElement::<B>::borrowing`, `DoesNotContainElement::<B>::borrowing` | Select `B` explicitly and borrow through `BorrowFor<B>`. Direct methods use these definitions. |

Renderer requirements follow the selected view, as specified by
[rendering capabilities](diagnostic-rendering.md#capabilities-and-structure).
See [range rustdoc](../assertr/src/assertions/core/range.rs) for inference examples.

## Formatted-value comparison

| Assertion | Text compared |
|---|---|
| `has_debug_string` | Actual `Debug` output against expected preformatted text verbatim. |
| `has_debug_value` | Both operands' `Debug` output. |
| `has_display_value` | Both operands' `Display` output. |

These checks compare the complete generated text before diagnostic limits truncate it. On rejection, they retain that
text and render it through `ValueRenderer<str>` without formatting the operands again. Probes still format the operands
because the comparison needs their text. Rootcause current-context checks follow the same rule using the report's
formatter hook.
