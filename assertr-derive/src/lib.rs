#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used)]
#![deny(missing_docs)]
#![allow(clippy::needless_continue)]
//! Procedural macros for `assertr`.
//!
//! The `derive` feature of `assertr` re-exports [`AssertrEq`]. [`fluent_aliases`] supports
//! assertion-trait authors and is used internally by `assertr` when the `fluent` feature is
//! enabled.

mod assertr_eq;
mod fluent_aliases;

use proc_macro::TokenStream;
use syn::{DeriveInput, ItemTrait, parse_macro_input};

/// Derives a companion matcher type for partial equality assertions.
///
/// Annotating `struct Person` with `#[derive(AssertrEq)]` generates `struct PersonAssertrEq`. The
/// matcher has one [`Eq<T>`] field for every public field of `Person`. Fill a field with
/// `eq(value)` to require that value, or with `any()` to ignore the field. Passing the
/// matcher to `is_equal_to` compares only the `eq` fields:
///
/// ```
/// use assertr::prelude::*;
///
/// #[derive(Debug, AssertrEq)]
/// pub struct Person {
///     pub name: String,
///     pub age: u32,
/// }
///
/// let alice = Person { name: "Alice".to_owned(), age: 30 };
///
/// assert_that!(alice).is_equal_to(PersonAssertrEq {
///     name: eq("Alice".to_owned()),
///     age: any(),
/// });
/// ```
///
/// The matcher implements `Default` with every field set to `any()`, so `..Default::default()`
/// leaves unspecified fields ignored. A failed comparison lists every mismatched field in
/// declaration order:
///
/// ```text
/// Details: [
///     Differences: [
///         "name": expected "Alicia", but was "Alice",
///     ],
/// ]
/// ```
///
/// # What is generated
///
/// - The matcher struct, `<Name>AssertrEq`, with the same generics as the source struct.
///   Lifetimes, type parameters, const generics, and where-clauses are carried over.
/// - `Default` and `Debug` for the matcher.
/// - `AssertrPartialEq<NameAssertrEq>` for `Name` and `&Name`, so the matcher works with
///   collections of values and collections of references.
///
/// Only public named fields take part. Private fields are neither present on the matcher nor
/// compared. Tuple structs, unit structs, and enums are rejected with a compile error. The source
/// struct needs `Debug` (or a custom renderer) so that it can be rendered on failure, but it does
/// not need `PartialEq`. The generated code also resolves a renamed `assertr` dependency.
///
/// # Nested structs: `map_type`
///
/// A matcher field is `Eq<FieldType>` by default and compares the whole field value. To match a
/// nested struct partially as well, point `map_type` at that struct's own matcher:
///
/// ```
/// use assertr::prelude::*;
///
/// #[derive(Debug, AssertrEq)]
/// pub struct Child {
///     pub id: i32,
/// }
///
/// #[derive(Debug, AssertrEq)]
/// pub struct Parent {
///     #[assertr_eq(map_type = "ChildAssertrEq")]
///     pub child: Child,
/// }
///
/// assert_that!(Parent { child: Child { id: 1 } }).is_equal_to(ParentAssertrEq {
///     child: eq(ChildAssertrEq { id: eq(1) }),
/// });
/// ```
///
/// # Collections: `compare_with` and `compare_bounds`
///
/// For a `Vec<Child>` field the expected type becomes `Vec<ChildAssertrEq>`, and there is no `==`
/// between the two. `compare_with` names the function that performs the comparison instead, and
/// `compare_bounds` adds the where-predicates that function needs. The derive does not inspect
/// field types, so it cannot infer either of them. Inside `compare_bounds`, `R` is the
/// renderer type parameter of the generated implementation. The two built-in comparison functions
/// each have a companion trait for their required bound:
///
/// ```
/// use assertr::prelude::*;
///
/// #[derive(Debug, AssertrEq)]
/// pub struct Child {
///     pub id: i32,
/// }
///
/// #[derive(Debug, AssertrEq)]
/// pub struct Parent {
///     #[assertr_eq(
///         map_type = "Vec<ChildAssertrEq>",
///         compare_with = "::assertr::cmp::slice::compare",
///         compare_bounds = "Child: ::assertr::cmp::slice::CompareElement<ChildAssertrEq, R>"
///     )]
///     pub children: Vec<Child>,
/// }
///
/// let parent = Parent {
///     children: vec![Child { id: 1 }, Child { id: 2 }],
/// };
///
/// assert_that!(parent).is_equal_to(ParentAssertrEq {
///     children: eq(vec![ChildAssertrEq { id: eq(1) }, ChildAssertrEq { id: any() }]),
/// });
/// ```
///
/// [`cmp::hashmap::compare`] does the same for `HashMap<K, Child>` fields, with
/// `map_type = "HashMap<K, ChildAssertrEq>"` and the bound
/// `Child: ::assertr::cmp::hashmap::CompareValue<ChildAssertrEq, R>`.
///
/// A custom `compare_with` function is called as `f(&actual_field, &expected_field, ctx)`, where
/// `ctx` is an `Option<&mut EqContext<'_, R>>`, and returns `bool`. Record what differed through
/// the context so that the failure message can show it. `compare_bounds` takes any where-predicate
/// syntax, with several predicates separated by commas.
///
/// [`Eq<T>`]: https://docs.rs/assertr/latest/assertr/enum.Eq.html
/// [`AssertrPartialEq`]: https://docs.rs/assertr/latest/assertr/trait.AssertrPartialEq.html
/// [`cmp::hashmap::compare`]: https://docs.rs/assertr/latest/assertr/cmp/hashmap/fn.compare.html
#[proc_macro_derive(AssertrEq, attributes(assertr_eq))]
pub fn derive_assertr_eq(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    assertr_eq::derive_assertr_eq_impl(&input).into()
}

/// Attribute macro that generates fluent aliases for assertion trait methods.
///
/// Place on a trait definition to auto-generate `be_*` aliases for `is_*` methods,
/// `have_*` aliases for `has_*` methods, and imperative forms for other third-person
/// verbs (`contains` -> `contain`, `starts_with` -> `start_with`, `panics` -> `panic`, and
/// `needs_*` -> `need_*`). Negated methods put `not` first: `is_not_*` -> `not_be_*`,
/// `has_not_*` -> `not_have_*`, and `does_not_*` -> `not_*`. Namespace prefixes such as
/// `into_iter_` stay at the front. Methods beginning with `get_` are already imperative and get no
/// alias.
///
/// Generated aliases are gated by `#[cfg(feature = "fluent")]`. Their documentation links to the
/// original method, and they inherit its documentation and attributes, including `must_use` and
/// `deprecated`.
///
/// Use `#[fluent_alias("custom_name")]` on a method for a custom alias name.
/// Use `#[no_fluent_alias]` on a method to skip alias generation.
#[proc_macro_attribute]
pub fn fluent_aliases(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let trait_def = parse_macro_input!(item as ItemTrait);
    fluent_aliases::fluent_aliases_impl(trait_def).into()
}
