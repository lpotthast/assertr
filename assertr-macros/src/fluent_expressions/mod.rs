use proc_macro_crate::{FoundCrate, crate_name};
use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{
    Expr, Ident, Item,
    visit_mut::{self, VisitMut},
};

pub(crate) fn fluent_expressions_impl(mut item: Item) -> syn::Result<TokenStream> {
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
        assertr: assertr_path(),
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
        let is_word = matches!(
            token,
            proc_macro2::TokenTree::Ident(_) | proc_macro2::TokenTree::Literal(_)
        );
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
        let error = fluent_expressions_impl(item).expect_err("module must be inline");
        assert_that!(error.to_string()).is_equal_to("fluent_expressions requires an inline module");
    }

    #[test]
    fn rejects_other_items() {
        let item = syn::parse2(quote!(
            struct Tests;
        ))
        .expect("valid struct");
        let error = fluent_expressions_impl(item).expect_err("struct is not supported");
        assert_that!(error.to_string())
            .is_equal_to("fluent_expressions can only be applied to a function or inline module");
    }
}
