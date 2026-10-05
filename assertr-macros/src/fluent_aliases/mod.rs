//! Implementation of the `fluent_aliases` attribute macro.

mod attributes;
mod generate;
mod naming;

use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{Ident, ItemTrait, TraitItem, TraitItemFn, ext::IdentExt};

use self::{
    attributes::{fluent_alias_literal, has_attribute, remove_helper_attributes},
    generate::{alias_ident, generate_alias},
    naming::automatic_alias,
};

/// Adds fluent aliases to eligible trait methods and removes the helper attributes consumed by the
/// macro from the emitted trait.
///
/// Invalid helper attributes are reported as compile errors next to the trait, which is still
/// emitted so its other uses keep resolving.
pub(super) fn fluent_aliases_impl(mut trait_definition: ItemTrait) -> TokenStream {
    let mut items = Vec::with_capacity(trait_definition.items.len());
    let mut errors: Option<syn::Error> = None;

    for item in std::mem::take(&mut trait_definition.items) {
        let TraitItem::Fn(mut method) = item else {
            items.push(item);
            continue;
        };

        let alias = alias_name(&method);
        remove_helper_attributes(&mut method.attrs);
        let alias = match alias {
            Ok(alias) => alias.map(|alias| generate_alias(&method, alias)),
            Err(error) => {
                match &mut errors {
                    Some(errors) => errors.combine(error),
                    None => errors = Some(error),
                }
                None
            }
        };
        items.push(TraitItem::Fn(method));
        items.extend(alias.map(TraitItem::Fn));
    }

    trait_definition.items = items;
    let errors = errors.map(syn::Error::into_compile_error);
    quote! { #trait_definition #errors }
}

/// Selects the explicit or automatic alias of a method, if it should have one.
fn alias_name(method: &TraitItemFn) -> syn::Result<Option<Ident>> {
    if has_attribute(&method.attrs, "no_fluent_alias") {
        return Ok(None);
    }

    if let Some(literal) = fluent_alias_literal(&method.attrs)? {
        let name = literal.value();
        return alias_ident(&name, literal.span()).map(Some).ok_or_else(|| {
            syn::Error::new(
                literal.span(),
                format!("`{name}` is not a valid method name for a fluent alias"),
            )
        });
    }

    let name = method.sig.ident.unraw().to_string();
    Ok(automatic_alias(&name).and_then(|alias| alias_ident(&alias, Span::call_site())))
}
