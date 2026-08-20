//! Allocation of hygienic helper identifiers for generated code.

use darling::ast;
use proc_macro2::{Span, TokenStream, TokenTree};
use quote::ToTokens;
use syn::{DeriveInput, Ident};

use super::input::AssertrEqField;

/// Allocates helper names that collide with neither user input nor earlier generated names.
pub(super) struct IdentifierAllocator {
    input_tokens: TokenStream,
    generated: Vec<Ident>,
}

impl IdentifierAllocator {
    /// Captures every identifier that can enter the expansion, including types and bounds parsed
    /// from string-valued `assertr_eq` attributes.
    pub(super) fn from_input(input: &DeriveInput, fields: &ast::Fields<AssertrEqField>) -> Self {
        Self {
            input_tokens: derive_input_tokens(input, fields),
            generated: Vec::new(),
        }
    }

    /// Returns the requested stem or the first available numeric-suffixed variant.
    pub(super) fn fresh(&mut self, stem: &str) -> Ident {
        let mut suffix = None;
        loop {
            let name = suffix.map_or_else(|| stem.to_owned(), |suffix| format!("{stem}{suffix}"));
            let candidate = Ident::new(&name, Span::call_site());
            if !tokens_mention_ident(self.input_tokens.clone(), &candidate)
                && !self.generated.contains(&candidate)
            {
                self.generated.push(candidate.clone());
                return candidate;
            }
            suffix = Some(suffix.map_or(0, |suffix| suffix + 1));
        }
    }
}

/// Combines ordinary derive tokens with syntax parsed from string-valued attributes.
fn derive_input_tokens(input: &DeriveInput, fields: &ast::Fields<AssertrEqField>) -> TokenStream {
    let mut tokens = input.to_token_stream();
    for field in fields.iter() {
        if let Some(map_type) = &field.map_type {
            map_type.to_tokens(&mut tokens);
        }
        if let Some(compare_with) = &field.compare_with {
            compare_with.to_tokens(&mut tokens);
        }
        if let Some(compare_bounds) = &field.compare_bounds
            && let Ok(compare_bounds) = compare_bounds.parse::<TokenStream>()
        {
            tokens.extend(compare_bounds);
        }
    }
    tokens
}

/// Recursively checks whether a token stream contains an identifier with the requested name.
pub(super) fn tokens_mention_ident(tokens: TokenStream, ident: &Ident) -> bool {
    tokens.into_iter().any(|token| match token {
        TokenTree::Ident(token_ident) => token_ident == *ident,
        TokenTree::Group(group) => tokens_mention_ident(group.stream(), ident),
        _ => false,
    })
}
