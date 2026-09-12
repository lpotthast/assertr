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
  - assertr/tests/failure_adapters.rs
  - assertr-no-std-tests/src/lib.rs
---

# Failure processing

[Architecture overview](README.md)

A failed assertion becomes an owned `AssertionFailure` before capture storage or panic presentation. Failure adapters
inspect that structure without parsing the default report or retaining the original Rust values.

## Structured construction and ownership

The [failure model](../assertr/src/failure/mod.rs) separates meaning, evidence, and location:

| Fields                                         | Meaning                                                                                                                                    |
|------------------------------------------------|--------------------------------------------------------------------------------------------------------------------------------------------|
| `kind`                                         | Non-exhaustive family classification for grouping and filtering. It does not identify a specific assertion.                                |
| `actual`, `relation`, `expected`, `unexpected` | Rendered operands and the sentence connecting them. Relations are lowercase, contain no values, and have no trailing period.               |
| `facts`                                        | Additional evidence. `Fact::labelled` names it. `Fact::note` uses an empty label.                                                          |
| `children`, `constraint`, omission counts      | Nested rejections and descriptions of unmet expectations. A constraint is another `AssertionFailure`, not a separate schema.               |
| `path` and subject/caller metadata             | Relative field, tuple, variant, stable index, or rendered-key location, plus type, name, expression, caller location, and detail messages. |

[`FailureBuilder`](../assertr/src/failure/builder.rs) has two completion targets. `AssertThat::failure` creates an
`Attached` builder. `.raise()` adds chain metadata and delivers the failure through the active mode.
`FailureBuilder::detached::<T>` creates a `Detached` builder. `.build()` returns data without raising or collecting
chain metadata. [Expectation explanation](expectation-execution.md#evaluation-and-explanation) uses either target.

Diagnostic values pass through the active rendering context before entering the builder. Failure adapters receive
[budgeted `Rendered` trees](diagnostic-rendering.md#bounded-retention) and cannot recover omitted values.
`AssertionFailure`, `Fact`, and `Rendered` expose fields and read-only accessors for inspection without rerendering.

Capture forwards descendant failures to the root in raise order. [
`AssertionFailures`](../assertr/src/failure/failures.rs)
is the ordered aggregate returned on [capture completion](assertion-lifecycle.md#entry-subject-ownership-and-mode). It
supports slice access, iteration, and conversion to a vector. Fluent expression bookkeeping stays private and never
appears as a placeholder in public fields. Its attachment rules live
in [fluent entry](fluent-entry.md#scoped-expression-capture).

## Presentation and fallback

[`Adapter<Input>`](../assertr/src/failure/adapter/mod.rs) converts borrowed input to a declared output or error.
`AdapterExt::then` composes conversions. `ToHumanReadableText` renders a failure or aggregate using the default report
grammar and returns `HumanReadableText`, an owned text wrapper.

With `std`, `Writer<W>` is an explicit sink accepting `AsRef<[u8]>`, including `HumanReadableText`, strings, and
byte buffers. Its `Adapter` implementation writes the complete input to a configured `std::io::Write` target and
flushes it, returning `()` or the I/O error. It adds no separators and preserves the input bytes. The target may be
owned or borrowed. Constructors select standard output or standard error without performing I/O. The writer uses
interior mutability for the shared `Adapter::adapt` receiver. Reentrant use returns an I/O error.

With `tokio`, the same `Writer` accepts `AsyncWrite + Unpin` targets through `adapt_async(&mut self, input)`.
Tokio stdout/stderr constructors are available. Callers await writing and flushing explicitly after any synchronous
adapter stages. This method never creates or blocks on a runtime and holds no interior borrow guard across an await.
Errors and cancellation can leave partial output. Neither capture nor panic presentation automatically writes to a
stream, and a sink's `()` output cannot be installed as panic presentation.

Capture leaves adaptation to the caller. Panic mode uses `ToHumanReadableText` unless `with_panic_presentation` installs
an owned `'static + RefUnwindSafe` text adapter with a `Display` error. Derived chains share it through `Rc`. It needs
neither `Send`, `Sync`, nor `Clone`. The private `PanicPresentation` trait object retains the unwind-safety bound.

[Panic presentation](../assertr/src/failure/panic_presentation.rs) falls back to the default report and appends a
presentation diagnostic when the adapter returns an error. With `std`, it also catches adapter panics, including error
formatting panics. Without `std`, those panics propagate. Fallback does not retry the adapter, though a later assertion
may use it again. Explicit adapter calls have neither this fallback nor the presentation-specific unwind bound.

The regressions [`a_presentation_error_falls_back_without_std`](../assertr-no-std-tests/src/lib.rs) and
[`a_panicking_error_formatter_preserves_the_original_failure`](../assertr/tests/failure_adapters.rs) pin these two
boundaries.
