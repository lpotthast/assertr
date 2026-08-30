//! Rendering individual values in assertion failure messages.

use core::{
    borrow::Borrow,
    fmt::{self, Debug},
    marker::PhantomData,
};

use crate::assertions::collection::CollectionStyle;

/// Formats individual values in assertion diagnostics.
///
/// A `ValueRenderer<T>` writes one `&T` to a [`fmt::Formatter`]. Assertr controls the surrounding
/// failure layout. One renderer type may implement this trait for any set of value types, and each
/// assertion method requires only the implementations its failure path uses.
///
/// The renderer is tracked as type state on [`crate::AssertThat`] (the `R` type parameter) and
/// defaults to [`DebugRenderer`], which delegates to [`fmt::Debug`]. Supply a custom renderer
/// with [`crate::AssertThat::with_renderer`] (or [`crate::AssertThat::with_debug_format`] for an
/// inline closure) for types that do not implement `Debug`.
///
/// # Capability bounds belong to methods
///
/// [`crate::AssertThat`] does not require `R: ValueRenderer<T>` at the struct or assertion-trait
/// implementation level. A chain must exist before installing a renderer for a non-`Debug`
/// subject. Projections can change `T`, and some methods never render the subject.
///
/// Each assertion method therefore declares only the `ValueRenderer<U>` capabilities used by
/// its own failure path. Blanket assertion-trait implementations remain available for every `R`.
/// This keeps unrelated methods available and reports a missing capability on the method that
/// needs it. Projection and extraction methods preserve `R` instead of resetting it to
/// [`DebugRenderer`].
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
/// exact requirement.
///
/// # `Clone` requirement
///
/// Assertions that derive a child [`crate::AssertThat`] (notably the [`crate::AssertThat::derive`]
/// and [`crate::AssertThat::satisfies`] families, `is_some_satisfying`, `is_ok_satisfying`, and
/// assertion methods implemented by composing those operations) require the renderer to be
/// `Clone` so each derived child receives its own copy. [`DebugRenderer`] is `Copy`, so the
/// default adds no constraint. A custom renderer used in derived contexts must implement `Clone`
/// or `Copy`.
///
/// # Pretty-printing
///
/// Assertion templates render values with `{value:#?}`, so the [`fmt::Formatter`] passed to
/// [`ValueRenderer::fmt`] carries `f.alternate() == true` (the same flag `{:#?}` sets for
/// [`fmt::Debug`]). Renderers that want to honor pretty vs. compact output should branch on it.
/// [`DebugRenderer`] forwards directly to [`fmt::Debug::fmt`], so it honors the flag automatically.
///
/// ```
/// use core::fmt;
/// use assertr::ValueRenderer;
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

/// A [`Debug`] adapter that formats one value through a [`ValueRenderer`].
///
/// [`crate::AssertThat::render_value`] creates this adapter for custom assertion diagnostics.
pub struct Renderable<'a, T: ?Sized, R> {
    pub(crate) value: &'a T,
    pub(crate) renderer: &'a R,
}

impl<T: ?Sized, R> Debug for Renderable<'_, T, R>
where
    R: ValueRenderer<T>,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.renderer.fmt(self.value, f)
    }
}

/// A [`Debug`] adapter that formats a collection of values through a [`ValueRenderer`].
///
/// [`crate::AssertThat::render_values`] creates this adapter for custom assertion diagnostics.
pub struct RenderableValues<'a, T: ?Sized, R, B = &'a T> {
    pub(crate) values: &'a [B],
    pub(crate) renderer: &'a R,
    pub(crate) style: CollectionStyle,
    pub(crate) item: PhantomData<fn() -> T>,
}

impl<T: ?Sized, B, R> Debug for RenderableValues<'_, T, R, B>
where
    B: Borrow<T>,
    R: ValueRenderer<T>,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let values = self.values.iter().map(|value| Renderable {
            value: value.borrow(),
            renderer: self.renderer,
        });
        match self.style {
            CollectionStyle::List => f.debug_list().entries(values).finish(),
            CollectionStyle::Set => f.debug_set().entries(values).finish(),
        }
    }
}

/// A crate-internal adapter for borrowed map entries.
pub(crate) struct RenderableMap<'a, K, V, R> {
    pub(crate) entries: &'a [(&'a K, &'a V)],
    pub(crate) renderer: &'a R,
}

/// A crate-internal adapter for one-field enum variants such as `Some`, `Ok`, and `Ready`.
pub(crate) struct RenderableVariant<'a, T: ?Sized, R> {
    pub(crate) name: &'static str,
    pub(crate) value: &'a T,
    pub(crate) renderer: &'a R,
}

impl<T: ?Sized, R> Debug for RenderableVariant<'_, T, R>
where
    R: ValueRenderer<T>,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple(self.name)
            .field(&Renderable {
                value: self.value,
                renderer: self.renderer,
            })
            .finish()
    }
}

/// A crate-internal adapter for one-field structural wrappers such as `RefCell` and `RwLock`.
pub(crate) struct RenderableStructField<'a, T: ?Sized, R> {
    pub(crate) name: &'static str,
    pub(crate) field: &'static str,
    pub(crate) value: &'a T,
    pub(crate) renderer: &'a R,
}

impl<T: ?Sized, R> Debug for RenderableStructField<'_, T, R>
where
    R: ValueRenderer<T>,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct(self.name)
            .field(
                self.field,
                &Renderable {
                    value: self.value,
                    renderer: self.renderer,
                },
            )
            .finish()
    }
}

/// A crate-internal adapter for structural wrappers whose field is temporarily inaccessible.
pub(crate) struct RenderableUnavailableStructField {
    pub(crate) name: &'static str,
    pub(crate) field: &'static str,
    pub(crate) unavailable: &'static str,
}

struct Unavailable(&'static str);

impl Debug for Unavailable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.0)
    }
}

impl Debug for RenderableUnavailableStructField {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct(self.name)
            .field(self.field, &Unavailable(self.unavailable))
            .finish()
    }
}

impl<K, V, R> Debug for RenderableMap<'_, K, V, R>
where
    R: ValueRenderer<K> + ValueRenderer<V>,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_map()
            .entries(self.entries.iter().map(|(key, value)| {
                (
                    Renderable {
                        value: *key,
                        renderer: self.renderer,
                    },
                    Renderable {
                        value: *value,
                        renderer: self.renderer,
                    },
                )
            }))
            .finish()
    }
}
