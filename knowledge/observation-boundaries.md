---
id: observation-boundaries
depends_on: [ expectation-execution ]
sources:
  - assertr/src/assert_that/execution.rs
  - assertr/src/assertions/core/fn.rs
  - assertr/src/assertions/core/pattern.rs
  - assertr/src/entry/panic.rs
  - assertr/src/assertions/iterator/mod.rs
  - assertr/src/assertions/iterator/tests.rs
  - assertr/src/assertions/std/mutex.rs
  - assertr/src/assertions/std/path.rs
  - assertr/src/assertions/tokio/mutex.rs
  - assertr/src/assertions/tokio/rw_lock.rs
  - assertr/src/assertions/tokio/watch.rs
  - assertr/src/assertions/program.rs
  - assertr/src/assertions/reqwest/response.rs
  - assertr/src/assertions/rootcause/report.rs
  - assertr/src/assertions/alloc/boxed.rs
  - assertr/src/assertions/alloc/panic_value.rs
  - assertr/src/conversion.rs
---

# Observation and consumption boundaries

[Architecture overview](README.md)

An execution adapter connects an operation to the shared expectation executor. It owns invocation, polling, traversal,
or ownership transfer and supplies the resulting observation for evaluation and explanation. It is distinct from a
failure `Adapter` and from a rendering adapter. Internal observation types and their definitions stay private.

The private `test_once_after_tracking` executor accepts an `FnOnce` observation and an `FnOnce` explanation. The adapter
tracks before invoking user code and supplies its captured caller location. The executor constructs the context and
failure builder, then routes a rejection through the active mode. Reusable expectations use this same execution path
without changing their borrowed `evaluate` protocol. Consuming scans and ordinary pattern guards do not implement a
reusable expectation by hiding a one-use value in interior mutable storage.

Ordinary pattern assertions consume their guard closure once. Its temporary captures are released when observation
returns, before rendering the subject or continuing the chain. Reusable pattern matchers still require `Fn` guards.

## Retaining checks versus extraction

A retaining check can support both modes. An extraction that cannot produce the promised subject after rejection
requires panic mode. Rust enforces the method's mode bounds. Consuming a borrowed `Actual` instead fails at runtime as
misuse. Success follows the ordinary [mapping or derivation rules](assertion-lifecycle.md#projections-and-continuation).

For example, [program existence](../assertr/src/assertions/program.rs) retains its subject, while resolved-path
extraction returns an owned `PathBuf`. [Erased boxes](../assertr/src/assertions/alloc/boxed.rs),
[panic payloads](../assertr/src/assertions/alloc/panic_value.rs), and
[rootcause contexts](../assertr/src/assertions/rootcause/report.rs) similarly separate retaining type checks from
extraction. An owned payload can transfer ownership. A borrowed payload remains borrowed.

[JSON/TOML conversion](../assertr/src/conversion.rs) serializes the borrowed subject once and maps to an owned
`Result<String, Error>` in either mode. It counts no assertion. Errors remain `Err` for result assertions. This is a
transformation whose continuation exists even when serialization fails.

## Invocation and polling

[Function assertions](../assertr/src/assertions/core/fn.rs) require ownership and panic mode. `panics` catches
invocation and dropping a produced output. `does_not_panic` returns the output, so its later drop is outside the catch
boundary. Async variants also catch poll panics and never repoll a panicked future. `panics_async` also observes output
drop.

Tracking and caller capture have operation-specific timing:

| Execution adapter              | Caller location                                | Assertion count and effects                                                                  |
|--------------------------------|------------------------------------------------|----------------------------------------------------------------------------------------------|
| Synchronous function assertion | Captured at the call.                          | Tracks before invoking the function.                                                         |
| Async function assertion       | Captured when the method is called.            | Tracks and invokes the function when the returned future is first polled.                    |
| Reqwest body extraction        | Captured at the call and carried across await. | Tracks and rejects borrowed ownership before returning a future. Reading occurs when polled. |

Retained invocation results reach the shared executor before output or panic-payload transfer. Explanation cannot invoke
or poll the function again. Localized `AssertUnwindSafe` permits mutable captures without restoring their state. The
resulting `PanicValue` contains `Box<dyn Any>` and has neither unwind-safety trait. These exemptions do not change the
[chain's unwind-safety bounds](assertion-lifecycle.md#unwind-safety). Cancellation of pending operations has no recovery
guarantee.

## Filesystem existence observations

[Path absence checks](../assertr/src/assertions/std/path.rs) inspect `Path::try_exists` once. `Ok(false)`
establishes absence and passes `does_not_exist` or `DoesNotExist`. `Ok(true)` rejects with evidence that the path
unexpectedly exists. An inspection error also rejects, with a relation stating that existence could not be determined
and the original I/O error rendered as a labelled fact. Explanation consumes the retained observation without inspecting
the filesystem again. The public rejection type is opaque, with private variants distinguishing the two failure cases.

Evaluation requires no renderer capability. The ordinary method and diagnostic implementation require both the path
subject's renderer and `ValueRenderer<std::io::Error>`, so custom rendering and the leaf budget apply to error evidence.
The `does_not_exist::observations` tests pin the three outcomes directly. Invalid-path tests exercise filesystem
inspection errors without depending on permissions or the process's privileges.

## Guarded observations

Execution must keep a guard long enough to render the observed value and release it before raising or evaluating a
sibling. The [guarded rejection trace](expectation-execution.md#guarded-rejection-trace) explains the handoff to owned
evidence. Successful observations returned by `test_assertion` instead remain under their caller's control.

[Standard mutex checks](../assertr/src/assertions/std/mutex.rs) use `try_lock`. Success and an acquirable poisoned guard
mean unlocked. `WouldBlock` means locked. Poison state has separate assertions. A failing `is_locked` renders through
the acquired guard, then releases it so the assertion's raised panic does not itself poison the mutex.

[Tokio mutex value callbacks](../assertr/src/assertions/tokio/mutex.rs) run nested capture only after immediate
acquisition. Contention fails the check without invoking the callback.
[Tokio RwLock checks](../assertr/src/assertions/tokio/rw_lock.rs) try write acquisition, then read acquisition if
needed. Write success means unlocked, read-only success means read-locked, and both failing means write-locked. Queued
waiters, reader limits, and concurrent changes affect these observations. They are not synchronized guard counts.
Without acquisition, diagnostics mark the value unavailable.

[Watch receiver checks](../assertr/src/assertions/tokio/watch.rs) borrow the current value without marking it seen.
Ordinary `has_changed` and `has_not_changed` remain panic-only despite retaining the receiver. Their execution observes
`has_changed` once and treats channel closure as failure.

## Awaiting and consuming a response

[Reqwest body extraction](../assertr/src/assertions/reqwest/response.rs) consumes an owned response in panic mode. Read
failures retain the URL and read error. JSON decoding follows a successful text read. Decode failures retain that text,
URL, expected type, and original parser error. Successful continuations carry no temporary error details. Reading and
JSON decoding together count as one `get_json` assertion. Partial consumption has no response-recovery guarantee.

Header extraction only checks presence and continues on a clone of the first header value. Header evidence follows the
[renderer sensitivity policy](diagnostic-rendering.md#sensitive-http-header-evidence). The regression
[`panics_synchronously_when_the_response_is_only_borrowed`](../assertr/src/assertions/reqwest/response.rs)
pins the ownership check before a body-extraction future is returned.

## Traversal

Streaming execution owns one iterator for one scan. Retained observations explain decisions without repeating `next`
or `size_hint`. Expected sequence views convert after tracking and before scanning. The iterator remains alive through
explanation. Every private `Scan` observes through `&mut I`. The execution adapter returns the owning iterator with
rejection evidence, renders that evidence into owned diagnostic values, and drops the iterator before failure routing.
This also applies when scanning exits before exhaustion or the iterator owns a guard. Successful scans drop their
iterator before returning. Neither the iterator nor its items require `Clone` or repeatable traversal.

The [streaming regressions](../assertr/src/assertions/iterator/tests.rs) check resource-dependent rendering for every
family, direct and borrowed adapters, early exits, exhaustion, `next` and `size_hint` counts, and guard release before
panic presentation. Cardinality uses the same ownership boundary.
[Collection and iterator semantics](collection-semantics.md#borrowed-traversal-versus-terminal-streams) owns stopping
conditions, preview limits, and the distinction between borrowed traversal and terminal consumption.
