# assertr

Fluent assertions for the Rust programming language.

## Goals

- No requirement to use any macros. A simple
  assert_that(...)
  suffices.s

- One import suffices to access all possible assertions through autocomplete.
  use assertr::prelude::*;

- No requirement to always use explicit references. If you are free to give ownership, thats fine and possibly easier to
  read.
  assert_that([1, 2, 3]).contains_exactly_in_any_order([3, 1, 2])

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

## Contribution

## Testing

To test all creates, run with --all when in root

    cargo test --all

This crate uses features. Some tests are declared under conditional compilation.

Run all tests using

    cargo test --all-features
