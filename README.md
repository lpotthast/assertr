# assertr

[![Crates.io](https://img.shields.io/crates/v/assertr.svg)](https://crates.io/crates/assertr)
[![Docs.rs](https://docs.rs/assertr/badge.svg)](https://docs.rs/assertr)
[![CI](https://github.com/lpotthast/assertr/actions/workflows/ci.yml/badge.svg)](https://github.com/lpotthast/assertr/actions/workflows/ci.yml)
[![MSRV](https://img.shields.io/badge/MSRV-1.89.0-blue.svg)](https://github.com/lpotthast/assertr/blob/main/assertr/Cargo.toml)
[![License: MIT OR Apache-2.0](https://img.shields.io/crates/l/assertr.svg)](#license)

Assertr is a fluent assertion library for Rust. Assertions are methods on the subject, so a chain reads as a statement
about one value and autocomplete lists only the assertions available for its type. A failure shows the subject, the
expectation, and the relation that did not hold. Assertr supports `std` and `no_std` builds.

```rust
use assertr::prelude::*;

assert_that!("hello, world!")
    .starts_with("hello")
    .ends_with("!");
```

Change the `"!"` to `"?"` and the test fails with:

```text
-------- assertr --------
Assertion failed at tests/greeting.rs:5:6

Expression: `"hello, world!"`

Actual: "hello, world!"

does not end with

Expected: "?"
-------- assertr --------
```

## Why a fluent API

The subject comes first. This distinguishes it from the expected value, and one chain replaces one `assert!` per
assertion:

```rust
let vec = vec![1, 2, 3];
assert_eq!(vec.len(), 3);
assert!(vec.contains(&2));
```

becomes

```rust
use assertr::prelude::*;

let vec = vec![1, 2, 3];
assert_that!(vec).has_length(3).contains(2);
```

## Installation

```toml
[dependencies]
assertr = "0.7.1"
```

The default features are `std` and `num`. Everything else is opt-in:

| feature                                                    | enables                                                                             |
|------------------------------------------------------------|-------------------------------------------------------------------------------------|
| `std`                                                      | Assertions for standard library types (`HashMap`, `Path`, `Command`, `Mutex`, ...). |
| `num`                                                      | Assertions for numeric types (`is_zero`, `is_positive`, `is_close_to`, ...).        |
| `libm`                                                     | Floating-point classifications for `num` assertions without `std`.                  |
| `fluent`                                                   | Fluent assertion entry points and aliases (`42.must().be_positive()`).              |
| `derive`                                                   | The `AssertrEq` derive macro for partial equality assertions.                       |
| `serde-json`                                               | `json()` and `as_json()` conversions.                                               |
| `serde-toml`                                               | `toml()` and `as_toml()` conversions.                                               |
| `serde`                                                    | Combined `serde-json` and `serde-toml`.                                             |
| `program`                                                  | Assertions that resolve an executable name or path.                                 |
| `http`, `jiff`, `reqwest`, `rootcause`, `tokio`            | Assertions for the types of the crate of the same name.                             |
| `full`                                                     | All of the above.                                                                   |

### no_std

Disable the default features. `derive`, `fluent`, `num`, `libm`, and `rootcause` support embedded `no_std` targets.
The `http` feature leaves Assertr in `no_std` mode but currently requires a hosted target through its dependencies.
Every other feature enables `std`. Add `libm` next to `num` if numeric assertions need floating-point
classifications. `libm` does not enable `num` by itself.

## Quick start

Import the prelude. It brings the enabled assertion traits into scope, so autocomplete lists the methods available for
the subject:

```rust
use assertr::prelude::*;

assert_that!("42".parse::<i32>()).is_ok_satisfying(|value| {
    value.is_greater_than(0).is_less_than(100);
});
```

`assert_that!(value)` borrows its input. Named values stay usable after the assertion, and temporaries live until the
end of the enclosing statement. The few assertions that consume their subject, such as `panics()` on a closure or
terminal iterator assertions, need `assert_that_owned!(value)`, which takes ownership instead:

```rust
use assertr::prelude::*;

assert_that_owned!((1..=3).map(|n| n * n)).contains_exactly([1, 4, 9]);
```

With the `fluent` feature, an assertion context can be entered from the value itself. `must()` panics on the first
failure, `verify(...)` collects the failures and returns them. Both borrow. The consuming variants are named
`must_owned()` and `verify_owned()`.

```rust
use assertr::prelude::*;

# #[cfg(feature = "fluent")]
# {
"hello, world!"
    .must()
    .start_with("hello")
    .end_with("!");

let failures = 3.verify(|it| it.be_equal_to(4));
assert_that!(failures).has_length(1);

let mut values = vec![1, 2, 3];
let reference = &mut values;
reference.must().contain(2).have_length(3);
reference.push(4);
# }
```

Fluent names follow fixed rules. `is_x` becomes `be_x`, `has_x` becomes `have_x`, other verbs become imperative
(`contains` -> `contain`), and negations put `not` first (`is_not_x` -> `not_be_x`). See
[`IntoAssertContext`](https://docs.rs/assertr/latest/assertr/trait.IntoAssertContext.html) for the complete rules.

## Finding assertions

Autocomplete on the subject is the fastest way. For a browsable reference, start with the
[assertion families](https://docs.rs/assertr/latest/assertr/assertions/index.html) on docs.rs. Each assertion trait page
is the authoritative list of its methods, signatures, and required bounds.

Blanket implementations make general assertions available to user-defined types. A `PartialEq` type has
`is_equal_to`, a `PartialOrd` type has `is_greater_than`, and a `HasLength` type has `has_length`.

## Guides

Failure reporting has three responsibilities:

- **Failure construction:** Every assertion builds an `AssertionFailure` containing structured evidence.
- **Failure handling:** Capture mode stores it. Panic mode asks a presentation adapter to produce the panic text.
- **Presentation:** An adapter converts the failure into another representation. `.then()` allows intermediate transformations.

The detailed material lives on the API item that owns it:

- **Capture mode**: collect failures as structured `AssertionFailure` values instead of panicking. See
  [`AssertThat::capture`](https://docs.rs/assertr/latest/assertr/struct.AssertThat.html#method.capture) and
  [`AssertionFailure`](https://docs.rs/assertr/latest/assertr/failure/struct.AssertionFailure.html).
- **Failure adapters**: inspect retained value trees or transform failures through typed, chainable adapters. See
  [`Rendered`](https://docs.rs/assertr/latest/assertr/renderer/struct.Rendered.html),
  [`Adapter`](https://docs.rs/assertr/latest/assertr/failure/adapter/trait.Adapter.html),
  [`AdapterExt::map_err`](https://docs.rs/assertr/latest/assertr/failure/adapter/trait.AdapterExt.html#method.map_err),
  and [`ToHumanReadableText`](https://docs.rs/assertr/latest/assertr/failure/adapter/struct.ToHumanReadableText.html).
  Select panic presentation with
  [`AssertThat::with_panic_presentation`](https://docs.rs/assertr/latest/assertr/struct.AssertThat.html#method.with_panic_presentation)
  for an owned `'static` adapter. It converts displayable errors to strings internally. Move or clone local data into
  the adapter, or share owned data through `Rc`. Derived assertions share the adapter without requiring `Clone`.
  Presentation returns `HumanReadableText` and defaults to `ToHumanReadableText`.
  Capture mode retains structured failures for explicit adapter processing.
- **Partial equality**: compare only some fields of a struct with `#[derive(AssertrEq)]`, including nested structs and
  collections of them. See [`AssertrEq`](https://docs.rs/assertr/latest/assertr/prelude/derive.AssertrEq.html).
- **Rendering values without `Debug`**: swap the renderer that failure messages use. See
  [`AssertThat::with_debug_format`](https://docs.rs/assertr/latest/assertr/struct.AssertThat.html#method.with_debug_format),
  [`AssertThat::with_renderer`](https://docs.rs/assertr/latest/assertr/struct.AssertThat.html#method.with_renderer),
  and [`ValueRenderer`](https://docs.rs/assertr/latest/assertr/renderer/trait.ValueRenderer.html). Type-specific
  structural assertions compose leaf renderers into collection, map, range, and wrapper syntax. Generic assertions
  that render the whole subject require a renderer for that subject.
- **Assertions for custom types**: define an assertion trait and implement it on `AssertThat`, either by composing
  existing assertions or by deciding the outcome yourself. See
  [custom assertions](https://docs.rs/assertr/latest/assertr/#custom-assertions).
- **Assertions on a part of the subject**: the projections
  [`AssertThat::derive`](https://docs.rs/assertr/latest/assertr/struct.AssertThat.html#method.derive) and
  [`AssertThat::satisfies`](https://docs.rs/assertr/latest/assertr/struct.AssertThat.html#method.satisfies). See
  [the core model](https://docs.rs/assertr/latest/assertr/#core-model).
- **Assertions about types**: `needs_drop`, type name, and size. See
  [`assert_that_type`](https://docs.rs/assertr/latest/assertr/fn.assert_that_type.html).

## API stability

Publicly exported items follow the usual Semantic Versioning rules unless their documentation explicitly says
otherwise. The `*Assertions` traits are public for method discovery, not as downstream implementation interfaces, so
adding a method to one of them is considered compatible. `assertr::__private` is the explicitly unsupported macro
plumbing and must not be named directly.

## MSRV

The minimum supported Rust version is `1.89.0` for both crates. Version history is recorded in the changelog.

## Contributing

Run `just install-tools` once, then `just verify` before submitting a pull request. Record notable changes under
`## [Unreleased]`, or under the latest version section if it has not been published yet.

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](https://github.com/lpotthast/assertr/blob/main/LICENSE-APACHE))
- MIT License ([LICENSE-MIT](https://github.com/lpotthast/assertr/blob/main/LICENSE-MIT))
