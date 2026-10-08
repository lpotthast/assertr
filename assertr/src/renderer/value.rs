use core::fmt;

/// Formats individual values in assertion diagnostics.
///
/// A `ValueRenderer<T>` writes one `&T` to a [`fmt::Formatter`]. Assertr controls the surrounding
/// failure layout. One renderer type may implement this trait for any set of value types, and each
/// assertion method requires only the implementations its failure path uses.
///
/// The renderer is tracked as type state on [`crate::AssertThat`] (the `R` type parameter) and
/// defaults to [`DebugRenderer`], which delegates to [`fmt::Debug`]. Supply a custom renderer with
/// [`crate::AssertThat::with_renderer`] (or [`crate::AssertThat::with_debug_format`] for an inline
/// closure) for types that do not implement `Debug`.
///
/// # Capability bounds belong to methods
///
/// [`crate::AssertThat`] does not require `R: ValueRenderer<T>` at the struct or assertion-trait
/// implementation level. A chain must exist before installing a renderer for a non-`Debug` subject.
/// Projections can change `T`, and some methods never render the subject.
///
/// Each assertion method therefore declares only the `ValueRenderer<U>` capabilities used by its
/// own failure path. Blanket assertion-trait implementations remain available for every `R`. This
/// keeps unrelated methods available and reports a missing capability on the method that needs it.
/// Projection and extraction methods preserve `R` instead of resetting it to [`DebugRenderer`].
///
/// Custom leaf assertions access the active renderer through
/// [`AssertThat::render`](crate::AssertThat::render). Render every value included in their failure
/// text with [`RenderingContext::value`](crate::renderer::RenderingContext::value) or the context's
/// structural methods, such as
/// [`RenderingContext::borrowed_values`](crate::renderer::RenderingContext::borrowed_values), so
/// custom renderers and the chain's [`RenderingBudget`](crate::renderer::RenderingBudget) remain
/// effective. Use [`Rendered::show_type_hint`](crate::renderer::Rendered::show_type_hint) to
/// control whether text output shows a value's short type hint.
///
/// # Render leaf values, not structural wrappers
///
/// Type-specific structural assertions own the syntax for collections, iterators, sets, maps,
/// ranges, `Option`, `Result`, `Poll`, `RefCell`, mutexes, and locks. They require renderers only
/// for the leaf values they display. For example, collection membership assertions on `Vec<T>`
/// require `ValueRenderer<T>`, and map assertions compose key and value renderers themselves.
///
/// Generic assertions that treat their subject as opaque still require a renderer for the whole
/// subject. This includes direct equality and length assertions. Each method signature shows the
/// exact requirement. Identity assertions display pointer addresses through Assertr's internal
/// formatter and require no target renderer. Exact positional identity comparisons additionally
/// require `ValueRenderer<usize>` for length evidence. Their diagnostics respect the chain's
/// rendering budget.
///
/// # `Clone` requirement
///
/// Assertions that derive a child [`crate::AssertThat`] (notably the [`crate::AssertThat::derive`]
/// and [`crate::AssertThat::satisfies`] families, `is_some_satisfying`, `is_ok_satisfying`, and
/// assertion methods implemented by composing those operations) require the renderer to be `Clone`
/// so each derived child receives its own copy. [`DebugRenderer`] is `Copy`, so the default adds no
/// constraint. A custom renderer used in derived contexts must implement `Clone` or `Copy`, or be
/// installed by reference, as in `with_renderer(&renderer)`. A reference renders exactly like the
/// renderer it refers to.
///
/// # Pretty-printing
///
/// Assertions render values with `{value:#?}`, so the [`fmt::Formatter`] passed to
/// [`ValueRenderer::fmt`] carries `f.alternate() == true` (the same flag `{:#?}` sets for
/// [`fmt::Debug`]). Inline evidence, such as map keys in failure paths, uses the compact form.
/// Renderers that want to honor pretty vs. compact output should branch on it.
/// [`DebugRenderer`] forwards directly to [`fmt::Debug::fmt`], so it honors the flag automatically.
///
/// ```
/// use core::fmt;
/// use assertr::renderer::ValueRenderer;
///
/// struct MyType { field: u32 }
///
/// struct PrettyRenderer;
///
/// impl ValueRenderer<MyType> for PrettyRenderer {
///     fn fmt(&self, value: &MyType, f: &mut fmt::Formatter<'_>) -> fmt::Result {
///         if f.alternate() {
///             // multi-line, indented form for failure messages
///             write!(f, "MyType {{\n    field: {},\n}}", value.field)
///         } else {
///             write!(f, "MyType {{ field: {} }}", value.field)
///         }
///     }
/// }
/// ```
#[diagnostic::on_unimplemented(
    message = "`{Self}` cannot render values of type `{T}`",
    label = "missing value rendering support for `{T}`",
    note = "derive `Debug` for `{T}` or call `.with_debug_format(...)` / `.with_renderer(...)` before this assertion"
)]
pub trait ValueRenderer<T: ?Sized> {
    /// Formats `value` for assertion diagnostics.
    ///
    /// See the [trait docs](ValueRenderer#pretty-printing) for how to honor `f.alternate()`.
    ///
    /// # Errors
    ///
    /// Returns an error if writing to `f` fails.
    fn fmt(&self, value: &T, f: &mut fmt::Formatter<'_>) -> fmt::Result;
}

/// A borrowed renderer renders exactly like the renderer it refers to.
///
/// References are `Copy`, so `with_renderer(&renderer)` satisfies the `Clone` requirement of
/// derived assertions even when the renderer itself does not implement `Clone`.
impl<T: ?Sized, R: ValueRenderer<T> + ?Sized> ValueRenderer<T> for &R {
    fn fmt(&self, value: &T, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        (**self).fmt(value, f)
    }
}

/// The default renderer. Delegates to [`fmt::Debug`].
#[derive(Clone, Copy, Debug, Default)]
pub struct DebugRenderer;

#[diagnostic::do_not_recommend]
impl<T: fmt::Debug + ?Sized> ValueRenderer<T> for DebugRenderer {
    fn fmt(&self, value: &T, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(value, f)
    }
}

/// A [`ValueRenderer`] backed by a formatter function.
///
/// [`crate::AssertThat::with_debug_format`] creates this adapter. `F` is the formatter function
/// type.
#[derive(Clone, Copy)]
pub struct CustomRenderer<F>(pub(crate) F);

impl<T: ?Sized, F> ValueRenderer<T> for CustomRenderer<F>
where
    F: Fn(&T, &mut fmt::Formatter<'_>) -> fmt::Result,
{
    fn fmt(&self, value: &T, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0(value, f)
    }
}

#[cfg(test)]
mod tests {
    use core::fmt;

    use crate::prelude::*;

    /// Deliberately neither `Clone` nor `Copy`.
    struct Labelled(&'static str);

    impl ValueRenderer<i32> for Labelled {
        fn fmt(&self, value: &i32, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "{}({value})", self.0)
        }
    }

    mod borrowed_renderer {
        use super::*;

        #[test]
        fn renders_derived_assertions_without_a_clone_renderer() {
            let renderer = Labelled("value");

            let failures = assert_that!(Some(1))
                .with_renderer(&renderer)
                .capture(|it| {
                    it.is_some_satisfying(|value| {
                        value.is_equal_to(2);
                    })
                });

            assert_that!(failures).has_length(1);
            assert_that!(failures[0].to_string()).contains("value(1)");
        }
    }
}
