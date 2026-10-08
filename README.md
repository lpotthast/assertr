<!-- Generated from the landing-page rustdoc in assertr/src/lib.rs. Edit that source and run `just readme`. -->

<!-- cargo-rdme start -->

# assertr

[![Crates.io](https://img.shields.io/crates/v/assertr.svg)](https://crates.io/crates/assertr)
[![Docs.rs](https://docs.rs/assertr/badge.svg)](https://docs.rs/assertr)
[![CI](https://github.com/lpotthast/assertr/actions/workflows/ci.yml/badge.svg)](https://github.com/lpotthast/assertr/actions/workflows/ci.yml)
[![MSRV](https://img.shields.io/badge/MSRV-1.89.0-blue.svg)](https://github.com/lpotthast/assertr/blob/main/assertr/Cargo.toml)
[![License: MIT OR Apache-2.0](https://img.shields.io/crates/l/assertr.svg)](#license)

Fluent assertions for Rust. Pass the value under test to `assert_that!` and chain the checks it
should pass. Autocomplete offers only the assertions that fit the value's type, and every
failure explains what was found and what was expected.

```rust
use assertr::prelude::*;

assert_that!("hello, world!")
    .starts_with("hello")
    .ends_with("!");

let numbers = vec![1, 2, 3];
assert_that!(numbers).has_length(3).contains(2);
```

Had the first chain expected `"?"` instead of `"!"`, the test would fail with:

```text
-------- assertr --------
Assertion failed at tests/greeting.rs:5:6

Expression: `"hello, world!"`

Actual: "hello, world!"

does not end with

Expected: "?"
-------- assertr --------
```

Compared to `assert!` and `assert_eq!`, the value under test always comes first, one chain
replaces a series of separate macro calls, and the report names the relation that did not hold.
assertr works in `std` and `no_std` builds.

## Installation

```toml
[dev-dependencies]
assertr = "0.8.0"
```

Everything beyond `std` and `num` is opt-in:

| Feature | Adds |
|---|---|
| `std` (default) | Assertions for `HashMap`, `HashSet`, `Path`, `Command`, `Mutex`, and panicking closures. |
| `num` (default) | Numeric assertions such as `is_zero`, `is_positive`, `is_nan`, and `is_close_to`. |
| `libm` | Floating-point checks like `is_nan` for `num` in `no_std` builds. Does not enable `num`. |
| `partial` | The `partial!` macro for matching selected struct fields. |
| `fluent` | The `value.must()` and `value.verify(..)` entry points with fluent method names. |
| `serde-json`, `serde-toml` | `as_json()` and `as_toml()` to assert on a value's serialized form. |
| `serde` | Both `serde-json` and `serde-toml`. |
| `program` | Checks that a program name or path resolves to an executable, like `which`. |
| `http`, `jiff`, `reqwest`, `rootcause`, `tokio` | Assertions for types of the crate with the same name. |
| `full` | All of the above. |

For `no_std`, disable the default features. `num`, `libm`, `partial`, `fluent`, `rootcause`, and
the `serde` features work with `alloc` alone. All other features enable `std`.

## Writing assertions

Import the prelude. It brings every assertion trait into scope, so autocomplete can list them.
Then start a chain with `assert_that!` and add assertions. The first failing assertion panics.

`assert_that!` borrows its argument, so you can keep using a value after asserting on it. A few
assertions consume their subject, for example running a closure or draining an iterator. Start
those with `assert_that_owned!`:

```rust
use assertr::prelude::*;

let name = String::from("Ada");
assert_that!(name).is_equal_to("Ada");
assert_that!(name.len()).is_equal_to(3); // `name` is still usable.

assert_that_owned!((1..=3).map(|n| n * n)).contains_exactly([1, 4, 9]);
```

Some assertions continue with a different subject. Use this to check what is inside an `Option`,
a `Result`, or a panic:

```rust
use assertr::prelude::*;

assert_that!(Some(42)).get_some().is_greater_than(40);
assert_that!("42".parse::<u32>()).get_ok().is_equal_to(42);
assert_that_owned!(|| panic!("boom"))
    .panics()
    .has_type::<&str>()
    .is_equal_to("boom");
```

Expected values can be owned or borrowed, and string literals compare with `String` subjects.
Pass a reference to reuse an expected value without cloning it:

```rust
use assertr::prelude::*;

let expected = String::from("Ada");
assert_that!(String::from("Ada")).is_equal_to(&expected);
assert_that!(vec![String::from("Ada")]).contains_exactly(["Ada"]);
```

Reports show values through their `Debug` implementation. For types without one, or to show a
value differently, see [rendering values](https://docs.rs/assertr/latest/assertr/renderer/).

## Checking parts of a value

Use `derive` to assert on a field or a computed value. The parent chain stays usable, so you can
check several fields one after another:

```rust
use assertr::prelude::*;

struct User {
    name: String,
    age: u32,
}

let user = User { name: "Ada".into(), age: 36 };
let user = assert_that!(user);
user.derive(|u| &u.name).starts_with("A");
user.derive(|u| &u.age).is_greater_or_equal_to(18);
```

With the `partial` feature, `partial!` describes the fields that matter in one expression and
ignores the rest with `..`. The struct needs no derives or annotations:

```rust
use assertr::{matchers::{eq, ge}, prelude::*};

struct User {
    name: String,
    age: u32,
}

let user = User { name: "Ada".into(), age: 36 };
assert_that!(user).matches(partial!(User { name: eq("Ada"), age: ge(18), .. }));
```

For a user named Bob aged 17, the report lists both mismatches:

```text
does not match

Nested failures:
  - At .name:
    Expected: "Ada"

      Actual: "Bob"

  - At .age:
    Actual: 17

    is not greater than or equal to

    Expected: 18
```

## Reusable checks

`eq` and `ge` above are matchers. A matcher is a check stored in a value, so you can define it
once and apply it to a whole value, to the elements of a collection, or to a field in
`partial!`:

```rust
use assertr::{matchers::{all_of, string, HasLengthOf}, prelude::*};

let short_name = all_of(matchers![string::IsNotBlank, HasLengthOf::new(3)]);
assert_that!("Ada").matches(&short_name);
assert_that!(["", "Ada", "Grace"]).contains_matching(&short_name);
```

The [matcher catalog](https://docs.rs/assertr/latest/assertr/matchers/) lists every built-in matcher. To name your own
domain checks, see the [custom assertions
guide](https://docs.rs/assertr/latest/assertr/#custom-assertions).

## Collecting failures

`capture` runs a chain without panicking and returns all failures. Use it to report several
problems at once, or to test your own assertions:

```rust
use assertr::prelude::*;

let failures = assert_that!(42).capture(|it| it.is_less_than(10).is_equal_to(43));
assert_that!(failures).has_length(2);
assert_that!(failures[0].to_string()).contains("is not less than");
```

Each failure is structured data. Its fields hold the rendered actual and expected values, the
relation between them, and nested failures. Its `Display` implementation produces the report.

Assertions that switch to a different subject, such as `get_some()`, are not available in
capture mode, because a failure leaves no value to continue with. Use their `*_satisfying`
variants instead, for example `is_some_satisfying(|value| ..)`.

## Fluent entry points

The `fluent` feature lets you start a chain from the value itself. `must()` panics on the first
failure. `verify(..)` collects failures like `capture`. Assertion names read as requirements:
`is_x` becomes `be_x`, `has_x` becomes `have_x`, `contains` becomes `contain`, and `is_not_x`
becomes `not_be_x`.

```rust
use assertr::prelude::*;

"hello, world!".must().start_with("hello").end_with("!");

let failures = 3.verify(|it| it.be_equal_to(4));
assert_that!(failures).has_length(1);
```

Both borrow the value. `must_owned()` and `verify_owned()` take ownership. See
[`IntoAssertContext`](https://docs.rs/assertr/latest/assertr/trait.IntoAssertContext.html) for
all naming rules and for recording the source expression in reports.

## Finding assertions

Autocomplete on a chain is the quickest way to discover assertions. To browse, start at the
[assertion families](https://docs.rs/assertr/latest/assertr/assertions/). Each assertion trait lists its methods and what they
require from the subject.

Many assertions apply to any type with the right capabilities. Your own types get `is_equal_to`
from `PartialEq`, `is_greater_than` from `PartialOrd`, and collection assertions by implementing
[`Collection`](https://docs.rs/assertr/latest/assertr/assertions/collection/trait.Collection.html).

## Guides

The crate documentation on docs.rs goes deeper:

- [Core concepts](https://docs.rs/assertr/latest/assertr/#core-concepts): subjects, ownership,
  child chains, and the two assertion modes.
- [Partial matching](https://docs.rs/assertr/latest/assertr/matchers/index.html#structural-syntax):
  nested fields, enum variants, and collection policies such as `each` and `elements_are!`.
- [Custom assertions](https://docs.rs/assertr/latest/assertr/#custom-assertions): reusable
  checks, your own chainable methods, and diagnostics in the standard report format.
- [Rendering values](https://docs.rs/assertr/latest/assertr/renderer/): types without `Debug`, custom formatting, and limits for
  large values.
- [Failure handling](https://docs.rs/assertr/latest/assertr/failure/): inspect failures as data and customize panic messages.
- [Async code](https://docs.rs/assertr/latest/assertr/#async-code): async projections, futures
  that panic, and `Send` limitations.
- [Type properties](https://docs.rs/assertr/latest/assertr/fn.assert_that_type.html): size, type
  name, and drop behavior of a type.

## API stability

assertr follows Semantic Versioning for its public items unless their documentation says
otherwise. The `*Assertions` traits are public so that their methods are available, not for you
to implement. Adding methods to them is not a breaking change. `assertr::__private` is internal
macro support and must not be used directly.

## MSRV

Current MSRV:

- `assertr`: `1.89.0`
- `assertr-macros`: `1.89.0`

Previous MSRV values:

- As of `0.1.0`, the MSRV was `1.76.0`
- As of `0.2.0`, the MSRV was `1.85.0`
- As of `0.4.0`, the MSRV was `1.89.0`

## Contributing

Run `just install-tools` once, then `just verify` before opening a pull request. Record notable
changes in `CHANGELOG.md` under `## [Unreleased]`.

This README is generated from the crate documentation in `assertr/src/lib.rs`. Edit it there and
run `just readme`.

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](https://github.com/lpotthast/assertr/blob/main/LICENSE-APACHE))
- MIT License ([LICENSE-MIT](https://github.com/lpotthast/assertr/blob/main/LICENSE-MIT))

<!-- cargo-rdme end -->
