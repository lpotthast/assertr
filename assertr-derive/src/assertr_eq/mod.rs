//! Implementation of the `AssertrEq` derive macro.

mod comparison;
mod expand;
mod generics;
mod identifiers;
mod input;
mod matcher;
mod rendering;

use proc_macro2::TokenStream;
use syn::DeriveInput;

/// Expands a parsed derive input into its matcher type and equality implementations.
pub(super) fn derive_assertr_eq_impl(input: &DeriveInput) -> TokenStream {
    expand::derive_assertr_eq_impl(input)
}
