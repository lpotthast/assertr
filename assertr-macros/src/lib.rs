#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used)]
#![deny(missing_docs)]
//! Procedural macros for `assertr`.
//!
//! Use these macros through `assertr`, which re-exports them behind its features:
//!
//! - [`fluent_expressions`], available as `assertr::fluent_expressions` with the `fluent` feature,
//!   captures receiver expressions of fluent entry calls in a test scope.
//! - `partial!`, available as `assertr::partial!` with the `partial` feature, builds structural
//!   matchers. Its documentation lives in `assertr`.
//! - [`fluent_aliases`], available as `assertr::fluent_aliases` with the `fluent` feature,
//!   generates fluent aliases for assertion traits, including custom ones.

mod fluent_aliases;
mod fluent_expressions;
mod partial;

use proc_macro::TokenStream;
use syn::{Item, ItemTrait, parse_macro_input};

/// Attribute macro that generates fluent aliases for assertion trait methods.
///
/// Place it on a trait definition. Each eligible method gets a delegating alias named by these
/// rules:
///
/// - `is_x` -> `be_x` and `has_x` -> `have_x`. The possessive `has_no_x` keeps its order as
///   `have_no_x`.
/// - Negations put `not` first, as in "must not be equal to": `is_not_x` -> `not_be_x`, `has_not_x`
///   -> `not_have_x`, and `does_not_x` -> `not_x`.
/// - `contains`, `exists`, `panics`, and `satisfies` turn imperative, alone or as a prefix:
///   `contains` -> `contain`, `exists` -> `exist`, `panics_async` -> `panic_async`, `satisfies` ->
///   `satisfy`.
/// - The prefixes `starts_`, `ends_`, and `needs_` turn imperative: `starts_with` -> `start_with`,
///   `ends_with` -> `end_with`, `needs_drop` -> `need_drop`.
/// - `matches` -> `match_expectation`, because `match` is a keyword and `be_matching` belongs to
///   `is_matching`.
/// - Names starting with `get_` are already imperative and get no alias. Other names get an alias
///   only when one is given explicitly.
///
/// Use `#[fluent_alias("custom_name")]` on a method for a custom alias name. Keywords become raw
/// identifiers. Use `#[no_fluent_alias]` on a method to skip alias generation. A method takes at
/// most one of these helpers. Repeated or combined helpers, helpers on items other than methods,
/// and aliases that collide with another item of the trait are compile errors. A method gated by
/// `#[cfg]` is checked only against items without `#[cfg]`, because gated items may share a name
/// across configurations.
///
/// Aliases are documented as aliases of their original method. They copy its other attributes,
/// including `must_use`, `deprecated`, and `cfg`, track the caller, and require `Self: Sized`.
/// The aliases themselves are not feature-gated. To make them optional, apply the attribute and
/// its helper attributes conditionally:
///
/// ```ignore
/// #[cfg_attr(feature = "fluent", assertr::fluent_aliases)]
/// pub trait ReadinessAssertions {
///     fn is_ready(self) -> Self; // Alias: `be_ready`.
///
///     #[cfg_attr(feature = "fluent", fluent_alias("be_set_up"))]
///     fn is_initialized(self) -> Self;
/// }
/// ```
#[proc_macro_attribute]
pub fn fluent_aliases(attr: TokenStream, item: TokenStream) -> TokenStream {
    if !attr.is_empty() {
        return syn::Error::new(
            proc_macro2::Span::call_site(),
            "fluent_aliases does not accept arguments",
        )
        .into_compile_error()
        .into();
    }

    let trait_def = parse_macro_input!(item as ItemTrait);
    fluent_aliases::fluent_aliases_impl(trait_def).into()
}

/// Captures receiver expressions for fluent assertion entry points in a test scope.
///
/// Place this attribute on a test function or an inline test module. Failures from visible
/// `value.must()`, `value.must_owned()`, `value.verify(...)`, and `value.verify_owned(...)` calls
/// in that scope then report `value` as their expression, as `assert_that!(value)` does. Calls
/// outside the annotated scope remain unchanged.
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
/// A macro invocation can be the receiver, as in `fixture!().must()`. The attribute cannot inspect
/// later macro expansion, so a macro that itself expands to `value.must()` or `value.verify(...)`
/// does not gain expression capture.
///
/// The attribute recognizes entry calls by method name and keeps ordinary method resolution.
/// Callback arguments are never rewritten, so they keep their type, call traits, and coercions.
/// For `verify` and `verify_owned`, the expression reaches failures raised directly on the
/// callback's input chain. Derived chains keep their own expressions, and results of other methods
/// with these names are returned unchanged. A
/// user-defined zero-argument `must` or `must_owned` method that does not return an assertion chain
/// fails to compile, because its result receives the expression too. Keep such calls outside
/// annotated scopes.
///
/// Limitation: a user-defined `#[track_caller]` `verify` or `verify_owned` method that returns the
/// failures of an inner Assertr verification reports them at its own call site. Those failures
/// therefore receive the outer receiver expression.
///
/// # Runtime path
///
/// Generated code refers to `assertr`'s runtime support. By default, the attribute finds the
/// `assertr` dependency in the calling crate's manifest, including under a renamed dependency key.
/// A crate that reaches `assertr` only through a re-export, such as a facade crate declaring
/// `pub use assertr;`, names the re-exported path with the `crate` argument:
///
/// ```ignore
/// #[my_facade::assertr::fluent_expressions(crate = ::my_facade::assertr)]
/// #[test]
/// fn reports_the_receiver() {
///     response.status().must().be_equal_to(200);
/// }
/// ```
///
/// The path is unquoted and resolves where the annotated calls appear, so prefer an absolute path
/// such as `::my_facade::assertr` or `crate::support::assertr`. Unknown or repeated arguments are
/// rejected.
#[proc_macro_attribute]
pub fn fluent_expressions(attr: TokenStream, item: TokenStream) -> TokenStream {
    let arguments = parse_macro_input!(attr as fluent_expressions::Arguments);
    let item = parse_macro_input!(item as Item);
    fluent_expressions::fluent_expressions_impl(arguments, item)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

/// Implementation of `assertr::partial!`, which forwards its `$crate` path before a `;`.
///
/// Not a public API. Use `assertr::partial!` instead.
#[doc(hidden)]
#[proc_macro]
pub fn __partial(input: TokenStream) -> TokenStream {
    partial::expand(input.into())
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}
