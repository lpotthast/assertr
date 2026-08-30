//! Recognition and parsing of fluent-alias helper attributes.

use syn::punctuated::Punctuated;
use syn::{Attribute, Meta, Token};

const HELPER_ATTRIBUTES: [&str; 2] = ["fluent_alias", "no_fluent_alias"];

/// Returns whether an attribute list contains `name` directly or inside `cfg_attr`.
pub(super) fn has_attribute(attributes: &[Attribute], name: &str) -> bool {
    attributes
        .iter()
        .any(|attribute| attribute.path().is_ident(name) || cfg_attr_contains(attribute, name))
}

/// Removes attributes consumed by the fluent-alias macro.
///
/// A `cfg_attr` is retained when it also contains attributes unrelated to alias generation.
pub(super) fn remove_helper_attributes(attributes: &mut Vec<Attribute>) {
    attributes.retain_mut(|attribute| {
        if is_helper_meta(&attribute.meta) {
            return false;
        }

        let Some(arguments) = cfg_attr_arguments(attribute) else {
            return true;
        };
        let mut arguments = arguments.into_iter();
        let Some(predicate) = arguments.next() else {
            return true;
        };
        let nested = arguments.collect::<Vec<_>>();
        let retained = nested
            .iter()
            .filter(|meta| !is_helper_meta(meta))
            .collect::<Vec<_>>();

        if retained.len() == nested.len() {
            return true;
        }
        if retained.is_empty() {
            return false;
        }

        attribute.meta = syn::parse_quote! {
            cfg_attr(#predicate, #(#retained),*)
        };
        true
    });
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
    Some(cfg_attr_arguments(attribute)?.into_iter().skip(1).collect())
}

/// Parses all arguments inside `cfg_attr(predicate, attr, ...)`.
fn cfg_attr_arguments(attribute: &Attribute) -> Option<Punctuated<Meta, Token![,]>> {
    if !attribute.path().is_ident("cfg_attr") {
        return None;
    }

    attribute
        .parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)
        .ok()
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

/// Returns whether a meta item is a helper consumed by the fluent-alias macro.
fn is_helper_meta(meta: &Meta) -> bool {
    HELPER_ATTRIBUTES
        .iter()
        .any(|name| meta.path().is_ident(name))
}

#[cfg(test)]
mod tests {
    use syn::{Attribute, parse_quote};

    use super::remove_helper_attributes;

    #[test]
    fn removes_only_helpers_from_cfg_attr() {
        let mut attributes: Vec<Attribute> = vec![parse_quote! {
            #[cfg_attr(
                feature = "fluent",
                allow(non_snake_case),
                fluent_alias("Have_NAME"),
                must_use
            )]
        }];

        remove_helper_attributes(&mut attributes);

        assert_eq!(
            attributes,
            vec![parse_quote! {
                #[cfg_attr(feature = "fluent", allow(non_snake_case), must_use)]
            }]
        );
    }

    #[test]
    fn removes_direct_helpers_and_cfg_attr_containing_only_helpers() {
        let mut attributes: Vec<Attribute> = vec![
            parse_quote! { #[fluent_alias("be_ready")] },
            parse_quote! { #[cfg_attr(feature = "fluent", no_fluent_alias)] },
            parse_quote! { #[track_caller] },
        ];

        remove_helper_attributes(&mut attributes);

        assert_eq!(attributes, vec![parse_quote! { #[track_caller] }]);
    }
}
