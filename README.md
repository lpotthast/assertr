# assertr

Fluent assertions for the Rust programming language.

## Goals

- Assertions that read like english:
  `assert_that("foobar").stars_with("foo")`
  `assert_that((Ok(42)).is_ok().is_equal_to(42)`

- No requirement to use any macros. A simple
  `assert_that(...)` suffices to get into an assertion context.

- One import should be enough to access all possible assertions through autocomplete.
  `use assertr::prelude::*;`

- No requirement to always use explicit references. If you are free to give ownership, that's fine.
  `assert_that(MyStruct {}).is_equal_to(MyStruct {})`

- Chainable assertions.

      assert_that("foobar")
          .is_not_empty()
          .starts_with("foo")
          .ends_with("bar")

- Extensibility.

- Partial equality assertions (meaning that only some fields of a struct are compared, and some are ignored).
  ```rust
  use assertr::prelude::*;
  use indoc::formatdoc;

  // Deriving `Debug` is necessary, as we want to actually use `Foo` in an assertion.
  #[derive(Debug, AssertrEq)]
  pub struct Foo {
      pub id: i32,
      pub name: String,
      pub data: (u32, u32),
  }

  fn main() {
      let foo = Foo {
          id: 1,
          name: "bob".to_string(),
          data: (42, 100),
      };

      assert_that_ref(&foo).is_equal_to(FooAssertrEq {
          id: any(),
          name: any(),
          data: any(),
      });

      assert_that_ref(&foo).is_equal_to(FooAssertrEq {
          id: eq(1),
          name: eq("bob".to_string()),
          data: any(),
      });
  }
  ```

## Open questions

Many assertions require `std::fmt::Debug`, limiting usability to types implementing Debug.
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

## Acknowledgements

Midway through implementing this, I found out that "spectral" already exists, which uses a very similar style of
assertions.
After looking into it, I took the concept of generally relying on `*Assertions` trait definitions instead of directly
implementing Assertions with multiple impl blocks on the `AssertThat` type.
