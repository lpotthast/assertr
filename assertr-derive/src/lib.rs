#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used)]
#![allow(clippy::needless_continue)]

mod fluent_aliases;

use proc_macro::TokenStream;

use darling::{Error, FromDeriveInput, FromField, ast};
use proc_macro2::Span;
use quote::{format_ident, quote};
use syn::{DeriveInput, Ident, ItemTrait, Path, Type, Visibility, WhereClause, parse_macro_input};

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

    // Extra trait bounds for the generated `AssertrPartialEq` impl when this field uses
    // `compare_with`. Specified as the body of a `where` clause without the leading `where`
    // keyword, e.g. `compare_bounds = "Bar: ::assertr::cmp::slice::CompareElement<BarAssertrEq, R>"`.
    // Per-field renderer bounds (`R: AssertionRenderer<actual_ty> + AssertionRenderer<expected_ty>`)
    // are added automatically; `compare_bounds` is appended on top, so list only the
    // comparison-specific predicates here.
    #[darling(default)]
    compare_bounds: Option<String>,
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

// Emits an autoref-specialization fallback so generated `DebugRenderer` impls compile for any
// field type, including those without `Debug`. Method-resolution prefers the impl on `Self`
// over the impl on `&Self`; the `Self` impl is gated on `DebugRenderer: AssertionRenderer<T>`,
// so it only applies for types `DebugRenderer` can actually render. Otherwise, resolution falls
// back through autoref to the `&Self` impl, which prints `<unrendered>`. Do not collapse the
// two impls — that would force every field type to satisfy the bound and break non-`Debug`
// fields.
fn debug_value_wrapper_tokens(wrapper: &Ident) -> proc_macro2::TokenStream {
    let render_trait = format_ident!("{wrapper}Render");

    quote! {
        struct #wrapper<'a, T: ?Sized> {
            value: &'a T,
            renderer: &'a ::assertr::DebugRenderer,
        }

        trait #render_trait {
            fn render(self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result;
        }

        impl<T: ?Sized> #render_trait for #wrapper<'_, T>
        where
            ::assertr::DebugRenderer: ::assertr::AssertionRenderer<T>,
        {
            fn render(self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                ::assertr::AssertionRenderer::<T>::fmt(self.renderer, self.value, f)
            }
        }

        impl<T: ?Sized> #render_trait for &#wrapper<'_, T> {
            fn render(self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                f.write_str("<unrendered>")
            }
        }
    }
}

fn render_debug_value_tokens(
    value: &proc_macro2::TokenStream,
    renderer: &proc_macro2::TokenStream,
    formatter: &proc_macro2::TokenStream,
    wrapper_prefix: &Ident,
) -> proc_macro2::TokenStream {
    let value_wrapper = format_ident!("{wrapper_prefix}Value");
    let wrapper = debug_value_wrapper_tokens(&value_wrapper);

    quote! {{
        #wrapper

        #value_wrapper {
            value: #value,
            renderer: #renderer,
        }.render(#formatter)
    }}
}

fn render_debug_eq_tokens(
    value: &proc_macro2::TokenStream,
    renderer: &proc_macro2::TokenStream,
    formatter: &proc_macro2::TokenStream,
    wrapper_prefix: &Ident,
) -> proc_macro2::TokenStream {
    let value_renderer =
        render_debug_value_tokens(&quote! { value }, renderer, formatter, wrapper_prefix);

    quote! {
        match #value {
            ::assertr::Eq::Any => #formatter.write_str("Eq::Any"),
            ::assertr::Eq::Eq(value) => {
                #formatter.write_str("Eq::Eq(")?;
                #value_renderer?;
                #formatter.write_str(")")
            }
        }
    }
}

fn comparison_bounds_for_field(
    actual_ty: &Type,
    expected_ty: &Type,
    compare_with: Option<&Path>,
    compare_bounds: Option<&String>,
) -> Result<Vec<proc_macro2::TokenStream>, syn::Error> {
    if compare_with.is_none() {
        return Ok(Vec::from([quote! {
            #actual_ty: ::assertr::AssertrPartialEq<#expected_ty, R>
        }]));
    }

    match compare_bounds {
        Some(bounds) => {
            let where_clause: WhereClause = syn::parse_str(&format!("where {bounds}"))?;
            Ok(where_clause
                .predicates
                .into_iter()
                .map(|predicate| quote! { #predicate })
                .collect())
        }
        None => Ok(Vec::new()),
    }
}

/// Derive macro for `AssertrEq`.
///
/// # Panics
///
/// This proc macro will panic if applied to an enum, as only structs are supported.
#[proc_macro_derive(AssertrEq, attributes(assertr_eq))]
#[allow(clippy::too_many_lines)]
pub fn store(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);

    let input: MyInputReceiver = match FromDeriveInput::from_derive_input(&ast) {
        Ok(args) => args,
        Err(err) => return Error::write_errors(err).into(),
    };

    let original_struct_ident = input.ident.clone();

    let filtered_fields = input
        .fields()
        .iter()
        .filter(|field| matches!(field.vis, Visibility::Public(_)))
        .collect::<Vec<_>>();

    let eq_struct_ident = Ident::new(
        format!("{}AssertrEq", input.ident).as_str(),
        Span::call_site(),
    );
    let eq_struct_fields = filtered_fields.iter().map(|field| {
        let vis = &field.vis;
        let ident = &field.ident;
        let ty = match &field.map_type {
            None => &field.ty,
            Some(ty) => ty,
        };
        quote! { #vis #ident: ::assertr::Eq<#ty> }
    });

    let eq_impls = filtered_fields.iter().map(|field| {
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
            None => quote! { ::assertr::AssertrPartialEq::<#ty, R>::eq(#eq_args) },
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
                            ctx.add_field_difference_rendered(#ident_string, v, &self.#ident);
                        }
                    }
                    eq
                },
            }
        }
    });

    let mut field_renderer_bounds = Vec::new();
    for field in &filtered_fields {
        let actual_ty = &field.ty;
        let expected_ty = match &field.map_type {
            None => &field.ty,
            Some(ty) => ty,
        };
        field_renderer_bounds.push(quote! {
            R: ::assertr::AssertionRenderer<#actual_ty> + ::assertr::AssertionRenderer<#expected_ty>
        });
        let comparison_bounds = match comparison_bounds_for_field(
            actual_ty,
            expected_ty,
            field.compare_with.as_ref(),
            field.compare_bounds.as_ref(),
        ) {
            Ok(bounds) => bounds,
            Err(err) => return Error::from(err).write_errors().into(),
        };
        field_renderer_bounds.extend(comparison_bounds);
    }

    let renderer_bounds = field_renderer_bounds;
    let renderer_where = if renderer_bounds.is_empty() {
        quote! {}
    } else {
        quote! { where #(#renderer_bounds),* }
    };

    let debug_renderer_fields = filtered_fields.iter().enumerate().map(|(index, field)| {
        let ident = field
            .ident
            .as_ref()
            .expect("only named fields are supported!");
        let ident_string = ident.to_string();
        let ty = match &field.map_type {
            None => &field.ty,
            Some(ty) => ty,
        };
        let wrapper = format_ident!("Field{index}Renderer");
        let render_eq = render_debug_eq_tokens(
            &quote! { self.value },
            &quote! { self.renderer },
            &quote! { f },
            &wrapper,
        );

        quote! {
            struct #wrapper<'a> {
                value: &'a ::assertr::Eq<#ty>,
                renderer: &'a ::assertr::DebugRenderer,
            }

            impl ::core::fmt::Debug for #wrapper<'_> {
                fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                    #render_eq
                }
            }

            debug_struct.field(
                #ident_string,
                &#wrapper {
                    value: &value.#ident,
                    renderer,
                },
            );
        }
    });

    Into::into(quote! {
        #[derive(::core::default::Default)]
        pub struct #eq_struct_ident {
            #(#eq_struct_fields),*
        }

        impl ::core::fmt::Debug for #eq_struct_ident {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                let mut debug_struct = f.debug_struct(stringify!(#eq_struct_ident));
                let value = self;
                let renderer = &::assertr::DebugRenderer;
                #(#debug_renderer_fields)*
                debug_struct.finish()
            }
        }

        impl<R> ::assertr::AssertrPartialEq<#eq_struct_ident, R> for &#original_struct_ident
        #renderer_where
        {
            fn eq(&self, other: &#eq_struct_ident, mut ctx: Option<&mut ::assertr::EqContext<'_, R>>) -> bool {
                true #(#eq_impls)*
            }
        }

        impl<R> ::assertr::AssertrPartialEq<#eq_struct_ident, R> for #original_struct_ident
        #renderer_where
        {
            fn eq(&self, other: &#eq_struct_ident, ctx: Option<&mut ::assertr::EqContext<'_, R>>) -> bool {
                ::assertr::AssertrPartialEq::eq(&self, other, ctx)
            }
        }
    })
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
