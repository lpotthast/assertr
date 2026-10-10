//! Recognition and parsing of fluent-alias helper attributes.

use syn::{Attribute, LitStr, Meta, Token, punctuated::Punctuated};

const HELPER_ATTRIBUTES: [&str; 2] = ["fluent_alias", "no_fluent_alias"];

/// Collects the helper attributes consumed by the fluent-alias macro, in source order, whether
/// written directly or inside `cfg_attr`.
pub(super) fn helper_metas(attributes: &[Attribute]) -> Vec<Meta> {
    attributes
        .iter()
        .flat_map(|attribute| {
            if is_helper_meta(&attribute.meta) {
                vec![attribute.meta.clone()]
            } else {
                cfg_attr_nested_attributes(attribute)
                    .unwrap_or_default()
                    .into_iter()
                    .filter(is_helper_meta)
                    .collect()
            }
        })
        .collect()
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

/// Parses the alias name of a `fluent_alias` helper.
///
/// Rejects a `fluent_alias` whose argument is not a single string literal.
pub(super) fn alias_literal(meta: &Meta) -> syn::Result<LitStr> {
    meta.require_list()
        .and_then(syn::MetaList::parse_args::<LitStr>)
        .map_err(|error| {
            syn::Error::new(
                error.span(),
                "expected the alias name as a string literal, as in `#[fluent_alias(\"be_ready\")]`",
            )
        })
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

/// Returns whether a meta item is a helper consumed by the fluent-alias macro.
fn is_helper_meta(meta: &Meta) -> bool {
    HELPER_ATTRIBUTES
        .iter()
        .any(|name| meta.path().is_ident(name))
}

#[cfg(test)]
mod tests {
    use renamed_assertr::prelude::*;
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

        let expected: Vec<Attribute> = vec![parse_quote! {
            #[cfg_attr(feature = "fluent", allow(non_snake_case), must_use)]
        }];
        assert_that!(attributes).is_equal_to(expected);
    }

    #[test]
    fn removes_direct_helpers_and_cfg_attr_containing_only_helpers() {
        let mut attributes: Vec<Attribute> = vec![
            parse_quote! { #[fluent_alias("be_ready")] },
            parse_quote! { #[cfg_attr(feature = "fluent", no_fluent_alias)] },
            parse_quote! { #[track_caller] },
        ];

        remove_helper_attributes(&mut attributes);

        let expected: Vec<Attribute> = vec![parse_quote! { #[track_caller] }];
        assert_that!(attributes).is_equal_to(expected);
    }
}
