use alloc::string::String;
use core::{fmt, marker::PhantomData};

use crate::{
    AssertThat, EqContext,
    assertions::collection::CollectionStyle,
    mode::Mode,
    renderer::{
        CustomRenderer, Renderable, RenderableMap, RenderableStructField,
        RenderableUnavailableStructField, RenderableValues, RenderableVariant,
    },
};

impl<'t, T, M: Mode, R> AssertThat<'t, T, M, R> {
    /// Borrows the current assertion subject.
    ///
    /// Custom assertion implementations use this to inspect the value being asserted.
    pub fn actual(&self) -> &T {
        self.actual.borrowed()
    }

    /// Adapts a value so its [`Debug`] output uses this assertion's renderer.
    ///
    /// Custom assertions can interpolate the returned [`Renderable`] into their diagnostics with
    /// `{:?}` or `{:#?}`.
    pub fn render_value<'a, U: ?Sized>(&'a self, value: &'a U) -> Renderable<'a, U, R> {
        Renderable {
            value,
            renderer: &self.state.renderer,
        }
    }

    /// Adapts a slice of references so its [`Debug`] output uses this assertion's renderer for
    /// every value and the requested structural style.
    pub fn render_values<'a, U: ?Sized>(
        &'a self,
        values: &'a [&'a U],
        style: CollectionStyle,
    ) -> RenderableValues<'a, U, R> {
        RenderableValues {
            values,
            renderer: &self.state.renderer,
            style,
            item: PhantomData,
        }
    }

    pub(crate) fn render_borrowed_values<'a, U: ?Sized, B>(
        &'a self,
        values: &'a [B],
        style: CollectionStyle,
    ) -> RenderableValues<'a, U, R, B>
    where
        B: core::borrow::Borrow<U>,
    {
        RenderableValues {
            values,
            renderer: &self.state.renderer,
            style,
            item: PhantomData,
        }
    }

    pub(crate) fn render_map<'a, K, V>(
        &'a self,
        entries: &'a [(&'a K, &'a V)],
    ) -> RenderableMap<'a, K, V, R> {
        RenderableMap {
            entries,
            renderer: &self.state.renderer,
        }
    }

    pub(crate) fn render_variant<'a, U: ?Sized>(
        &'a self,
        name: &'static str,
        value: &'a U,
    ) -> RenderableVariant<'a, U, R> {
        RenderableVariant {
            name,
            value,
            renderer: &self.state.renderer,
        }
    }

    pub(crate) fn render_struct_field<'a, U: ?Sized>(
        &'a self,
        name: &'static str,
        field: &'static str,
        value: &'a U,
    ) -> RenderableStructField<'a, U, R> {
        RenderableStructField {
            name,
            field,
            value,
            renderer: &self.state.renderer,
        }
    }

    pub(crate) fn render_unavailable_struct_field(
        name: &'static str,
        field: &'static str,
        unavailable: &'static str,
    ) -> RenderableUnavailableStructField {
        RenderableUnavailableStructField {
            name,
            field,
            unavailable,
        }
    }

    pub(crate) fn eq_context(&self) -> EqContext<'_, R> {
        EqContext::with_renderer(&self.state.renderer)
    }

    /// Sets the subject name shown in failure messages.
    #[must_use]
    pub fn with_subject_name(mut self, subject_name: impl Into<String>) -> Self {
        self.state.subject_name = Some(subject_name.into());
        self
    }

    /// Controls whether failures record the source file, line, and column.
    ///
    /// Disable locations when comparing a rendered failure exactly in a test.
    ///
    /// Assertions derived from this one (through `satisfies` and friends) inherit the setting.
    #[must_use]
    pub fn with_location(mut self, value: bool) -> Self {
        self.state.print_location = value;
        self
    }

    /// Renders the subject with the given closure in failure messages instead of through `Debug`.
    ///
    /// The closure has the shape of [`fmt::Debug::fmt`]. Use it to assert on a
    /// subject that does not implement `Debug`:
    ///
    /// ```
    /// use assertr::prelude::*;
    ///
    /// #[derive(PartialEq)]
    /// struct Secret(u32);
    ///
    /// assert_that!(Secret(1))
    ///     .with_debug_format(|value, f| write!(f, "Secret({})", value.0))
    ///     .is_equal_to(Secret(1));
    /// ```
    ///
    /// The closure renders `T` only. When a chain also has to render other types, for example the
    /// elements of a collection or an expected value of a different type, use a renderer that
    /// implements [`crate::ValueRenderer`] for each of them and pass it to
    /// [`AssertThat::with_renderer`].
    #[must_use]
    pub fn with_debug_format<F>(self, renderer: F) -> AssertThat<'t, T, M, CustomRenderer<F>>
    where
        F: Fn(&T, &mut fmt::Formatter<'_>) -> fmt::Result,
    {
        self.with_renderer(CustomRenderer(renderer))
    }

    /// Replaces the renderer used in failure messages.
    ///
    /// The renderer is type state on `AssertThat`, so every later assertion renders through `R2`.
    /// Implement [`ValueRenderer<T>`](crate::ValueRenderer) for each value a failure displays.
    /// Type-specific structural assertions compose leaf renderers into collection, iterator, map,
    /// range, and wrapper syntax. Generic assertions such as `has_length` treat the whole subject
    /// as opaque and require a renderer for it. Implement `Clone` when derived assertions such as
    /// `satisfies` need a copy.
    ///
    /// ```
    /// use core::fmt;
    /// use assertr::prelude::*;
    ///
    /// #[derive(PartialEq)]
    /// struct Secret(u32);
    ///
    /// #[derive(Clone, Copy)]
    /// struct SecretRenderer;
    ///
    /// impl ValueRenderer<Secret> for SecretRenderer {
    ///     fn fmt(&self, value: &Secret, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    ///         write!(f, "Secret({})", value.0)
    ///     }
    /// }
    ///
    /// assert_that!(Secret(1))
    ///     .with_renderer(SecretRenderer)
    ///     .is_equal_to(Secret(1));
    /// ```
    ///
    /// See [`crate::ValueRenderer`] for the `Clone` requirement and for honoring pretty-printing.
    #[must_use]
    pub fn with_renderer<R2>(self, renderer: R2) -> AssertThat<'t, T, M, R2> {
        let AssertThat { actual, state } = self;
        AssertThat {
            actual,
            state: state.with_renderer(renderer),
        }
    }
}
