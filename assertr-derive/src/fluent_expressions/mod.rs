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
    MustOwned,
    Verify,
    VerifyOwned,
}

struct FluentExpressions {
    assertr: TokenStream,
}

impl VisitMut for FluentExpressions {
    fn visit_expr_mut(&mut self, expression: &mut Expr) {
        let rewrite = match expression {
            Expr::MethodCall(call) if call.method == "must" && call.args.is_empty() => Some((
                EntryCall::Must,
                receiver_tokens(&call.receiver),
                call.method.span(),
            )),
            Expr::MethodCall(call) if call.method == "must_owned" && call.args.is_empty() => {
                Some((
                    EntryCall::MustOwned,
                    receiver_tokens(&call.receiver),
                    call.method.span(),
                ))
            }
            Expr::MethodCall(call) if call.method == "verify" && call.args.len() == 1 => Some((
                EntryCall::Verify,
                receiver_tokens(&call.receiver),
                call.method.span(),
            )),
            Expr::MethodCall(call) if call.method == "verify_owned" && call.args.len() == 1 => {
                Some((
                    EntryCall::VerifyOwned,
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

        match entry {
            EntryCall::Must | EntryCall::MustOwned => {
                let entry_call = expression.clone();
                *expression = syn::parse_quote_spanned! {span=>
                    #entry_call.with_expression(::core::stringify!(#receiver))
                };
            }
            EntryCall::Verify | EntryCall::VerifyOwned => {
                let Expr::MethodCall(call) = expression else {
                    unreachable!("the visitor does not replace verify calls")
                };
                let assertions = call
                    .args
                    .first()
                    .cloned()
                    .expect("verify rewrites have exactly one argument");
                let assertr = &self.assertr;
                call.args.clear();
                call.args
                    .push(adapt_verify_callback(assertions, assertr, &receiver, span));
            }
        }
    }
}

fn adapt_verify_callback(
    assertions: Expr,
    assertr: &TokenStream,
    receiver: &TokenStream,
    span: Span,
) -> Expr {
    match assertions {
        Expr::Closure(mut closure) if closure.inputs.len() == 1 => {
            let input = closure
                .inputs
                .first()
                .cloned()
                .expect("the closure has exactly one input");
            let body = closure.body;
            let assertion = Ident::new("__assertr_assertion", Span::mixed_site());

            closure.inputs.clear();
            closure
                .inputs
                .push(syn::parse_quote_spanned! {span=> #assertion});
            closure.body = Box::new(syn::parse_quote_spanned! {span=>
                {
                    let #input = #assertr::__private::fluent_expressions::AttachExpression::new(
                        #assertion,
                        ::core::stringify!(#receiver),
                    )
                    .attach();
                    #body
                }
            });

            Expr::Closure(closure)
        }
        Expr::Closure(closure) => Expr::Closure(closure),
        assertions => syn::parse_quote_spanned! {span=>
            #assertr::__private::fluent_expressions::adapt_callback(
                #assertions,
                move |__assertr_assertion| {
                    #assertr::__private::fluent_expressions::AttachExpression::new(
                        __assertr_assertion,
                        ::core::stringify!(#receiver),
                    )
                    .attach()
                },
            )
        },
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
