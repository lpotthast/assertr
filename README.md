# assertr

A fluent assertion library for Rust that enables clear, intuitive, and readable test code\
with detailed failure messages to help pinpoint issues quickly.

## Features

- 🔗 **Fluent API**: Chain multiple assertions for improved readability.
  Fluent assertions provide better IDE support through method chaining. The IDE can show you exactly what
  assertions are available for your specific type, making it hard to write invalid assertions and easier to discover
  available checks.
- 🎯 **Type-specific Assertions**: Specialized checks for many Rust types as well as a broad generic type coverage.
- 📝 **Detailed Error Messages**: Clear, context-rich failure messages. Any assertion can extend the context with
  additional descriptive output.
- 🔄 **Capture Mode**: Collect assertion failures for manual inspection instead of immediately panicking.
- 🛠 **Extensible**: Easily add custom assertions for your own types.
- ⚡ **Derive Macros**: Perform partial struct assertions with the help of the `#[derive(AssertrEq)]` macro.

## Installation

```tome
[dependencies]
assertr = "0.3.5"
```

or

```tome
[dependencies]
assertr = { version = "0.3.5", features = ["derive"] }
```

if you want the `AssertrEq` derive macro allowing you to perform partial equality assertions on struct value on a
field-by-field value. More on that later.

- You may disable the default features for no-std environments.

- You may activate any of the following features:

| feature | description                                                           | default feature |
|---------|-----------------------------------------------------------------------|-----------------|
| std     | Assertions for types from the standard library.                       | yes             |
| derive  | Enables the `AssertrEq` derive macro.                                 | no              |
| num     | Assertions for numeric types.                                         | yes             |
| libm    | Use fallback implementations for Rust's float math functions in core. | no              |
| serde   | Assertions for serializable types (supporting json and toml).         | no              |
| jiff    | Assertions for types from the `jiff` crate.                           | no              |
| tokio   | Assertions for types from the `tokio` crate.                          | no              |
| reqwest | Assertions for types from the `reqwest` crate.                        | no              |

## Quick start

Always prefer importing the entire prelude, as in:

```rust
use assertr::prelude::*;

#[test]
fn test() {
    assert_that("hello, world!")
        .starts_with("hello")
        .ends_with("!");
}
```

This way, you are ensured to get full IDE autocompletion and always see all available assertions for the type you are
currently writing an assertion on.

## Available Assertions

| type / required bounds             | assertion                                          | note                          | required features |
|------------------------------------|----------------------------------------------------|-------------------------------|-------------------|
| `T: PartialEq`                     | `is_equal_to(expected)`                            |                               |                   |
| `T: PartialEq`                     | `is_not_equal_to(expected)`                        |                               |                   |
| `T: PartialOrd<E>`                 | `is_less_than(expected)`                           |                               |                   |
| `T: PartialOrd<E>`                 | `is_greater_than(expected)`                        |                               |                   |
| `T: PartialOrd<E>`                 | `is_less_or_equal_to(expected)`                    |                               |                   |
| `T: PartialOrd<E>`                 | `is_greater_or_equal_to(expected)`                 |                               |                   |
| `bool`                             | `is_true()`                                        |                               |                   |
| `bool`                             | `is_false()`                                       |                               |                   |
| `char`                             | `is_equal_to_ignoring_ascii_case(expected)`        |                               |                   |
| `char`                             | `is_lowercase()`                                   |                               |                   |
| `char`                             | `is_uppercase()`                                   |                               |                   |
| `char`                             | `is_ascii_lowercase()`                             |                               |                   |
| `char`                             | `is_ascii_uppercase()`                             |                               |                   |
| `&str`                             | `is_blank()`                                       |                               |                   |
| `&str`                             | `is_blank_ascii()`                                 |                               |                   |
| `&str`                             | `contains(expected)`                               |                               |                   |
| `&str`                             | `starts_with(expected)`                            |                               |                   |
| `&str`                             | `ends_with(expected)`                              |                               |                   |
| `String`                           | `contains(expected)`                               |                               | alloc             |
| `String`                           | `starts_with(expected)`                            |                               | alloc             |
| `String`                           | `ends_with(expected)`                              |                               | alloc             |
| `&[T]`                             | `contains(expected)`                               |                               |                   |
| `&[T]`                             | `contains_exactly(expected)`                       |                               |                   |
| `&[T]`                             | `contains_exactly_in_any_order(expected)`          |                               |                   |
| `&[T]`                             | `contains_exactly_matching_in_any_order(expected)` |                               |                   |
| `[T; N]`                           | `contains(expected)`                               |                               |                   |
| `[T; N]`                           | `contains_exactly(expected)`                       |                               |                   |
| `[T; N]`                           | `contains_exactly_matching_in_any_order(expected)` |                               |                   |
| `Vec<T>`                           | `contains(expected)`                               |                               | alloc             |
| `Vec<T>`                           | `contains_exactly(expected)`                       |                               | alloc             |
| `Vec<T>`                           | `contains_exactly_matching_in_any_order(expected)` |                               | alloc             |
| `T: Debug`                         | `has_debug_string(expected)`                       |                               |                   |
| `T: Debug`                         | `has_debug_value(expected)`                        |                               |                   |
| `T: Display`                       | `has_display_value(expected)`                      |                               |                   |
| `F: FnOnce -> R`                   | `panics()`                                         |                               |                   |
| `F: FnOnce -> R`                   | `does_not_panic()`                                 |                               |                   |
| `I: Iterator<Item = T>`            | `contains(expected)`                               |                               |                   |
| `I: Iterator<Item = T>`            | `contains_exactly(expected)`                       |                               |                   |
| `T: HasLength`                     | `is_empty()`                                       | implemented for: ``           |                   |
| `T: HasLength`                     | `is_not_empty()`                                   | implemented for: ``           |                   |
| `T: HasLength`                     | `has_length(expected)`                             | implemented for: ``           |                   |
| `T: Num`                           | `is_zero()`                                        |                               | num               |
| `T: Num`                           | `is_additive_identity()`                           | Synonym for `is_zero`         | num               |
| `T: Num`                           | `is_one()`                                         |                               | num               |
| `T: Num`                           | `is_multiplicative_identity()`                     | Synonym for `is_zero`         | num               |
| `T: Num + Signed`                  | `is_negative()`                                    |                               | num               |
| `T: Num + Signed`                  | `is_positive()`                                    |                               | num               |
| `T: Num + PartialOrd + Clone`      | `is_close_to()`                                    |                               | num               |
| `T: Num + Float`                   | `is_nan()`                                         |                               | num               |
| `T: Num + Float`                   | `is_finite()`                                      |                               | num               |
| `T: Num + Float`                   | `is_infinite()`                                    |                               | num               |
| `Option<T>`                        | `is_some()`                                        |                               |                   |
| `Option<T>`                        | `is_none()`                                        |                               |                   |
| `Poll<T>`                          | `is_pending()`                                     |                               |                   |
| `Poll<T>`                          | `is_ready()`                                       |                               |                   |
| `R: RangeBounds<B>, B: PartialOrd` | `contains_element(expected)`                       |                               |                   |
| `R: RangeBounds<B>, B: PartialOrd` | `does_not_contain_element(expected)`               |                               |                   |
| `B: PartialOrd`                    | `is_in_range(expected)`                            |                               |                   |
| `B: PartialOrd`                    | `is_not_in_range(expected)`                        |                               |                   |
| `B: PartialOrd`                    | `is_outside_of_range(expected)`                    | Synonym for `is_not_in_range` |                   |
| `RefCell<T>`                       | `is_borrowed()`                                    |                               |                   |
| `RefCell<T>`                       | `is_mutably_borrowed()`                            |                               |                   |
| `RefCell<T>`                       | `is_not_mutably_borrowed()`                        |                               |                   |
| `Mutex<T>`                         | `is_locked()`                                      |                               |                   |
| `Mutex<T>`                         | `is_not_locked()`                                  |                               |                   |
| `Mutex<T>`                         | `is_free()`                                        | Synonym for`is_not_locked`    |                   |
| `Result<T, E>`                     | `is_ok()`                                          |                               |                   |
| `Result<T, E>`                     | `is_err()`                                         |                               |                   |
| `Result<T, E>`                     | `is_ok_satisfying(assertions)`                     |                               |                   |
| `Result<T, E>`                     | `is_err_satisfying(assertions)`                    |                               |                   |
| `PathBuf`                          | `exists()`                                         |                               | std               |
| `PathBuf`                          | `does_not_exist()`                                 |                               | std               |
| `PathBuf`                          | `is_a_file()`                                      |                               | std               |
| `PathBuf`                          | `is_a_directory()`                                 |                               | std               |
| `PathBuf`                          | `is_a_symlink()`                                   |                               | std               |
| `PathBuf`                          | `has_a_root()`                                     |                               | std               |
| `PathBuf`                          | `is_relative()`                                    |                               | std               |
| `PathBuf`                          | `has_file_name(expected)`                          |                               | std               |
| `PathBuf`                          | `has_file_stem(expected)`                          |                               | std               |
| `PathBuf`                          | `has_extension(expected)`                          |                               | std               |
| `&Path`                            | `exists()`                                         |                               | std               |
| `&Path`                            | `does_not_exist()`                                 |                               | std               |
| `&Path`                            | `is_a_file()`                                      |                               | std               |
| `&Path`                            | `is_a_directory()`                                 |                               | std               |
| `&Path`                            | `is_a_symlink()`                                   |                               | std               |
| `&Path`                            | `has_a_root()`                                     |                               | std               |
| `&Path`                            | `is_relative()`                                    |                               | std               |
| `&Path`                            | `has_file_name(expected)`                          |                               | std               |
| `&Path`                            | `has_file_stem(expected)`                          |                               | std               |
| `&Path`                            | `has_extension(expected)`                          |                               | std               |
| `HashMap<K, V>`                    | `contains_key(expected)`                           |                               | std               |
| `HashMap<K, V>`                    | `does_not_contain_key(not_expected)`               |                               | std               |
| `HashMap<K, V>`                    | `contains_value(expected)`                         |                               | std               |
| `HashMap<K, V>`                    | `contains_entry(expected_key, expected_value)`     |                               | std               |
| `Command`                          | `has_arg(expected)`                                |                               | std               |
| `Type<T>`                          | `needs_drop()`                                     |                               | std               |
| `Box<dyn Any>`                     | `has_type::<Expected>()`                           |                               | alloc             |
| `Box<dyn Any>`                     | `has_type_ref::<Expected>()`                       |                               | alloc             |
| `PanicValue`                       | `has_type::<Expected>()`                           |                               | alloc             |
| `PanicValue`                       | `has_type_ref::<Expected>()`                       |                               | alloc             |
| `tokio::sync::Mutex<T>`            | `is_locked()`                                      |                               | tokio             |
| `tokio::sync::Mutex<T>`            | `is_not_locked()`                                  |                               | tokio             |
| `tokio::sync::Mutex<T>`            | `is_free()`                                        | Synonym for `is_not_locked`   | tokio             |
| `tokio::sync::RwLock<T>`           | `is_not_locked()`                                  |                               | tokio             |
| `tokio::sync::RwLock<T>`           | `is_free()`                                        | Synonym for `is_not_locked`   | tokio             |
| `tokio::sync::RwLock<T>`           | `is_read_locked()`                                 |                               | tokio             |
| `tokio::sync::RwLock<T>`           | `is_write_locked()`                                |                               | tokio             |
| `tokio::sync::watch::Receiver<T>`  | `has_current_value(expected)`                      |                               | tokio             |
| `tokio::sync::watch::Receiver<T>`  | `has_changed()`                                    |                               | tokio             |
| `tokio::sync::watch::Receiver<T>`  | `has_not_changed()`                                |                               | tokio             |
| `reqwest::Response`                | `has_status_code(expected)`                        |                               | reqwest           |
| `jiff::SignedDuration`             | `is_zero()`                                        |                               | jiff              |
| `jiff::SignedDuration`             | `is_negative()`                                    |                               | jiff              |
| `jiff::SignedDuration`             | `is_positive()`                                    |                               | jiff              |
| `jiff::SignedDuration`             | `is_close_to(expected, allowed_deviation)`         |                               | jiff              |
| `jiff::Span`                       | `is_zero()`                                        |                               | jiff              |
| `jiff::Span`                       | `is_negative()`                                    |                               | jiff              |
| `jiff::Span`                       | `is_positive()`                                    |                               | jiff              |
| `jiff::Zoned`                      | `is_in_time_zone(expected)`                        |                               | jiff              |
| `jiff::Zoned`                      | `is_in_time_zone_named(expected)`                  |                               | jiff              |

*The generic types (`T`, `E`, ...) nearly always also require to be `Debug`. Otherwise, we could not print values in
case an assertion is violated. We chose to not explicitly list these bounds in the table above.

### Conditional Assertions

- `satisfies(condition)`: Asserts that a value satisfies a condition
- `satisfies_ref(condition)`: Asserts that a referenced value satisfies a condition

## Advanced Features

### Capture Mode

Instead of immediately panicking on assertion failure, you can capture failures for later analysis:

```rust
let failures = assert_that(3)
    .with_capture()
    .is_equal_to(4)
    .is_less_than(2)
    .capture_failures();

assert_that(failures).has_length(2);
```

### Partial equality assertions

You can derive a helper struct, allowing you to perform partial equality comparisons, for any owned struct type
by annotating it with the `#[derive(AssertrEq)`.

**Make sure that this crates `derive` feature is active!**

```toml
assertr = { version = "0.2.0", features = ["derive"] }
```

```rust
// Deriving `AssertrEq` provides us an additional `PersonAssertrEq` type.
// Deriving `Debug` is necessary, as we want to actually use `Foo` in an assertion.
#[derive(Debug, AssertrEq)]
pub struct Person {
    pub name: String,
    pub age: i32,
    pub data: (u32, u32),
}

#[test]
fn test() {
    let alice = Person {
        name: "Alice".to_owned(),
        age: 30,
        data: (100, 998)
    };
    
    // We can still perform a standard (full) equality check.
    assert_that_ref(&alice).is_equal_to(Person {
        name: "Alice".to_owned(),
        age: 30,
        data: (100, 998),
    });
    
    // But we can also do a partial equality check!
    assert_that_ref(&alice).is_equal_to(PersonAssertrEq {
        name: eq("Alice".to_owned()),
        age: any(), // Match any age
        data: any() // Match any data
    });
}
```

### Write assertions for your own types.

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

### Type Testing

Test properties of types:

``` rust
assert_that_type::<MyType>()
    .needs_drop()
    .satisfies(|it| it.size(), |size| {
        size.is_equal_to(32);
    });
```

## Goals

```rust
// Assertions that read like English.
assert_that("foobar").starts_with("foo").contains("ooba");
assert_that(vec![1, 2, 3]).has_length(3).contains(2);
assert_that((Ok(42)).is_ok().is_equal_to(42);
assert_that(Person { id: 42 }).has_debug_string("Person { id: 42 }");

// Chainable,
assert_that("foobar")
.is_not_empty()
.starts_with("foo")
.ends_with("bar")
.has_length(6);
```

- Partial equality assertions (meaning that only some fields of a struct are compared, while some are ignored).
  Add the `AssertrEq` annotation to one of your struct to enable this.


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

## Technical decisions

- Derived assertions are not allowed to control whether the location is printed.
- Detail messages are collected from the current assertion upwards, taking the messages of all parents into account.
- Failures are stored at the root assertion.
- Failures can only be extracted from the root assertion.
- Assertions to be defined on common traits as often as possible. Allowing, for example, all types implementing `Eq`
  to allow `is_equal_to`, `PartialOrd` types to allow `is_greater_than` assertions and all types implementing the
  `HasLength` trait to support the `has_length` assertion.
- No requirement to use macros for simple assertions. An
  `assert_that(...)` suffices to get into an assertion context. Use `assert_that_ref(&val)` if you cant give up
  ownership and instead want to assert on a reference.
- One import should be enough to access all possible assertions through **autocomplete**.
  `use assertr::prelude::*;`

## Testing

To test all crates, run with --all when in root

    cargo test --all

This crate uses features. Some tests are declared under conditional compilation.

Run all tests using

    cargo test --all-features

## MSRV

- As of `0.1.0` the MSRV is `1.76.0`
- As of `0.2.0` the MSRV is `1.85.0`

## Open questions

- Many assertions require `std::fmt::Debug`, limiting usability to types implementing Debug.
  Can we implement fallback rendering? Will probably require the currently unstable specialization feature.

- The differentiation between `assert_that` for owned values and `assert_that_ref` for references is not ideal.
  One `assert_that` function, not being a macro and accepting both owned values and references would be much preferred.
  But that would probably also require the specialization feature to be able to detect the use of a reference type at
  compiletime.

## Contributing

Contributions are welcome. Feel free to open a PR with your suggested changes.

## Acknowledgements

Midway through implementing this, I found out that "spectral" already exists, which uses a very similar style of
assertions.
After looking into it, I took the concept of generally relying on `*Assertions` trait definitions instead of directly
implementing Assertions with multiple impl blocks on the `AssertThat` type.
