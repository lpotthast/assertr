---
id: failure-processing
refines:
  - assertr
depends_on:
  - diagnostic-rendering
related_to:
  - assertion-lifecycle
  - extension-contract
sources:
  - assertr/src/failure/mod.rs
  - assertr/src/failure/builder.rs
  - assertr/src/failure/failures.rs
  - assertr/src/renderer/rendered.rs
  - assertr/src/failure/adapter/**
  - assertr/src/failure/panic_presentation.rs
---

# Failure processing

[Architecture overview](README.md)

Assertion failures become structured data before capture storage or panic presentation. Adapters can inspect that data
without parsing the default report.

## Structured construction and ownership

After tracking a failed leaf, the assertion creates a `FailureBuilder` with `AssertThat::failure(FailureKind)`. Its
fields hold:

- Rendered actual, expected, and unexpected values, plus the relation between them.
- Facts, notes, nested failures, matcher constraints, and omission counts.
- A path, subject type, subject name, source expression, caller location, and detail messages.

`FailureKind` is non-exhaustive and classifies the assertion family for adapters. The relation and evidence explain the
specific failure.

An attached builder targets a chain. Calling `.raise()` collects its metadata and either stores the failure at the
capture root or presents it and panics. A detached builder creates a child failure with `.build()`. Typed field, tuple,
variant, index, and rendered-key paths locate nested evidence.

`AssertionFailure` is the public data model. Descendant failures reach the root in the order they are raised. Capture
returns them as `AssertionFailures`, an ordered aggregate supporting slice access and iteration.
See [assertion lifecycle](assertion-lifecycle.md#entry-subject-ownership-and-mode) for capture completion.

`AssertionFailure`, `Fact`, and `Rendered` provide read-only accessors alongside their public fields. Getters borrow
strings, slices, and rendered trees, or copy small metadata values. Optional trees use `Option<&Rendered>` without
cloning or rendering again. Pass getters returning a borrowed sized value to `derive`, and getters returning slices,
string slices, optional views, or copied values to `derive_owned`.

Adapters receive rendered values with [budgets already applied](diagnostic-rendering.md#bounded-retention). They can
change presentation but cannot recover omitted original values.

## Presentation and fallback

Capture leaves presentation to the caller. Panic mode uses the configured adapter or `ToHumanReadableText` by default.

If a custom panic adapter returns an error, Assertr uses the default report and appends a presentation diagnostic. With
`std`, it also catches an adapter panic, including a panic while formatting the adapter's error. It does not retry the
adapter after unwinding. Without `std`, returned errors still fall back, but adapter panics propagate.

This fallback applies only to panic presentation. Explicit adapter calls return their declared result directly.

## Sources

[FailureBuilder](../assertr/src/failure/builder.rs) implements attached and detached construction.
The [failure model](../assertr/src/failure/mod.rs) defines structured fields and root
storage. [AssertionFailures](../assertr/src/failure/failures.rs) provides aggregate
access. [Adapters](../assertr/src/failure/adapter/mod.rs) process failures,
and [panic presentation](../assertr/src/failure/panic_presentation.rs) handles fallback.
