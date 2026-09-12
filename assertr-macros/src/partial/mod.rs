use proc_macro2::{Span, TokenStream};
use quote::{quote, quote_spanned};
use std::collections::BTreeSet;
use syn::{
    Expr, Ident, Path, Token,
    parse::{Parse, ParseStream},
};

mod keyword {
    syn::custom_keyword!(variant);
}

struct Input {
    variant: bool,
    path: Path,
    shape: Shape,
}
enum Shape {
    Named(Vec<(Ident, Expr)>, bool),
    Tuple(Vec<Option<Expr>>),
    Unit,
}

impl Parse for Input {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let variant = if input.peek(keyword::variant) && input.peek2(Ident) {
            input.parse::<keyword::variant>()?;
            true
        } else {
            false
        };
        let path = input.parse::<Path>()?;
        let shape = if input.peek(syn::token::Brace) {
            let content;
            syn::braced!(content in input);
            let mut fields = Vec::new();
            let mut rest = false;
            let mut names = BTreeSet::new();
            while !content.is_empty() {
                if content.peek(Token![..]) {
                    content.parse::<Token![..]>()?;
                    rest = true;
                    if content.peek(Token![,]) {
                        content.parse::<Token![,]>()?;
                    }
                    if !content.is_empty() {
                        return Err(content.error("`..` must be the final field"));
                    }
                    break;
                }
                let name: Ident = content.parse()?;
                if !names.insert(name.to_string().trim_start_matches("r#").to_owned()) {
                    return Err(syn::Error::new(name.span(), "duplicate matcher field"));
                }
                content.parse::<Token![:]>()?;
                let expression: Expr = content.parse()?;
                fields.push((name, expression));
                if content.is_empty() {
                    break;
                }
                content.parse::<Token![,]>()?;
            }
            Shape::Named(fields, rest)
        } else if input.peek(syn::token::Paren) {
            let content;
            syn::parenthesized!(content in input);
            let mut fields = Vec::new();
            let mut rest = false;
            while !content.is_empty() {
                if content.peek(Token![..]) {
                    if rest {
                        return Err(content.error("only one tuple rest is allowed"));
                    }
                    content.parse::<Token![..]>()?;
                    rest = true;
                    fields.push(None);
                } else {
                    fields.push(Some(content.parse()?));
                }
                if content.is_empty() {
                    break;
                }
                content.parse::<Token![,]>()?;
                if rest && !content.is_empty() {
                    return Err(content
                        .error("tuple `..` must be final so selected tuple indexes remain exact"));
                }
            }
            Shape::Tuple(fields)
        } else {
            Shape::Unit
        };
        if !input.is_empty() {
            return Err(input.error("expected a named, tuple, or unit constructor"));
        }
        Ok(Self {
            variant,
            path,
            shape,
        })
    }
}

fn runtime() -> TokenStream {
    match proc_macro_crate::crate_name("assertr") {
        Ok(proc_macro_crate::FoundCrate::Name(name)) => {
            let name = Ident::new(&name, Span::call_site());
            quote!(::#name)
        }
        Ok(proc_macro_crate::FoundCrate::Itself) | Err(_) => quote!(::assertr),
    }
}

fn constructor_label(path: &Path) -> String {
    path.segments
        .iter()
        .map(|segment| {
            segment
                .ident
                .to_string()
                .trim_start_matches("r#")
                .to_owned()
        })
        .collect::<Vec<_>>()
        .join("::")
}

pub(crate) fn expand(input: TokenStream) -> syn::Result<TokenStream> {
    let Input {
        path,
        shape,
        variant,
    } = syn::parse2(input)?;
    let runtime = runtime();
    let constructor_name = constructor_label(&path);
    let actual = Ident::new("__assertr_actual", Span::mixed_site());
    let value = Ident::new("__assertr_value", Span::mixed_site());
    let mut expectations = Vec::new();
    let mut projections = Vec::new();
    let pattern;
    match shape {
        Shape::Named(fields, rest) => {
            let names = fields.iter().map(|(name, _)| name).collect::<Vec<_>>();
            let rest = rest.then(|| quote!(..));
            pattern = quote!(#path {#(#names: _,)* #rest});
            for (name, expression) in &fields {
                let field_name = name.to_string().trim_start_matches("r#").to_owned();
                expectations.push(quote!(#expression));
                projections.push((
                    quote_spanned!(name.span()=> |#actual| {
                        #[allow(unreachable_patterns)]
                        match #actual {
                            #path { #name: #value, .. } => ::core::option::Option::Some(#value),
                            _ => ::core::option::Option::None,
                        }
                    }),
                    quote!(#runtime::failure::PathSegment::Field(#field_name)),
                ));
            }
        }
        Shape::Tuple(fields) => {
            let slots = fields
                .iter()
                .map(|field| {
                    if field.is_none() {
                        quote!(..)
                    } else {
                        quote!(_)
                    }
                })
                .collect::<Vec<_>>();
            pattern = quote!(#path (#(#slots),*));
            for (index, expression) in fields.iter().enumerate() {
                let Some(expression) = expression else {
                    continue;
                };
                if matches!(expression, Expr::Infer(_)) {
                    continue;
                }
                let mut projection = slots.clone();
                projection[index] = quote!(#value);
                expectations.push(quote!(#expression));
                projections.push((
                    quote!(|#actual| {
                        #[allow(unreachable_patterns)]
                        match #actual {
                            #path (#(#projection),*) => ::core::option::Option::Some(#value),
                            _ => ::core::option::Option::None,
                        }
                    }),
                    quote!(#runtime::failure::PathSegment::TupleIndex(#index)),
                ));
            }
        }
        Shape::Unit => {
            // Braces force constructor resolution even for a single unqualified identifier.
            pattern = quote!(#path {});
        }
    }
    // Keep expectations in one expression so borrowed temporaries live through the caller's
    // statement. Nested constructor arguments evaluate each expectation once in source order.
    let list = matcher_fields(expectations, projections, &runtime);
    let variant = if variant {
        quote!(::core::option::Option::Some(#constructor_name))
    } else {
        quote!(::core::option::Option::None)
    };
    Ok(quote!(
        #runtime::__private::partial_match(
            |#actual| {
                #[allow(unreachable_patterns)]
                match #actual { #pattern => true, _ => false }
            },
            #list,
            #constructor_name,
            #variant,
        )
    ))
}

/// Builds the nested matcher list in source order from the expectations and projections.
fn matcher_fields(
    expectations: Vec<TokenStream>,
    projections: Vec<(TokenStream, TokenStream)>,
    runtime: &TokenStream,
) -> TokenStream {
    let fields = expectations
        .into_iter()
        .zip(projections)
        .map(|(expectation, (projection, path))| {
            quote!(#runtime::__private::field(#projection,#expectation,#path))
        })
        .collect::<Vec<_>>();
    fields.into_iter().rev().fold(
        quote!(#runtime::__private::Nil),
        |tail, head| quote!(#runtime::__private::Cons(#head,#tail)),
    )
}
