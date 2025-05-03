# assertr

Fluent assertions for the Rust programming language.

## Goals

- Assertions that read like english:
  ```rust
  assert_that("foobar").starts_with("foo").contains("ooba");
  assert_that(vec![1, 2, 3]).has_length(3).contains(2);
  assert_that((Ok(42)).is_ok().is_equal_to(42);
  assert_that(Person { id: 42 }).has_debug_value("Person { id: 42 }");
  ```


- Assertions to be defined on common traits as often as possible. Allowing, for example, all types implementing `Eq`
  to allow `is_equal_to`, `PartialOrd` types to allow `is_greater_than` assertions and all types implementing the
  `HasLength` trait to support the `has_length` assertion.


- No requirement to use macros for simple assertions. An
  `assert_that(...)` suffices to get into an assertion context. Use `assert_that_ref(&val)` if you cant give up
  ownership and instead want to assert on a reference.


- One import should be enough to access all possible assertions through **autocomplete**.
  `use assertr::prelude::*;`


- Chainable assertions.
  ```rust
  assert_that("foobar")
      .is_not_empty()
      .starts_with("foo")
      .ends_with("bar")
      .has_length(6);
  ```


- Extensibility. Write assertions for your own types.
  ```rust
  #[derive(Debug, PartialEq)]
  struct Person {
      age: u32,
  }

  trait PersonAssertions {
      fn has_age(self, expected: u32) -> Self;
  }

  impl<M: Mode> PersonAssertions for AssertThat<'_, Person, M> {
      fn has_age(self, expected: u32) -> Self {
          self.satisfies(|p| p.age, |age| { age.is_equal_to(expected); })
      }
  }

  #[test]
  fn test() {
      assert_that(Person { age: 30 })
        .has_age(30);
  }
  ```


- Partial equality assertions (meaning that only some fields of a struct are compared, while some are ignored).
  Add the `AssertrEq` annotation to one of your struct to enable this.
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

      assert_that(foo).is_equal_to(FooAssertrEq {
          id: eq(1),
          name: eq("bob".to_string()),
          data: any(),
      });
  }
  ```

## Compared to other assertion styles

One other style of assertions in Rust is the "individual macros" approach.
The standard library already comes with a few of them, like the `assert_eq!` macro, many libraries provide a more
exhaustive list of macros specifically tailored for specific types and operations.

Let me point out a few benefits of fluent assertions compared to individual assert macros.

#### Chainability and Readability

The fluent interface allows you to chain multiple assertions naturally, following the way we think about validating
properties. Instead of writing multiple separate assertions, you can express related checks in a single, flowing
statement that reads almost like natural language.

Additionally, having a concrete entrance into the assertion context using a function like `assert_that` with assertions
coming after makes it totally obvious which value is the "actual" and which is the "expected" value. This provides
a clear schema for how assertions are written, compared to an assertion macro, like std's `assert_eq!`, in which the
order of arguments can be chosen freely, making it non-obvious when coming into a new codebase which style
was chosen.

#### Better Error Messages

Fluent assertions can provide more detailed and structured error messages out of the box. Rather than just showing
the values that didn't match, they can include context about what specific check failed within the chain and clearer
descriptions of the expected vs actual values. Descriptive messages can be collected throughout the call chain.

#### Reduced Code Duplication

With traditional assert macros, you often need to reference the same value multiple times:

```rust
let vec = vec![1, 2, 3];
assert_eq!(vec.len(), 3);
assert!(vec.contains(&2));
```

Versus the fluent style:

```rust
assert_that(vec![1, 2, 3]).has_length(3).contains(2);
```

#### Type Safety and IDE Support

Fluent assertions provide better IDE support through method chaining. The IDE can show you exactly what
assertions are available for your specific type, making it harder to write invalid assertions and easier to discover
available checks.

#### Extensibility

It's generally easier to add new assertion types in a fluent interface - you just need to implement new methods on the
assertion type. With macros, you'd need to create entirely new macros for each assertion type, which can be more complex
and harder to maintain.

## Open questions

- Many assertions require `std::fmt::Debug`, limiting usability to types implementing Debug.
  Can we implement fallback rendering? Will probably require the currently unstable specialization feature.

- The differentiation between `assert_that` for owned values and `assert_that_ref` for references is bad.
  One `assert_that` function, not being macro, accepting both owned values and references would be much preferred.
  But that would also require the specialization feature to be able to detect the use of a reference type at
  compiletime.

## Decisions

- Derived assertions are not allowed to control whether the location is printed.
- Detail messages are collected from the current assertion upwards, taking the messages of all parents into account.
- Failures are stored at the root assertion.
- Failures can only be extracted from the root assertion.

## Testing

To test all creates, run with --all when in root

    cargo test --all

This crate uses features. Some tests are declared under conditional compilation.

Run all tests using

    cargo test --all-features

## MSRV

- As of `0.1.0` the MSRV is `1.76.0`
- As of `0.2.0` the MSRV is `1.85.0`

## Contributing

Contributions are welcome. Feel free to open a PR with your suggested changes.

## Acknowledgements

Midway through implementing this, I found out that "spectral" already exists, which uses a very similar style of
assertions.
After looking into it, I took the concept of generally relying on `*Assertions` trait definitions instead of directly
implementing Assertions with multiple impl blocks on the `AssertThat` type.
