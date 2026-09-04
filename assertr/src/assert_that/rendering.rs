use core::fmt;

use crate::{
    AssertThat, EqContext,
    mode::Mode,
    renderer::{CustomRenderer, RenderingBudget, RenderingContext},
};

impl<'t, T, M: Mode, R> AssertThat<'t, T, M, R> {
    /// Returns this chain's diagnostic rendering context.
    ///
    /// Custom assertion implementations use [`RenderingContext::value`],
    /// [`RenderingContext::values`], and [`RenderingContext::borrowed_values`] instead of formatting
    /// diagnostic values directly. This honors both the active
    /// [`ValueRenderer`](crate::ValueRenderer) and [`RenderingBudget`]. A rendered value always
    /// retains type metadata. Customize its hint through
    /// [`Typed::with_type_hint`](crate::renderer::Typed::with_type_hint) and its text visibility
    /// through [`Typed::show_type_hint`](crate::renderer::Typed::show_type_hint).
    ///
    /// ```
    /// use assertr::prelude::*;
    /// use assertr::failure::FailureKind;
    ///
    /// trait EvenAssertions<R = DebugRenderer> {
    ///     fn is_even(self) -> Self
    ///     where
    ///         R: ValueRenderer<u32>;
    /// }
    ///
    /// impl<M: Mode, R> EvenAssertions<R> for AssertThat<'_, u32, M, R> {
    ///     #[track_caller]
    ///     fn is_even(self) -> Self
    ///     where
    ///         R: ValueRenderer<u32>,
    ///     {
    ///         self.track_assertion();
    ///         if self.actual() % 2 != 0 {
    ///             self.failure(FailureKind::Predicate)
    ///                 .actual(self.render().value(self.actual()))
    ///                 .relation("is not even")
    ///                 .raise();
    ///         }
    ///         self
    ///     }
    /// }
    ///
    /// assert_that!(4).is_even();
    /// ```
    #[must_use]
    pub const fn render(&self) -> RenderingContext<'_, R> {
        RenderingContext::new(&self.state.renderer, self.state.rendering_budget)
    }

    pub(crate) fn eq_context(&self) -> EqContext<'_, R> {
        EqContext::with_rendering(self.render())
    }

    /// Sets the limits applied when this chain renders diagnostic values and collections.
    ///
    /// The budget is inherited by assertions derived through `satisfies`, `derive`, and related
    /// methods. [`RenderingBudget::default`] keeps diagnostics generous but bounded.
    /// [`RenderingBudget::unlimited`] restores complete rendering.
    ///
    /// ```
    /// use assertr::prelude::*;
    ///
    /// let failures = assert_that!([1, 2, 3, 4])
    ///     .with_rendering_budget(
    ///         RenderingBudget::builder()
    ///             .max_items(2)
    ///             .max_leaf_characters(1_000)
    ///             .build(),
    ///     )
    ///     .with_location(false)
    ///     .capture(|it| it.contains(5));
    ///
    /// assert_that!(TextReporter.report(&failures[0])).contains("... 2 more elements ...");
    /// ```
    #[must_use]
    pub fn with_rendering_budget(mut self, budget: RenderingBudget) -> Self {
        self.state.rendering_budget = budget;
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

#[cfg(test)]
mod tests {
    use crate::prelude::*;

    #[derive(PartialEq)]
    struct Secret(u32);

    #[test]
    fn default_budget_is_bounded_and_unlimited_restores_complete_output() {
        let values = (0..260).collect::<Vec<_>>();
        let bounded = assert_that!(&values)
            .with_location(false)
            .capture(|it| it.contains(999));
        assert_that!(TextReporter.report(&bounded[0])).contains("... 4 more elements ...");

        let unlimited = assert_that!(&values)
            .with_rendering_budget(RenderingBudget::unlimited())
            .with_location(false)
            .capture(|it| it.contains(999));
        assert_that!(TextReporter.report(&unlimited[0])).does_not_contain("more elements");
    }

    #[test]
    fn budget_is_inherited_by_derived_assertions() {
        let failures = assert_that!((123_456,))
            .with_rendering_budget(RenderingBudget::builder().max_leaf_characters(3).build())
            .with_location(false)
            .capture(|it| {
                it.satisfies(
                    |tuple| &tuple.0,
                    |value| {
                        value.is_equal_to(0);
                    },
                )
            });

        assert_that!(TextReporter.report(&failures[0]))
            .contains("Actual: 123... 3 more characters ...");
    }

    #[test]
    fn non_debug_subject_can_use_debug_format_closure() {
        let failures = assert_that!(Secret(1))
            .with_debug_format(|value, f| write!(f, "Secret({})", value.0))
            .with_location(false)
            .capture(|it| it.is_equal_to(Secret(2)));

        assert_that!(TextReporter.report(&failures[0]))
            .contains("Expected: Secret(2)")
            .contains("Actual: Secret(1)");
    }
}
