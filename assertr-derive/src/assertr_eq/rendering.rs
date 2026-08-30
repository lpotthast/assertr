//! Token generation for matcher diagnostics that tolerate non-`Debug` field types.

use proc_macro2::TokenStream;
use quote::quote;

/// Renders one `Eq<T>` matcher value.
///
/// `render_value` must be an expression of type `::core::fmt::Result` that renders the `Eq::Eq`
/// payload. It can reference the payload as `value` and the formatter through `formatter`.
pub(super) fn render_debug_eq(
    value: &TokenStream,
    formatter: &TokenStream,
    render_value: &TokenStream,
    assertr: &TokenStream,
) -> TokenStream {
    quote! {
        match #value {
            #assertr::Eq::Any => #formatter.write_str("Eq::Any"),
            #assertr::Eq::Eq(value) => {
                #formatter.write_str("Eq::Eq(")?;
                #render_value?;
                #formatter.write_str(")")
            }
        }
    }
}

/// Renders a payload of concrete type through `assertr::__private::SpecializedDebug`, an
/// autoref-specialization fallback that writes `<unrendered>` for types the `DebugRenderer`
/// cannot render.
///
/// The specialization resolves where this expression is emitted, so the payload type must be
/// concrete at that location. Payload types naming matcher generics use bounds on the matcher's
/// `Debug` impl instead (see `RenderableEq`).
pub(super) fn specialized_render_value(
    value: &TokenStream,
    renderer: &TokenStream,
    formatter: &TokenStream,
    assertr: &TokenStream,
) -> TokenStream {
    quote! {{
        use #assertr::__private::SpecializedDebugRender as _;

        #assertr::__private::SpecializedDebug {
            value: #value,
            renderer: #renderer,
        }
        .render(#formatter)
    }}
}
