---
id: assertion-lifecycle
refines:
  - assertr
depends_on:
  - failure-processing
  - diagnostic-rendering
related_to:
  - fluent-entry
  - integration-boundaries
sources:
  - assertr/src/lib.rs
  - assertr/src/actual.rs
  - assertr/src/mode.rs
  - assertr/src/entry/mod.rs
  - assertr/src/assert_that/capture.rs
  - assertr/src/assert_that/projection.rs
  - assertr/src/tracking.rs
  - assertr/src/assertions/core/fn.rs
  - assertr/src/entry/panic.rs
---

# Assertion lifecycle

[Architecture overview](README.md)

An assertion chain combines the subject, a failure mode, and diagnostic state. Mapping carries that state into a new
subject. Derivation creates a child that reports back to its parent.

## Chain representation

`AssertThat<'t, T, M, R>` contains an `Actual<'t, T>` and a private `ChainState<'t, M, R>`.

| Part         | Purpose                                                                                              |
|--------------|------------------------------------------------------------------------------------------------------|
| `T`          | Selects the available assertion methods.                                                             |
| `Actual`     | Stores either `Borrowed(&T)` or `Owned(T)`. Ownership is checked at runtime by consuming methods.    |
| `M: Mode`    | Selects panic or capture behavior at compile time. `Mode` is sealed to `Panic` and `Capture`.        |
| `R`          | Stores the renderer. Each assertion method requires only the rendering capabilities it uses.         |
| `ChainState` | Holds metadata, rendering settings, assertion count, captured failures, and an optional parent link. |

The parent link is a borrowed, private `DynAssertThat` trait object. A child forwards failures and assertion counts
through this link and reads ancestor detail messages when building a failure. Counters, messages, and failure storage
use `RefCell` so assertions on borrowed children can update their root.

## Entry, subject ownership, and mode

`assert_that!(expression)` borrows its input and records the expression text. For sized pointees, a value and one
reference layer produce the same subject type. Unsized strings and slices use reference-typed subjects.
`assert_that_owned!` takes ownership. A consuming assertion called on a borrowed subject panics as misuse when it tries
to take the value. Ownership is not a separate type parameter that could reject the call at compile time.

Entry normally creates a panic-mode root. `capture` starts a capture-mode root for its closure, resets the assertion
count, and severs any panic-mode parent link. It retains existing detail messages by collecting them from the old chain
and its ancestors.

The closure must return the supplied chain or a mapped continuation. Capture checks the returned chain's count and takes
its failures. An empty assertion count panics as misuse. Capture does not catch user panics or invoke panic
presentation. Completion happens when the closure returns, with no check deferred to `Drop`.

## Projections and continuation

| Operation                                       | Chain state                                                          | Subject name and expression                    | Renderer                      |
|-------------------------------------------------|----------------------------------------------------------------------|------------------------------------------------|-------------------------------|
| `map`, `map_owned`, `map_async`                 | Moves the existing state into the continuation.                      | Preserved.                                     | Moved, with no `Clone` bound. |
| `derive`, `derive_owned`, `derive_async` | Creates a child borrowing the parent. | Start empty. | Cloned. |
| `satisfies`, `satisfies_owned`, `satisfies_ref` | Gives a derived child to a closure, then returns the original chain. | Empty on the child. Unchanged on the original. | Cloned for the child.         |

`map` receives `Actual<T>` and returns `Actual<U>`. `map_owned` first makes a `ToOwned` copy of the subject. `map_async`
awaits a new owned subject. `derive` borrows its projection. Use it to project a field and then assert on the child,
while keeping the parent available for other field checks. `derive_owned` and `derive_async` store the mapper's result
as the child subject. They borrow the parent without cloning it. Derived children inherit the mode, rendering budget,
location setting, and panic presentation. Projection itself does not count as an assertion. See the
[`derive` rustdoc](https://docs.rs/assertr/latest/assertr/struct.AssertThat.html#method.derive) for field-check examples.

This capture collects a child failure, then returns a mapped continuation with the original metadata:

```rust
use assertr::{actual::Actual, prelude::*};

let values = [1, 2];
let failures = assert_that!(values)
    .with_subject_name("values")
    .capture(|root| {
        root.derive(|values| &values[0]).is_equal_to(9);
        root.map(|actual| Actual::Owned(actual.borrowed().len()))
            .is_equal_to(3)
    });

assert_eq!(failures.len(), 2);
assert_eq!(failures[0].subject_name, None);
assert_eq!(failures[0].expression, None);
assert_eq!(failures[1].subject_name.as_deref(), Some("values"));
assert_eq!(failures[1].expression, Some("values"));
```

The root receives the child's failure through its parent link. Returning the mapped root lets capture extract both
failures. A derived child borrows its parent and cannot replace that local parent as the closure's returned chain.

## Panic observation boundaries

Synchronous and asynchronous `FnOnce` assertions require ownership and panic mode. `panics` observes invocation and
drops a produced output inside the unwind boundary. A panic from either phase becomes the next `PanicValue` subject.
`does_not_panic` returns the output, so its later `Drop` is outside that boundary.

The async variants also observe each poll. After a poll panics, the future is not polled again. `panics_async` observes
dropping the output as well. External cancellation of a pending projection or panic-checking future has no recovery
guarantee.

## Sources

[Chain representation](../assertr/src/lib.rs), [Actual](../assertr/src/actual.rs), and [Mode](../assertr/src/mode.rs)
define the state
model. [Entry](../assertr/src/entry/mod.rs), [capture](../assertr/src/assert_that/capture.rs), [projections](../assertr/src/assert_that/projection.rs),
and [tracking](../assertr/src/tracking.rs) implement its
lifecycle. [Function assertions](../assertr/src/assertions/core/fn.rs) define panic observation.
See [failure processing](failure-processing.md) for raising failures and [diagnostic rendering](diagnostic-rendering.md)
for renderer behavior.
