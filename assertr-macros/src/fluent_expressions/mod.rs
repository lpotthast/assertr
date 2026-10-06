use proc_macro_crate::{FoundCrate, crate_name};
use proc_macro2::{Span, TokenStream, TokenTree};
use quote::{ToTokens, quote};
use syn::{
    Expr, Ident, Item, LitStr, Path, Token,
    parse::{Parse, ParseStream},
    visit_mut::{self, VisitMut},
};

/// Arguments of `#[fluent_expressions(...)]`.
#[derive(Default)]
pub(crate) struct Arguments {
    /// Runtime path given as `crate = path`, for example through a facade crate.
    runtime: Option<Path>,
}

impl Parse for Arguments {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let mut arguments = Self::default();
        while !input.is_empty() {
            if input.peek(Token![crate]) {
                let keyword = input.parse::<Token![crate]>()?;
                input.parse::<Token![=]>()?;
                if input.peek(LitStr) {
                    return Err(input.error(
                        "expected an unquoted path such as `my_facade::assertr`, not a string literal",
                    ));
                }
                let path = Path::parse_mod_style(input)?;
                if arguments.runtime.is_some() {
                    return Err(syn::Error::new(
                        keyword.span,
                        "duplicate fluent_expressions argument `crate`",
                    ));
                }
                arguments.runtime = Some(path);
            } else {
                let token = input.parse::<TokenTree>()?;
                let message = match &token {
                    TokenTree::Ident(name) => format!(
                        "unknown fluent_expressions argument `{name}`, expected `crate = <path>`"
                    ),
                    _ => String::from("expected fluent_expressions argument `crate = <path>`"),
                };
                return Err(syn::Error::new(token.span(), message));
            }
            if input.is_empty() {
                break;
            }
            input.parse::<Token![,]>()?;
        }
        Ok(arguments)
    }
}

pub(crate) fn fluent_expressions_impl(
    arguments: Arguments,
    mut item: Item,
) -> syn::Result<TokenStream> {
    match &item {
        Item::Fn(_) => {}
        Item::Mod(module) if module.content.is_some() => {}
        Item::Mod(module) => {
            return Err(syn::Error::new_spanned(
                module,
                "fluent_expressions requires an inline module",
            ));
        }
        other => {
            return Err(syn::Error::new_spanned(
                other,
                "fluent_expressions can only be applied to a function or inline module",
            ));
        }
    }

    FluentExpressions {
        assertr: arguments
            .runtime
            .map_or_else(assertr_path, |path| path.to_token_stream()),
    }
    .visit_item_mut(&mut item);
    Ok(quote!(#item))
}

#[derive(Clone, Copy)]
enum EntryCall {
    Must,
    Verify,
}

struct FluentExpressions {
    assertr: TokenStream,
}

impl VisitMut for FluentExpressions {
    fn visit_expr_mut(&mut self, expression: &mut Expr) {
        let rewrite = match expression {
            Expr::MethodCall(call)
                if (call.method == "must" || call.method == "must_owned")
                    && call.args.is_empty() =>
            {
                Some((
                    EntryCall::Must,
                    receiver_tokens(&call.receiver),
                    call.method.span(),
                ))
            }
            Expr::MethodCall(call)
                if (call.method == "verify" || call.method == "verify_owned")
                    && call.args.len() == 1 =>
            {
                Some((
                    EntryCall::Verify,
                    receiver_tokens(&call.receiver),
                    call.method.span(),
                ))
            }
            _ => None,
        };

        visit_mut::visit_expr_mut(self, expression);

        let Some((entry, receiver, span)) = rewrite else {
            return;
        };
        let entry_call = expression.clone();

        *expression = match entry {
            EntryCall::Must => syn::parse_quote_spanned! {span=>
                #entry_call.with_expression(::core::stringify!(#receiver))
            },
            EntryCall::Verify => {
                // A function call records the start of its path as its caller location. Give the
                // resolved crate path the method span too, rather than the attribute's call site,
                // so `finish` observes the same location as the tracked entry method.
                let assertr: TokenStream = self
                    .assertr
                    .clone()
                    .into_iter()
                    .map(|mut token| {
                        token.set_span(span);
                        token
                    })
                    .collect();
                let result = Ident::new("__assertr_result", Span::mixed_site());
                let location = Ident::new("__assertr_location", Span::mixed_site());
                // The callback argument stays untouched, so the method keeps its callback type,
                // call traits, and coercions. Only the completed result is inspected.
                syn::parse_quote_spanned! {span=>
                    #assertr::__private::fluent_expressions::finish(
                        #entry_call,
                        |mut #result, #location| {
                            #[allow(unused_imports)]
                            use #assertr::__private::fluent_expressions::AttachExpressionFallback as _;
                            #assertr::__private::fluent_expressions::AttachExpression::new(
                                &mut #result,
                            )
                            .attach(::core::stringify!(#receiver), #location);
                            #result
                        },
                    )
                }
            }
        };
    }
}

fn assertr_path() -> TokenStream {
    match crate_name("assertr") {
        Ok(FoundCrate::Name(name)) => {
            let ident = Ident::new(&name, Span::call_site());
            quote!(::#ident)
        }
        Ok(FoundCrate::Itself) | Err(_) => quote!(::assertr),
    }
}

fn receiver_tokens(receiver: &Expr) -> TokenStream {
    let fallback = quote!(#receiver);
    let mut source = String::new();
    let mut previous_was_word = false;

    for token in fallback.clone() {
        let is_word = matches!(token, TokenTree::Ident(_) | TokenTree::Literal(_));
        if previous_was_word && is_word {
            source.push(' ');
        }
        if let Some(token_source) = token.span().source_text() {
            source.push_str(&token_source);
        } else {
            source.push_str(&token.to_string());
        }
        previous_was_word = is_word;
    }

    source.parse().unwrap_or(fallback)
}

#[cfg(test)]
mod tests {
    use super::*;
    use quote::quote;
    use renamed_assertr::prelude::*;

    #[test]
    fn rejects_out_of_line_modules() {
        let item = syn::parse2(quote!(
            mod tests;
        ))
        .expect("valid module");
        let error =
            fluent_expressions_impl(Arguments::default(), item).expect_err("module must be inline");
        assert_that!(error.to_string()).is_equal_to("fluent_expressions requires an inline module");
    }

    #[test]
    fn rejects_other_items() {
        let item = syn::parse2(quote!(
            struct Tests;
        ))
        .expect("valid struct");
        let error = fluent_expressions_impl(Arguments::default(), item)
            .expect_err("struct is not supported");
        assert_that!(error.to_string())
            .is_equal_to("fluent_expressions can only be applied to a function or inline module");
    }

    mod arguments {
        use super::*;

        fn parse(tokens: TokenStream) -> syn::Result<Arguments> {
            syn::parse2(tokens)
        }

        fn runtime(tokens: TokenStream) -> Option<String> {
            parse(tokens)
                .expect("valid arguments")
                .runtime
                .map(|path| path.to_token_stream().to_string())
        }

        fn error(tokens: TokenStream) -> String {
            match parse(tokens) {
                Ok(_) => panic!("arguments must be rejected"),
                Err(error) => error.to_string(),
            }
        }

        #[test]
        fn accepts_no_arguments() {
            assert_that!(runtime(quote!())).is_none();
        }

        #[test]
        fn accepts_a_runtime_path() {
            assert_that!(runtime(quote!(crate = my_facade::assertr)))
                .is_equal_to(Some(String::from("my_facade :: assertr")));
            assert_that!(runtime(quote!(crate = ::my_facade::assertr,)))
                .is_equal_to(Some(String::from(":: my_facade :: assertr")));
            assert_that!(runtime(quote!(crate = crate::support::assertr)))
                .is_equal_to(Some(String::from("crate :: support :: assertr")));
        }

        #[test]
        fn uses_the_given_runtime_path_in_generated_code() {
            let item = syn::parse2(quote!(
                fn check() {
                    let _ = 1.verify(|it| it);
                }
            ))
            .expect("valid function");
            let arguments = parse(quote!(crate = my_facade::assertr)).expect("valid arguments");
            let expanded = fluent_expressions_impl(arguments, item)
                .expect("function is supported")
                .to_string();
            assert_that!(expanded)
                .contains("my_facade :: assertr :: __private :: fluent_expressions :: finish");
        }

        #[test]
        fn rejects_unknown_arguments() {
            assert_that!(error(quote!(krate = my_facade::assertr))).is_equal_to(
                "unknown fluent_expressions argument `krate`, expected `crate = <path>`",
            );
            assert_that!(error(quote!("my_facade::assertr")))
                .is_equal_to("expected fluent_expressions argument `crate = <path>`");
        }

        #[test]
        fn rejects_duplicate_runtime_paths() {
            assert_that!(error(quote!(crate = a::assertr, crate = b::assertr)))
                .is_equal_to("duplicate fluent_expressions argument `crate`");
        }

        #[test]
        fn rejects_quoted_and_missing_paths() {
            assert_that!(error(quote!(crate = "my_facade::assertr"))).is_equal_to(
                "expected an unquoted path such as `my_facade::assertr`, not a string literal",
            );
            assert_that!(error(quote!(crate))).is_equal_to("expected `=`");
            assert_that!(error(quote!(crate = my_facade::assertr<T>))).is_equal_to("expected `,`");
        }
    }
}
