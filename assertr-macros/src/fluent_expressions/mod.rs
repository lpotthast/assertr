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
                let Expr::MethodCall(mut entry_call) = expression.clone() else {
                    unreachable!("the visitor does not replace the entry itself")
                };
                // A function call records the start of its path as its caller location. Give the
                // resolved crate path the method span too, rather than the attribute's call site.
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
                let callback = Ident::new("__assertr_callback", Span::mixed_site());
                let callback_type = Ident::new("__assertr_callback_type", Span::mixed_site());
                let assertions = entry_call
                    .args
                    .first_mut()
                    .expect("verify rewrites have exactly one argument");
                if !adapt_verify_callback(assertions, &assertr, &receiver, &callback_type, span) {
                    *expression = Expr::MethodCall(entry_call);
                    return;
                }

                *expression = syn::parse_quote_spanned! {span=>
                    {
                        let mut #callback_type = ::core::option::Option::None;
                        #assertr::__private::fluent_expressions::finish(
                            #entry_call,
                            |mut #result, #location| {
                                if let ::core::option::Option::Some(#callback) = #callback_type {
                                    #[allow(unused_imports)]
                                    use #assertr::__private::fluent_expressions::{
                                        CaptureCallback as _, CaptureCallbackFallback as _,
                                        AttachExpressionFallback as _,
                                    };
                                    if (&#callback).accepts_capture() {
                                        #assertr::__private::fluent_expressions::AttachExpression::new(
                                            &mut #result,
                                        )
                                        .attach(::core::stringify!(#receiver), #location);
                                    }
                                }
                                #result
                            },
                        )
                    }
                };
            }
        }
    }
}

/// Literal closures retain the method's expected callback signature, including coercions to
/// function pointers. Other callback values keep their concrete type for inspection after the
/// original call has resolved it. Returns whether a callback type needs to be retained.
fn adapt_verify_callback(
    expression: &mut Expr,
    assertr: &TokenStream,
    receiver: &TokenStream,
    callback_type: &Ident,
    span: Span,
) -> bool {
    match expression {
        Expr::Closure(closure) => {
            if closure.inputs.len() == 1 {
                let input = closure
                    .inputs
                    .first()
                    .cloned()
                    .expect("the closure has exactly one input");
                let assertion = Ident::new("__assertr_assertion", Span::mixed_site());
                let body = &closure.body;
                closure.inputs.clear();
                let mut parameter = input.clone();
                if let syn::Pat::Type(typed) = &mut parameter {
                    *typed.pat = syn::parse_quote_spanned! {span=> #assertion};
                } else {
                    parameter = syn::parse_quote_spanned! {span=> #assertion};
                }
                closure.inputs.push(parameter);
                *closure.body = syn::parse_quote_spanned! {span=> {
                    #[allow(unused_imports)]
                    use #assertr::__private::fluent_expressions::AttachExpressionFallback as _;
                    let mut #assertion = #assertion;
                    #assertr::__private::fluent_expressions::AttachExpression::new(&mut #assertion)
                        .attach_to_input(::core::stringify!(#receiver));
                    let #input = #assertion;
                    #body
                }};
            }
            false
        }
        Expr::Block(block) => {
            if let Some(syn::Stmt::Expr(tail, None)) = block.block.stmts.last_mut() {
                adapt_verify_callback(tail, assertr, receiver, callback_type, span)
            } else {
                false
            }
        }
        Expr::Paren(inner) => {
            adapt_verify_callback(&mut inner.expr, assertr, receiver, callback_type, span)
        }
        Expr::Group(inner) => {
            adapt_verify_callback(&mut inner.expr, assertr, receiver, callback_type, span)
        }
        Expr::Reference(inner) => {
            let mut value = (*inner.expr).clone();
            if !adapt_verify_callback(&mut value, assertr, receiver, callback_type, span) {
                *inner.expr = value;
                return false;
            }
            // Remember the reference itself. Moving its referent into the helper would consume
            // a callback that the original call only borrowed.
            *expression = syn::parse_quote_spanned! {span=>
                #assertr::__private::fluent_expressions::remember_callback(#expression, &mut #callback_type)
            };
            true
        }
        Expr::Cast(inner) => {
            adapt_verify_callback(&mut inner.expr, assertr, receiver, callback_type, span)
        }
        Expr::If(branch) => {
            let then_retained =
                if let Some(syn::Stmt::Expr(tail, None)) = branch.then_branch.stmts.last_mut() {
                    adapt_verify_callback(tail, assertr, receiver, callback_type, span)
                } else {
                    false
                };
            let else_retained = branch.else_branch.as_mut().is_some_and(|(_, tail)| {
                adapt_verify_callback(tail, assertr, receiver, callback_type, span)
            });
            then_retained || else_retained
        }
        Expr::Match(branch) => {
            let mut retained = false;
            for arm in &mut branch.arms {
                retained |=
                    adapt_verify_callback(&mut arm.body, assertr, receiver, callback_type, span);
            }
            retained
        }
        _ => {
            *expression = syn::parse_quote_spanned! {span=>
                #assertr::__private::fluent_expressions::remember_callback(#expression, &mut #callback_type)
            };
            true
        }
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

    #[test]
    fn rejects_out_of_line_modules() {
        let item = syn::parse2(quote!(
            mod tests;
        ))
        .expect("valid module");
        let error = fluent_expressions_impl(item).expect_err("module must be inline");
        assert_eq!(
            error.to_string(),
            "fluent_expressions requires an inline module"
        );
    }

    #[test]
    fn rejects_other_items() {
        let item = syn::parse2(quote!(
            struct Tests;
        ))
        .expect("valid struct");
        let error = fluent_expressions_impl(item).expect_err("struct is not supported");
        assert_eq!(
            error.to_string(),
            "fluent_expressions can only be applied to a function or inline module"
        );
    }
}
