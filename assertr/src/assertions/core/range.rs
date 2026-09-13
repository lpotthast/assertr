use crate::borrow_for::{BorrowFor, borrow_for};
use crate::{
    AssertThat, AssertionContext, Expectation, ExpectationDiagnostics, Mode, ValueRenderer,
    failure::{FailureBuilder, FailureKind},
    renderer::RenderingContext,
};
use alloc::{format, string::String};
use core::marker::PhantomData;
use core::ops::{
    Bound::{Excluded, Included, Unbounded},
    RangeBounds,
};

/// Checks whether a range contains an element.
/// Uses [`RangeBounds::contains`], preserving inclusive, exclusive, and unbounded endpoints.
pub struct ContainsElement<B, E = B>(E, PhantomData<fn() -> B>);

impl<B> ContainsElement<B> {
    /// Owns the expected operand, using its type as the range bound type.
    ///
    /// The operand determines the type even before this definition is used with a range.
    /// Use [`Self::borrowing`] to select a different bound type for a borrowed operand.
    ///
    /// ```
    /// use assertr::{matchers::range::ContainsElement, prelude::*};
    ///
    /// let expected = ContainsElement::new(&2);
    /// assert_that!(&1..&3).matches(&expected);
    /// assert_that!(..).matches(ContainsElement::new(String::from("a")));
    /// ```
    #[must_use]
    pub const fn new(expected: B) -> Self {
        Self(expected, PhantomData)
    }

    /// Owns an operand whose borrowed view is selected for bound type `B`.
    ///
    /// Pass a reference to reuse an expected value. Construction does not borrow or clone the
    /// operand. Evaluation borrows it through [`BorrowFor`]. Diagnostics require renderers for
    /// `B` and `E::View`, without requiring one for the operand wrapper.
    /// For borrowed bounds, selecting the pointee type also avoids needing a reference renderer.
    ///
    /// ```
    /// use assertr::{matchers::range::ContainsElement, prelude::*};
    ///
    /// let value = String::from("b");
    /// let expected = ContainsElement::<String>::borrowing(&value);
    /// assert_that!(String::from("a")..String::from("c")).matches(&expected);
    /// ```
    #[must_use]
    pub const fn borrowing<E: BorrowFor<B>>(expected: E) -> ContainsElement<B, E> {
        ContainsElement(expected, PhantomData)
    }
}

impl<B, E: BorrowFor<B>, Range: RangeBounds<B> + ?Sized, R> Expectation<Range, R>
    for ContainsElement<B, E>
where
    B: PartialOrd<E::View>,
    E::View: PartialOrd<B>,
{
    type Success<'a>
        = ()
    where
        Self: 'a,
        Range: 'a;
    type Rejection<'a>
        = &'a E::View
    where
        Self: 'a,
        Range: 'a;
    fn evaluate<'a>(
        &'a self,
        actual: &'a Range,
        _: &AssertionContext<'_, R>,
    ) -> Result<(), Self::Rejection<'a>> {
        let expected = borrow_for::<B, _>(&self.0);
        if actual.contains(expected) {
            Ok(())
        } else {
            Err(expected)
        }
    }
}

impl<
    B,
    E: BorrowFor<B>,
    Range: RangeBounds<B> + ?Sized,
    R: ValueRenderer<B> + ValueRenderer<E::View>,
> ExpectationDiagnostics<Range, R> for ContainsElement<B, E>
where
    B: PartialOrd<E::View>,
    E::View: PartialOrd<B>,
{
    const KIND: FailureKind = FailureKind::Membership;
    fn explain<'a, Target>(
        &'a self,
        rejected: Option<(&'a Range, Self::Rejection<'a>)>,
        failure: FailureBuilder<Target>,
        context: &AssertionContext<'_, R>,
    ) -> FailureBuilder<Target> {
        let render = context.render();
        let (failure, expected) = match rejected {
            None => (failure.relation("contains"), borrow_for::<B, _>(&self.0)),
            Some((actual, expected)) => (
                failure
                    .actual(render_range(render, actual))
                    .relation("does not contain"),
                expected,
            ),
        };
        failure.expected(render.value(expected))
    }
}

/// Checks whether a range does not contain an element.
/// Uses [`RangeBounds::contains`], preserving inclusive, exclusive, and unbounded endpoints.
pub struct DoesNotContainElement<B, E = B>(E, PhantomData<fn() -> B>);

impl<B> DoesNotContainElement<B> {
    /// Owns the unexpected operand, using its type as the range bound type.
    ///
    /// The operand determines the type even before this definition is used with a range.
    /// Use [`Self::borrowing`] to select a different bound type for a borrowed operand.
    ///
    /// ```
    /// use assertr::{matchers::range::DoesNotContainElement, prelude::*};
    ///
    /// let unexpected = DoesNotContainElement::new(&3);
    /// assert_that!(&1..&3).matches(&unexpected);
    /// ```
    #[must_use]
    pub const fn new(expected: B) -> Self {
        Self(expected, PhantomData)
    }

    /// Owns an operand whose borrowed view is selected for bound type `B`.
    ///
    /// Construction, borrowing, and rendering follow [`ContainsElement::borrowing`].
    ///
    /// ```
    /// use assertr::{matchers::range::DoesNotContainElement, prelude::*};
    ///
    /// let value = String::from("z");
    /// let unexpected = DoesNotContainElement::<String>::borrowing(&value);
    /// assert_that!(String::from("a")..String::from("c")).matches(&unexpected);
    /// ```
    #[must_use]
    pub const fn borrowing<E: BorrowFor<B>>(expected: E) -> DoesNotContainElement<B, E> {
        DoesNotContainElement(expected, PhantomData)
    }
}

impl<B, E: BorrowFor<B>, Range: RangeBounds<B> + ?Sized, R> Expectation<Range, R>
    for DoesNotContainElement<B, E>
where
    B: PartialOrd<E::View>,
    E::View: PartialOrd<B>,
{
    type Success<'a>
        = ()
    where
        Self: 'a,
        Range: 'a;
    type Rejection<'a>
        = &'a E::View
    where
        Self: 'a,
        Range: 'a;
    fn evaluate<'a>(
        &'a self,
        actual: &'a Range,
        _: &AssertionContext<'_, R>,
    ) -> Result<(), Self::Rejection<'a>> {
        let expected = borrow_for::<B, _>(&self.0);
        if actual.contains(expected) {
            Err(expected)
        } else {
            Ok(())
        }
    }
}

impl<
    B,
    E: BorrowFor<B>,
    Range: RangeBounds<B> + ?Sized,
    R: ValueRenderer<B> + ValueRenderer<E::View>,
> ExpectationDiagnostics<Range, R> for DoesNotContainElement<B, E>
where
    B: PartialOrd<E::View>,
    E::View: PartialOrd<B>,
{
    const KIND: FailureKind = FailureKind::Membership;
    fn explain<'a, Target>(
        &'a self,
        rejected: Option<(&'a Range, Self::Rejection<'a>)>,
        failure: FailureBuilder<Target>,
        context: &AssertionContext<'_, R>,
    ) -> FailureBuilder<Target> {
        let render = context.render();
        let (failure, expected) = match rejected {
            None => (
                failure.relation("does not contain"),
                borrow_for::<B, _>(&self.0),
            ),
            Some((actual, expected)) => (
                failure
                    .actual(render_range(render, actual))
                    .relation("contains"),
                expected,
            ),
        };
        failure.unexpected(render.value(expected))
    }
}

/// Checks whether a value is in range.
/// Uses [`RangeBounds::contains`], preserving inclusive, exclusive, and unbounded endpoints.
pub struct IsInRange<Range>(Range);

impl<Range> IsInRange<Range> {
    /// Owns the expected operand.
    #[must_use]
    pub const fn new(expected: Range) -> Self {
        Self(expected)
    }
}

impl<B: PartialOrd, Range: RangeBounds<B>, R> Expectation<B, R> for IsInRange<Range> {
    type Success<'a>
        = ()
    where
        Self: 'a,
        B: 'a;
    type Rejection<'a>
        = ()
    where
        Self: 'a,
        B: 'a;
    fn evaluate<'a>(&'a self, actual: &'a B, _: &AssertionContext<'_, R>) -> Result<(), ()> {
        if self.0.contains(actual) {
            Ok(())
        } else {
            Err(())
        }
    }
}

impl<B: PartialOrd, Range: RangeBounds<B>, R: ValueRenderer<B>> ExpectationDiagnostics<B, R>
    for IsInRange<Range>
{
    const KIND: FailureKind = FailureKind::Ordering;
    fn explain<Target>(
        &self,
        rejected: Option<(&B, ())>,
        failure: FailureBuilder<Target>,
        context: &AssertionContext<'_, R>,
    ) -> FailureBuilder<Target> {
        let render = context.render();
        let failure = match rejected {
            None => failure.relation("is in range"),
            Some((actual, ())) => failure
                .actual(render.value(actual))
                .relation("is not in range"),
        };
        failure.expected(render_range(render, &self.0))
    }
}

/// Checks whether a value is not in range.
/// Uses [`RangeBounds::contains`], preserving inclusive, exclusive, and unbounded endpoints.
pub struct IsNotInRange<Range>(Range);

impl<Range> IsNotInRange<Range> {
    /// Owns the expected operand.
    #[must_use]
    pub const fn new(expected: Range) -> Self {
        Self(expected)
    }
}

impl<B: PartialOrd, Range: RangeBounds<B>, R> Expectation<B, R> for IsNotInRange<Range> {
    type Success<'a>
        = ()
    where
        Self: 'a,
        B: 'a;
    type Rejection<'a>
        = ()
    where
        Self: 'a,
        B: 'a;
    fn evaluate<'a>(&'a self, actual: &'a B, _: &AssertionContext<'_, R>) -> Result<(), ()> {
        if self.0.contains(actual) {
            Err(())
        } else {
            Ok(())
        }
    }
}

impl<B: PartialOrd, Range: RangeBounds<B>, R: ValueRenderer<B>> ExpectationDiagnostics<B, R>
    for IsNotInRange<Range>
{
    const KIND: FailureKind = FailureKind::Ordering;
    fn explain<Target>(
        &self,
        rejected: Option<(&B, ())>,
        failure: FailureBuilder<Target>,
        context: &AssertionContext<'_, R>,
    ) -> FailureBuilder<Target> {
        let render = context.render();
        let failure = match rejected {
            None => failure.relation("is not in range"),
            Some((actual, ())) => failure.actual(render.value(actual)).relation("is in range"),
        };
        failure.unexpected(render_range(render, &self.0))
    }
}

/// Assertions over a range subject's membership.
///
/// Diagnostics use Rust range notation when possible. Ranges with excluded lower bounds use
/// explicit bound tuples, such as `(Excluded(1), Included(3))`.
///
/// Standard ranges with borrowed, sized bounds have inherent methods that select the bounds'
/// pointee type. This keeps owned and borrowed elements inferable even though those ranges
/// implement both `RangeBounds<B>` and `RangeBounds<&B>`. An unbounded `..` range selects the
/// operand's own type. Custom ranges use this trait, and fully qualified calls can select `B`
/// explicitly when a range supports more than one bound type.
/// These methods execute [`ContainsElement::borrowing`] and [`DoesNotContainElement::borrowing`]
/// with the selected bound type. Reusable matchers' `new` constructors instead infer the bound
/// type from the operand, including its reference type when a reference is passed.
///
/// ```
/// use assertr::prelude::*;
/// let lower = String::from("a");
/// let upper = String::from("z");
/// let element = String::from("m");
/// assert_that!(&lower..&upper).contains_element(&element);
/// assert_that!(..).contains_element(&element);
/// ```
#[cfg_attr(feature = "fluent", assertr_macros::fluent_aliases)]
#[allow(clippy::return_self_not_must_use)]
pub trait RangeBoundAssertions<B, Range: RangeBounds<B>, R = crate::DebugRenderer> {
    /// Asserts that the range contains `expected`.
    fn contains_element<E: BorrowFor<B>>(self, expected: E) -> Self
    where
        B: PartialOrd<E::View>,
        E::View: PartialOrd<B>,
        R: ValueRenderer<B> + ValueRenderer<E::View>;

    /// Asserts that the range does not contain `expected`.
    fn does_not_contain_element<E: BorrowFor<B>>(self, expected: E) -> Self
    where
        B: PartialOrd<E::View>,
        E::View: PartialOrd<B>,
        R: ValueRenderer<B> + ValueRenderer<E::View>;
}

/// Assertions over a value subject's membership in a range.
///
/// Ranges are displayed as described in [`RangeBoundAssertions`].
#[allow(clippy::return_self_not_must_use)]
#[cfg_attr(feature = "fluent", assertr_macros::fluent_aliases)]
pub trait RangeAssertions<B, R = crate::DebugRenderer> {
    /// Asserts that the subject is within `expected`.
    fn is_in_range(self, expected: impl RangeBounds<B>) -> Self
    where
        B: PartialOrd,
        R: ValueRenderer<B>;

    /// Asserts that the subject is outside `expected`.
    fn is_not_in_range(self, expected: impl RangeBounds<B>) -> Self
    where
        B: PartialOrd,
        R: ValueRenderer<B>;

    /// Alias of [`RangeAssertions::is_not_in_range`].
    #[track_caller]
    fn is_outside_of_range(self, expected: impl RangeBounds<B>) -> Self
    where
        Self: Sized,
        B: PartialOrd,
        R: ValueRenderer<B>,
    {
        self.is_not_in_range(expected)
    }
}

impl<B, Range: RangeBounds<B>, M: Mode, R> RangeBoundAssertions<B, Range, R>
    for AssertThat<'_, Range, M, R>
{
    #[track_caller]
    fn contains_element<E: BorrowFor<B>>(self, expected: E) -> Self
    where
        B: PartialOrd<E::View>,
        E::View: PartialOrd<B>,
        R: ValueRenderer<B> + ValueRenderer<E::View>,
    {
        self.apply_assertion(ContainsElement::<B>::borrowing(expected))
    }

    #[track_caller]
    fn does_not_contain_element<E: BorrowFor<B>>(self, expected: E) -> Self
    where
        B: PartialOrd<E::View>,
        E::View: PartialOrd<B>,
        R: ValueRenderer<B> + ValueRenderer<E::View>,
    {
        self.apply_assertion(DoesNotContainElement::<B>::borrowing(expected))
    }
}

// Standard ranges with borrowed bounds implement both RangeBounds<B> and RangeBounds<&B>.
// Inherent methods select the pointee view before borrowing the operand, avoiding ambiguity
// in the blanket assertion trait. Custom ranges keep using that trait unchanged.
macro_rules! borrowed_range_assertions {
    ($($range:ty),+ $(,)?) => {$(
        #[allow(clippy::return_self_not_must_use)]
        impl<'b, B, M: Mode, R> AssertThat<'_, $range, M, R> {
            /// Asserts that this range contains `expected`, using its bounds' pointee type.
            ///
            /// This is [`RangeBoundAssertions::contains_element`] with the native
            /// `RangeBounds<B>` view selected explicitly for borrowed bounds.
            #[track_caller]
            pub fn contains_element<E: BorrowFor<B>>(self, expected: E) -> Self
            where
                B: PartialOrd<E::View>,
                E::View: PartialOrd<B>,
                R: ValueRenderer<B> + ValueRenderer<E::View>,
            {
                RangeBoundAssertions::<B, $range, R>::contains_element(self, expected)
            }

            /// Asserts that this range excludes `expected`, using its bounds' pointee type.
            ///
            /// This is [`RangeBoundAssertions::does_not_contain_element`] with the native
            /// `RangeBounds<B>` view selected explicitly for borrowed bounds.
            #[track_caller]
            pub fn does_not_contain_element<E: BorrowFor<B>>(self, expected: E) -> Self
            where
                B: PartialOrd<E::View>,
                E::View: PartialOrd<B>,
                R: ValueRenderer<B> + ValueRenderer<E::View>,
            {
                RangeBoundAssertions::<B, $range, R>::does_not_contain_element(self, expected)
            }

            /// Fluent alias of [`Self::contains_element`].
            #[cfg(feature = "fluent")]
            #[track_caller]
            pub fn contain_element<E: BorrowFor<B>>(self, expected: E) -> Self
            where
                B: PartialOrd<E::View>,
                E::View: PartialOrd<B>,
                R: ValueRenderer<B> + ValueRenderer<E::View>,
            {
                self.contains_element(expected)
            }

            /// Fluent alias of [`Self::does_not_contain_element`].
            #[cfg(feature = "fluent")]
            #[track_caller]
            pub fn not_contain_element<E: BorrowFor<B>>(self, expected: E) -> Self
            where
                B: PartialOrd<E::View>,
                E::View: PartialOrd<B>,
                R: ValueRenderer<B> + ValueRenderer<E::View>,
            {
                self.does_not_contain_element(expected)
            }
        }
    )+};
}

borrowed_range_assertions!(
    core::ops::Range<&'b B>,
    core::ops::RangeInclusive<&'b B>,
    core::ops::RangeFrom<&'b B>,
    core::ops::RangeTo<&'b B>,
    core::ops::RangeToInclusive<&'b B>,
    (core::ops::Bound<&'b B>, core::ops::Bound<&'b B>),
);

#[allow(clippy::return_self_not_must_use)]
impl<M: Mode, R> AssertThat<'_, core::ops::RangeFull, M, R> {
    /// Asserts that this unbounded range contains `expected`.
    ///
    /// With no bound to select a comparison type, the operand's own type is used.
    #[track_caller]
    pub fn contains_element<E: PartialOrd>(self, expected: E) -> Self
    where
        R: ValueRenderer<E>,
    {
        RangeBoundAssertions::<E, core::ops::RangeFull, R>::contains_element(self, expected)
    }

    /// Asserts that this unbounded range excludes `expected`, which always fails.
    #[track_caller]
    pub fn does_not_contain_element<E: PartialOrd>(self, expected: E) -> Self
    where
        R: ValueRenderer<E>,
    {
        RangeBoundAssertions::<E, core::ops::RangeFull, R>::does_not_contain_element(self, expected)
    }

    /// Fluent alias of [`Self::contains_element`].
    #[cfg(feature = "fluent")]
    #[track_caller]
    pub fn contain_element<E: PartialOrd>(self, expected: E) -> Self
    where
        R: ValueRenderer<E>,
    {
        self.contains_element(expected)
    }

    /// Fluent alias of [`Self::does_not_contain_element`].
    #[cfg(feature = "fluent")]
    #[track_caller]
    pub fn not_contain_element<E: PartialOrd>(self, expected: E) -> Self
    where
        R: ValueRenderer<E>,
    {
        self.does_not_contain_element(expected)
    }
}

impl<B, M: Mode, R> RangeAssertions<B, R> for AssertThat<'_, B, M, R> {
    #[track_caller]
    fn is_in_range(self, expected: impl RangeBounds<B>) -> Self
    where
        B: PartialOrd,
        R: ValueRenderer<B>,
    {
        self.apply_assertion(IsInRange::new(expected))
    }

    #[track_caller]
    fn is_not_in_range(self, expected: impl RangeBounds<B>) -> Self
    where
        B: PartialOrd,
        R: ValueRenderer<B>,
    {
        self.apply_assertion(IsNotInRange::new(expected))
    }
}

fn render_range<B, Range: RangeBounds<B> + ?Sized, R>(
    rendering: RenderingContext<'_, R>,
    range: &Range,
) -> String
where
    R: ValueRenderer<B>,
{
    let start = range.start_bound().map(|value| rendering.value(value));
    let end = range.end_bound().map(|value| rendering.value(value));

    match (start, end) {
        // Rust's range operators cannot express an excluded start. Format the bounds around
        // their rendering adapters so the active renderer and budget still apply to each leaf.
        (start @ Excluded(_), end) => format!("({start:?}, {end:?})"),
        (Included(start), Included(end)) => format!("{start:?}..={end:?}"),
        (Included(start), Excluded(end)) => format!("{start:?}..{end:?}"),
        (Included(start), Unbounded) => format!("{start:?}.."),
        (Unbounded, Included(end)) => format!("..={end:?}"),
        (Unbounded, Excluded(end)) => format!("..{end:?}"),
        (Unbounded, Unbounded) => "..".into(),
    }
}

#[cfg(test)]
mod tests {
    use core::ops::Bound::{self, Excluded, Included, Unbounded};

    use crate::prelude::*;
    use indoc::formatdoc;

    type Bounds = (Bound<i32>, Bound<i32>);

    // Every range contains 2 and excludes 1. Cover ordinary notation and each excluded-start form.
    const RANGE_CASES: [(Bounds, &str); 4] = [
        ((Included(2), Excluded(3)), "2..3"),
        ((Excluded(1), Included(3)), "(Excluded(1), Included(3))"),
        ((Excluded(1), Excluded(3)), "(Excluded(1), Excluded(3))"),
        ((Excluded(1), Unbounded), "(Excluded(1), Unbounded)"),
    ];

    mod renderer_contract {
        use core::ops::{Bound, RangeBounds};

        use crate::{
            prelude::*,
            test_support::{
                NoRenderer, SENTINEL, SentinelRenderer, assert_trait_impl, rendered_text,
            },
        };

        #[test]
        fn traits_are_implemented_without_renderer_support() {
            assert_trait_impl!(
                AssertThat<'static, core::ops::Range<i32>, Panic, NoRenderer>
                    => RangeBoundAssertions<i32, core::ops::Range<i32>, NoRenderer>
            );
            assert_trait_impl!(
                AssertThat<'static, i32, Panic, NoRenderer>
                    => RangeAssertions<i32, NoRenderer>
            );

            assert_trait_impl!(super::super::ContainsElement<i32> => crate::Expectation<core::ops::Range<i32>, NoRenderer>);
            assert_trait_impl!(super::super::DoesNotContainElement<i32> => crate::Expectation<core::ops::Range<i32>, NoRenderer>);
            assert_trait_impl!(super::super::ContainsElement<&'static i32> => crate::Expectation<core::ops::Range<&'static i32>, NoRenderer>);
            assert_trait_impl!(super::super::DoesNotContainElement<&'static i32> => crate::Expectation<core::ops::Range<&'static i32>, NoRenderer>);
            assert_trait_impl!(super::super::ContainsElement<i32, &'static i32> => crate::Expectation<core::ops::Range<i32>, NoRenderer>);
            assert_trait_impl!(super::super::DoesNotContainElement<i32, &'static i32> => crate::Expectation<core::ops::Range<i32>, NoRenderer>);
            assert_trait_impl!(super::super::IsInRange<core::ops::Range<i32>> => crate::Expectation<i32, NoRenderer>);
            assert_trait_impl!(super::super::IsNotInRange<core::ops::Range<i32>> => crate::Expectation<i32, NoRenderer>);
        }

        #[test]
        fn failures_render_bounds_and_values_with_the_active_renderer() {
            let bound_failures = assert_that!(1..3)
                .with_renderer(SentinelRenderer)
                .with_location(false)
                .capture(|it| it.contains_element(4));
            assert_that!(ToHumanReadableText.render(&bound_failures[0]))
                .contains(format!("{SENTINEL}..{SENTINEL}"));

            let value_failures = assert_that!(1)
                .with_renderer(SentinelRenderer)
                .with_location(false)
                .capture(|it| it.is_in_range(2..=3));
            assert_that!(ToHumanReadableText.render(&value_failures[0]))
                .contains(SENTINEL)
                .contains(format!("{SENTINEL}..={SENTINEL}"));
        }

        #[test]
        #[allow(clippy::needless_borrows_for_generic_args)] // Borrowed operands are the regression.
        fn borrowed_bounds_render_pointees_without_reference_or_clone_support() {
            struct BoundRenderer;
            impl ValueRenderer<i32> for BoundRenderer {
                fn fmt(&self, value: &i32, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                    write!(f, "bound({value})")
                }
            }

            let failures = assert_that!(&1..&3)
                .with_renderer(BoundRenderer)
                .with_expression("range")
                .with_location(false)
                .capture(|it| {
                    it.contains_element(&4)
                        .matches(super::super::ContainsElement::<i32>::borrowing(&4))
                });
            assert_that!(failures).has_length(2);
            for failure in &failures {
                assert_that!(failure).has_text_report(indoc::indoc! {"
                -------- assertr --------
                Expression: `range`

                Actual: bound(1)..bound(3)

                does not contain

                Expected: bound(4)
                -------- assertr --------
                "});
            }
        }

        #[test]
        fn custom_ranges_with_excluded_starts_honor_the_renderer_and_budget() {
            struct OpenStartRange {
                start: i32,
                end: Bound<i32>,
            }

            impl RangeBounds<i32> for OpenStartRange {
                fn start_bound(&self) -> Bound<&i32> {
                    Bound::Excluded(&self.start)
                }

                fn end_bound(&self) -> Bound<&i32> {
                    self.end.as_ref()
                }
            }

            for (end, rendered_end) in [
                (
                    Bound::Included(3),
                    "Included(<ren... 6 more characters ...)",
                ),
                (
                    Bound::Excluded(3),
                    "Excluded(<ren... 6 more characters ...)",
                ),
                (Bound::Unbounded, "Unbounded"),
            ] {
                let range = OpenStartRange { start: 1, end };
                let failures = assert_that!(range)
                    .with_renderer(SentinelRenderer)
                    .with_rendering_budget(RenderingBudget::default().with_max_leaf_characters(4))
                    .capture(|it| {
                        it.contains_element(1)
                            .matches(super::super::ContainsElement::<i32>::borrowing(&1))
                    });
                assert_that!(failures).has_length(2);
                for failure in &failures {
                    assert_that!(rendered_text(failure.actual.as_ref().unwrap())).is_equal_to(
                        format!("(Excluded(<ren... 6 more characters ...), {rendered_end})"),
                    );
                    assert_that!(rendered_text(failure.expected.as_ref().unwrap()))
                        .is_equal_to("<ren... 6 more characters ...");
                }
            }
        }
    }

    mod contains_element {
        use super::*;

        #[test]
        #[cfg(feature = "fluent")]
        fn fluent_alias_is_as_expected() {
            ("aa"..="zz").must().contain_element("aa");
        }

        #[test]
        fn caller_location_is_as_expected() {
            assert_caller_location!(assert_that!("aa".."zz"), contains_element("zz"));
        }

        #[test]
        #[cfg(feature = "fluent")]
        #[allow(clippy::needless_borrows_for_generic_args)] // Borrowed operands are the regression.
        fn fluent_aliases_infer_borrowed_bounds_and_unbounded_ranges() {
            (&1..&4).must().contain_element(&2);
            (&1..=&4).must().contain_element(&2);
            (&1..).must().contain_element(&2);
            (..&4).must().contain_element(&2);
            (..=&4).must().contain_element(&2);
            (Included(&1), Excluded(&4)).must().contain_element(&2);
            (..).must().contain_element(&2);
        }

        #[test]
        #[allow(clippy::needless_borrows_for_generic_args)] // Borrowed operands are the regression.
        fn borrowed_bounds_preserve_caller_locations() {
            assert_caller_location!(assert_that!(&1..&4), contains_element(&4));
            assert_caller_location!(assert_that!(&1..=&4), contains_element(&5));
            assert_caller_location!(assert_that!(&1..), contains_element(&0));
            assert_caller_location!(assert_that!(..&4), contains_element(&4));
            assert_caller_location!(assert_that!(..=&4), contains_element(&5));
            assert_caller_location!(
                assert_that!((Included(&1), Excluded(&4))),
                contains_element(&4)
            );
        }

        #[test]
        fn succeeds_when_element_is_contained() {
            assert_that!("aa"..="zz").contains_element("aa");
            assert_that!("aa"..="zz").contains_element("ab");
            assert_that!("aa"..="zz").contains_element("ac");
            assert_that!("aa"..="zz").contains_element("zx");
            assert_that!("aa"..="zz").contains_element("zy");
            assert_that!("aa"..="zz").contains_element("zz");
        }

        #[test]
        #[allow(clippy::needless_borrows_for_generic_args)] // Borrowed operands are the regression.
        fn borrowed_bounds_and_elements_infer_without_annotations() {
            let lower = 1;
            let upper = 4;
            let element = 2;
            assert_that!(&lower..&upper)
                .contains_element(element)
                .contains_element(&element);
            assert_that!(&lower..=&upper)
                .contains_element(element)
                .contains_element(&element);
            assert_that!(&lower..)
                .contains_element(element)
                .contains_element(&element);
            assert_that!(..&upper)
                .contains_element(element)
                .contains_element(&element);
            assert_that!(..=&upper)
                .contains_element(element)
                .contains_element(&element);
            assert_that!((Included(&lower), Excluded(&upper)))
                .contains_element(element)
                .contains_element(&element);
        }

        #[test]
        #[allow(clippy::needless_borrows_for_generic_args)] // Borrowed operands are the regression.
        fn unbounded_ranges_infer_owned_and_borrowed_elements() {
            let element = String::from("element");
            assert_that!(..).contains_element(2).contains_element(&2);
            assert_that!(..)
                .contains_element(String::from("element"))
                .contains_element(&element);
        }

        #[test]
        fn fails_when_element_is_not_contained() {
            for (range, rendered_range) in RANGE_CASES {
                assert_that!(range).contains_element(2);
                let failures = assert_that!(range)
                    .with_location(false)
                    .capture(|it| it.contains_element(1));
                assert_that!(failures).contains_exactly_satisfying([
                    |element: AssertThat<AssertionFailure, Capture>| {
                        element.derive(|value| value).has_text_report(formatdoc! {r"
                    -------- assertr --------
                    Expression: `range`

                    Actual: {rendered_range}

                    does not contain

                    Expected: 1
                    -------- assertr --------
                "});
                    },
                ]);
            }
        }
    }

    mod does_not_contain_element {
        use super::*;

        #[test]
        #[cfg(feature = "fluent")]
        fn fluent_alias_is_as_expected() {
            ("aa"..="zz").must().not_contain_element("a");
        }

        #[test]
        fn caller_location_is_as_expected() {
            assert_caller_location!(assert_that!("aa".."zz"), does_not_contain_element("cc"));
        }

        #[test]
        #[cfg(feature = "fluent")]
        #[allow(clippy::needless_borrows_for_generic_args)] // Borrowed operands are the regression.
        fn fluent_aliases_infer_borrowed_bounds_and_unbounded_ranges() {
            (&1..&4).must().not_contain_element(&4);
            (&1..=&4).must().not_contain_element(&5);
            (&1..).must().not_contain_element(&0);
            (..&4).must().not_contain_element(&4);
            (..=&4).must().not_contain_element(&5);
            (Included(&1), Excluded(&4)).must().not_contain_element(&4);
            let failures = (..).must().capture(|it| it.not_contain_element(&2));
            assert_that!(failures).has_length(1);
        }

        #[test]
        #[allow(clippy::needless_borrows_for_generic_args)] // Borrowed operands are the regression.
        fn borrowed_bounds_and_unbounded_ranges_preserve_caller_locations() {
            assert_caller_location!(assert_that!(&1..&4), does_not_contain_element(&2));
            assert_caller_location!(assert_that!(&1..=&4), does_not_contain_element(&2));
            assert_caller_location!(assert_that!(&1..), does_not_contain_element(&2));
            assert_caller_location!(assert_that!(..&4), does_not_contain_element(&2));
            assert_caller_location!(assert_that!(..=&4), does_not_contain_element(&2));
            assert_caller_location!(
                assert_that!((Included(&1), Excluded(&4))),
                does_not_contain_element(&2)
            );
            assert_caller_location!(assert_that!(..), does_not_contain_element(&2));
        }

        #[test]
        fn succeeds_when_element_is_not_contained() {
            assert_that!("aa"..="zz").does_not_contain_element("a");
            assert_that!("aa"..="zz").does_not_contain_element("AA");
        }

        #[test]
        #[allow(clippy::needless_borrows_for_generic_args)] // Borrowed operands are the regression.
        fn borrowed_bounds_and_elements_infer_without_annotations() {
            let lower = 1;
            let upper = 4;
            assert_that!(&lower..&upper)
                .does_not_contain_element(upper)
                .does_not_contain_element(&upper);
            assert_that!(&lower..=&upper)
                .does_not_contain_element(0)
                .does_not_contain_element(&0);
            assert_that!(&lower..)
                .does_not_contain_element(0)
                .does_not_contain_element(&0);
            assert_that!(..&upper)
                .does_not_contain_element(upper)
                .does_not_contain_element(&upper);
            assert_that!(..=&upper)
                .does_not_contain_element(5)
                .does_not_contain_element(&5);
            assert_that!((Included(&lower), Excluded(&upper)))
                .does_not_contain_element(upper)
                .does_not_contain_element(&upper);
        }

        #[test]
        #[allow(clippy::needless_borrows_for_generic_args)] // Borrowed operands are the regression.
        fn unbounded_ranges_infer_owned_and_borrowed_elements() {
            let element = String::from("element");
            let failures = assert_that!(..).capture(|it| {
                it.does_not_contain_element(2)
                    .does_not_contain_element(&2)
                    .does_not_contain_element(String::from("element"))
                    .does_not_contain_element(&element)
            });
            assert_that!(failures).has_length(4);
        }

        #[test]
        fn fails_when_element_is_contained() {
            for (range, rendered_range) in RANGE_CASES {
                assert_that!(range).does_not_contain_element(1);
                let failures = assert_that!(range)
                    .with_location(false)
                    .capture(|it| it.does_not_contain_element(2));
                assert_that!(failures).contains_exactly_satisfying([
                    |element: AssertThat<AssertionFailure, Capture>| {
                        element.derive(|value| value).has_text_report(formatdoc! {r"
                    -------- assertr --------
                    Expression: `range`

                    Actual: {rendered_range}

                    contains

                    Unexpected: 2
                    -------- assertr --------
                "});
                    },
                ]);
            }
        }
    }

    mod is_in_range {
        use super::*;

        #[test]
        #[cfg(feature = "fluent")]
        fn fluent_alias_is_as_expected() {
            'a'.must().be_in_range('a'..='z');
        }

        #[test]
        fn caller_location_is_as_expected() {
            assert_caller_location!(assert_that!('A'), is_in_range('a'..='z'));
        }

        #[test]
        fn succeeds_when_in_range() {
            assert_that!('a').is_in_range('a'..='z');
            assert_that!('p').is_in_range('a'..='z');
            assert_that!('z').is_in_range('a'..='z');
        }

        #[test]
        fn fails_when_not_in_range() {
            for (range, rendered_range) in RANGE_CASES {
                assert_that!(2).is_in_range(range);
                let failures = assert_that!(1)
                    .with_location(false)
                    .capture(|it| it.is_in_range(range));
                assert_that!(failures).contains_exactly_satisfying([
                    |element: AssertThat<AssertionFailure, Capture>| {
                        element.derive(|value| value).has_text_report(formatdoc! {r"
                    -------- assertr --------
                    Expression: `1`

                    Actual: 1

                    is not in range

                    Expected: {rendered_range}
                    -------- assertr --------
                "});
                    },
                ]);
            }
        }
    }

    mod is_not_in_range {
        use super::*;

        #[test]
        #[cfg(feature = "fluent")]
        fn fluent_alias_is_as_expected() {
            (-1).must().not_be_in_range(0..=7);
        }

        #[test]
        fn caller_location_is_as_expected() {
            assert_caller_location!(assert_that!(5), is_not_in_range(0..=7));
        }

        #[test]
        fn succeeds_when_not_in_range() {
            assert_that!(-1).is_not_in_range(0..=7);
            assert_that!(8).is_not_in_range(0..=7);
            assert_that!(9).is_not_in_range(0..=7);
        }

        #[test]
        fn fails_when_in_range() {
            for (range, rendered_range) in RANGE_CASES {
                assert_that!(1).is_not_in_range(range);
                let failures = assert_that!(2)
                    .with_location(false)
                    .capture(|it| it.is_not_in_range(range));
                assert_that!(failures).contains_exactly_satisfying([
                    |element: AssertThat<AssertionFailure, Capture>| {
                        element.derive(|value| value).has_text_report(formatdoc! {r"
                    -------- assertr --------
                    Expression: `2`

                    Actual: 2

                    is in range

                    Unexpected: {rendered_range}
                    -------- assertr --------
                "});
                    },
                ]);
            }
        }
    }

    /// Synonym of `is_not_in_range`. The fluent name and caller location are pinned here. The
    /// behavior is covered by that module.
    mod is_outside_of_range {
        use crate::prelude::*;

        #[test]
        #[cfg(feature = "fluent")]
        fn fluent_alias_is_as_expected() {
            5.must().be_outside_of_range(1..3);
        }

        #[test]
        fn caller_location_is_as_expected() {
            assert_caller_location!(assert_that!(5), is_outside_of_range(0..=10));
        }
    }

    mod reusable_definitions {
        use super::super::{ContainsElement, DoesNotContainElement};
        use super::*;
        use crate::{
            matchers::{all_of, each},
            test_support::BorrowSpy,
        };
        use core::cell::Cell;

        #[test]
        fn constructors_infer_without_a_subject_or_comparison_capabilities() {
            struct Opaque;
            let _ = ContainsElement::new(Opaque);
            let _ = DoesNotContainElement::new(Opaque);
            let _ = ContainsElement::new(&2);
            let _ = DoesNotContainElement::new(&2);
            let _ = ContainsElement::new(String::from("a"));
            let _ = DoesNotContainElement::new(String::from("a"));
        }

        #[test]
        fn borrowed_endpoints_infer_for_each_range_shape() {
            assert_that!(&1..&3)
                .matches(ContainsElement::new(&2))
                .matches(DoesNotContainElement::new(&3));
            assert_that!(&1..=&3)
                .matches(ContainsElement::new(&3))
                .matches(DoesNotContainElement::new(&4));
            assert_that!(&1..)
                .matches(ContainsElement::new(&2))
                .matches(DoesNotContainElement::new(&0));
            assert_that!(..&3)
                .matches(ContainsElement::new(&2))
                .matches(DoesNotContainElement::new(&3));
            assert_that!(..=&3)
                .matches(ContainsElement::new(&3))
                .matches(DoesNotContainElement::new(&4));
            assert_that!((Included(&1), Excluded(&3)))
                .matches(ContainsElement::new(&2))
                .matches(DoesNotContainElement::new(&3));
        }

        #[test]
        fn unbounded_ranges_infer_owned_and_borrowed_operands() {
            let value = String::from("a");
            assert_that!(..)
                .matches(ContainsElement::new(String::from("a")))
                .matches(ContainsElement::new(&value))
                .matches(ContainsElement::new(2))
                .matches(ContainsElement::new(&2));
            let failures = assert_that!(..).capture(|it| {
                it.matches(DoesNotContainElement::new(String::from("a")))
                    .matches(DoesNotContainElement::new(&value))
                    .matches(DoesNotContainElement::new(2))
                    .matches(DoesNotContainElement::new(&2))
            });
            assert_that!(failures).has_length(4);
        }

        #[test]
        fn definitions_are_reusable_in_nested_composition() {
            let expected = ContainsElement::new(&2);
            let unexpected = DoesNotContainElement::new(&4);
            let matcher = all_of((&expected, &unexpected));
            assert_that!(&1..&3).matches(&expected).matches(&unexpected);
            assert_that!(&1..&4).matches(&matcher);
            assert_that!([&1..&3, &0..&4]).matches(each(&matcher));
        }

        #[test]
        fn borrowing_definitions_are_reusable_across_owned_and_borrowed_bounds() {
            let lower = String::from("a");
            let upper = String::from("c");
            let value = String::from("b");
            let absent = String::from("z");
            let matcher = all_of((
                ContainsElement::<String>::borrowing(&value),
                DoesNotContainElement::<String>::borrowing(&absent),
            ));
            assert_that!(String::from("a")..String::from("c")).matches(&matcher);
            assert_that!(&lower..&upper).matches(&matcher);
            assert_that!([&lower..&upper]).matches(each(&matcher));
            assert_that!(value).is_equal_to("b");
            assert_that!(absent).is_equal_to("z");
        }

        #[test]
        fn borrowed_matchers_preserve_caller_locations() {
            assert_caller_location!(assert_that!(&1..&3), matches(ContainsElement::new(&4)));
            assert_caller_location!(
                assert_that!(&1..&3),
                matches(DoesNotContainElement::new(&2))
            );
        }

        #[test]
        fn matchers_preserve_membership_diagnostics() {
            let failures = assert_that!(&1..&3)
                .with_expression("range")
                .with_location(false)
                .capture(|it| {
                    it.matches(ContainsElement::new(&4))
                        .matches(DoesNotContainElement::new(&2))
                });
            assert_that!(failures).has_length(2);
            for (failure, relation, operand) in [
                (&failures[0], "does not contain", "Expected: 4"),
                (&failures[1], "contains", "Unexpected: 2"),
            ] {
                assert_that!(failure).has_text_report(formatdoc! {r"
                    -------- assertr --------
                    Expression: `range`

                    Actual: 1..3

                    {relation}

                    {operand}
                    -------- assertr --------
                "});
            }
        }

        #[test]
        fn borrowing_happens_once_after_tracking_and_rejections_retain_the_view() {
            for (negative, value) in [(false, 2), (false, 9), (true, 9), (true, 2)] {
                let calls = Cell::new(0);
                let failures = assert_that!(()).capture(|root| {
                    let expected = BorrowSpy {
                        value,
                        observe: || {
                            assert_that!(root.state.records.assertion_count()).is_equal_to(1);
                            calls.set(calls.get() + 1);
                        },
                    };
                    let it = root.derive_owned(|()| 1..3);
                    if negative {
                        let matcher = DoesNotContainElement::<i32>::borrowing(expected);
                        assert_that!(calls.get()).is_equal_to(0);
                        it.matches(&matcher);
                    } else {
                        let matcher = ContainsElement::<i32>::borrowing(expected);
                        assert_that!(calls.get()).is_equal_to(0);
                        it.matches(&matcher);
                    }
                    root
                });
                assert_that!(calls.get()).is_equal_to(1);
                assert_that!(failures).has_length(usize::from(negative == (value == 2)));
            }
        }
    }

    mod borrowed_elements {
        use super::super::{ContainsElement, DoesNotContainElement};
        use crate::{prelude::*, test_support::BorrowSpy};
        use core::cell::Cell;
        #[test]
        fn non_copy_bounds_accept_borrowed_elements_and_definitions() {
            let element = String::from("b");
            let absent = String::from("z");
            assert_that!(String::from("a")..String::from("c"))
                .contains_element(&element)
                .does_not_contain_element(&absent)
                .matches(ContainsElement::<String>::borrowing(&element))
                .matches(DoesNotContainElement::<String>::borrowing(&absent));
        }

        #[test]
        fn borrowed_non_copy_bounds_accept_owned_and_borrowed_elements() {
            let lower = String::from("a");
            let upper = String::from("c");
            let element = String::from("b");
            let absent = String::from("z");
            assert_that!(&lower..&upper)
                .contains_element(String::from("b"))
                .contains_element(&element)
                .does_not_contain_element(String::from("z"))
                .does_not_contain_element(&absent);
            assert_that!(element).is_equal_to("b");
            assert_that!(absent).is_equal_to("z");
        }

        #[test]
        fn borrowed_bounds_borrow_operands_once_after_tracking() {
            let lower = 1;
            let upper = 3;
            for (negative, value) in [(false, 2), (false, 9), (true, 9), (true, 2)] {
                let calls = Cell::new(0);
                let failures = assert_that!(()).capture(|root| {
                    let expected = BorrowSpy {
                        value,
                        observe: || {
                            assert_that!(root.state.records.assertion_count()).is_equal_to(1);
                            calls.set(calls.get() + 1);
                        },
                    };
                    let it = root.derive_owned(|()| &lower..&upper);
                    if negative {
                        it.does_not_contain_element(expected);
                    } else {
                        it.contains_element(expected);
                    }
                    root
                });
                assert_that!(calls.get()).is_equal_to(1);
                assert_that!(failures).has_length(usize::from(negative == (value == 2)));
            }
        }

        #[test]
        #[allow(clippy::needless_borrows_for_generic_args)] // Borrowed operands are the regression.
        fn unbounded_ranges_track_each_assertion_once() {
            let failures = assert_that!(..).capture(|it| {
                let it = it.contains_element(&2).does_not_contain_element(&2);
                assert_that!(it.state.records.assertion_count()).is_equal_to(2);
                it
            });
            assert_that!(failures).has_length(1);
        }

        #[test]
        fn borrows_once_after_tracking_and_reuses_rejections() {
            for negative in [false, true] {
                let calls = Cell::new(0);
                let failures = assert_that!(()).capture(|root| {
                    let expected = BorrowSpy {
                        value: if negative { 2 } else { 9 },
                        observe: || {
                            assert_that!(root.state.records.assertion_count()).is_equal_to(1);
                            calls.set(calls.get() + 1);
                        },
                    };
                    let it = root.derive_owned(|()| 1..3);
                    if negative {
                        it.does_not_contain_element(expected);
                    } else {
                        it.contains_element(expected);
                    }
                    root
                });
                assert_that!(calls.get()).is_equal_to(1);
                assert_that!(failures).has_length(1);
            }
        }
    }
}
