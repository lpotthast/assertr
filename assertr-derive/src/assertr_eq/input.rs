//! Darling input models and selection of fields exposed by the generated matcher.

use darling::{FromDeriveInput, FromField, ast};
use syn::{Generics, Ident, Path, Type, Visibility};

/// Parsed source-field metadata and `#[assertr_eq(...)]` options.
#[derive(Debug, FromField)]
#[darling(attributes(assertr_eq))]
pub(super) struct AssertrEqField {
    pub(super) ident: Option<Ident>,
    pub(super) ty: Type,
    pub(super) vis: Visibility,

    #[darling(default)]
    pub(super) map_type: Option<Type>,

    #[darling(default)]
    pub(super) compare_with: Option<Path>,

    // Extra trait bounds for the generated `AssertrPartialEq` impl when this field uses
    // `compare_with`. This is the body of a `where` clause without the leading keyword.
    // Renderer bounds are added automatically, so this contains comparison-specific bounds only.
    // Kept as the literal so predicate parse errors point into the user's attribute input.
    #[darling(default)]
    pub(super) compare_bounds: Option<syn::LitStr>,
}

impl AssertrEqField {
    /// Returns the type stored by the matcher field.
    ///
    /// `map_type` may intentionally differ from the source field type when comparison is delegated
    /// to a custom mapping or comparison function.
    pub(super) fn expected_type(&self) -> &Type {
        self.map_type.as_ref().unwrap_or(&self.ty)
    }
}

/// Parsed derive input for a supported named-field struct.
#[derive(Debug, FromDeriveInput)]
#[darling(attributes(assertr_eq), supports(struct_named))]
pub(super) struct AssertrEqInput {
    pub(super) ident: Ident,
    pub(super) generics: Generics,
    pub(super) data: ast::Data<(), AssertrEqField>,
}

/// A public source field paired with its required identifier.
pub(super) type PublicField<'a> = (&'a AssertrEqField, &'a Ident);

/// Selects the public named fields that participate in the generated matcher.
///
/// Fields without an identifier cannot occur here: darling's `supports(struct_named)` already
/// rejected tuple and unit structs before this runs.
pub(super) fn public_fields(fields: &ast::Fields<AssertrEqField>) -> Vec<PublicField<'_>> {
    fields
        .iter()
        .filter(|field| matches!(field.vis, Visibility::Public(_)))
        .filter_map(|field| field.ident.as_ref().map(|ident| (field, ident)))
        .collect()
}
