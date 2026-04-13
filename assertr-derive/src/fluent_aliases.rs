use proc_macro2::{Ident, Span, TokenStream};
use quote::{ToTokens, quote};
use syn::{Attribute, FnArg, GenericParam, ItemTrait, Pat, TraitItem, TraitItemFn};

pub fn fluent_aliases_impl(mut trait_def: ItemTrait) -> TokenStream {
    let mut new_items: Vec<TraitItem> = Vec::new();

    for item in &trait_def.items {
        // Always keep the original item.
        new_items.push(item.clone());

        if let TraitItem::Fn(method) = item {
            // Skip methods marked with #[no_fluent_alias].
            if has_attr(&method.attrs, "no_fluent_alias") {
                continue;
            }

            // Determine alias name: explicit #[fluent_alias("name")] or auto-derived.
            let alias_name = get_fluent_alias_name(&method.attrs)
                .or_else(|| auto_derive_alias(&method.sig.ident.to_string()));

            if let Some(alias_name) = alias_name {
                new_items.push(TraitItem::Fn(generate_alias(method, &alias_name)));
            }
        }
    }

    trait_def.items = new_items;

    // Strip helper attributes from all methods in the output.
    for item in &mut trait_def.items {
        if let TraitItem::Fn(method) = item {
            method.attrs.retain(|attr| {
                !attr.path().is_ident("fluent_alias")
                    && !attr.path().is_ident("no_fluent_alias")
                    && !has_attr_in_cfg_attr(attr, "fluent_alias")
                    && !has_attr_in_cfg_attr(attr, "no_fluent_alias")
            });
        }
    }

    quote! { #trait_def }
}

/// Auto-derive an alias name from the original method name.
///
/// Transforms third-person singular verb forms to infinitive/imperative mood:
/// - `is_*` → `be_*` (e.g., `is_empty` → `be_empty`)
/// - `has_*` → `have_*` (e.g., `has_length` → `have_length`)
/// - `does_not_*` → `not_*` (e.g., `does_not_contain` → `not_contain`)
/// - `contains` / `contains_*` → `contain` / `contain_*`
/// - `starts_*` → `start_*` (e.g., `starts_with` → `start_with`)
/// - `ends_*` → `end_*` (e.g., `ends_with` → `end_with`)
/// - `exists` / `exists_*` → `exist` / `exist_*`
/// - `satisfies` / `satisfies_*` → `satisfy` / `satisfy_*`
///
/// Returns `None` if no auto-derivation rule matches — use `#[fluent_alias("name")]` instead.
fn auto_derive_alias(name: &str) -> Option<String> {
    // Check more specific prefixes first to avoid partial matches.
    if let Some(rest) = name.strip_prefix("does_not_") {
        return Some(format!("not_{rest}"));
    }
    if let Some(rest) = name.strip_prefix("is_") {
        return Some(format!("be_{rest}"));
    }
    if let Some(rest) = name.strip_prefix("has_") {
        return Some(format!("have_{rest}"));
    }
    if let Some(rest) = name.strip_prefix("contains_") {
        return Some(format!("contain_{rest}"));
    }
    if name == "contains" {
        return Some("contain".into());
    }
    if let Some(rest) = name.strip_prefix("starts_") {
        return Some(format!("start_{rest}"));
    }
    if let Some(rest) = name.strip_prefix("ends_") {
        return Some(format!("end_{rest}"));
    }
    if let Some(rest) = name.strip_prefix("exists_") {
        return Some(format!("exist_{rest}"));
    }
    if name == "exists" {
        return Some("exist".into());
    }
    if let Some(rest) = name.strip_prefix("satisfies_") {
        return Some(format!("satisfy_{rest}"));
    }
    if name == "satisfies" {
        return Some("satisfy".into());
    }
    None
}

fn has_attr(attrs: &[Attribute], name: &str) -> bool {
    attrs
        .iter()
        .any(|a| a.path().is_ident(name) || has_attr_in_cfg_attr(a, name))
}

/// Check if an attribute is `#[cfg_attr(..., <name>)]` or `#[cfg_attr(..., <name>(...))]]`.
fn has_attr_in_cfg_attr(attr: &Attribute, name: &str) -> bool {
    if !attr.path().is_ident("cfg_attr") {
        return false;
    }
    // Use string-based matching since cfg_attr contents have complex parse requirements.
    let tokens = attr.meta.to_token_stream().to_string();
    // Look for ", <name>" or ", <name> (" pattern after the cfg condition.
    tokens.contains(&format!(", {name}"))
}

/// Extract the alias name from `#[fluent_alias("name")]`
/// or `#[cfg_attr(..., fluent_alias("name"))]`.
fn get_fluent_alias_name(attrs: &[Attribute]) -> Option<String> {
    for attr in attrs {
        if attr.path().is_ident("fluent_alias") {
            if let Ok(lit) = attr.parse_args::<syn::LitStr>() {
                return Some(lit.value());
            }
        }
        // Also check cfg_attr-wrapped form.
        if let Some(name) = get_fluent_alias_from_cfg_attr(attr) {
            return Some(name);
        }
    }
    None
}

/// Extract the alias name from `#[cfg_attr(..., fluent_alias("name"))]`.
fn get_fluent_alias_from_cfg_attr(attr: &Attribute) -> Option<String> {
    if !attr.path().is_ident("cfg_attr") {
        return None;
    }
    // Use string-based extraction: find `fluent_alias` followed by a string literal.
    let tokens = attr.meta.to_token_stream().to_string();
    let idx = tokens.find("fluent_alias")?;
    let after = &tokens[idx + "fluent_alias".len()..];
    // Find the first string literal (between double quotes) after `fluent_alias`.
    let quote_start = after.find('"')?;
    let rest = &after[quote_start + 1..];
    let quote_end = rest.find('"')?;
    Some(rest[..quote_end].to_string())
}

/// Generate an alias trait method that delegates to the original.
fn generate_alias(original: &TraitItemFn, alias_name: &str) -> TraitItemFn {
    let mut alias = original.clone();

    // Set the new name.
    alias.sig.ident = Ident::new(alias_name, Span::call_site());

    // Keep only #[cfg(...)] attributes from the original, then add #[cfg(feature = "fluent")].
    alias.attrs.retain(|attr| attr.path().is_ident("cfg"));
    alias
        .attrs
        .insert(0, syn::parse_quote! { #[cfg(feature = "fluent")] });

    // Add #[track_caller] so caller location propagates through the delegation.
    alias.attrs.push(syn::parse_quote! { #[track_caller] });

    // Add `Self: Sized` to the where clause.
    let where_clause = alias.sig.generics.make_where_clause();
    where_clause
        .predicates
        .push(syn::parse_quote! { Self: Sized });

    // Generate the delegation body: `{ self.original_name(arg1, arg2, ...) }`.
    let original_name = &original.sig.ident;
    let generics: Vec<TokenStream> = original
        .sig
        .generics
        .params
        .iter()
        .map(|param| match param {
            GenericParam::Lifetime(param) => param.lifetime.to_token_stream(),
            GenericParam::Type(param) => param.ident.to_token_stream(),
            GenericParam::Const(param) => param.ident.to_token_stream(),
        })
        .collect();
    let args: Vec<&Ident> = original
        .sig
        .inputs
        .iter()
        .filter_map(|arg| match arg {
            FnArg::Receiver(_) => None,
            FnArg::Typed(pat_type) => {
                if let Pat::Ident(pat_ident) = &*pat_type.pat {
                    Some(&pat_ident.ident)
                } else {
                    None
                }
            }
        })
        .collect();

    alias.default = if generics.is_empty() {
        Some(syn::parse_quote! {
            { self.#original_name(#(#args),*) }
        })
    } else {
        Some(syn::parse_quote! {
            { self.#original_name::<#(#generics),*>(#(#args),*) }
        })
    };
    alias.semi_token = None;

    alias
}
