# assertr

Fluent assertions for the Rust programming language.

## Goals

- Assertions that read like english:
  `assert_that("foobar").stars_with("foo")`
  `assert_that((Ok(42)).is_ok().is_equal_to(42)`

- No requirement to use a macro. A simple
  `assert_that(...)` suffices.

- One import should be enough to access all possible assertions through autocomplete.
  `use assertr::prelude::*;`

- No requirement to always use explicit references. If you are free to give ownership, that's fine.
  `assert_that(MyStruct {}).is_equal_to(MyStruct {})`

- Chainable assertions.

      assert_that("foobar")
          .is_not_empty()
          .starts_with("foo")
          .ends_with("bar")

- Extensible.

## Open questions

Many assertions require std::fmt::Debug, limiting usability to types implementing Debug.
Can we implement fallback rendering? Will probably require the currently unstable specialization feature.

## Decisions

- Derived assertions are not allowed to control whether the location is printed.
- Detail messages are collected from the current assertion upwards, taking the messages of all parents into account.
- Failures are stored at the root assertion.
- Failures can only be extracted from the root assertion.

## Examples

    use assertr::prelude::*;

    assert_that("foobar")
        .is_not_empty()
        .starts_with("foo")
        .ends_with("bar")

## Extensibility

## Testing

To test all creates, run with --all when in root

    cargo test --all

This crate uses features. Some tests are declared under conditional compilation.

Run all tests using

    cargo test --all-features

## Contribution

Midway through implementing this, I found out that "spectral" already exists, which uses a very similar style of
assertions.
After looking into it, I took the concept of generally relying on `*Assertions` trait definitions instead of directly
implementing Assertions with multiple impl blocks on `AssertThat`.
