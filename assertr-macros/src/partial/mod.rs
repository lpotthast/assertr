use std::collections::BTreeSet;

use proc_macro2::{Delimiter, Group, Span, TokenStream, TokenTree};
use quote::{ToTokens, quote_spanned};
use syn::{
    Expr, Ident, Path, Token,
    ext::IdentExt,
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
/// Field shapes keep the span of their delimiters for compiler suggestions on the pattern.
enum Shape {
    Named(Vec<(Ident, Expr)>, bool, Span),
    Tuple(Vec<Option<Expr>>, Span),
    Unit,
}

impl Parse for Input {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let variant = if input.peek(keyword::variant) {
            let fork = input.fork();
            fork.parse::<keyword::variant>()?;
            if fork.parse::<Path>().is_ok() {
                input.parse::<keyword::variant>()?;
                true
            } else {
                false
            }
        } else {
            false
        };
        let path = input.parse::<Path>()?;
        let shape = if input.peek(syn::token::Brace) {
            let content;
            let delimiter = syn::braced!(content in input).span.join();
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
                if !names.insert(name.unraw().to_string()) {
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
            Shape::Named(fields, rest, delimiter)
        } else if input.peek(syn::token::Paren) {
            let content;
            let delimiter = syn::parenthesized!(content in input).span.join();
            let mut fields = Vec::new();
            let mut rest = false;
            while !content.is_empty() {
                if content.peek(Token![..]) {
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
            Shape::Tuple(fields, delimiter)
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

/// Splits the runtime crate path, forwarded by `assertr::partial!` as `$crate`, from the matcher
/// input at the first top-level `;`.
///
/// Also returns the span for generated tokens. They resolve at the call site but point at the
/// user's input rather than into the `macro_rules!` wrapper, keeping diagnostics and closure type
/// names in the caller's file.
fn split_runtime(input: TokenStream) -> syn::Result<(TokenStream, TokenStream, Span)> {
    let mut tokens = input.into_iter();
    let runtime = tokens
        .by_ref()
        .take_while(|token| !matches!(token, TokenTree::Punct(punct) if punct.as_char() == ';'))
        .collect::<TokenStream>();
    if runtime.is_empty() {
        return Err(syn::Error::new(
            Span::call_site(),
            "use `partial!` through the `assertr` crate",
        ));
    }
    let input = tokens.collect::<TokenStream>();
    let location = input
        .clone()
        .into_iter()
        .next()
        .map_or_else(Span::call_site, |token| token.span());
    Ok((runtime, input, Span::call_site().located_at(location)))
}

/// Wraps generated pattern fields in delimiters located at the user's original delimiters.
fn delimited(delimiter: Delimiter, fields: TokenStream, span: Span) -> TokenStream {
    let mut group = Group::new(delimiter, fields);
    group.set_span(Span::call_site().located_at(span));
    group.into_token_stream()
}

fn constructor_label(path: &Path) -> String {
    path.segments
        .iter()
        .map(|segment| segment.ident.unraw().to_string())
        .collect::<Vec<_>>()
        .join("::")
}

pub(crate) fn expand(input: TokenStream) -> syn::Result<TokenStream> {
    let (runtime, input, span) = split_runtime(input)?;
    let Input {
        path,
        shape,
        variant,
    } = syn::parse2(input)?;
    let constructor_name = constructor_label(&path);
    let actual = Ident::new("__assertr_actual", Span::mixed_site().located_at(span));
    let value = Ident::new("__assertr_value", Span::mixed_site().located_at(span));
    let field = |projection: TokenStream, expression: &Expr, path: TokenStream| quote_spanned!(span=> #runtime::__private::field(#projection,#expression,#path));
    let mut matcher_fields = Vec::new();
    let pattern;
    match shape {
        Shape::Named(fields, rest, delimiter) => {
            let names = fields.iter().map(|(name, _)| name).collect::<Vec<_>>();
            let rest = rest.then(|| quote_spanned!(span=> ..));
            let fields_pattern = delimited(
                Delimiter::Brace,
                quote_spanned!(span=> #(#names: _,)* #rest),
                delimiter,
            );
            pattern = quote_spanned!(span=> #path #fields_pattern);
            for (name, expression) in &fields {
                let field_name = name.unraw().to_string();
                matcher_fields.push(field(
                    quote_spanned!(name.span()=> |#actual| {
                        #[allow(unreachable_patterns)]
                        match #actual {
                            #path { #name: #value, .. } => ::core::option::Option::Some(#value),
                            _ => ::core::option::Option::None,
                        }
                    }),
                    expression,
                    quote_spanned!(span=> #runtime::failure::PathSegment::Field(#field_name)),
                ));
            }
        }
        Shape::Tuple(fields, delimiter) => {
            let slots = fields
                .iter()
                .map(|field| {
                    if field.is_none() {
                        quote_spanned!(span=> ..)
                    } else {
                        quote_spanned!(span=> _)
                    }
                })
                .collect::<Vec<_>>();
            let fields_pattern = delimited(
                Delimiter::Parenthesis,
                quote_spanned!(span=> #(#slots),*),
                delimiter,
            );
            pattern = quote_spanned!(span=> #path #fields_pattern);
            for (index, expression) in fields.iter().enumerate() {
                let Some(expression) = expression else {
                    continue;
                };
                if matches!(expression, Expr::Infer(_)) {
                    continue;
                }
                let mut projection = slots.clone();
                projection[index] = quote_spanned!(span=> #value);
                matcher_fields.push(field(
                    quote_spanned!(span=> |#actual| {
                        #[allow(unreachable_patterns)]
                        match #actual {
                            #path (#(#projection),*) => ::core::option::Option::Some(#value),
                            _ => ::core::option::Option::None,
                        }
                    }),
                    expression,
                    quote_spanned!(span=> #runtime::failure::PathSegment::TupleIndex(#index)),
                ));
            }
        }
        Shape::Unit => {
            // Braces force constructor resolution even for a single unqualified identifier.
            pattern = quote_spanned!(span=> #path {});
        }
    }
    // Keep expectations in one expression so borrowed temporaries live through the caller's
    // statement. Nested constructor arguments evaluate each expectation once in source order.
    let list = matcher_fields.into_iter().rev().fold(
        quote_spanned!(span=> #runtime::__private::Nil),
        |tail, head| quote_spanned!(span=> #runtime::__private::Cons(#head,#tail)),
    );
    let variant = if variant {
        quote_spanned!(span=> ::core::option::Option::Some(#constructor_name))
    } else {
        quote_spanned!(span=> ::core::option::Option::None)
    };
    Ok(quote_spanned!(span=>
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
