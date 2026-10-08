---
id: assertion-lifecycle
depends_on: [ ]
sources:
  - assertr/src/lib.rs
  - assertr/src/crate_docs.md
  - assertr/src/actual.rs
  - assertr/src/mode.rs
  - assertr/src/entry/mod.rs
  - assertr/src/assert_that/mod.rs
  - assertr/src/assert_that/capture.rs
  - assertr/src/assert_that/projection.rs
  - assertr/src/details.rs
  - assertr/src/tracking.rs
  - assertr/src/conversion.rs
  - assertr/src/assertions/program.rs
---

# Assertion lifecycle

[Architecture overview](README.md)

An assertion chain owns or borrows a subject. Mapping transfers its state to a new subject. Derivation creates a child
that reports to its parent.

## Chain representation

| Part of `AssertThat<'t, T, M, R>` | Purpose |
|---|---|
| `Actual<'t, T>` | `Borrowed(&T)` or `Owned(T)`. `actual()` returns `&T`. |
| `M: Mode` | Compile-time failure handling, sealed to `Panic` and `Capture`. |
| `R` | Active renderer. Methods require only the rendering capabilities they use. |
| `ChainState` (private) | Mode, renderer, diagnostic settings, subject name, expression, records. Independent of subject type. |
| `ChainRecords` (private) | Local messages, assertion count, captured failures, optional parent-record link. |

A root has no parent link. A child's link gives it access to ancestor records, but not to ancestor subjects, renderers,
or panic presentations. The child's subject may also borrow a value projected from its parent. That borrow has its
own lifetime and unwind-safety requirements, separate from the record link.

Counts propagate through every ancestor. Captured failures reach the root in the order they are raised. Diagnostics
collect local messages before ancestor messages.

## Entry, subject ownership, and mode

| Entry | Subject storage |
|---|---|
| `assert_that!` | Borrows and records expression text. Values and one reference layer select the same subject type for sized pointees. Unsized strings and slices remain reference-typed subjects. |
| `assert_that_owned!` | Owns the input. Given `&T`, owns that reference. |

Ownership is a runtime property of `Actual`, independent of mode. `Panic` presents and panics on rejection. `Capture`
stores failures and continues. Whether a method supports capture depends on
[what subject it can return after failure](#continuation-availability).

`capture` converts a panic-mode chain into a capture root:

1. Reset the count, detach parent records, retain inherited messages and diagnostic settings. Inherited messages keep
   following every local message, including those added in the callback.
2. Run the synchronous callback. It must return the supplied root or a mapped continuation, after checking any children.
3. Check the returned chain's count and take its failures. Zero assertions causes a misuse panic.

Capture reads only the returned chain. Returning an unrelated chain loses the original root's results. A derived child
borrows the local root and cannot replace it as the callback result. Capture neither catches user panics nor invokes
panic presentation. Dropping a chain does not check whether capture completed.
Regression: [`returned_context_collects_projections_and_renderer_changes_once`](../assertr/src/assert_that/capture.rs).

## Projections and continuation

| Operation | State and metadata | Renderer |
|---|---|---|
| `map`, `map_owned`, `map_async` | Move existing state, including records, name, expression. | Moved, no `Clone` bound. |
| `derive`, `derive_owned`, `derive_async` | Create a child with inherited settings, mode, ancestor messages. Clear name and expression. | Cloned. |
| `satisfies`, `satisfies_owned`, `satisfies_ref` | Check a derived child in a callback, then return the original chain. | Cloned for child. |

- `map` transforms `Actual<T>` into `Actual<U>`. `map_owned` first copies through `ToOwned`, even for owned input.
  `map_async` awaits a new owned subject.
- `derive` borrows a sized projection. `derive_owned` and `derive_async` store the mapper's result, which may itself
  reference an unsized target. Derivation does not clone the subject.

### Continuation availability

| Operation | Failure and continuation |
|---|---|
| Retaining check | Can support both modes because the original subject remains available. |
| Extraction that cannot provide a subject after failure | Requires `Panic`, enforced by method bounds. |
| Taking a borrowed `Actual` | Runtime misuse panic, independent of mode. |
| JSON/TOML conversion | Serialize the borrowed subject once, map to owned `Result<String, Error>` in either mode. Errors remain `Err` for later checks. This transformation counts no assertion. |

For example, program existence retains its subject, while resolved-path extraction promises a `PathBuf`. Type checks and
extractions on erased boxes, panic payloads, and rootcause contexts use the same distinction. Owned payloads can transfer
ownership. Borrowed payloads remain borrowed.

### Async constraints

Chains are neither `Send` nor `Sync`: their state contains an `Rc` for panic presentation and interior-mutable records.
A future retaining a chain across suspension cannot be `Send`. Async projections can be awaited locally. For a `Send` task,
construct and finish the chain after required awaits. Capture callbacks return chains, never futures.
The [async compile-fail examples](../assertr/src/crate_docs.md#async-limitations) demonstrate these limits.
See [invocation and polling](observation-boundaries.md#invocation-and-polling) for operation timing and cancellation.

## Unwind safety

| Chain auto trait | Subject bounds | Renderer bound |
|---|---|---|
| `UnwindSafe` | `T: UnwindSafe + RefUnwindSafe` | `R: UnwindSafe` |
| `RefUnwindSafe` | `T: RefUnwindSafe` | `R: RefUnwindSafe` |

These bounds apply in both modes and both storage variants. Construction, projection, and callbacks add no unwind
bounds. Only the message, count, and failure cells have local `AssertUnwindSafe` exemptions. The count is a
`Cell<usize>`. Inherited messages are a plain `Vec<String>` that is never mutated after construction and needs no
exemption. Conversions and rendering finish before records are borrowed mutably. Unwinding releases guards without
undoing records or user effects. Subjects, renderers, and panic presentations remain outside those exemptions.

[Panic-catching execution adapters](observation-boundaries.md#invocation-and-polling) use localized exemptions for
mutable captures. They do not make arbitrary chains unwind-safe or roll back user state.
