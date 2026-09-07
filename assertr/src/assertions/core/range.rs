use alloc::{format, string::String};
use core::ops::Bound::{Excluded, Included, Unbounded};
use core::ops::RangeBounds;

use crate::{AssertThat, Mode, ValueRenderer, failure::FailureKind};

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
        self.track_assertion();
        if !self.actual().contains(&expected) {
            let range = render_range(&self, self.actual());
            self.failure(FailureKind::Membership)
                .actual(format_args!("{range}"))
                .relation("does not contain")
                .expected(self.render().value(&expected))
                .raise();
        }
        self
    }

    #[track_caller]
    fn does_not_contain_element(self, expected: B) -> Self
    where
        B: PartialOrd,
        R: ValueRenderer<B>,
    {
        self.track_assertion();
        if self.actual().contains(&expected) {
            let range = render_range(&self, self.actual());
            self.failure(FailureKind::Membership)
                .actual(format_args!("{range}"))
                .relation("contains")
                .unexpected(self.render().value(&expected))
                .raise();
        }
        self
    }
}

impl<B, M: Mode, R> RangeAssertions<B, R> for AssertThat<'_, B, M, R> {
    #[track_caller]
    fn is_in_range(self, expected: impl RangeBounds<B>) -> Self
    where
        B: PartialOrd,
        R: ValueRenderer<B>,
    {
        self.track_assertion();

        let actual = self.actual();

        if !expected.contains(actual) {
            let range = render_range(&self, &expected);
            self.failure(FailureKind::Ordering)
                .actual(self.render().value(actual))
                .relation("is not in range")
                .expected(format_args!("{range}"))
                .raise();
        }

        self
    }

    #[track_caller]
    fn is_not_in_range(self, expected: impl RangeBounds<B>) -> Self
    where
        B: PartialOrd,
        R: ValueRenderer<B>,
    {
        self.track_assertion();

        let actual = self.actual();

        if expected.contains(actual) {
            let range = render_range(&self, &expected);
            self.failure(FailureKind::Ordering)
                .actual(self.render().value(actual))
                .relation("is in range")
                .unexpected(format_args!("{range}"))
                .raise();
        }

        self
    }
}

fn render_range<B, S, Range: RangeBounds<B> + ?Sized, M: Mode, R>(
    assert_that: &AssertThat<'_, S, M, R>,
    range: &Range,
) -> String
where
    R: ValueRenderer<B>,
{
    let rendering = assert_that.render();
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

        use crate::prelude::*;
        use crate::test_support::{
            NoRenderer, SENTINEL, SentinelRenderer, assert_trait_impl, rendered_text,
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
                    .with_rendering_budget(
                        RenderingBudget::builder().max_leaf_characters(4).build(),
                    )
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
