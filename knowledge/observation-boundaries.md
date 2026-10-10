---
id: observation-boundaries
depends_on: [ expectation-execution ]
sources:
  - assertr/src/assert_that/execution.rs
  - assertr/src/crate_docs.md
  - assertr/src/assertions/alloc/boxed.rs
  - assertr/src/assertions/core/fn.rs
  - assertr/src/assertions/core/pattern.rs
  - assertr/src/entry/panic.rs
  - assertr/src/assertions/std/mutex.rs
  - assertr/src/assertions/std/path.rs
  - assertr/src/assertions/std/eventually/mod.rs
  - assertr/src/assert_that/detached.rs
  - assertr/src/assertions/tokio/mutex.rs
  - assertr/src/assertions/tokio/rw_lock.rs
  - assertr/src/assertions/tokio/watch.rs
  - assertr/src/assertions/reqwest/response.rs
---

# Observation boundaries

[Architecture overview](README.md)

Some assertions invoke user code, acquire guards, inspect external state, or consume input. Their diagnostics must
describe what the assertion observed without performing the operation again. Iterator scans are covered in
[iterator execution](iterator-execution.md). [Assertion lifecycle](assertion-lifecycle.md#continuation-availability)
explains which operations require ownership or panic mode.

## Execution adapters

Execution adapters invoke functions, poll futures, or consume input before handing the result to the
[shared executor](expectation-execution.md#chain-execution). They track the assertion at the times described below and
preserve its caller location. Internal observation types stay private.
Reusable expectation hooks remain synchronous. Do not hide one-use values in interior mutability to simulate reusability.

Built-in adapters use executor methods that skip tracking. Downstream adapters cannot access those private methods.
If an operation cannot use public expectation execution, its adapter tracks explicitly, builds a
[failure](failure-processing.md#builder-completion) with `self.failure(..)` and `self.render()`, and passes it to
`self.raise(..)`.

Pattern assertions and reusable pattern matchers require `Fn` guards and run through ordinary expectation execution.

## Invocation and polling

Function assertions require ownership and panic mode:

| Assertion | Caught operations |
|---|---|
| `panics` | Catch panics while invoking the function or dropping its output. |
| `does_not_panic` | Catch panics while invoking the function. A later panic from dropping the returned output is not caught. |
| Async variants | Also catch panics while polling. Never poll a panicked future again. `panics_async` also catches panics from dropping the output. |

### Tracking and caller timing

| Adapter | Caller captured | Tracking and operation |
|---|---|---|
| Synchronous function assertion | At method call. | Track before invocation. |
| Async function assertion | At method call. | Track and invoke on first poll. |
| Reqwest body extraction | At method call, retained across await. | Track and reject borrowed responses before returning the future. Read the body when polled. |

The executor receives the retained result before the output or panic payload is transferred. Explanation never invokes
the function or polls the future again. Localized `AssertUnwindSafe` permits mutable captures without restoring state.
`PanicValue` contains `Box<dyn Any + Send>` and has neither unwind-safety trait. These exemptions do not change
[chain bounds](assertion-lifecycle.md#unwind-safety). Cancelling a pending operation does not guarantee recovery of its
input or state.

Async assertions are adapters awaited in the calling task, under the chain's
[async constraints](assertion-lifecycle.md#async-constraints).
The [function tests](../assertr/src/assertions/core/fn.rs), including
`invocation_is_lazy_and_panicked_futures_are_never_repolled` and caller-location pins, verify timing, not cancellation safety.

## Eventual observations

`EventualAssertions` take an observation, a closure returning a future of the current value, and observe it
repeatedly in panic mode. Their builders configure the [patience](glossary.md#execution-and-presentation). The final
`matches` or `satisfies` call is the assertion:

| Assertion | Passes | Fails |
|---|---|---|
| `eventually` | At the first observation meeting the expectation. Continues with that value. | When the last observation at or after the timeout does not. |
| `consistently` | When every observation until the consistency duration ends meets it. Continues with the last value. | At the first observation that does not. |
| `_ok` variants | Same, for an observation returning `Result`: `eventually_ok` retries an `Err`, `consistently_ok` fails on it. | An `Err` fails with relation "could not be observed" and an `Error` fact. |

The final call captures the caller location, tracks one assertion on the chain and its ancestors, resolves the patience
(global patience with the chain's overrides), and [detaches](glossary.md#chain-state) the chain: ancestor
messages are collected as in `capture`, and the future keeps only the observation, the diagnostic settings, and the
renderer. It is therefore `Send` whenever they are, unlike other async adapters. The continuation is a new root chain on
the observed value. Panic mode raises immediately, so it needs neither the records nor the parent link.

Expectations are evaluated on every observation, but explained only for the failing one. Each observation is rendered
for the history. A failure is the expectation's own failure plus a `Waited` or `Held` fact (duration and number of
observations) and, when the value changed, `Observed values`: up to eight distinct values with their offsets. Pauses
use a private timer thread rather than a runtime's timer, so any executor can await them. Cancelling the future stops
observing. [Eventual tests](../assertr/src/assertions/std/eventually/mod.rs) cover timing, reports, messages, and `Send`.

## Filesystem existence observations

`exists`, `does_not_exist`, and their expectations `Exists` and `DoesNotExist` call `Path::try_exists` once. A
`NotFound` or `NotADirectory` error confirms absence, because the latter means that an ancestor is not a directory.

| Observation | `exists` | `does_not_exist` |
|---|---|---|
| `Ok(true)` | Pass. | Reject with unexpected existence. |
| `Ok(false)`, `NotFound`, or `NotADirectory` | Reject as absent. | Pass. |
| Any other `Err(error)` | Reject because existence is undetermined. Retain the original error as an `I/O error` fact. | Same as `exists`. |

`IsAFile`, `IsADirectory`, and `IsASymlink` read metadata once, following symbolic links except for `IsASymlink`. They
retain the observed entry kind or the metadata error. `NotFound` and `NotADirectory` report the path as absent. Any
other error reports an inspection failure with an `I/O error` fact.

Rejections retain the raw observation: `io::Result<bool>` for existence and `io::Result<FileType>` for entry kinds.
Explanation classifies it and never inspects the filesystem again. Diagnostics require the path-subject renderer and
`ValueRenderer<std::io::Error>`, honoring leaf budgets. [Path tests](../assertr/src/assertions/std/path.rs) cover the
outcomes above. Invalid-path cases avoid privilege-dependent permission failures.

## Guarded observations

When diagnostics need a guarded value, these adapters keep the guard alive until that value has been rendered. They
follow the shared [observation lifetime rules](expectation-execution.md#child-scopes-and-evidence), but differ in how they
acquire guards:

| Subject | Observation contract |
|---|---|
| Standard mutex | `try_lock` success, including acquisition of a poisoned guard, means unlocked. `WouldBlock` means locked. Poison checks are separate. A failing `is_locked` renders through the guard, then releases it before raising to avoid poisoning the mutex. |
| Tokio mutex callback | Try immediate acquisition. If the lock is held, reject without invoking the callback. Otherwise, `has_value_satisfying` runs its `FnOnce` callback in the chain's mode while holding the guard, like `is_some_satisfying`. The reusable `Fn` form `HasValueSatisfying` runs the callback in nested capture and reports its failures as children. |
| Tokio RwLock | Try a write lock, then a read lock if needed. Write success means unlocked. Read-only success means read-locked. Neither succeeding means write-locked. Values that cannot be acquired are shown as unavailable. |
| Watch receiver | Borrow the current value without marking it seen. `has_changed` and `has_not_changed` observe once, preserve the value and seen state, reject a closed channel, and continue on the receiver in capture mode. |

These RwLock checks report what could be acquired at that moment. Queued waiters, reader limits, and concurrent changes
can affect the result, so it does not establish how many guards are held.

## Awaiting and consuming a response

Reqwest body extraction consumes an owned response in panic mode:

| Outcome | Retained failure evidence |
|---|---|
| Body-read failure | URL and original read error. |
| JSON-decode failure after successful text read | Text, URL, expected type, original parser error. |
| Success | No temporary error details on continuation. |

Reading and decoding count as one `get_json` assertion. A partially consumed response cannot be recovered. Header
extraction checks presence and continues on a clone of the first value. Header diagnostics follow the [sensitive header
rules](diagnostic-rendering.md#sensitive-http-header-evidence). Regression:
[`panics_synchronously_when_the_response_is_only_borrowed`](../assertr/src/assertions/reqwest/response.rs).
