use core::fmt::{self, Debug};

/// Renders assertion values for diagnostics.
///
/// The renderer is tracked as type state on [`crate::AssertThat`] (the `R` type parameter) and
/// defaults to [`DebugRenderer`], which delegates to [`fmt::Debug`]. Supply a custom renderer
/// with [`crate::AssertThat::with_renderer`] (or [`crate::AssertThat::with_debug_format`] for an
/// inline closure) to produce diagnostics for types that do not implement `Debug`.
///
/// # `Clone` requirement
///
/// Assertions that derive a child [`crate::AssertThat`] (notably [`crate::AssertThat::derive`],
/// [`crate::AssertThat::satisfies`], `is_some_satisfying`, `is_ok_satisfying`, and the chained
/// `Vec`/`Array`/`Path`/`String` assertions that delegate through `derive`) require the renderer
/// to be `Clone` so each derived child receives its own copy. [`DebugRenderer`] is `Copy`, so the
/// default has no constraint; custom renderers used in chained contexts should derive `Clone`
/// (or `Copy`).
///
/// # Pretty-printing
///
/// Assertion templates render values with `{value:#?}`, so the [`fmt::Formatter`] passed to
/// [`AssertionRenderer::fmt`] carries `f.alternate() == true` (the same flag `{:#?}` sets for
/// [`fmt::Debug`]). Renderers that want to honor pretty vs. compact output should branch on it.
/// [`DebugRenderer`] forwards directly to [`fmt::Debug::fmt`], so it honors the flag automatically.
///
/// ```
/// use core::fmt;
/// use assertr::AssertionRenderer;
///
/// struct MyType { field: u32 }
///
/// struct PrettyRenderer;
///
/// impl AssertionRenderer<MyType> for PrettyRenderer {
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
    message = "`{Self}` cannot render assertion values of type `{T}`",
    label = "missing assertion rendering support for `{T}`",
    note = "derive `Debug` for `{T}` or call `.with_debug_format(...)` / `.with_renderer(...)` before this assertion"
)]
pub trait AssertionRenderer<T: ?Sized> {
    /// Formats `value` for assertion diagnostics.
    ///
    /// See the [trait docs](AssertionRenderer#pretty-printing) for how to honor `f.alternate()`.
    ///
    /// # Errors
    ///
    /// Returns an error if writing to `f` fails.
    fn fmt(&self, value: &T, f: &mut fmt::Formatter<'_>) -> fmt::Result;
}

#[derive(Clone, Copy, Debug, Default)]
pub struct DebugRenderer;

#[diagnostic::do_not_recommend]
impl<T: fmt::Debug + ?Sized> AssertionRenderer<T> for DebugRenderer {
    fn fmt(&self, value: &T, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(value, f)
    }
}

/// Renders values of type `F` using a custom formatter function.
#[derive(Clone, Copy)]
pub struct CustomRenderer<F>(pub(crate) F);

impl<T: ?Sized, F> AssertionRenderer<T> for CustomRenderer<F>
where
    F: Fn(&T, &mut fmt::Formatter<'_>) -> fmt::Result,
{
    fn fmt(&self, value: &T, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0(value, f)
    }
}

/// Tracks a value and the renderer it is to be rendered with.
pub struct Renderable<'a, T: ?Sized, R> {
    pub(crate) value: &'a T,
    pub(crate) renderer: &'a R,
}

impl<T: ?Sized, R> Debug for Renderable<'_, T, R>
where
    R: AssertionRenderer<T>,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.renderer.fmt(self.value, f)
    }
}

pub struct RenderableValues<'a, T, R> {
    pub(crate) values: &'a [&'a T],
    pub(crate) renderer: &'a R,
}

impl<T, R> Debug for RenderableValues<'_, T, R>
where
    R: AssertionRenderer<T>,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list()
            .entries(self.values.iter().map(|value| Renderable {
                value: *value,
                renderer: self.renderer,
            }))
            .finish()
    }
}
