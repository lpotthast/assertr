<!-- Generated from the landing-page rustdoc in assertr/src/lib.rs. Edit that source and run `just readme`. -->

<!-- cargo-rdme start -->

# assertr

[![Crates.io](https://img.shields.io/crates/v/assertr.svg)](https://crates.io/crates/assertr)
[![Docs.rs](https://docs.rs/assertr/badge.svg)](https://docs.rs/assertr)
[![CI](https://github.com/lpotthast/assertr/actions/workflows/ci.yml/badge.svg)](https://github.com/lpotthast/assertr/actions/workflows/ci.yml)
[![MSRV](https://img.shields.io/badge/MSRV-1.89.0-blue.svg)](https://github.com/lpotthast/assertr/blob/main/assertr/Cargo.toml)
[![License: MIT OR Apache-2.0](https://img.shields.io/crates/l/assertr.svg)](#license)

Assertr is a fluent assertion library for Rust. Assertions are methods on the subject, so a
chain reads as a statement about one value and autocomplete lists only the assertions available
for its type. A failure shows the subject, the expectation, and the relation that did not hold.
Assertr supports `std` and `no_std` builds.

```rust
use assertr::prelude::*;

assert_that!("hello, world!")
    .starts_with("hello")
    .ends_with("!");
```

Match only the struct fields that matter with `partial!`. Enable the `matchers` feature for this
example:

```rust
use assertr::prelude::*;

struct User {
    name: &'static str,
    age: u32,
}

let user = User { name: "Alice", age: 30 };
assert_that!(user).matches(partial!(User { name: "Alice", .. }));
```

Here `..` ignores the remaining fields. The struct needs no derives or annotations. Field
expectations can also use constraints, existing assertion methods, and nested partial matches.
See the [partial matching guide](https://docs.rs/assertr/latest/assertr/matchers/index.html).

Changing `"!"` to `"?"` in the greeting assertion above produces:

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

The subject comes first. This distinguishes it from the expected value, and one chain replaces
one `assert!` per assertion:

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
| `matchers`                                                 | The `partial!` macro for structural matching. Runtime matchers need no feature.     |
| `serde-json`                                               | `as_json()` serializes to a JSON `Result` subject.                                               |
| `serde-toml`                                               | `as_toml()` serializes to a TOML `Result` subject.                                               |
| `serde`                                                    | Combined `serde-json` and `serde-toml`.                                             |
| `program`                                                  | Assertions that resolve an executable name or path.                                 |
| `http`, `jiff`, `reqwest`, `rootcause`, `tokio`            | Assertions for the types of the crate of the same name.                             |
| `full`                                                     | All of the above.                                                                   |

### no_std

Disable the default features. `matchers`, `fluent`, `num`, `libm`, and `rootcause` support
embedded `no_std` targets. The `http` feature leaves Assertr in `no_std` mode but currently
requires a hosted target through its dependencies. Every other feature enables `std`. Add `libm`
next to `num` if numeric assertions need floating-point classifications. `libm` does not enable
`num` by itself.

## Quick start

Import the prelude. It brings the enabled assertion traits into scope, so autocomplete lists the
methods available for the subject:

```rust
use assertr::prelude::*;

assert_that!("42".parse::<i32>()).is_ok_satisfying(|value| {
    value.is_greater_than(0).is_less_than(100);
});
```

`assert_that!(value)` borrows its input. Named values stay usable after the assertion, and
temporaries live until the end of the enclosing statement. The few assertions that consume their
subject, such as `panics()` on a closure or terminal iterator assertions, need
`assert_that_owned!(value)`, which takes ownership instead:

```rust
use assertr::prelude::*;

assert_that_owned!((1..=3).map(|n| n * n)).contains_exactly([1, 4, 9]);
```

With the `fluent` feature, an assertion context can be entered from the value itself. `must()`
panics on the first failure, `verify(...)` collects the failures and returns them. Both borrow.
The consuming variants are named `must_owned()` and `verify_owned()`.

```rust
use assertr::prelude::*;

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
```

Fluent names follow fixed rules. `is_x` becomes `be_x`, `has_x` becomes `have_x`, other verbs
become imperative (`contains` -> `contain`), and negations put `not` first (`is_not_x` ->
`not_be_x`). See [`IntoAssertContext`](https://docs.rs/assertr/latest/assertr/trait.IntoAssertContext.html) for the complete rules.

## Finding assertions

Autocomplete on the subject is the fastest way. For a browsable reference, start with the
[assertion families](https://docs.rs/assertr/latest/assertr/assertions/index.html) on docs.rs. Each assertion trait page
is the authoritative list of its methods, signatures, and required bounds.

Blanket implementations make general assertions available to user-defined types. A `PartialEq`
type has `is_equal_to`, a `PartialOrd` type has `is_greater_than`, and a `HasLength` type has
`has_length`.

## Guides

These guides build on the quick start. Each lives with the API it explains and includes examples
you can adapt:

- [Assert on part of a subject](https://docs.rs/assertr/latest/assertr/struct.AssertThat.html#method.derive):
  project a field with `derive`, then chain assertions on it. Check several fields from the same
  parent, use `derive_owned` for computed values and borrowed slices, or await `derive_async`
  projections.
- [Match selected fields and nested values](https://docs.rs/assertr/latest/assertr/matchers/index.html):
  use `partial!` with plain values, selected matcher constraints, or existing assertions through
  `satisfying`. Nest expectations through structs, collections, and maps. Only `partial!`
  requires the `matchers` feature.
- [Collect failures without panicking](https://docs.rs/assertr/latest/assertr/struct.AssertThat.html#method.capture):
  run several checks, inspect their structured failures, and render a report when needed.
- [Customize diagnostic values](https://docs.rs/assertr/latest/assertr/renderer/index.html):
  render types without `Debug`, preserve a renderer across projections, and limit diagnostic
  output.
- [Process failures and customize reports](https://docs.rs/assertr/latest/assertr/failure/adapter/index.html):
  transform captured failures with adapters or select the presentation used by a panicking
  assertion.
- [Write assertions for custom types](https://docs.rs/assertr/latest/assertr/#custom-assertions):
  add chainable methods by composing existing assertions or building a structured failure
  yourself.
- [Assert properties of a type](https://docs.rs/assertr/latest/assertr/fn.assert_that_type.html):
  check size, type name, or drop requirements without constructing a value.

## API stability

Publicly exported items follow the usual Semantic Versioning rules unless their documentation
explicitly says otherwise. The `*Assertions` traits are public for method discovery, not as
downstream implementation interfaces, so adding a method to one of them is considered
compatible. `assertr::__private` is the explicitly unsupported macro plumbing and must not be
named directly.

## MSRV

The minimum supported Rust version is `1.89.0` for both crates. Version history is recorded in
the changelog.

## Contributing

Run `just install-tools` once, then `just verify` before submitting a pull request. Record
notable changes under `## [Unreleased]`, or under the latest version section if it has not been
published yet.

The README is generated from the landing-page rustdoc in `assertr/src/lib.rs`. Edit that source,
format it, then run `just readme`. `just check-readme` verifies that the generated README is
current.

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](https://github.com/lpotthast/assertr/blob/main/LICENSE-APACHE))
- MIT License ([LICENSE-MIT](https://github.com/lpotthast/assertr/blob/main/LICENSE-MIT))

<!-- cargo-rdme end -->
