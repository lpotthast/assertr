use crate::{
    AssertThat, AssertionContext, Expectation, ExpectationDiagnostics, Mode, ValueRenderer,
    failure::{FailureBuilder, FailureKind},
    renderer::RenderingContext,
};
use alloc::{format, string::String};
use core::ops::{
    Bound::{Excluded, Included, Unbounded},
    RangeBounds,
};

/// Checks whether a range contains an element.
/// Uses [`RangeBounds::contains`], preserving inclusive, exclusive, and unbounded endpoints.
pub struct ContainsElement<B>(B);

impl<B> ContainsElement<B> {
    /// Owns the expected operand.
    #[must_use]
    pub const fn new(expected: B) -> Self {
        Self(expected)
    }
}

impl<B: PartialOrd, Range: RangeBounds<B> + ?Sized, R> Expectation<Range, R>
    for ContainsElement<B>
{
    type Success<'a>
        = ()
    where
        Self: 'a,
        Range: 'a;
    type Rejection<'a>
        = ()
    where
        Self: 'a,
        Range: 'a;
    fn evaluate<'a>(&'a self, actual: &'a Range, _: &AssertionContext<'_, R>) -> Result<(), ()> {
        if actual.contains(&self.0) {
            Ok(())
        } else {
            Err(())
        }
    }
}

impl<B: PartialOrd, Range: RangeBounds<B> + ?Sized, R: ValueRenderer<B>>
    ExpectationDiagnostics<Range, R> for ContainsElement<B>
{
    const KIND: FailureKind = FailureKind::Membership;
    fn explain<Target>(
        &self,
        rejected: Option<(&Range, ())>,
        failure: FailureBuilder<Target>,
        context: &AssertionContext<'_, R>,
    ) -> FailureBuilder<Target> {
        let render = context.render();
        let failure = match rejected {
            None => failure.relation("contains"),
            Some((actual, ())) => failure
                .actual(render_range(render, actual))
                .relation("does not contain"),
        };
        failure.expected(render.value(&self.0))
    }
}

/// Checks whether a range does not contain an element.
/// Uses [`RangeBounds::contains`], preserving inclusive, exclusive, and unbounded endpoints.
pub struct DoesNotContainElement<B>(B);

impl<B> DoesNotContainElement<B> {
    /// Owns the expected operand.
    #[must_use]
    pub const fn new(expected: B) -> Self {
        Self(expected)
    }
}

impl<B: PartialOrd, Range: RangeBounds<B> + ?Sized, R> Expectation<Range, R>
    for DoesNotContainElement<B>
{
    type Success<'a>
        = ()
    where
        Self: 'a,
        Range: 'a;
    type Rejection<'a>
        = ()
    where
        Self: 'a,
        Range: 'a;
    fn evaluate<'a>(&'a self, actual: &'a Range, _: &AssertionContext<'_, R>) -> Result<(), ()> {
        if actual.contains(&self.0) {
            Err(())
        } else {
            Ok(())
        }
    }
}

impl<B: PartialOrd, Range: RangeBounds<B> + ?Sized, R: ValueRenderer<B>>
    ExpectationDiagnostics<Range, R> for DoesNotContainElement<B>
{
    const KIND: FailureKind = FailureKind::Membership;
    fn explain<Target>(
        &self,
        rejected: Option<(&Range, ())>,
        failure: FailureBuilder<Target>,
        context: &AssertionContext<'_, R>,
    ) -> FailureBuilder<Target> {
        let render = context.render();
        let failure = match rejected {
            None => failure.relation("does not contain"),
            Some((actual, ())) => failure
                .actual(render_range(render, actual))
                .relation("contains"),
        };
        failure.unexpected(render.value(&self.0))
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
#[cfg_attr(feature = "fluent", assertr_macros::fluent_aliases)]
#[allow(clippy::return_self_not_must_use)]
pub trait RangeBoundAssertions<B, Range: RangeBounds<B>, R = crate::DebugRenderer> {
    /// Asserts that the range contains `expected`.
    fn contains_element(self, expected: B) -> Self
    where
        B: PartialOrd,
        R: ValueRenderer<B>;

    /// Asserts that the range does not contain `expected`.
    fn does_not_contain_element(self, expected: B) -> Self
    where
        B: PartialOrd,
        R: ValueRenderer<B>;
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
    fn contains_element(self, expected: B) -> Self
    where
        B: PartialOrd,
        R: ValueRenderer<B>,
    {
        self.apply_assertion(ContainsElement::new(expected))
    }

    #[track_caller]
    fn does_not_contain_element(self, expected: B) -> Self
    where
        B: PartialOrd,
        R: ValueRenderer<B>,
    {
        self.apply_assertion(DoesNotContainElement::new(expected))
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
                    .capture(|it| it.contains_element(1));
                assert_that!(failures).contains_exactly_satisfying([
                    |element: AssertThat<AssertionFailure, Capture>| {
                        element
                            .derive_owned(|value| rendered_text(value.actual.as_ref().unwrap()))
                            .is_equal_to(format!(
                                "(Excluded(<ren... 6 more characters ...), {rendered_end})"
                            ));
                    },
                ]);
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
        fn succeeds_when_element_is_contained() {
            assert_that!("aa"..="zz").contains_element("aa");
            assert_that!("aa"..="zz").contains_element("ab");
            assert_that!("aa"..="zz").contains_element("ac");
            assert_that!("aa"..="zz").contains_element("zx");
            assert_that!("aa"..="zz").contains_element("zy");
            assert_that!("aa"..="zz").contains_element("zz");
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
        fn succeeds_when_element_is_not_contained() {
            assert_that!("aa"..="zz").does_not_contain_element("a");
            assert_that!("aa"..="zz").does_not_contain_element("AA");
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
}
