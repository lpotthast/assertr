//! Expansion of field comparisons and their required implementation bounds.

use proc_macro2::{TokenStream, TokenTree};
use quote::{ToTokens, quote};
use syn::punctuated::Punctuated;
use syn::{Generics, Ident, Token, parse_quote};

use super::input::{AssertrEqField, PublicField};

/// Generates the comparison statement for every public field.
///
/// Each generated statement honors `Eq::Any`, delegates to a field-specific `compare_with`
/// function when configured, and records a rendered difference without stopping later fields
/// from being compared.
pub(super) fn equality_checks(
    fields: &[PublicField<'_>],
    renderer: &Ident,
    assertr: &TokenStream,
) -> Vec<TokenStream> {
    fields
        .iter()
        .map(|(field, ident)| {
            let field_name = ident.to_string();
            let expected_type = field.expected_type();
            let arguments = quote! { &self.#ident, expected, ctx.as_deref_mut() };
            let equality_check = match &field.compare_with {
                None => quote! {
                    #assertr::AssertrPartialEq::<#expected_type, #renderer>::eq(#arguments)
                },
                Some(compare_with) => quote! { #compare_with(#arguments) },
            };

            quote! {
                match &other.#ident {
                    #assertr::Eq::Any => {}
                    #assertr::Eq::Eq(expected) => {
                        if !(#equality_check) {
                            if let Some(ctx) = ctx.as_mut() {
                                ctx.add_field_difference_rendered(
                                    #field_name,
                                    expected,
                                    &self.#ident,
                                );
                            }
                            equal = false;
                        }
                    }
                }
            }
        })
        .collect()
}

/// Extends the source type's generics for the generated `AssertrPartialEq` implementations.
///
/// The added renderer parameter must render both the actual and expected field types. Types
/// shared between fields are only bounded once. Fields using the default comparison also
/// receive an `AssertrPartialEq` bound. Custom comparisons may contribute predicates through
/// `compare_bounds`.
pub(super) fn build_implementation_generics(
    original: &Generics,
    fields: &[PublicField<'_>],
    renderer: &Ident,
    assertr: &TokenStream,
) -> syn::Result<Generics> {
    let mut generics = original.clone();
    generics.params.push(parse_quote! { #renderer });

    let mut rendered_types = Vec::new();
    for (field, _) in fields {
        let where_clause = generics.make_where_clause();
        for ty in [&field.ty, field.expected_type()] {
            let type_key = ty.to_token_stream().to_string();
            if rendered_types.contains(&type_key) {
                continue;
            }
            rendered_types.push(type_key);
            where_clause.predicates.push(parse_quote! {
                #renderer: #assertr::ValueRenderer<#ty>
            });
        }
        where_clause
            .predicates
            .extend(comparison_bounds(field, renderer, assertr)?);
    }

    Ok(generics)
}

/// Builds the comparison-specific predicates for one field.
///
/// The default comparison requires `AssertrPartialEq`. A custom comparison owns its comparison
/// contract and therefore receives only explicitly configured `compare_bounds` predicates.
fn comparison_bounds(
    field: &AssertrEqField,
    renderer: &Ident,
    assertr: &TokenStream,
) -> syn::Result<Vec<syn::WherePredicate>> {
    let actual_type = &field.ty;
    let expected_type = field.expected_type();
    if field.compare_with.is_none() {
        return Ok(vec![parse_quote! {
            #actual_type: #assertr::AssertrPartialEq<#expected_type, #renderer>
        }]);
    }

    let Some(bounds) = field.compare_bounds.as_ref() else {
        return Ok(Vec::new());
    };
    // `LitStr::parse_with` spans the parsed syntax into the literal, so errors in the user's
    // `compare_bounds` input point at the attribute instead of generated code.
    let predicates =
        bounds.parse_with(Punctuated::<syn::WherePredicate, Token![,]>::parse_terminated)?;
    predicates
        .into_iter()
        .map(|predicate| replace_renderer(quote! { #predicate }, renderer))
        .map(syn::parse2)
        .collect()
}

/// Replaces the documented renderer placeholder `R` inside a parsed `compare_bounds` predicate.
///
/// Replacement recurses through token groups and preserves their spans so diagnostics continue
/// to point at the user's attribute input.
fn replace_renderer(tokens: TokenStream, renderer: &Ident) -> TokenStream {
    tokens
        .into_iter()
        .map(|token| match token {
            TokenTree::Ident(ident) if ident == "R" => {
                TokenTree::Ident(Ident::new(&renderer.to_string(), ident.span()))
            }
            TokenTree::Group(group) => {
                let mut replaced = proc_macro2::Group::new(
                    group.delimiter(),
                    replace_renderer(group.stream(), renderer),
                );
                replaced.set_span(group.span());
                TokenTree::Group(replaced)
            }
            token => token,
        })
        .collect()
}
