#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used)]

use proc_macro::TokenStream;

use darling::*;
use proc_macro2::{Ident, Span};
use quote::quote;
use syn::{DeriveInput, parse_macro_input, Visibility};

#[derive(Debug, FromField)]
#[darling(attributes(assertr_eq))]
struct MyFieldReceiver {
    ident: Option<syn::Ident>,

    ty: syn::Type,

    vis: syn::Visibility,

    map_type: Option<syn::Type>,
}

#[derive(Debug, FromDeriveInput)]
#[darling(attributes(assertr_eq), supports(struct_any))]
struct MyInputReceiver {
    ident: syn::Ident,

    data: ast::Data<(), MyFieldReceiver>,
}

impl MyInputReceiver {
    pub fn fields(&self) -> &ast::Fields<MyFieldReceiver> {
        match &self.data {
            ast::Data::Enum(_) => panic!("Only structs are supported"),
            ast::Data::Struct(fields) => fields,
        }
    }
}

#[proc_macro_derive(AssertrEq, attributes(assertr_eq))]
pub fn store(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);

    let input: MyInputReceiver = match FromDeriveInput::from_derive_input(&ast) {
        Ok(args) => args,
        Err(err) => return darling::Error::write_errors(err).into(),
    };

    let original_struct_ident = input.ident.clone();

    let filtered_fields = input
        .fields()
        .iter()
        .filter(|field| match field.vis {
            Visibility::Public(_) => true,
            Visibility::Restricted(_) => false,
            Visibility::Inherited => false,
        });

    let eq_struct_ident = Ident::new(format!("{}AssertrEq", input.ident).as_str(), Span::call_site());
    let eq_struct_fields = filtered_fields.clone()
        .map(|field| {
            let vis = &field.vis;
            let ident = &field.ident;
            let ty = match &field.map_type {
                None => &field.ty,
                Some(ty) => ty,
            };
            quote! { #vis #ident: ::assertr::Eq<#ty> }
        });

    let eq_impls = filtered_fields.map(|field| {
        let ident = &field.ident;
        let eq_check = quote! { ::core::cmp::PartialEq::eq(&self.#ident, v) };
        quote! {
            && match &other.#ident {
                ::assertr::Eq::Any => true,
                ::assertr::Eq::Eq(v) => { #eq_check },
            }
        }
    }).collect::<Vec<_>>();

    quote! {
        #[derive(::core::fmt::Debug)]
        pub struct #eq_struct_ident {
            #(#eq_struct_fields),*
        }

        impl ::core::cmp::PartialEq<#eq_struct_ident> for #original_struct_ident {
            fn eq(&self, other: &#eq_struct_ident) -> bool {
                true #(#eq_impls)*
            }
        }
    }.into()
}
