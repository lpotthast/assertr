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
mod fluent_expressions;

use proc_macro::TokenStream;
use syn::{DeriveInput, Item, ItemTrait, parse_macro_input};

/// Derives a companion matcher type for partial equality assertions.
///
/// Annotating `struct Person` with `#[derive(AssertrEq)]` generates `struct PersonAssertrEq`. The
/// matcher has one [`Eq<T>`] field for every public field of `Person`. Fill a field with
/// `eq(value)` to require that value, or with `any()` to ignore the field. Passing the
/// matcher to `is_equal_to` compares only the `eq` fields:
///
/// ```
/// # extern crate renamed_assertr as assertr;
/// # use assertr_derive::AssertrEq;
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
/// Details:
///   - Differences: [
///         "name": expected "Alicia", but was "Alice",
///     ]
/// ```
///
/// # What is generated
///
/// - The matcher struct, `<Name>AssertrEq`, with the source generics required by its public
///   fields. Parameters and dependent bounds used only by private fields are omitted.
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
/// # extern crate renamed_assertr as assertr;
/// # use assertr_derive::AssertrEq;
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
/// # extern crate renamed_assertr as assertr;
/// # use assertr_derive::AssertrEq;
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

/// Captures receiver expressions for fluent assertion entry points in a test scope.
///
/// Place this attribute on a test function or an inline test module. It rewrites syntactically
/// visible `value.must()` and `value.must_owned()` calls to attach `stringify!(value)`, and wraps
/// the callbacks passed to visible `value.verify(...)` and `value.verify_owned(...)` calls with
/// macro-only expression-aware support. Calls outside the annotated scope remain unchanged.
///
/// A macro invocation can be the receiver, as in `fixture!().must()`, because the fluent call is
/// visible to this attribute. The attribute cannot inspect later macro expansion, so a macro that
/// itself expands to `value.must()` or `value.verify(...)` does not gain expression capture.
///
/// Put this attribute above `#[test]` and proc-macro test attributes such as `#[tokio::test]` or
/// `#[rstest]`, so expression capture runs before those attributes transform the function body:
///
/// ```ignore
/// #[assertr::fluent_expressions]
/// #[test]
/// fn reports_the_receiver() {
///     response.status().must().be_equal_to(200);
/// }
/// ```
///
/// The rewrite keeps ordinary method resolution. A user-defined zero-argument `must` method is
/// still called, after which the generated expression attachment fails to compile if its return
/// type is not an assertion chain. User-defined `verify` and `verify_owned` methods likewise remain
/// selected, and callback inputs unrelated to assertr pass through unchanged.
#[proc_macro_attribute]
pub fn fluent_expressions(attr: TokenStream, item: TokenStream) -> TokenStream {
    if !attr.is_empty() {
        return syn::Error::new(
            proc_macro2::Span::call_site(),
            "fluent_expressions does not accept arguments",
        )
        .into_compile_error()
        .into();
    }

    let item = parse_macro_input!(item as Item);
    fluent_expressions::fluent_expressions_impl(item)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}
