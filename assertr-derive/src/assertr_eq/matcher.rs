//! Expansion of the generated matcher struct and its `Debug` implementation.

use super::{
    generics::mentions_generics,
    identifiers::IdentifierAllocator,
    input::PublicField,
    rendering::{render_debug_eq, specialized_render_value},
};
use proc_macro2::TokenStream;
use quote::quote;
use syn::Generics;

/// Generates the `Eq<FieldType>` members of the companion matcher struct.
pub(super) fn matcher_field_definitions(
    fields: &[PublicField<'_>],
    assertr: &TokenStream,
) -> Vec<TokenStream> {
    fields
        .iter()
        .map(|(field, ident)| {
            let visibility = &field.vis;
            let expected_type = field.expected_type();
            quote! { #visibility #ident: #assertr::Eq<#expected_type> }
        })
        .collect()
}

/// Initializes every generated matcher field to `assertr::Eq::Any`.
///
/// The path is emitted through `assertr` rather than referenced directly because downstream
/// crates may rename their `assertr` dependency.
pub(super) fn default_field_values(
    fields: &[PublicField<'_>],
    assertr: &TokenStream,
) -> Vec<TokenStream> {
    fields
        .iter()
        .map(|(_, ident)| quote! { #ident: #assertr::Eq::Any })
        .collect()
}

/// Generates the per-field entries used by the matcher's custom `Debug` implementation.
///
/// Fields of concrete type render through an autoref-specialization fallback that tolerates
/// non-`Debug` types. Fields whose types name matcher generics cannot use that fallback (the
/// specialization never resolves for a generic type), so they render through
/// `assertr::__private::RenderableEq`, whose conditional `Debug` impl is satisfied by the
/// `DebugRenderer: AssertionRenderer<FieldType>` bounds that `matcher_debug_generics` adds to
/// the matcher's `Debug` impl.
///
/// The concrete-field wrapper identifier is allocated through `identifiers` because its
/// declaration is emitted into user code and must not collide with identifiers from the derive
/// input.
pub(super) fn debug_field_entries(
    fields: &[PublicField<'_>],
    matcher_generics: &Generics,
    identifiers: &mut IdentifierAllocator,
    assertr: &TokenStream,
) -> Vec<TokenStream> {
    fields
        .iter()
        .enumerate()
        .map(|(index, (field, ident))| {
            let field_name = ident.to_string();
            let expected_type = field.expected_type();

            if mentions_generics(matcher_generics, expected_type) {
                return quote! {
                    debug_struct.field(
                        #field_name,
                        &#assertr::__private::RenderableEq {
                            value: &value.#ident,
                            renderer,
                        },
                    );
                };
            }

            let wrapper = identifiers.fresh(&format!("Field{index}Renderer"));
            let render_value = specialized_render_value(
                &quote! { value },
                &quote! { self.renderer },
                &quote! { f },
                assertr,
            );
            let render_eq = render_debug_eq(
                &quote! { self.value },
                &quote! { f },
                &render_value,
                assertr,
            );

            quote! {
                struct #wrapper<'a> {
                    value: &'a #assertr::Eq<#expected_type>,
                    renderer: &'a #assertr::DebugRenderer,
                }

                impl ::core::fmt::Debug for #wrapper<'_> {
                    fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                        #render_eq
                    }
                }

                debug_struct.field(
                    #field_name,
                    &#wrapper {
                        value: &value.#ident,
                        renderer,
                    },
                );
            }
        })
        .collect()
}
