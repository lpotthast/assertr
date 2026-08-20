//! Construction of delegating fluent alias methods.

use proc_macro2::{Ident, Span, TokenStream};
use quote::ToTokens;
use syn::{FnArg, GenericParam, Pat, TraitItemFn};

/// Clones a trait method and turns it into a feature-gated delegating alias.
///
/// Only conditional-compilation attributes are copied. The alias receives `track_caller` and a
/// `Self: Sized` bound, then forwards the original generic and value arguments unchanged.
pub(super) fn generate_alias(original: &TraitItemFn, alias_name: &str) -> TraitItemFn {
    let mut alias = original.clone();
    alias.sig.ident = Ident::new(alias_name, Span::call_site());

    alias
        .attrs
        .retain(|attribute| attribute.path().is_ident("cfg"));
    alias
        .attrs
        .insert(0, syn::parse_quote! { #[cfg(feature = "fluent")] });
    alias.attrs.push(syn::parse_quote! { #[track_caller] });
    alias
        .sig
        .generics
        .make_where_clause()
        .predicates
        .push(syn::parse_quote! { Self: Sized });

    let original_name = &original.sig.ident;
    let generics = generic_arguments(original);
    let arguments = value_arguments(original);
    alias.default = if generics.is_empty() {
        Some(syn::parse_quote! {
            { self.#original_name(#(#arguments),*) }
        })
    } else {
        Some(syn::parse_quote! {
            { self.#original_name::<#(#generics),*>(#(#arguments),*) }
        })
    };
    alias.semi_token = None;
    alias
}

/// Converts declared generic parameters into turbofish arguments for delegation.
fn generic_arguments(method: &TraitItemFn) -> Vec<TokenStream> {
    method
        .sig
        .generics
        .params
        .iter()
        .map(|parameter| match parameter {
            GenericParam::Lifetime(parameter) => parameter.lifetime.to_token_stream(),
            GenericParam::Type(parameter) => parameter.ident.to_token_stream(),
            GenericParam::Const(parameter) => parameter.ident.to_token_stream(),
        })
        .collect()
}

/// Collects simple identifier arguments forwarded by the generated method body.
fn value_arguments(method: &TraitItemFn) -> Vec<&Ident> {
    method
        .sig
        .inputs
        .iter()
        .filter_map(|argument| match argument {
            FnArg::Receiver(_) => None,
            FnArg::Typed(argument) => match &*argument.pat {
                Pat::Ident(pattern) => Some(&pattern.ident),
                _ => None,
            },
        })
        .collect()
}
