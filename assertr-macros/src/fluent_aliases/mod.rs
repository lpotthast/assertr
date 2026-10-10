//! Implementation of the `fluent_aliases` attribute macro.

mod attributes;
mod generate;
mod naming;

use std::collections::BTreeSet;

use proc_macro2::TokenStream;
use quote::quote;
use syn::{Attribute, Ident, ItemTrait, Meta, TraitItem, TraitItemFn, ext::IdentExt};

use self::{
    attributes::{alias_literal, helper_metas, remove_helper_attributes},
    generate::{alias_ident, generate_alias},
    naming::automatic_alias,
};

/// Adds fluent aliases to eligible trait methods and removes the helper attributes consumed by the
/// macro from the emitted trait.
///
/// Invalid helper attributes and aliases colliding with other trait items are reported as compile
/// errors next to the trait, which is still emitted so its other uses keep resolving.
pub(super) fn fluent_aliases_impl(mut trait_definition: ItemTrait) -> TokenStream {
    let mut items = Vec::with_capacity(trait_definition.items.len());
    let mut errors = Errors::default();
    // Items gated by `cfg` may legitimately share a name across configurations, so only
    // unconditional items take part in collision checks.
    let mut names = trait_definition
        .items
        .iter()
        .filter_map(unconditional_item_name)
        .collect::<BTreeSet<_>>();

    for mut item in std::mem::take(&mut trait_definition.items) {
        let TraitItem::Fn(method) = &mut item else {
            if let Some(attributes) = item_attributes(&mut item) {
                for meta in helper_metas(attributes) {
                    let name = helper_name(&meta);
                    errors.push(syn::Error::new_spanned(
                        meta,
                        format!("`{name}` applies only to trait methods"),
                    ));
                }
                remove_helper_attributes(attributes);
            }
            items.push(item);
            continue;
        };

        let alias = alias_name(method);
        remove_helper_attributes(&mut method.attrs);
        match alias {
            Ok(Some(alias)) => {
                let name = alias.unraw().to_string();
                let collides = if is_conditional(&method.attrs) {
                    names.contains(&name)
                } else {
                    !names.insert(name)
                };
                if !collides {
                    let generated = generate_alias(method, alias);
                    items.push(item);
                    items.push(TraitItem::Fn(generated));
                    continue;
                }
                errors.push(syn::Error::new(
                    alias.span(),
                    format!(
                        "fluent alias `{alias}` of `{}` collides with another item of this trait",
                        method.sig.ident
                    ),
                ));
            }
            Ok(None) => {}
            Err(error) => errors.push(error),
        }
        items.push(item);
    }

    trait_definition.items = items;
    let errors = errors.0.map(syn::Error::into_compile_error);
    quote! { #trait_definition #errors }
}

/// Compile errors combined into one diagnostic stream.
#[derive(Default)]
struct Errors(Option<syn::Error>);

impl Errors {
    fn push(&mut self, error: syn::Error) {
        match &mut self.0 {
            Some(errors) => errors.combine(error),
            None => self.0 = Some(error),
        }
    }
}

/// Selects the explicit or automatic alias of a method, if it should have one.
///
/// Rejects a `no_fluent_alias` with arguments, repeated helpers, and the combination of
/// `fluent_alias` with `no_fluent_alias`.
fn alias_name(method: &TraitItemFn) -> syn::Result<Option<Ident>> {
    let mut explicit: Option<Meta> = None;
    let mut skipped: Option<Meta> = None;
    for meta in helper_metas(&method.attrs) {
        let name = helper_name(&meta);
        let is_skip = name == "no_fluent_alias";
        if is_skip && !matches!(meta, Meta::Path(_)) {
            return Err(syn::Error::new_spanned(
                meta,
                "`no_fluent_alias` takes no arguments",
            ));
        }
        let slot = if is_skip { &mut skipped } else { &mut explicit };
        if slot.is_some() {
            return Err(syn::Error::new_spanned(meta, format!("duplicate `{name}`")));
        }
        *slot = Some(meta);
    }

    match (explicit, skipped) {
        (Some(_), Some(skipped)) => Err(syn::Error::new_spanned(
            skipped,
            "`fluent_alias` and `no_fluent_alias` cannot be combined",
        )),
        (None, Some(_)) => Ok(None),
        (Some(explicit), None) => {
            let literal = alias_literal(&explicit)?;
            let name = literal.value();
            alias_ident(&name, literal.span()).map(Some).ok_or_else(|| {
                syn::Error::new(
                    literal.span(),
                    format!("`{name}` is not a valid method name for a fluent alias"),
                )
            })
        }
        (None, None) => {
            let ident = &method.sig.ident;
            Ok(automatic_alias(&ident.unraw().to_string())
                .and_then(|alias| alias_ident(&alias, ident.span())))
        }
    }
}

/// Returns the name of a trait item that is not gated by `cfg`.
fn unconditional_item_name(item: &TraitItem) -> Option<String> {
    let (attributes, ident) = match item {
        TraitItem::Fn(item) => (&item.attrs, &item.sig.ident),
        TraitItem::Const(item) => (&item.attrs, &item.ident),
        TraitItem::Type(item) => (&item.attrs, &item.ident),
        _ => return None,
    };
    (!is_conditional(attributes)).then(|| ident.unraw().to_string())
}

/// Returns whether an item is gated by `cfg`.
fn is_conditional(attributes: &[Attribute]) -> bool {
    attributes
        .iter()
        .any(|attribute| attribute.path().is_ident("cfg"))
}

/// Returns the name of a helper attribute collected by [`helper_metas`].
fn helper_name(meta: &Meta) -> &'static str {
    if meta.path().is_ident("fluent_alias") {
        "fluent_alias"
    } else {
        "no_fluent_alias"
    }
}

/// Returns the attributes of a trait item other than a method, if it has any.
fn item_attributes(item: &mut TraitItem) -> Option<&mut Vec<Attribute>> {
    match item {
        TraitItem::Const(item) => Some(&mut item.attrs),
        TraitItem::Type(item) => Some(&mut item.attrs),
        TraitItem::Macro(item) => Some(&mut item.attrs),
        _ => None,
    }
}
