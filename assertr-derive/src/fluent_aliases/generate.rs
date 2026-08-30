//! Construction of delegating fluent alias methods.

use std::collections::BTreeSet;

use proc_macro2::{Ident, Span, TokenStream};
use quote::ToTokens;
use syn::{FnArg, GenericParam, Pat, TraitItemFn};

/// Clones a trait method and turns it into a feature-gated delegating alias.
///
/// The original method's attributes are copied. The alias receives `track_caller` when the
/// original did not already have it and a `Self: Sized` bound, then forwards every original
/// generic and value argument.
pub(super) fn generate_alias(original: &TraitItemFn, alias_name: &str) -> TraitItemFn {
    let mut alias = original.clone();
    alias.sig.ident = Ident::new(alias_name, Span::call_site());

    alias
        .attrs
        .insert(0, syn::parse_quote! { #[cfg(feature = "fluent")] });
    let original_name = &original.sig.ident;
    let documentation = format!("Fluent alias for [`{original_name}`](Self::{original_name}).");
    alias
        .attrs
        .insert(1, syn::parse_quote! { #[doc = #documentation] });
    alias.attrs.insert(2, syn::parse_quote! { #[doc = ""] });
    if !alias
        .attrs
        .iter()
        .any(|attribute| attribute.path().is_ident("track_caller"))
    {
        alias.attrs.push(syn::parse_quote! { #[track_caller] });
    }
    alias
        .sig
        .generics
        .make_where_clause()
        .predicates
        .push(syn::parse_quote! { Self: Sized });

    let generics = generic_arguments(original);
    let arguments = value_arguments(&mut alias);
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

/// Converts declared type and const parameters into turbofish arguments for delegation.
///
/// Lifetimes are inferred from the forwarded arguments and return type. Forwarding them explicitly
/// is rejected for late-bound lifetime parameters.
fn generic_arguments(method: &TraitItemFn) -> Vec<TokenStream> {
    method
        .sig
        .generics
        .params
        .iter()
        .filter_map(|parameter| match parameter {
            GenericParam::Lifetime(_) => None,
            GenericParam::Type(parameter) => Some(parameter.ident.to_token_stream()),
            GenericParam::Const(parameter) => Some(parameter.ident.to_token_stream()),
        })
        .collect()
}

/// Makes every declared value argument directly addressable and returns the names to forward.
///
/// Plain identifier patterns keep their public names. Identifier bindings such as `ref value` are
/// reduced to their by-value name, and patterns that do not provide a name receive an internal,
/// collision-free one in the alias signature.
fn value_arguments(method: &mut TraitItemFn) -> Vec<Ident> {
    let mut unavailable_names = method
        .sig
        .generics
        .params
        .iter()
        .filter_map(|parameter| match parameter {
            GenericParam::Lifetime(_) => None,
            GenericParam::Type(parameter) => Some(parameter.ident.to_string()),
            GenericParam::Const(parameter) => Some(parameter.ident.to_string()),
        })
        .collect::<BTreeSet<_>>();

    unavailable_names.extend(
        method
            .sig
            .inputs
            .iter()
            .filter_map(|argument| match argument {
                FnArg::Receiver(_) => None,
                FnArg::Typed(argument) => match &*argument.pat {
                    Pat::Ident(pattern) => Some(pattern.ident.to_string()),
                    _ => None,
                },
            }),
    );

    let mut next_internal_name = 0;
    method
        .sig
        .inputs
        .iter_mut()
        .filter_map(|argument| match argument {
            FnArg::Receiver(_) => None,
            FnArg::Typed(argument) => {
                let ident = match &*argument.pat {
                    Pat::Ident(pattern) => pattern.ident.clone(),
                    _ => fresh_argument_ident(&mut unavailable_names, &mut next_internal_name),
                };
                *argument.pat = syn::parse_quote! { #ident };
                Some(ident)
            }
        })
        .collect()
}

/// Allocates a hygienic argument name that cannot collide with preserved arguments or generics.
fn fresh_argument_ident(
    unavailable_names: &mut BTreeSet<String>,
    next_internal_name: &mut usize,
) -> Ident {
    loop {
        let name = format!("__assertr_fluent_argument_{next_internal_name}");
        *next_internal_name += 1;
        if unavailable_names.insert(name.clone()) {
            return Ident::new(&name, Span::call_site());
        }
    }
}

#[cfg(test)]
mod tests {
    use quote::quote;
    use syn::{Attribute, TraitItemFn, parse_quote};

    use super::generate_alias;

    fn attributes_tokens(attributes: &[Attribute]) -> String {
        quote! { #(#attributes)* }.to_string()
    }

    #[test]
    fn prepends_alias_documentation_and_preserves_original_method_attributes() {
        let original: TraitItemFn = parse_quote! {
            /// Returns whether the subject is ready.
            #[must_use = "the assertion result must be used"]
            #[deprecated(since = "1.2.3", note = "use `is_prepared` instead")]
            #[cfg(any(unix, windows))]
            #[cfg_attr(docsrs, doc(cfg(feature = "std")))]
            #[allow(clippy::needless_pass_by_value)]
            #[track_caller]
            fn is_ready(self, expected: bool) -> Self;
        };

        let alias = generate_alias(&original, "be_ready");

        assert_eq!(alias.attrs[0], parse_quote! { #[cfg(feature = "fluent")] });
        assert_eq!(
            alias.attrs[1],
            parse_quote! {
                #[doc = "Fluent alias for [`is_ready`](Self::is_ready)."]
            }
        );
        assert_eq!(alias.attrs[2], parse_quote! { #[doc = ""] });
        assert_eq!(
            attributes_tokens(&alias.attrs[3..]),
            attributes_tokens(&original.attrs)
        );
        assert_eq!(
            alias
                .attrs
                .iter()
                .filter(|attribute| attribute.path().is_ident("track_caller"))
                .count(),
            1
        );
    }

    #[test]
    fn forwards_arguments_with_non_identifier_patterns() {
        let original: TraitItemFn = parse_quote! {
            fn is_expected<const __assertr_fluent_argument_0: usize>(
                self,
                expected: usize,
                (left, right): (usize, usize),
                _: bool,
                ref label: String,
            ) -> Self {
                self
            }
        };

        let alias = generate_alias(&original, "be_expected");

        assert_eq!(
            alias.sig.inputs,
            parse_quote! {
                self,
                expected: usize,
                __assertr_fluent_argument_1: (usize, usize),
                __assertr_fluent_argument_2: bool,
                label: String,
            }
        );
        assert_eq!(
            alias.default,
            Some(parse_quote! {{
                self.is_expected::<__assertr_fluent_argument_0>(
                    expected,
                    __assertr_fluent_argument_1,
                    __assertr_fluent_argument_2,
                    label
                )
            }})
        );
    }

    #[test]
    fn infers_lifetimes_while_forwarding_type_and_const_generics() {
        let original: TraitItemFn = parse_quote! {
            fn is_borrowed_as<'a, T, const N: usize>(
                self,
                expected: &'a [T; N],
            ) -> Self {
                self
            }
        };

        let alias = generate_alias(&original, "borrow_as");

        assert_eq!(
            alias.default,
            Some(parse_quote! {{
                self.is_borrowed_as::<T, N>(expected)
            }})
        );
    }

    #[test]
    fn adds_alias_documentation_when_the_original_is_undocumented() {
        let original: TraitItemFn = parse_quote! {
            fn is_ready(self) -> Self;
        };

        let alias = generate_alias(&original, "be_ready");

        assert_eq!(
            alias.attrs[1],
            parse_quote! {
                #[doc = "Fluent alias for [`is_ready`](Self::is_ready)."]
            }
        );
    }
}
