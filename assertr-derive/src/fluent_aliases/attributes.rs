//! Recognition and parsing of fluent-alias helper attributes.

use syn::punctuated::Punctuated;
use syn::{Attribute, Meta, Token};

/// Returns whether an attribute list contains `name` directly or inside `cfg_attr`.
pub(super) fn has_attribute(attributes: &[Attribute], name: &str) -> bool {
    attributes
        .iter()
        .any(|attribute| attribute.path().is_ident(name) || cfg_attr_contains(attribute, name))
}

/// Returns whether an attribute is consumed by the fluent-alias macro.
pub(super) fn is_helper_attribute(attribute: &Attribute) -> bool {
    ["fluent_alias", "no_fluent_alias"]
        .iter()
        .any(|name| attribute.path().is_ident(name) || cfg_attr_contains(attribute, name))
}

/// Extracts an explicit alias from `fluent_alias`, including its `cfg_attr` form.
pub(super) fn fluent_alias_name(attributes: &[Attribute]) -> Option<String> {
    for attribute in attributes {
        if attribute.path().is_ident("fluent_alias")
            && let Ok(alias) = attribute.parse_args::<syn::LitStr>()
        {
            return Some(alias.value());
        }
        if let Some(alias) = fluent_alias_from_cfg_attr(attribute) {
            return Some(alias);
        }
    }
    None
}

/// Parses the attributes nested inside `cfg_attr(predicate, attr, ...)`, skipping the predicate.
///
/// Returns `None` when the attribute is not `cfg_attr` or its arguments do not parse as a
/// comma-separated meta list.
fn cfg_attr_nested_attributes(attribute: &Attribute) -> Option<Vec<Meta>> {
    if !attribute.path().is_ident("cfg_attr") {
        return None;
    }

    let arguments = attribute
        .parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)
        .ok()?;
    Some(arguments.into_iter().skip(1).collect())
}

/// Returns whether a `cfg_attr` attribute carries a nested attribute named `name`.
fn cfg_attr_contains(attribute: &Attribute, name: &str) -> bool {
    cfg_attr_nested_attributes(attribute)
        .is_some_and(|nested| nested.iter().any(|meta| meta.path().is_ident(name)))
}

/// Extracts the string argument from a `fluent_alias` nested inside `cfg_attr`.
fn fluent_alias_from_cfg_attr(attribute: &Attribute) -> Option<String> {
    cfg_attr_nested_attributes(attribute)?
        .into_iter()
        .find_map(|meta| match meta {
            Meta::List(list) if list.path.is_ident("fluent_alias") => {
                list.parse_args::<syn::LitStr>().ok().map(|lit| lit.value())
            }
            _ => None,
        })
}
