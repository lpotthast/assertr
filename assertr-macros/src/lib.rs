#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used)]
#![deny(missing_docs)]
#![allow(clippy::needless_continue)]
//! Procedural macros for `assertr`.
//!
//! The `matchers` feature of `assertr` re-exports [`partial`]. [`fluent_aliases`] supports
//! assertion-trait authors and is used internally by `assertr` when the `fluent` feature is
//! enabled.

mod fluent_aliases;
mod fluent_expressions;

use proc_macro::TokenStream;
use syn::{Item, ItemTrait, parse_macro_input};

/// Attribute macro that generates fluent aliases for assertion trait methods.
///
/// Place on a trait definition to auto-generate `be_*` aliases for `is_*` methods, `have_*` aliases
/// for `has_*` methods, and imperative forms for other third-person verbs (`contains` -> `contain`,
/// `starts_with` -> `start_with`, `panics` -> `panic`, and `needs_*` -> `need_*`). Negated methods
/// put `not` first: `is_not_*` -> `not_be_*`, `has_not_*` -> `not_have_*`, and `does_not_*` ->
/// `not_*`. Namespace prefixes such as `into_iter_` stay at the front. Methods beginning with
/// `get_` are already imperative and get no alias.
///
/// Generated aliases are gated by `#[cfg(feature = "fluent")]`. Their documentation links to the
/// original method, and they inherit its documentation and attributes, including `must_use` and
/// `deprecated`.
///
/// Use `#[fluent_alias("custom_name")]` on a method for a custom alias name. Use
/// `#[no_fluent_alias]` on a method to skip alias generation.
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
/// selected, and callback inputs unrelated to assertr pass through unchanged. Rewritten callbacks
/// retain their `Fn`, `FnMut`, or `FnOnce` capabilities.
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

mod partial;
/// Constructs a partial matcher without annotating the production type.
///
/// Use through `assertr::partial!` with the `matchers` feature enabled. Pass the result to
/// `.matches(...)` or a collection assertion such as `.contains_matching(...)`.
///
/// List the fields that matter to the test and use `..` to ignore the rest. A plain expected
/// value uses ordinary `PartialEq`. A field can instead use a matcher, another `partial!`, or
/// `assertr::matchers::satisfying` to check it with existing assertion methods. The built-in
/// matchers provide selected constraints rather than a counterpart for every assertion.
///
///
/// Without `..`, every field must be listed. Omitted fields need no comparison or rendering
/// support. Neither the whole type nor ignored fields need `PartialEq` or `Debug`, and private
/// fields follow ordinary Rust visibility rules. Field expectation expressions are evaluated
/// once when the matcher is constructed. Pass `&matcher` to reuse it.
///
/// Named structs, tuple structs, enum variants, and unit constructors are supported. Tuple `_`
/// positions are wildcards, and a tuple `..` must be final. Nested collection and map expectations
/// use `elements_are!`, `elements_are_in_any_order!`, `each`, and `entries_are!`.
///
/// See the [partial matching guide](https://docs.rs/assertr/latest/assertr/matchers/index.html)
/// for field constraints, nested examples, collection policies, and diagnostics.
#[proc_macro]
pub fn partial(input: TokenStream) -> TokenStream {
    partial::expand(input.into())
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}
