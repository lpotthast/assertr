use crate::mode::Mode;
use crate::{AssertThat, ValueRenderer};
use indoc::writedoc;
use jiff::Span;
use std::fmt::Write;

/// Assertions for [`Span`].
#[allow(clippy::return_self_not_must_use)]
#[cfg_attr(feature = "fluent", assertr_derive::fluent_aliases)]
pub trait SpanAssertions<R = crate::DebugRenderer> {
    /// Asserts that the span is zero.
    fn is_zero(self) -> Self
    where
        R: ValueRenderer<Span>;

    /// Asserts that the span is strictly negative.
    fn is_negative(self) -> Self
    where
        R: ValueRenderer<Span>;

    /// Asserts that the span is strictly positive.
    fn is_positive(self) -> Self
    where
        R: ValueRenderer<Span>;
}

impl<M: Mode, R> SpanAssertions<R> for AssertThat<'_, Span, M, R> {
    #[track_caller]
    fn is_zero(self) -> Self
    where
        R: ValueRenderer<Span>,
    {
        self.track_assertion();

        if !self.actual().is_zero() {
            let details = [String::from("Actual was not zero.")];

            let expected = Span::new();
            let actual = self.render().value(self.actual());
            let expected = self.render().value(&expected);
            self.fail_with_details(details, |w: &mut String| {
                writedoc! {w, r"
                    Expected: {expected:#?}

                      Actual: {actual:#?}
                "}
            });
        }

        self
    }

    #[track_caller]
    fn is_negative(self) -> Self
    where
        R: ValueRenderer<Span>,
    {
        self.track_assertion();

        if !self.actual().is_negative() {
            let actual = self.render().value(self.actual());
            self.fail(|w: &mut String| {
                writedoc! {w, r"
                    Expected value to be negative. But was

                      Actual: {actual:#?}
                "}
            });
        }

        self
    }

    #[track_caller]
    fn is_positive(self) -> Self
    where
        R: ValueRenderer<Span>,
    {
        self.track_assertion();

        if !self.actual().is_positive() {
            let actual = self.render().value(self.actual());
            self.fail(|w: &mut String| {
                writedoc! {w, r"
                    Expected value to be positive. But was

                      Actual: {actual:#?}
                "}
            });
        }

        self
    }
}

#[cfg(test)]
mod tests {
    mod renderer_contract {
        use crate::prelude::*;
        use crate::test_support::{NoRenderer, SENTINEL, SentinelRenderer, assert_trait_impl};
        use jiff::Span;

        #[test]
        fn trait_is_implemented_without_renderer_support() {
            assert_trait_impl!(
                AssertThat<'static, Span, Panic, NoRenderer> => SpanAssertions<NoRenderer>
            );
        }

        #[test]
        fn failures_use_the_active_renderer() {
            let failures = assert_that!(Span::new().hours(1))
                .with_renderer(SentinelRenderer)
                .with_location(false)
                .capture(SpanAssertions::is_zero);

            assert_that!(failures[0].description.as_str()).contains(SENTINEL);
        }
    }

    mod is_zero {
        use crate::prelude::*;
        use indoc::formatdoc;
        use jiff::{Span, ToSpan};

        #[test]
        #[cfg(feature = "fluent")]
        fn fluent_alias_is_as_expected() {
            Span::new().must().be_zero();
        }

        #[test]
        fn succeeds_when_zero() {
            assert_that!(Span::new()).is_zero();
        }

        #[test]
        fn panics_when_not_zero() {
            let duration: Span = 2.hours().minutes(30);

            assert_that_panic_by(|| assert_that!(duration).with_location(false).is_zero())
                .has_type::<String>()
                .is_equal_to(formatdoc! {r"
                    -------- assertr --------
                    Expression: `duration`

                    Expected: 0s

                      Actual: 2h 30m

                    Details:
                      - Actual was not zero.
                    -------- assertr --------
                "});
        }
    }

    mod is_negative {
        use crate::prelude::*;
        use indoc::formatdoc;
        use jiff::ToSpan;

        #[test]
        #[cfg(feature = "fluent")]
        fn fluent_alias_is_as_expected() {
            (-2).hours().minutes(30).must().be_negative();
        }

        #[test]
        fn succeeds_when_zero() {
            assert_that!((-2).hours().minutes(30)).is_negative();
        }

        #[test]
        fn panics_when_zero() {
            assert_that_panic_by(|| {
                assert_that!(0.seconds()).with_location(false).is_negative();
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {r"
                    -------- assertr --------
                    Expression: `0.seconds()`

                    Expected value to be negative. But was

                      Actual: 0s
                    -------- assertr --------
                "});
        }

        #[test]
        fn panics_when_positive() {
            assert_that_panic_by(|| {
                assert_that!(2.hours().minutes(30))
                    .with_location(false)
                    .is_negative();
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {r"
                    -------- assertr --------
                    Expression: `2.hours().minutes(30)`

                    Expected value to be negative. But was

                      Actual: 2h 30m
                    -------- assertr --------
                "});
        }
    }

    mod is_positive {
        use crate::prelude::*;
        use indoc::formatdoc;
        use jiff::ToSpan;

        #[test]
        #[cfg(feature = "fluent")]
        fn fluent_alias_is_as_expected() {
            2.hours().minutes(30).must().be_positive();
        }

        #[test]
        fn succeeds_when_positive() {
            assert_that!(2.hours().minutes(30)).is_positive();
        }

        #[test]
        fn panics_when_zero() {
            assert_that_panic_by(|| {
                assert_that!(0.seconds()).with_location(false).is_positive();
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {r"
                    -------- assertr --------
                    Expression: `0.seconds()`

                    Expected value to be positive. But was

                      Actual: 0s
                    -------- assertr --------
                "});
        }

        #[test]
        fn panics_when_negative() {
            assert_that_panic_by(|| {
                assert_that!((-2).hours().minutes(30))
                    .with_location(false)
                    .is_positive();
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {r"
                    -------- assertr --------
                    Expression: `(-2).hours().minutes(30)`

                    Expected value to be positive. But was

                      Actual: 2h 30m ago
                    -------- assertr --------
                "});
        }
    }
}
