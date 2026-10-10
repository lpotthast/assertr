//! Construction of delegating fluent alias methods.

use proc_macro2::{Ident, Span, TokenStream};
use quote::{ToTokens, quote};
use syn::{FnArg, GenericParam, Meta, Pat, TraitItemFn, ext::IdentExt};

/// Returns `name` as a method identifier, using a raw identifier for keywords.
///
/// Returns `None` when `name` is not a single valid identifier.
pub(super) fn alias_ident(name: &str, span: Span) -> Option<Ident> {
    let mut ident = syn::parse_str::<Ident>(name)
        .or_else(|_| syn::parse_str::<Ident>(&format!("r#{name}")))
        .ok()?;
    if ident.unraw() != name.trim_start_matches("r#") {
        return None;
    }
    ident.set_span(span);
    Some(ident)
}

/// Clones a trait method and turns it into a delegating alias.
///
/// The alias is documented as an alias of the original method instead of repeating its
/// documentation. Other attributes of the original method are copied. The alias receives
/// `track_caller` when the original did not already have it and a `Self: Sized` bound, then
/// forwards every original generic and value argument and awaits async methods.
pub(super) fn generate_alias(original: &TraitItemFn, alias_name: Ident) -> TraitItemFn {
    let mut alias = original.clone();
    alias.sig.ident = alias_name;

    // Keep `#[doc(hidden)]` and similar list forms, but replace the documentation text.
    alias.attrs.retain(|attribute| {
        !(attribute.path().is_ident("doc") && matches!(attribute.meta, Meta::NameValue(_)))
    });
    let original_name = &original.sig.ident;
    let documentation = format!("Fluent alias for [`{original_name}`](Self::{original_name}).");
    alias
        .attrs
        .insert(0, syn::parse_quote! { #[doc = #documentation] });
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
    let await_delegation = original.sig.asyncness.map(|_| quote! { .await });
    let turbofish = (!generics.is_empty()).then(|| quote! { ::<#(#generics),*> });
    alias.default = Some(syn::parse_quote! {
        { self.#original_name #turbofish (#(#arguments),*) #await_delegation }
    });
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
/// reduced to their by-value name. Patterns that do not provide a name receive `argument_{index}`
/// with mixed-site hygiene, which keeps it apart from the caller's argument names. Const generic
/// parameters are not hygienic, so a name they use is prefixed with underscores until it is free.
fn value_arguments(method: &mut TraitItemFn) -> Vec<Ident> {
    let const_parameters = method
        .sig
        .generics
        .const_params()
        .map(|parameter| parameter.ident.to_string())
        .collect::<Vec<_>>();
    method
        .sig
        .inputs
        .iter_mut()
        .enumerate()
        .filter_map(|(index, argument)| match argument {
            FnArg::Receiver(_) => None,
            FnArg::Typed(argument) => {
                let ident = if let Pat::Ident(pattern) = &*argument.pat {
                    pattern.ident.clone()
                } else {
                    let mut name = format!("argument_{index}");
                    while const_parameters.contains(&name) {
                        name.insert(0, '_');
                    }
                    Ident::new(&name, Span::mixed_site())
                };
                *argument.pat = syn::parse_quote! { #ident };
                Some(ident)
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use std::fmt::Debug;

    use quote::quote;
    use renamed_assertr::prelude::*;
    use syn::{Attribute, TraitItemFn, parse_quote};

    use super::{alias_ident, generate_alias};

    fn ident(name: &str) -> proc_macro2::Ident {
        alias_ident(name, proc_macro2::Span::call_site()).expect("valid alias")
    }

    fn attributes_tokens(attributes: &[Attribute]) -> String {
        quote! { #(#attributes)* }.to_string()
    }

    fn assert_equal<T>(actual: &T, expected: &T)
    where
        T: Debug + PartialEq,
    {
        assert_that_owned!(actual).is_equal_to(expected);
    }

    #[test]
    fn adds_caller_tracking_when_the_original_does_not_declare_it() {
        let original: TraitItemFn = parse_quote! {
            fn is_ready(self) -> Self;
        };
        let alias = generate_alias(&original, ident("be_ready"));
        assert_that!(
            alias
                .attrs
                .iter()
                .filter(|attribute| attribute.path().is_ident("track_caller"))
                .count()
        )
        .is_equal_to(1);
    }

    #[test]
    fn replaces_documentation_and_preserves_other_original_method_attributes() {
        let original: TraitItemFn = parse_quote! {
            /// Returns whether the subject is ready.
            ///
            /// ```
            /// assert!(true);
            /// ```
            #[doc(hidden)]
            #[must_use = "the assertion result must be used"]
            #[deprecated(since = "1.2.3", note = "use `is_prepared` instead")]
            #[cfg(any(unix, windows))]
            #[cfg_attr(docsrs, doc(cfg(feature = "std")))]
            #[allow(clippy::needless_pass_by_value)]
            #[track_caller]
            fn is_ready(self, expected: bool) -> Self;
        };

        let alias = generate_alias(&original, ident("be_ready"));

        assert_equal(
            &alias.attrs[0],
            &parse_quote! {
                #[doc = "Fluent alias for [`is_ready`](Self::is_ready)."]
            },
        );
        assert_that!(attributes_tokens(&alias.attrs[1..]))
            .is_equal_to(attributes_tokens(&original.attrs[5..]));
        assert_that!(
            alias
                .attrs
                .iter()
                .filter(|attribute| attribute.path().is_ident("track_caller"))
                .count()
        )
        .is_equal_to(1);
    }

    #[test]
    fn forwards_arguments_with_non_identifier_patterns() {
        let original: TraitItemFn = parse_quote! {
            fn is_expected<const argument_3: usize>(
                self,
                argument_2: usize,
                (left, right): (usize, usize),
                _: bool,
                ref label: String,
            ) -> Self {
                self
            }
        };

        let alias = generate_alias(&original, ident("be_expected"));

        // The generated `argument_2` has mixed-site hygiene and does not shadow the caller's.
        assert_equal(
            &alias.sig.inputs,
            &parse_quote! {
                self,
                argument_2: usize,
                argument_2: (usize, usize),
                _argument_3: bool,
                label: String,
            },
        );
        assert_equal(
            &alias.default,
            &Some(parse_quote! {{
                self.is_expected::<argument_3>(argument_2, argument_2, _argument_3, label)
            }}),
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

        let alias = generate_alias(&original, ident("borrow_as"));

        assert_equal(
            &alias.default,
            &Some(parse_quote! {{
                self.is_borrowed_as::<T, N>(expected)
            }}),
        );
    }

    #[test]
    fn adds_alias_documentation_when_the_original_is_undocumented() {
        let original: TraitItemFn = parse_quote! {
            fn is_ready(self) -> Self;
        };

        let alias = generate_alias(&original, ident("be_ready"));

        assert_equal(
            &alias.attrs[0],
            &parse_quote! {
                #[doc = "Fluent alias for [`is_ready`](Self::is_ready)."]
            },
        );
    }

    mod alias_identifiers {
        use super::*;

        #[test]
        fn uses_raw_identifiers_for_keywords() {
            let original: TraitItemFn = parse_quote! { fn matches<E>(self, expected: E) -> Self; };
            let alias = generate_alias(&original, ident("match"));
            assert_that!(alias.sig.ident.to_string()).is_equal_to("r#match");
            let _: TraitItemFn = syn::parse2(quote!(#alias)).expect("raw alias is valid Rust");
            assert_that!(ident("r#type").to_string()).is_equal_to("r#type");
        }

        #[test]
        fn rejects_names_that_are_not_one_identifier() {
            for name in ["", "be ready", " be_ready", "be-ready", "1st", "r#"] {
                assert_that!(alias_ident(name, proc_macro2::Span::call_site())).is_none();
            }
        }
    }
}
