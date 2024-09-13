#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used)]

use proc_macro::TokenStream;

use darling::*;
use proc_macro2::Span;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Ident, Path, Type, Visibility};

#[derive(Debug, FromField)]
#[darling(attributes(assertr_eq))]
struct MyFieldReceiver {
    ident: Option<Ident>,

    ty: Type,

    vis: Visibility,

    #[darling(default)]
    map_type: Option<Type>,

    #[darling(default)]
    compare_with: Option<Path>,
}

#[derive(Debug, FromDeriveInput)]
#[darling(attributes(assertr_eq), supports(struct_any))]
struct MyInputReceiver {
    ident: Ident,

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
        Err(err) => return Error::write_errors(err).into(),
    };

    let original_struct_ident = input.ident.clone();

    let filtered_fields = input.fields().iter().filter(|field| match field.vis {
        Visibility::Public(_) => true,
        Visibility::Restricted(_) => false,
        Visibility::Inherited => false,
    });

    let eq_struct_ident = Ident::new(
        format!("{}AssertrEq", input.ident).as_str(),
        Span::call_site(),
    );

    let eq_struct_fields = filtered_fields.clone().map(|field| {
        let vis = &field.vis;
        let ident = &field.ident;
        let ty = match &field.map_type {
            None => &field.ty,
            Some(ty) => ty,
        };
        quote! { #vis #ident: ::assertr::Eq<#ty> }
    });

    let eq_impls = filtered_fields.map(|field| {
        let ident = field
            .ident
            .as_ref()
            .expect("only named fields are supported!");
        let ident_string = ident.to_string();
        let ty = match &field.map_type {
            None => &field.ty,
            Some(ty) => ty,
        };
        let eq_args = quote! { &self.#ident, v, ctx.as_deref_mut() };
        let eq_check = match &field.compare_with {
            None => quote! { ::assertr::AssertrPartialEq::<#ty>::eq(#eq_args) },
            Some(eq_check) => {
                quote! { #eq_check(#eq_args) }
            }
        };
        quote! {
            && match &other.#ident {
                ::assertr::Eq::Any => true,
                ::assertr::Eq::Eq(v) => {
                    let eq = #eq_check;
                    if !eq {
                        if let Some(ctx) = ctx.as_mut() {
                            ctx.add_field_difference(#ident_string, v, &self.#ident);
                        }
                    }
                    eq
                },
            }
        }
    });

    quote! {
        #[derive(::core::fmt::Debug)]
        pub struct #eq_struct_ident {
            #(#eq_struct_fields),*
        }

        impl ::assertr::AssertrPartialEq<#eq_struct_ident> for &#original_struct_ident {
            fn eq(&self, other: &#eq_struct_ident, mut ctx: Option<&mut ::assertr::EqContext>) -> bool {
                true #(#eq_impls)*
            }
        }

        impl ::assertr::AssertrPartialEq<#eq_struct_ident> for #original_struct_ident {
            fn eq(&self, other: &#eq_struct_ident, ctx: Option<&mut ::assertr::EqContext>) -> bool {
                ::assertr::AssertrPartialEq::eq(&self, other, ctx)
            }
        }
    }.into()
}
