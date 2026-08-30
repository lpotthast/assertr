//! Implementation of the `fluent_aliases` attribute macro.

mod attributes;
mod generate;
mod naming;

use proc_macro2::TokenStream;
use quote::quote;
use syn::{ItemTrait, TraitItem};

use self::{
    attributes::{fluent_alias_name, has_attribute, remove_helper_attributes},
    generate::generate_alias,
    naming::{AutomaticAlias, automatic_alias},
};

/// Adds fluent aliases to eligible trait methods and removes the helper attributes consumed by
/// the macro from the emitted trait.
pub(super) fn fluent_aliases_impl(mut trait_definition: ItemTrait) -> TokenStream {
    let mut items = Vec::new();

    for item in &trait_definition.items {
        items.push(item.clone());

        if let TraitItem::Fn(method) = item {
            if has_attribute(&method.attrs, "no_fluent_alias") {
                continue;
            }

            let alias = fluent_alias_name(&method.attrs).map_or_else(
                || automatic_alias(&method.sig.ident.to_string()),
                AutomaticAlias::Generated,
            );
            if let AutomaticAlias::Generated(alias) = alias {
                items.push(TraitItem::Fn(generate_alias(method, &alias)));
            }
        }
    }

    trait_definition.items = items;
    for item in &mut trait_definition.items {
        if let TraitItem::Fn(method) = item {
            remove_helper_attributes(&mut method.attrs);
        }
    }

    quote! { #trait_definition }
}
