//! Syntax-aware retention of generics needed by public matcher fields.

use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::{
    GenericParam, Generics, Ident, Type, parse_quote,
    visit::{self, Visit},
};

use super::{identifiers::tokens_mention_ident, input::PublicField};

/// Records which source generic parameters are referenced by visited syntax.
struct GenericUsage<'a> {
    parameters: &'a [&'a GenericParam],
    used: Vec<bool>,
}

impl<'a> GenericUsage<'a> {
    fn new(parameters: &'a [&'a GenericParam]) -> Self {
        Self {
            parameters,
            used: vec![false; parameters.len()],
        }
    }

    fn mark_path_ident(&mut self, ident: &Ident, allow_const: bool) {
        for (index, parameter) in self.parameters.iter().enumerate() {
            let matches = match parameter {
                GenericParam::Lifetime(_) => false,
                GenericParam::Type(parameter) => parameter.ident == *ident,
                GenericParam::Const(parameter) => allow_const && parameter.ident == *ident,
            };
            self.used[index] |= matches;
        }
    }

    fn mark_lifetime(&mut self, lifetime: &syn::Lifetime) {
        for (index, parameter) in self.parameters.iter().enumerate() {
            if let GenericParam::Lifetime(parameter) = parameter {
                self.used[index] |= parameter.lifetime.ident == lifetime.ident;
            }
        }
    }

    fn mark_macro_tokens(&mut self, tokens: &TokenStream) {
        for (index, parameter) in self.parameters.iter().enumerate() {
            self.used[index] |=
                tokens_mention_ident(tokens.clone(), generic_parameter_ident(parameter));
        }
    }
}

impl<'ast> Visit<'ast> for GenericUsage<'_> {
    fn visit_type_path(&mut self, path: &'ast syn::TypePath) {
        if path.qself.is_none()
            && path.path.leading_colon.is_none()
            && let Some(segment) = path.path.segments.first()
        {
            self.mark_path_ident(&segment.ident, false);
        }
        visit::visit_type_path(self, path);
    }

    fn visit_expr_path(&mut self, path: &'ast syn::ExprPath) {
        if path.qself.is_none()
            && path.path.leading_colon.is_none()
            && let Some(segment) = path.path.segments.first()
        {
            self.mark_path_ident(&segment.ident, true);
        }
        visit::visit_expr_path(self, path);
    }

    fn visit_lifetime(&mut self, lifetime: &'ast syn::Lifetime) {
        self.mark_lifetime(lifetime);
    }

    fn visit_macro(&mut self, mac: &'ast syn::Macro) {
        self.mark_macro_tokens(&mac.tokens);
    }
}

/// Produces the generic parameters and predicates required by the generated matcher type.
///
/// Only generics referenced structurally by public expected-field types are retained. Bounds,
/// defaults, and where predicates that depend on omitted generics are removed as well, preventing
/// private implementation details from leaking into the public companion type.
pub(super) fn matcher_generics(original: &Generics, fields: &[PublicField<'_>]) -> Generics {
    let parameters = original.params.iter().collect::<Vec<_>>();
    let mut retained = vec![false; parameters.len()];

    for (field, _) in fields {
        let mut usage = GenericUsage::new(&parameters);
        usage.visit_type(field.expected_type());
        for (retained, mentioned) in retained.iter_mut().zip(usage.used) {
            *retained |= mentioned;
        }
    }

    let mut matcher = original.clone();
    matcher.params = original
        .params
        .iter()
        .enumerate()
        .filter(|(index, _)| retained[*index])
        .map(|(_, parameter)| filtered_parameter(&parameters, &retained, parameter))
        .collect();

    if matcher.params.is_empty() {
        matcher.lt_token = None;
        matcher.gt_token = None;
    }

    if let Some(where_clause) = &mut matcher.where_clause {
        where_clause.predicates = where_clause
            .predicates
            .iter()
            .filter(|predicate| {
                let dependencies = where_predicate_dependencies(&parameters, predicate);
                dependencies.iter().any(|mentioned| *mentioned)
                    && dependencies_are_retained(&dependencies, &retained)
            })
            .cloned()
            .collect();
        if where_clause.predicates.is_empty() {
            matcher.where_clause = None;
        }
    }

    matcher
}

/// Returns whether a type structurally references any of the given generic parameters.
pub(super) fn mentions_generics(generics: &Generics, ty: &Type) -> bool {
    let parameters = generics.params.iter().collect::<Vec<_>>();
    let mut usage = GenericUsage::new(&parameters);
    usage.visit_type(ty);
    usage.used.iter().any(|used| *used)
}

/// Extends the matcher generics with the renderer bounds required by its `Debug` impl.
///
/// Fields whose types reference matcher generics cannot render through the
/// autoref-specialization fallback (it never resolves for a generic type), so their `Debug`
/// rendering requires `DebugRenderer` to support the field type. This mirrors the bounds
/// `#[derive(Debug)]` would place on generic parameters: the matcher is `Debug` exactly when
/// its generic field types are renderable, and non-`Debug` payloads use a custom renderer.
pub(super) fn matcher_debug_generics(
    matcher: &Generics,
    fields: &[PublicField<'_>],
    assertr: &TokenStream,
) -> Generics {
    let mut generics = matcher.clone();
    let mut bounded_types = Vec::new();

    for (field, _) in fields {
        let expected_type = field.expected_type();
        if !mentions_generics(matcher, expected_type) {
            continue;
        }
        let type_key = expected_type.to_token_stream().to_string();
        if bounded_types.contains(&type_key) {
            continue;
        }
        bounded_types.push(type_key);
        generics.make_where_clause().predicates.push(parse_quote! {
            #assertr::DebugRenderer: #assertr::AssertionRenderer<#expected_type>
        });
    }

    generics
}

fn generic_parameter_ident(parameter: &GenericParam) -> &Ident {
    match parameter {
        GenericParam::Lifetime(parameter) => &parameter.lifetime.ident,
        GenericParam::Type(parameter) => &parameter.ident,
        GenericParam::Const(parameter) => &parameter.ident,
    }
}

fn dependencies_are_retained(dependencies: &[bool], retained: &[bool]) -> bool {
    dependencies
        .iter()
        .zip(retained)
        .all(|(mentioned, retained)| !mentioned || *retained)
}

/// Removes bounds or defaults whose generic dependencies are absent from the matcher.
fn filtered_parameter(
    parameters: &[&GenericParam],
    retained: &[bool],
    parameter: &GenericParam,
) -> GenericParam {
    let mut parameter = parameter.clone();
    match &mut parameter {
        GenericParam::Lifetime(parameter) => {
            parameter.bounds = parameter
                .bounds
                .iter()
                .filter(|bound| {
                    let mut usage = GenericUsage::new(parameters);
                    usage.visit_lifetime(bound);
                    dependencies_are_retained(&usage.used, retained)
                })
                .cloned()
                .collect();
        }
        GenericParam::Type(parameter) => {
            parameter.bounds = parameter
                .bounds
                .iter()
                .filter(|bound| {
                    let mut usage = GenericUsage::new(parameters);
                    usage.visit_type_param_bound(bound);
                    dependencies_are_retained(&usage.used, retained)
                })
                .cloned()
                .collect();
            if parameter.default.as_ref().is_some_and(|(_, default)| {
                let mut usage = GenericUsage::new(parameters);
                usage.visit_type(default);
                !dependencies_are_retained(&usage.used, retained)
            }) {
                parameter.default = None;
            }
        }
        GenericParam::Const(parameter) => {
            if parameter.default.as_ref().is_some_and(|(_, default)| {
                let mut usage = GenericUsage::new(parameters);
                usage.visit_expr(default);
                !dependencies_are_retained(&usage.used, retained)
            }) {
                parameter.default = None;
            }
        }
    }
    parameter
}

fn where_predicate_dependencies(
    parameters: &[&GenericParam],
    predicate: &syn::WherePredicate,
) -> Vec<bool> {
    let mut usage = GenericUsage::new(parameters);
    usage.visit_where_predicate(predicate);
    usage.used
}
