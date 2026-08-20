//! Top-level orchestration and final token assembly for `AssertrEq`.

use darling::{Error, FromDeriveInput, ast};
use proc_macro_crate::{FoundCrate, crate_name};
use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{DeriveInput, Ident};

use super::{
    comparison::{build_implementation_generics, equality_checks},
    generics::{matcher_debug_generics, matcher_generics},
    identifiers::IdentifierAllocator,
    input::{AssertrEqInput, public_fields},
    matcher::{debug_field_entries, default_field_values, matcher_field_definitions},
};

/// Resolves the path used to reference `assertr` from generated code.
///
/// Cargo exposes renamed dependencies under their local alias, so hard-coding `::assertr` would
/// break otherwise valid manifests such as `assertions = { package = "assertr", ... }`.
fn assertr_path() -> TokenStream {
    match crate_name("assertr") {
        Ok(FoundCrate::Name(name)) => {
            let ident = Ident::new(&name, Span::call_site());
            quote! { ::#ident }
        }
        Ok(FoundCrate::Itself) | Err(_) => quote! { ::assertr },
    }
}

/// Returns the companion matcher name generated for a source struct.
fn matcher_ident(input: &Ident) -> Ident {
    Ident::new(&format!("{input}AssertrEq"), Span::call_site())
}

/// Coordinates parsing, generic filtering, field expansion, and final token generation.
///
/// Darling errors and invalid custom bounds are returned as compile-error tokens so callers see
/// diagnostics at the derive site instead of a proc-macro panic.
pub(super) fn derive_assertr_eq_impl(ast: &DeriveInput) -> TokenStream {
    let input: AssertrEqInput = match FromDeriveInput::from_derive_input(ast) {
        Ok(args) => args,
        Err(err) => return Error::write_errors(err),
    };

    let original_struct_ident = input.ident.clone();
    let original_generics = input.generics.clone();
    let (_, original_ty_generics, _) = original_generics.split_for_impl();
    let assertr = assertr_path();

    // Unreachable in practice: darling's `supports(struct_named)` already rejected enums.
    let ast::Data::Struct(fields) = &input.data else {
        return Error::custom("AssertrEq can only be derived for structs").write_errors();
    };
    let mut identifiers = IdentifierAllocator::from_input(ast, fields);
    let renderer = identifiers.fresh("__AssertrRenderer");

    let public_fields = public_fields(fields);
    let matcher_generics = matcher_generics(&original_generics, &public_fields);
    let (matcher_impl_generics, matcher_ty_generics, matcher_where_clause) =
        matcher_generics.split_for_impl();
    let debug_generics = matcher_debug_generics(&matcher_generics, &public_fields, &assertr);
    let (debug_impl_generics, _, debug_where_clause) = debug_generics.split_for_impl();

    let eq_struct_ident = matcher_ident(&input.ident);
    let eq_struct_fields = matcher_field_definitions(&public_fields, &assertr);
    let default_fields = default_field_values(&public_fields, &assertr);
    let eq_impls = equality_checks(&public_fields, &renderer, &assertr);
    let implementation_generics = match build_implementation_generics(
        &original_generics,
        &public_fields,
        &renderer,
        &assertr,
    ) {
        Ok(generics) => generics,
        Err(err) => return Error::from(err).write_errors(),
    };
    let (implementation_impl_generics, _, implementation_where_clause) =
        implementation_generics.split_for_impl();

    let debug_renderer_fields = debug_field_entries(
        &public_fields,
        &matcher_generics,
        &mut identifiers,
        &assertr,
    );

    quote! {
        pub struct #eq_struct_ident #matcher_generics {
            #(#eq_struct_fields),*
        }

        impl #matcher_impl_generics ::core::default::Default
            for #eq_struct_ident #matcher_ty_generics
            #matcher_where_clause
        {
            fn default() -> Self {
                Self {
                    #(#default_fields),*
                }
            }
        }

        impl #debug_impl_generics ::core::fmt::Debug
            for #eq_struct_ident #matcher_ty_generics
            #debug_where_clause
        {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                let mut debug_struct = f.debug_struct(stringify!(#eq_struct_ident));
                let value = self;
                let renderer = &#assertr::DebugRenderer;
                #(#debug_renderer_fields)*
                debug_struct.finish()
            }
        }

        impl #implementation_impl_generics
            #assertr::AssertrPartialEq<#eq_struct_ident #matcher_ty_generics, #renderer>
            for &#original_struct_ident #original_ty_generics
            #implementation_where_clause
        {
            fn eq(
                &self,
                other: &#eq_struct_ident #matcher_ty_generics,
                mut ctx: Option<&mut #assertr::EqContext<'_, #renderer>>,
            ) -> bool {
                let mut equal = true;
                #(#eq_impls)*
                equal
            }
        }

        impl #implementation_impl_generics
            #assertr::AssertrPartialEq<#eq_struct_ident #matcher_ty_generics, #renderer>
            for #original_struct_ident #original_ty_generics
            #implementation_where_clause
        {
            fn eq(
                &self,
                other: &#eq_struct_ident #matcher_ty_generics,
                ctx: Option<&mut #assertr::EqContext<'_, #renderer>>,
            ) -> bool {
                #assertr::AssertrPartialEq::eq(&self, other, ctx)
            }
        }
    }
}
