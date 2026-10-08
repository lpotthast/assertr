---
id: failure-processing
depends_on: [ assertion-lifecycle, diagnostic-rendering ]
sources:
  - assertr/src/failure/mod.rs
  - assertr/src/failure/builder.rs
  - assertr/src/failure/failures.rs
  - assertr/src/failure/report.rs
  - assertr/src/failure/panic_presentation.rs
  - assertr/src/assert_that/diagnostics.rs
  - assertr/src/assert_that/execution.rs
  - assertr/src/expectation/context.rs
  - assertr/tests/panic_presentation.rs
  - assertr-no-std-tests/src/lib.rs
---

# Failure processing

[Architecture overview](README.md)

Failures become owned `AssertionFailure` values before capture or panic presentation. Consumers inspect structured data,
without parsing reports or retaining original Rust values.

## Structured construction and ownership

| Fields | Meaning |
|---|---|
| `kind` | Non-exhaustive failure family, not a specific assertion. |
| `actual`, `relation`, `expected`, `unexpected` | Rendered operands and their relation. Relations are lowercase sentences without embedded values or trailing periods. |
| `facts` | Labelled evidence via `Fact::labelled`, or an unlabelled note via `Fact::note` (empty label). |
| `children`, `constraint`, omission counts | Nested rejections and missing-subject descriptions. A constraint is another `AssertionFailure`. |
| `path` | Relative typed location within the parent subject. |
| Subject/caller metadata | Type, name, expression, caller location, detail messages. |

### Paths

`PathSegment` represents fields, tuple positions, variants, indexes, and rendered keys. All child locations use `path`,
including equality, identity, and matcher evidence.

- `Index`: stable collection position or direct iterator yield position. Order-free element evidence has no index.
- `Key`: rendered key tree with type metadata and omissions. Build key leaves in compact form through the active
  rendering context, using `render.compact()`, before storing the path. Presentation cannot compact an already rendered
  leaf.
- Paths compose parent to child. `FailureBuilder::path` appends relative segments.
- Human-readable child headings use paths, with compact keys. Facts always appear in `Details`. Labels such as `index`
  and `key` have no special location semantics.

### Builder completion

`FailureBuilder::new::<T>(kind)` starts a failure over a subject of type `T`. `build()` returns the data without chain
metadata. The [expectation executor](expectation-execution.md#chain-execution) passes a builder to
`Expectation::explain` and raises the returned builder itself. Child contexts build it and retain the failure as
evidence. [Execution adapters](observation-boundaries.md#execution-adapters) start a builder with `AssertThat::failure`
and pass it to `AssertThat::raise`, which adds the chain's caller location, subject name, expression, and detail
messages, then routes the failure through the active mode.

Every diagnostic value enters through the active rendering context. Consumers receive
[budgeted `Rendered` trees](diagnostic-rendering.md#bounded-retention) and cannot recover omissions. `AssertionFailure`,
`Fact`, and `Rendered` expose their data as public fields of non-exhaustive types.

`AssertionFailures` is the ordered aggregate returned by capture or fluent verification. It supports slices, iteration,
and vector conversion. See [assertion lifecycle](assertion-lifecycle.md#entry-subject-ownership-and-mode) for how capture
collects and returns failures, and [fluent entry](fluent-entry.md#pending-attachment) for delayed expression attachment.

## Report grammar

The `Display` implementation of `AssertionFailure` chooses the layout from the populated fields. `FailureKind` does
not select a format:

| Fields or section | Presentation |
|---|---|
| Ordinary body | Actual → relation → expected → unexpected. Omit absent parts. |
| Actual and expected, with no relation or unexpected operand | Aligned expected/actual comparison. |
| `constraint` | Nested description after the body, using the same grammar. |
| Messages, facts, children | `Messages`, `Details`, `Nested failures`, in that order. |
| Omitted children | Detail note from structured omission count. |

Caller, subject, and expression metadata precede the body. Children indent one level per depth and use
[typed path headings](#paths). Assertions supply structured fields rather than report text.
`Debug` prints the same report. `AssertionFailures` displays its failures separated by an empty line. Exact formatting
tests live in [report.rs](../assertr/src/failure/report.rs).

## Presentation and fallback

After capture, the caller chooses how to present the failures, for example through `to_string()`. Capture and panic
presentation never write to streams. Panic mode uses the default report unless `with_panic_presentation` installs an
owned `'static + RefUnwindSafe` closure from `&AssertionFailure` to `String`. Derived chains share it through `Rc`. It
requires none of `Send`, `Sync`, or `Clone`. The private `PanicPresentation` type preserves the unwind-safety bound.

| Presentation outcome | Behavior |
|---|---|
| Panics, with `std` | Catch, then panic with the default report plus a presentation diagnostic. |
| Panics, without `std` | Propagate. |

Fallback does not retry the closure for that failure, although later assertions may reuse it.
Regressions: [`a_panicking_presentation_propagates_without_std`](../assertr-no-std-tests/src/lib.rs) and
[`a_presentation_panic_preserves_the_failure_and_adds_a_diagnostic`](../assertr/tests/panic_presentation.rs).
