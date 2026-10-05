---
id: failure-processing
depends_on: [ assertion-lifecycle, diagnostic-rendering ]
sources:
  - assertr/src/failure/mod.rs
  - assertr/src/failure/builder.rs
  - assertr/src/failure/failures.rs
  - assertr/src/failure/adapter/mod.rs
  - assertr/src/failure/adapter/adapters/human_readable.rs
  - assertr/src/failure/adapter/adapters/writer.rs
  - assertr/src/failure/panic_presentation.rs
  - assertr/src/assert_that/diagnostics.rs
  - assertr/src/assert_that/execution.rs
  - assertr/src/expectation/context.rs
  - assertr/tests/failure_adapters.rs
  - assertr-no-std-tests/src/lib.rs
---

# Failure processing

[Architecture overview](README.md)

Failures become owned `AssertionFailure` values before capture or panic presentation. Adapters inspect structured data,
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
  rendering context, using `IntoRendered::into_rendered_compact`, before storing the path. Presentation cannot compact
  an already rendered leaf.
- Paths compose parent to child. `FailureBuilder::path` appends relative segments.
- Human-readable child headings use paths, with compact keys. Facts always appear in `Details`. Labels such as `index`
  and `key` have no special location semantics.

### Builder completion

| Target | Construction | Completion |
|---|---|---|
| `Attached` | `AssertThat::failure` | `.raise()` adds chain metadata and routes through the mode. The target is a concrete view borrowing the chain's records and settings. |
| `Detached` | `FailureBuilder::detached::<T>` | `.build()` returns data without raising or collecting chain metadata. |

The [expectation executor](expectation-execution.md#chain-execution) and
[execution adapters](observation-boundaries.md#execution-adapters) select and complete these targets.

Every diagnostic value enters through the active rendering context. Adapters receive
[budgeted `Rendered` trees](diagnostic-rendering.md#bounded-retention) and cannot recover omissions. `AssertionFailure`,
`Fact`, and `Rendered` expose fields and read-only accessors. Every `AssertionFailure` field has an accessor of the same
name.

`AssertionFailures` is the ordered aggregate returned by capture or fluent verification. It supports slices, iteration,
and vector conversion. See [assertion lifecycle](assertion-lifecycle.md#entry-subject-ownership-and-mode) for how capture
collects and returns failures, and [fluent entry](fluent-entry.md#pending-attachment) for delayed expression attachment.

## Report grammar

`ToHumanReadableText` chooses the layout from the populated fields. `FailureKind` does not select a format:

| Fields or section | Presentation |
|---|---|
| Ordinary body | Actual → relation → expected → unexpected. Omit absent parts. |
| Actual and expected, with no relation or unexpected operand | Aligned expected/actual comparison. |
| `constraint` | Nested description after the body, using the same grammar. |
| Messages, facts, children | `Messages`, `Details`, `Nested failures`, in that order. |
| Omitted children | Detail note from structured omission count. |

Caller, subject, and expression metadata precede the body. Children indent one level per depth and use
[typed path headings](#paths). Assertions supply structured fields rather than report text.
Exact formatting tests live in [human_readable.rs](../assertr/src/failure/adapter/adapters/human_readable.rs).

## Presentation and fallback

`Adapter<Input>` converts borrowed input to its declared output or error. `AdapterExt::then` composes conversions.
`AdapterExt` is implemented for every sized type, so importing it adds `then` and `map_err` to all types in that scope.
Import it only where adapters are composed, because it can clash with methods such as `FutureExt::then`. `ThenError`
displays only the failed stage and exposes the stage's error through `Error::source`. Its alternate form (`{:#}`)
appends that error. `ToHumanReadableText` converts one failure or an aggregate using the
[report grammar](#report-grammar). Its result, `HumanReadableText`, owns the text.

After capture, the caller chooses how to present the failures. Panic mode uses the default text adapter unless
`with_panic_presentation` installs an owned `'static + RefUnwindSafe` text adapter with a `Display` error. Its errors
are converted to text with the alternate form (`{error:#}`), so composed errors keep their stage's cause. Derived chains
share the adapter through `Rc`. It requires none of `Send`, `Sync`, or `Clone`. The private `PanicPresentation` type
preserves the unwind-safety bound.

| Panic-adapter outcome | Behavior |
|---|---|
| Returns error | Default report plus presentation diagnostic. |
| Panics, including while formatting an error, with `std` | Catch and fall back. |
| Panics without `std` | Propagate. |

Fallback does not retry the adapter for that failure, although later assertions may reuse it. Calling an adapter directly
does not provide this fallback or require the unwind-safety bound used for panic presentation.
Regressions: [`a_presentation_error_falls_back_without_std`](../assertr-no-std-tests/src/lib.rs) and
[`a_panicking_error_formatter_preserves_the_original_failure`](../assertr/tests/failure_adapters.rs).

### Explicit output sinks

| `Writer<W>` operation | Behavior |
|---|---|
| `Adapter::adapt` (`std`) | Write all `AsRef<[u8]>` bytes to `std::io::Write`, then flush. Return `()` or an I/O error. Interior mutability allows a shared receiver. Reentrant calls return an error. |
| `adapt_async(&mut self, input)` (`tokio`) | Await writing and flushing to `AsyncWrite + Unpin`. Do not create a runtime, block the thread, or hold an interior borrow guard across await. |

Targets may be owned or borrowed. Stdout/stderr constructors perform no I/O. Input bytes are preserved without added
separators. Async errors or cancellation may leave partial output. Capture and panic presentation never write to streams
automatically. A sink returning `()` cannot serve as panic presentation.
