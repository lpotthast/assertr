#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used)]
#![allow(clippy::needless_continue)]

mod assertr_eq;
mod fluent_aliases;

use proc_macro::TokenStream;
use syn::{DeriveInput, ItemTrait, parse_macro_input};

/// Derive macro for `AssertrEq`.
#[proc_macro_derive(AssertrEq, attributes(assertr_eq))]
pub fn derive_assertr_eq(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    assertr_eq::derive_assertr_eq_impl(&input).into()
}

/// Attribute macro that generates fluent aliases for assertion trait methods.
///
/// Place on a trait definition to auto-generate `be_*` aliases for `is_*` methods
/// and `have_*` aliases for `has_*` methods. Generated aliases are gated behind
/// `#[cfg(feature = "fluent")]`.
///
/// Use `#[fluent_alias("custom_name")]` on a method for a custom alias name.
/// Use `#[no_fluent_alias]` on a method to skip alias generation.
#[proc_macro_attribute]
pub fn fluent_aliases(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let trait_def = parse_macro_input!(item as ItemTrait);
    fluent_aliases::fluent_aliases_impl(trait_def).into()
}
