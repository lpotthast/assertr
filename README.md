# assertr

A fluent assertion library for Rust that enables clear, readable test code with detailed failure
messages that help pinpoint issues quickly.

## Features

- 🔗 **Fluent API**: Chain multiple assertions for improved readability.
  Fluent assertions provide better IDE support through method chaining. The IDE can show you exactly what
  assertions are available for your specific type, making it hard to write invalid assertions and easier to discover
  available checks.
- 🎯 **Type-specific Assertions**: Specialized checks for many Rust types, plus broad generic coverage.
- 📝 **Detailed Error Messages**: Clear, context-rich failure messages. Any assertion can extend the context with
  additional descriptive output.
- 🔄 **Capture Mode**: Collect assertion failures for manual inspection instead of immediately panicking.
- 🛠 **Extensible**: Easily add custom assertions for your own types.
- ⚡ **Derive Macros**: Perform partial struct assertions with the help of the `#[derive(AssertrEq)]` macro.

## Installation

### Default setup

```toml
[dependencies]
assertr = "0.5.7"
```

### Cargo features

Available individual features and feature groups:

| feature   | description                                                           | default feature |
|-----------|-----------------------------------------------------------------------|-----------------|
| std       | Assertions for types from the standard library.                       | yes             |
| derive    | Enables the `AssertrEq` derive macro.                                 | no              |
| fluent    | Enables `.must()` / `.verify()` entry points and fluent aliases.      | no              |
| num       | Assertions for numeric types.                                         | yes             |
| libm      | Use fallback implementations for Rust's float math functions in core. | no              |
| serde     | Assertions for serializable types (supporting json and toml).         | no              |
| jiff      | Assertions for types from the `jiff` crate.                           | no              |
| http      | Assertions for types from the `http` crate.                           | no              |
| tokio     | Assertions for types from the `tokio` crate.                          | no              |
| reqwest   | Assertions for types from the `reqwest` crate.                        | no              |
| rootcause | Assertions for types from the `rootcause` crate.                      | no              |
| program   | Assertions for the provided `Program` type.                           | no              |

| feature-group | description                                                          |
|---------------|----------------------------------------------------------------------|
| default       | Small set of features, enabling support for `std` types and numbers. |
| full          | Enables all features listed above.                                   |

All optional features are additive.

### no_std

Disable the default features in `no_std` environments:

```toml
[dependencies]
assertr = { version = "0.5.7", default-features = false }
```

If you still want numeric assertions in `no_std`, enable `num`. For floating-point classification
helpers such as `is_nan()`, `is_finite()`, or `is_infinite()` without `std`, also enable `libm`.

## Quick start

Always prefer importing the entire prelude:

```rust
use assertr::prelude::*;

#[test]
fn test() {
    assert_that!("hello, world!")
        .starts_with("hello")
        .ends_with("!");
}
```

This gives you full IDE autocomplete for the assertions available on the current subject type.

If the `fluent` feature is enabled, you can also enter assertion contexts directly from values:

```rust
use assertr::prelude::*;

#[test]
fn test() {
    "hello, world!"
        .must()
        .start_with("hello")
        .end_with("!");

    let failures = 3
        .verify()
        .be_equal_to(4)
        .capture_failures();

    assert_that!(failures).have_length(1);
}
```

## Available Assertions

This table is meant as a reference. In day-to-day use, `use assertr::prelude::*;` plus IDE
autocomplete is usually the fastest way to discover what is available.

Roughly, the table is grouped like this:

- Core assertions and collection/string helpers first
- `std` assertions next
- Optional integrations (`http`, `tokio`, `reqwest`, `program`, `rootcause`, `jiff`) last

| type / required bounds                    | assertion                                                     | note                                                                                                                                                | required features |
|-------------------------------------------|---------------------------------------------------------------|-----------------------------------------------------------------------------------------------------------------------------------------------------|-------------------|
| `T: PartialEq`                            | `is_equal_to(expected)`                                       |                                                                                                                                                     |                   |
| `T: PartialEq`                            | `is_not_equal_to(expected)`                                   |                                                                                                                                                     |                   |
| `T: PartialOrd<E>`                        | `is_less_than(expected)`                                      |                                                                                                                                                     |                   |
| `T: PartialOrd<E>`                        | `is_greater_than(expected)`                                   |                                                                                                                                                     |                   |
| `T: PartialOrd<E>`                        | `is_less_or_equal_to(expected)`                               |                                                                                                                                                     |                   |
| `T: PartialOrd<E>`                        | `is_greater_or_equal_to(expected)`                            |                                                                                                                                                     |                   |
| `bool`                                    | `is_true()`                                                   |                                                                                                                                                     |                   |
| `bool`                                    | `is_false()`                                                  |                                                                                                                                                     |                   |
| `char`                                    | `is_equal_to_ignoring_ascii_case(expected)`                   |                                                                                                                                                     |                   |
| `char`                                    | `is_lowercase()`                                              |                                                                                                                                                     |                   |
| `char`                                    | `is_uppercase()`                                              |                                                                                                                                                     |                   |
| `char`                                    | `is_ascii_lowercase()`                                        |                                                                                                                                                     |                   |
| `char`                                    | `is_ascii_uppercase()`                                        |                                                                                                                                                     |                   |
| `&str`                                    | `is_blank()`                                                  |                                                                                                                                                     |                   |
| `&str`                                    | `is_not_blank()`                                              |                                                                                                                                                     |                   |
| `&str`                                    | `is_blank_ascii()`                                            |                                                                                                                                                     |                   |
| `&str`                                    | `is_equal_to_ignoring_ascii_case(expected)`                   |                                                                                                                                                     |                   |
| `&str`                                    | `contains(expected)`                                          |                                                                                                                                                     |                   |
| `&str`                                    | `does_not_contain(unexpected)`                                |                                                                                                                                                     |                   |
| `&str`                                    | `starts_with(expected)`                                       |                                                                                                                                                     |                   |
| `&str`                                    | `does_not_start_with(unexpected)`                             |                                                                                                                                                     |                   |
| `&str`                                    | `ends_with(expected)`                                         |                                                                                                                                                     |                   |
| `&str`                                    | `does_not_end_with(unexpected)`                               |                                                                                                                                                     |                   |
| `String`                                  | `is_not_blank()`                                              |                                                                                                                                                     |                   |
| `String`                                  | `is_equal_to_ignoring_ascii_case(expected)`                   |                                                                                                                                                     |                   |
| `String`                                  | `contains(expected)`                                          |                                                                                                                                                     |                   |
| `String`                                  | `does_not_contain(unexpected)`                                |                                                                                                                                                     |                   |
| `String`                                  | `starts_with(expected)`                                       |                                                                                                                                                     |                   |
| `String`                                  | `does_not_start_with(unexpected)`                             |                                                                                                                                                     |                   |
| `String`                                  | `ends_with(expected)`                                         |                                                                                                                                                     |                   |
| `String`                                  | `does_not_end_with(unexpected)`                               |                                                                                                                                                     |                   |
| `&[T]`                                    | `contains(expected)`                                          |                                                                                                                                                     |                   |
| `&[T]`                                    | `does_not_contain(not_expected)`                              |                                                                                                                                                     |                   |
| `&[T]`                                    | `contains_exactly(expected)`                                  |                                                                                                                                                     |                   |
| `&[T]`                                    | `contains_exactly_in_any_order(expected)`                     |                                                                                                                                                     |                   |
| `&[T]`                                    | `contains_exactly_matching_in_any_order(expected)`            |                                                                                                                                                     |                   |
| `[T; N]`                                  | `contains(expected)`                                          |                                                                                                                                                     |                   |
| `[T; N]`                                  | `does_not_contain(not_expected)`                              |                                                                                                                                                     |                   |
| `[T; N]`                                  | `contains_exactly(expected)`                                  |                                                                                                                                                     |                   |
| `[T; N]`                                  | `contains_exactly_in_any_order(expected)`                     |                                                                                                                                                     |                   |
| `[T; N]`                                  | `contains_exactly_matching_in_any_order(expected)`            |                                                                                                                                                     |                   |
| `Vec<T>`                                  | `contains(expected)`                                          |                                                                                                                                                     |                   |
| `Vec<T>`                                  | `does_not_contain(not_expected)`                              |                                                                                                                                                     |                   |
| `Vec<T>`                                  | `contains_exactly(expected)`                                  |                                                                                                                                                     |                   |
| `Vec<T>`                                  | `contains_exactly_in_any_order(expected)`                     |                                                                                                                                                     |                   |
| `Vec<T>`                                  | `contains_exactly_matching_in_any_order(expected)`            |                                                                                                                                                     |                   |
| `VecDeque<T>`                             | `contains(expected)`                                          |                                                                                                                                                     |                   |
| `VecDeque<T>`                             | `does_not_contain(not_expected)`                              |                                                                                                                                                     |                   |
| `VecDeque<T>`                             | `contains_exactly(expected)`                                  |                                                                                                                                                     |                   |
| `VecDeque<T>`                             | `contains_exactly_in_any_order(expected)`                     |                                                                                                                                                     |                   |
| `VecDeque<T>`                             | `contains_exactly_matching_in_any_order(expected)`            |                                                                                                                                                     |                   |
| `T: Debug`                                | `has_debug_string(expected)`                                  |                                                                                                                                                     |                   |
| `T: Debug`                                | `has_debug_value(expected)`                                   |                                                                                                                                                     |                   |
| `T: Display`                              | `has_display_value(expected)`                                 |                                                                                                                                                     |                   |
| `F: FnOnce() -> R`                        | `panics()`                                                    | Panic mode only                                                                                                                                     | std               |
| `F: FnOnce() -> R`                        | `does_not_panic()`                                            | Panic mode only                                                                                                                                     | std               |
| `F: FnOnce() -> impl Future<Output = R>`  | `panics_async()`                                              | Panic mode only                                                                                                                                     | std               |
| `F: FnOnce() -> impl Future<Output = R>`  | `does_not_panic_async()`                                      | Panic mode only                                                                                                                                     | std               |
| `I: Iterator<Item = T>`                   | `contains(expected)`                                          | Terminal assertion                                                                                                                                  |                   |
| `I: Iterator<Item = T>`                   | `does_not_contain(not_expected)`                              | Terminal assertion                                                                                                                                  |                   |
| `I: Iterator<Item = T>`                   | `contains_exactly(expected)`                                  | Terminal assertion                                                                                                                                  |                   |
| `I where &I: IntoIterator<Item = &T>`     | `into_iter_contains(expected)`                                | Prefixed to avoid overlap with more specific collection assertions                                                                                  |                   |
| `I where &I: IntoIterator<Item = &T>`     | `into_iter_does_not_contain(not_expected)`                    | Prefixed to avoid overlap with more specific collection assertions                                                                                  |                   |
| `I where &I: IntoIterator<Item = &T>`     | `into_iter_contains_exactly(expected)`                        | Prefixed to avoid overlap with more specific collection assertions                                                                                  |                   |
| `I where &I: IntoIterator<Item = &T>`     | `into_iter_iterator_is_empty()`                               | Prefixed to avoid overlap with more specific collection assertions                                                                                  |                   |
| `T: HasLength`                            | `is_empty()`                                                  | Implemented for strings, slices, arrays, `Vec`/`VecDeque`, `HashMap`/`HashSet`, numeric ranges, and feature-gated rootcause collections/attachments |                   |
| `T: HasLength`                            | `is_not_empty()`                                              | Implemented for strings, slices, arrays, `Vec`/`VecDeque`, `HashMap`/`HashSet`, numeric ranges, and feature-gated rootcause collections/attachments |                   |
| `T: HasLength`                            | `has_length(expected)`                                        | Implemented for strings, slices, arrays, `Vec`/`VecDeque`, `HashMap`/`HashSet`, numeric ranges, and feature-gated rootcause collections/attachments |                   |
| `T: Num`                                  | `is_zero()`                                                   |                                                                                                                                                     | num               |
| `T: Num`                                  | `is_additive_identity()`                                      | Synonym for `is_zero`                                                                                                                               | num               |
| `T: Num`                                  | `is_one()`                                                    |                                                                                                                                                     | num               |
| `T: Num`                                  | `is_multiplicative_identity()`                                | Synonym for `is_one`                                                                                                                                | num               |
| `T: Num + Signed`                         | `is_negative()`                                               |                                                                                                                                                     | num               |
| `T: Num + Signed`                         | `is_positive()`                                               |                                                                                                                                                     | num               |
| `T: Num + PartialOrd + Clone`             | `is_close_to(expected, allowed_deviation)`                    |                                                                                                                                                     | num               |
| `T: Num + Float`                          | `is_nan()`                                                    | Requires either `std` or `libm` in addition to `num`                                                                                                | num               |
| `T: Num + Float`                          | `is_finite()`                                                 | Requires either `std` or `libm` in addition to `num`                                                                                                | num               |
| `T: Num + Float`                          | `is_infinite()`                                               | Requires either `std` or `libm` in addition to `num`                                                                                                | num               |
| `Option<T>`                               | `is_some()`                                                   | Panic mode only                                                                                                                                     |                   |
| `Option<T>`                               | `is_some_satisfying(assertions)`                              |                                                                                                                                                     |                   |
| `Option<T>`                               | `is_none()`                                                   |                                                                                                                                                     |                   |
| `Poll<T>`                                 | `is_pending()`                                                |                                                                                                                                                     |                   |
| `Poll<T>`                                 | `is_ready()`                                                  | Panic mode only                                                                                                                                     |                   |
| `Poll<T>`                                 | `is_ready_satisfying(assertions)`                             |                                                                                                                                                     |                   |
| `R: RangeBounds<B>, B: PartialOrd`        | `contains_element(expected)`                                  |                                                                                                                                                     |                   |
| `R: RangeBounds<B>, B: PartialOrd`        | `does_not_contain_element(expected)`                          |                                                                                                                                                     |                   |
| `B: PartialOrd`                           | `is_in_range(expected)`                                       |                                                                                                                                                     |                   |
| `B: PartialOrd`                           | `is_not_in_range(expected)`                                   |                                                                                                                                                     |                   |
| `B: PartialOrd`                           | `is_outside_of_range(expected)`                               | Synonym for `is_not_in_range`                                                                                                                       |                   |
| `RefCell<T>`                              | `is_borrowed()`                                               |                                                                                                                                                     |                   |
| `RefCell<T>`                              | `is_mutably_borrowed()`                                       |                                                                                                                                                     |                   |
| `RefCell<T>`                              | `is_not_mutably_borrowed()`                                   |                                                                                                                                                     |                   |
| `Mutex<T>`                                | `is_locked()`                                                 |                                                                                                                                                     | std               |
| `Mutex<T>`                                | `is_not_locked()`                                             |                                                                                                                                                     | std               |
| `Mutex<T>`                                | `is_free()`                                                   | Synonym for `is_not_locked`                                                                                                                         | std               |
| `Result<T, E>`                            | `is_ok()`                                                     | Panic mode only                                                                                                                                     |                   |
| `Result<T, E>`                            | `is_err()`                                                    | Panic mode only                                                                                                                                     |                   |
| `Result<T, E>`                            | `is_ok_satisfying(assertions)`                                |                                                                                                                                                     |                   |
| `Result<T, E>`                            | `is_err_satisfying(assertions)`                               |                                                                                                                                                     |                   |
| `PathBuf`                                 | `exists()`                                                    |                                                                                                                                                     | std               |
| `PathBuf`                                 | `does_not_exist()`                                            |                                                                                                                                                     | std               |
| `PathBuf`                                 | `is_a_file()`                                                 |                                                                                                                                                     | std               |
| `PathBuf`                                 | `is_a_directory()`                                            |                                                                                                                                                     | std               |
| `PathBuf`                                 | `is_a_symlink()`                                              |                                                                                                                                                     | std               |
| `PathBuf`                                 | `has_a_root()`                                                |                                                                                                                                                     | std               |
| `PathBuf`                                 | `is_relative()`                                               |                                                                                                                                                     | std               |
| `PathBuf`                                 | `has_file_name(expected)`                                     |                                                                                                                                                     | std               |
| `PathBuf`                                 | `has_file_stem(expected)`                                     |                                                                                                                                                     | std               |
| `PathBuf`                                 | `has_extension(expected)`                                     |                                                                                                                                                     | std               |
| `PathBuf`                                 | `starts_with(expected)`                                       |                                                                                                                                                     | std               |
| `PathBuf`                                 | `ends_with(expected)`                                         |                                                                                                                                                     | std               |
| `&Path`                                   | `exists()`                                                    |                                                                                                                                                     | std               |
| `&Path`                                   | `does_not_exist()`                                            |                                                                                                                                                     | std               |
| `&Path`                                   | `is_a_file()`                                                 |                                                                                                                                                     | std               |
| `&Path`                                   | `is_a_directory()`                                            |                                                                                                                                                     | std               |
| `&Path`                                   | `is_a_symlink()`                                              |                                                                                                                                                     | std               |
| `&Path`                                   | `has_a_root()`                                                |                                                                                                                                                     | std               |
| `&Path`                                   | `is_relative()`                                               |                                                                                                                                                     | std               |
| `&Path`                                   | `has_file_name(expected)`                                     |                                                                                                                                                     | std               |
| `&Path`                                   | `has_file_stem(expected)`                                     |                                                                                                                                                     | std               |
| `&Path`                                   | `has_extension(expected)`                                     |                                                                                                                                                     | std               |
| `&Path`                                   | `starts_with(expected)`                                       |                                                                                                                                                     | std               |
| `&Path`                                   | `ends_with(expected)`                                         |                                                                                                                                                     | std               |
| `HashMap<K, V>`                           | `contains_key(expected)`                                      |                                                                                                                                                     | std               |
| `HashMap<K, V>`                           | `does_not_contain_key(not_expected)`                          |                                                                                                                                                     | std               |
| `HashMap<K, V>`                           | `contains_value(expected)`                                    |                                                                                                                                                     | std               |
| `HashMap<K, V>`                           | `does_not_contain_value(not_expected)`                        |                                                                                                                                                     | std               |
| `HashMap<K, V>`                           | `contains_entry(expected_key, expected_value)`                |                                                                                                                                                     | std               |
| `HashMap<K, V>`                           | `does_not_contain_entry(unexpected_key, unexpected_value)`    |                                                                                                                                                     | std               |
| `HashMap<K, V>`                           | `contains_keys(expected)`                                     |                                                                                                                                                     | std               |
| `HashMap<K, V>`                           | `contains_exactly_entries(expected)`                          |                                                                                                                                                     | std               |
| `HashSet<T>`                              | `contains(expected)`                                          |                                                                                                                                                     | std               |
| `HashSet<T>`                              | `does_not_contain(not_expected)`                              |                                                                                                                                                     | std               |
| `HashSet<T>`                              | `contains_all(expected)`                                      |                                                                                                                                                     | std               |
| `HashSet<T>`                              | `is_subset_of(expected_superset)`                             |                                                                                                                                                     | std               |
| `HashSet<T>`                              | `is_superset_of(expected_subset)`                             |                                                                                                                                                     | std               |
| `HashSet<T>`                              | `is_disjoint_from(other)`                                     |                                                                                                                                                     | std               |
| `Command`                                 | `has_arg(expected)`                                           |                                                                                                                                                     | std               |
| `Type<T>`                                 | `needs_drop()`                                                |                                                                                                                                                     | std               |
| `Type<T>`                                 | `need_drop()`                                                 | Synonym for `needs_drop`                                                                                                                            | std               |
| `Box<dyn Any>`                            | `has_type::<Expected>()`                                      | Panic mode only                                                                                                                                     |                   |
| `Box<dyn Any>`                            | `has_type_ref::<Expected>()`                                  | Panic mode only                                                                                                                                     |                   |
| `PanicValue`                              | `has_type::<Expected>()`                                      | Panic mode only                                                                                                                                     |                   |
| `PanicValue`                              | `has_type_ref::<Expected>()`                                  | Panic mode only                                                                                                                                     |                   |
| `http::HeaderValue`                       | `is_empty()`                                                  |                                                                                                                                                     | http              |
| `http::HeaderValue`                       | `is_not_empty()`                                              |                                                                                                                                                     | http              |
| `http::HeaderValue`                       | `is_sensitive()`                                              |                                                                                                                                                     | http              |
| `http::HeaderValue`                       | `is_insensitive()`                                            |                                                                                                                                                     | http              |
| `http::HeaderValue`                       | `is_ascii_satisfying(assertions)`                             |                                                                                                                                                     | http              |
| `http::HeaderValue`                       | `is_ascii()`                                                  | Panic mode only                                                                                                                                     | http              |
| `tokio::sync::Mutex<T>`                   | `is_locked()`                                                 |                                                                                                                                                     | tokio             |
| `tokio::sync::Mutex<T>`                   | `is_not_locked()`                                             |                                                                                                                                                     | tokio             |
| `tokio::sync::Mutex<T>`                   | `is_free()`                                                   | Synonym for `is_not_locked`                                                                                                                         | tokio             |
| `tokio::sync::RwLock<T>`                  | `is_not_locked()`                                             |                                                                                                                                                     | tokio             |
| `tokio::sync::RwLock<T>`                  | `is_free()`                                                   | Synonym for `is_not_locked`                                                                                                                         | tokio             |
| `tokio::sync::RwLock<T>`                  | `is_read_locked()`                                            |                                                                                                                                                     | tokio             |
| `tokio::sync::RwLock<T>`                  | `is_write_locked()`                                           |                                                                                                                                                     | tokio             |
| `tokio::sync::watch::Receiver<T>`         | `has_current_value(expected)`                                 |                                                                                                                                                     | tokio             |
| `tokio::sync::watch::Receiver<T>`         | `has_changed()`                                               | Panic mode only                                                                                                                                     | tokio             |
| `tokio::sync::watch::Receiver<T>`         | `has_not_changed()`                                           | Panic mode only                                                                                                                                     | tokio             |
| `reqwest::Response`                       | `has_status_code(expected)`                                   |                                                                                                                                                     | reqwest           |
| `Program<'a>`                             | `exists()`                                                    |                                                                                                                                                     | program           |
| `Program<'a>`                             | `exists_and()`                                                | Panic mode only                                                                                                                                     | program           |
| `rootcause::ReportCollection<C, T>`       | `is_empty()`                                                  | via `HasLength`                                                                                                                                     | rootcause         |
| `rootcause::ReportCollection<C, T>`       | `is_not_empty()`                                              | via `HasLength`                                                                                                                                     | rootcause         |
| `rootcause::ReportCollection<C, T>`       | `has_length(expected)`                                        | via `HasLength`                                                                                                                                     | rootcause         |
| `rootcause::ReportAttachments<T>`         | `is_empty()`                                                  | via `HasLength`                                                                                                                                     | rootcause         |
| `rootcause::ReportAttachments<T>`         | `is_not_empty()`                                              | via `HasLength`                                                                                                                                     | rootcause         |
| `rootcause::ReportAttachments<T>`         | `has_length(expected)`                                        | via `HasLength`                                                                                                                                     | rootcause         |
| `rootcause::Report<C, O, T>`              | `has_child_count(expected)`                                   |                                                                                                                                                     | rootcause         |
| `rootcause::Report<C, O, T>`              | `has_attachment_count(expected)`                              |                                                                                                                                                     | rootcause         |
| `rootcause::Report<C, O, T>`              | `has_current_context_type::<Expected>()`                      |                                                                                                                                                     | rootcause         |
| `rootcause::Report<C, O, T>`              | `has_current_context_display_value(expected)`                 |                                                                                                                                                     | rootcause         |
| `rootcause::Report<C, O, T>`              | `has_current_context_debug_string(expected)`                  |                                                                                                                                                     | rootcause         |
| `rootcause::Report<Dynamic, O, T>`        | `has_current_context_satisfying::<Expected>(...)`             |                                                                                                                                                     | rootcause         |
| `rootcause::Report<Dynamic, O, T>`        | `has_current_context::<Expected>()`                           | Panic mode only                                                                                                                                     | rootcause         |
| `rootcause::ReportRef<'a, C, O, T>`       | Same as corresponding `rootcause::Report<C, O, T>` rows       |                                                                                                                                                     | rootcause         |
| `rootcause::ReportRef<'a, Dynamic, O, T>` | Same as corresponding `rootcause::Report<Dynamic, O, T>` rows |                                                                                                                                                     | rootcause         |
| `jiff::SignedDuration`                    | `is_zero()`                                                   |                                                                                                                                                     | jiff              |
| `jiff::SignedDuration`                    | `is_negative()`                                               |                                                                                                                                                     | jiff              |
| `jiff::SignedDuration`                    | `is_positive()`                                               |                                                                                                                                                     | jiff              |
| `jiff::SignedDuration`                    | `is_close_to(expected, allowed_deviation)`                    |                                                                                                                                                     | jiff              |
| `jiff::Span`                              | `is_zero()`                                                   |                                                                                                                                                     | jiff              |
| `jiff::Span`                              | `is_negative()`                                               |                                                                                                                                                     | jiff              |
| `jiff::Span`                              | `is_positive()`                                               |                                                                                                                                                     | jiff              |
| `jiff::Zoned`                             | `is_in_time_zone(expected)`                                   |                                                                                                                                                     | jiff              |
| `jiff::Zoned`                             | `is_in_time_zone_named(expected)`                             |                                                                                                                                                     | jiff              |

*The generic types (`T`, `E`, ...) nearly always also require `Debug`. Otherwise the library could
not print useful failure output. We chose not to list those bounds everywhere in the table above.

For `PartialOrd` assertions, unordered comparisons fail. That matters most for floating-point values like `NaN`, where
`partial_cmp()` returns `None`.

### Conditions

- `is(condition)` / `has(condition)`: Assert that a value satisfies a reusable `Condition<T>`.
- `are(condition)` / `have(condition)`: Assert that every element of an iterable satisfies a condition.

### Derived Assertions

Use derived assertions to map the current subject to one of its fields or views and then keep
asserting on the derived value:

```rust
use assertr::prelude::*;

#[derive(Debug)]
struct Person {
    age: u32,
}

#[test]
fn test() {
    assert_that!(Person { age: 30 }).satisfies(
        |person| person.age,
        |age| age.is_greater_or_equal_to(18),
    );
}
```

- `satisfies(mapper, assertions)`: Derive an owned view for nested assertions.
- `satisfies_ref(mapper, assertions)`: Derive a borrowed view for nested assertions.

## Advanced Features

### Capture Mode

Instead of immediately panicking on assertion failure, you can capture failures for later analysis:

```rust
#[test]
fn test() {
    let failures = assert_that!(3)
        .with_capture()
        .is_equal_to(4)
        .is_less_than(2)
        .capture_failures();

    assert_that!(failures).has_length(2);
}
```

With the `fluent` feature enabled, the same pattern can start from `.verify()` instead of
`assert_that!(...).with_capture()`.

### Partial equality assertions

You can derive a helper struct for partial equality comparisons by annotating an owned struct with
`#[derive(AssertrEq)]`.

**Make sure this crate's `derive` feature is enabled.**

```toml
assertr = { version = "0.5.7", features = ["derive"] }
```

```rust
// Deriving `AssertrEq` provides an additional `PersonAssertrEq` type.
// Deriving `Debug` is necessary because `Person` is used as an assertion subject.
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
    assert_that!(&alice).is_equal_to(Person {
        name: "Alice".to_owned(),
        age: 30,
        data: (100, 998),
    });

    // But we can also do a partial equality check!
    assert_that!(&alice).is_equal_to(PersonAssertrEq {
        name: eq("Alice".to_owned()),
        age: any(), // Match any age
        data: any() // Match any data
    });
}
```

### Write assertions for your own types.

Good custom assertions add domain-specific value. In practice, the most maintainable way to build
them is often to delegate to existing assertions so you keep the same failure formatting,
capture-mode behavior, and chaining style.

```rust
use assertr::prelude::*;

#[derive(Debug)]
struct Person {
    age: u32,
}

trait PersonAssertions {
    fn is_adult(self) -> Self;
}

impl<M: Mode> PersonAssertions for AssertThat<'_, Person, M> {
    #[track_caller]
    fn is_adult(self) -> Self {
        self.satisfies(|person| person.age, |age| {
            age.is_greater_or_equal_to(18);
        })
    }
}

#[test]
fn test() {
    assert_that!(Person { age: 30 })
        .is_adult();
}
```

### Type Testing

Test properties of types:

```rust
#[test]
fn test() {
    assert_that_type::<MyType>()
        .needs_drop()
        .satisfies(|it| it.size(), |size| {
            size.is_equal_to(32);
        });
}
```

## Examples

```rust
#[test]
fn test() {
    // Assertions that read like English.
    assert_that!("foobar").starts_with("foo").contains("ooba");
    assert_that!(vec![1, 2, 3]).has_length(3).contains(2);
    assert_that!(Ok(42)).is_ok().is_equal_to(42);
    assert_that!(Some(42)).has_debug_string("Some(42)");

    // Chainable.
    assert_that!("foobar")
        .is_not_empty()
        .starts_with("foo")
        .ends_with("bar")
        .has_length(6);
}
```

- Partial equality assertions (meaning that only some fields of a struct are compared, while some are ignored).
  Add the `AssertrEq` annotation to one of your structs to enable this.

## Compared to other assertion styles

One other style of assertions in Rust is the "individual macros" approach.
The standard library already comes with a few of them, like the `assert_eq!` macro, many libraries provide a more
exhaustive list of macros specifically tailored for specific types and operations.

Let me point out a few benefits of fluent assertions compared to individual assert macros.

#### Chainability and Readability

The fluent interface allows you to chain multiple assertions naturally, following the way we think about validating
properties. Instead of writing multiple separate assertions, you can express related checks in a single, flowing
statement that reads almost like natural language.

Additionally, having a concrete entrance into the assertion context using the `assert_that!` macro with assertions
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
#[test]
fn test() {
    let vec = vec![1, 2, 3];
    assert_eq!(vec.len(), 3);
    assert!(vec.contains(&2));
}
```

Versus the fluent style:

```rust
#[test]
fn test() {
    assert_that!(vec![1, 2, 3]).has_length(3).contains(2);
}
```

## Technical decisions

- Derived assertions are not allowed to control whether the location is printed.
- Detail messages are collected from the current assertion upwards, taking the messages of all parents into account.
- Failures are stored at the root assertion.
- Failures can only be extracted from the root assertion.
- Assertions to be defined on common traits as often as possible. Allowing, for example, all types implementing `Eq`
  to allow `is_equal_to`, `PartialOrd` types to allow `is_greater_than` assertions and all types implementing the
  `HasLength` trait to support the `has_length` assertion.
- A single `assert_that!(...)` macro suffices to get into an assertion context. It handles both owned values
  and references automatically — pass `assert_that!(value)` for owned values or `assert_that!(&value)` to borrow.
- One import should be enough to access all possible assertions through **autocomplete**.
  `use assertr::prelude::*;`

## Dev

Run the full maintenance pipeline, including formatting, checks, clippy, tests, and docs with:
(This will run checks and tests with default features, no default features and all features enabled.)

```bash
just tidy
```

### Testing

To test all crates from the workspace root:

```bash
cargo test --all
```

Some assertions are feature-gated. To run the full test suite, enable all features:

```bash
cargo test --all-features
```

## MSRV

Current MSRV:

- `assertr`: `1.89.0`
- `assertr-derive`: `1.85.0`

Previous MSRV values:

- As of `0.1.0`, the MSRV was `1.76.0`
- As of `0.2.0`, the MSRV was `1.85.0`
- As of `0.4.0`, the MSRV was `1.89.0`

## Open questions

- Many assertions require `std::fmt::Debug`, limiting usability to types implementing Debug.
  Can we implement fallback rendering? Will probably require the currently unstable specialization feature.

## Contributing

Contributions are welcome. Feel free to open a PR with your suggested changes.

## Acknowledgements

Midway through implementing this, I found out that "spectral" already exists, which uses a very similar style of
assertions.
After looking into it, I took the concept of generally relying on `*Assertions` trait definitions instead of directly
implementing Assertions with multiple impl blocks on the `AssertThat` type.
