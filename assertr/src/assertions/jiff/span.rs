use jiff::Span;

use super::sign_expectations;
use crate::{
    AssertThat,
    mode::Mode,
    renderer::{DebugRenderer, ValueRenderer},
};

sign_expectations!(Span, zero: Span::new());

/// Assertions for [`Span`].
#[allow(clippy::return_self_not_must_use)]
#[cfg_attr(feature = "fluent", assertr_macros::fluent_aliases)]
pub trait SpanAssertions<R = DebugRenderer> {
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
        self.matches(IsZero)
    }

    #[track_caller]
    fn is_negative(self) -> Self
    where
        R: ValueRenderer<Span>,
    {
        self.matches(IsNegative)
    }

    #[track_caller]
    fn is_positive(self) -> Self
    where
        R: ValueRenderer<Span>,
    {
        self.matches(IsPositive)
    }
}

#[cfg(test)]
mod tests {
    #[cfg(feature = "fluent")]
    mod fluent_aliases {
        use jiff::{Span, ToSpan};

        use crate::prelude::*;

        #[test]
        fn are_as_expected() {
            Span::new().must().be_zero();
            (-2).hours().minutes(30).must().be_negative();
            2.hours().minutes(30).must().be_positive();
        }
    }

    mod renderer_contract {
        use jiff::Span;

        use crate::{
            prelude::*,
            test_support::{NoRenderer, SENTINEL, SentinelRenderer, assert_trait_impl},
        };

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

            assert_that!(failures[0].to_string()).contains(SENTINEL);
        }
    }

    mod is_zero {
        use indoc::formatdoc;
        use jiff::{Span, ToSpan};

        use crate::prelude::*;

        #[test]
        fn caller_location_is_as_expected() {
            let duration: Span = 2.hours().minutes(30);
            assert_caller_location!(assert_that!(duration), is_zero());
        }

        #[test]
        fn succeeds_when_zero() {
            assert_that!(Span::new()).is_zero();
        }

        #[test]
        fn panics_when_not_zero() {
            let duration: Span = 2.hours().minutes(30);

            assert_that!(|| assert_that!(duration).with_location(false).is_zero())
                .panics()
                .has_message()
                .is_equal_to(formatdoc! {r"
                    -------- assertr --------
                    Expression: `duration`

                    Expected: 0s

                      Actual: 2h 30m
                    -------- assertr --------
                "});
        }
    }

    mod is_negative {
        use jiff::ToSpan;

        use crate::prelude::*;

        #[test]
        fn caller_location_is_as_expected() {
            assert_caller_location!(assert_that!(0.seconds()), is_negative());
        }

        #[test]
        fn accepts_only_negative_spans() {
            assert_that!((-2).hours().minutes(30)).is_negative();
            let failures = assert_that!(0.seconds())
                .capture(SpanAssertions::is_negative)
                .into_iter()
                .chain(assert_that!(2.hours()).capture(SpanAssertions::is_negative));
            for failure in failures {
                assert_that!(failure.relation.as_deref()).is_equal_to(Some("is not negative"));
            }
        }
    }

    mod is_positive {
        use indoc::formatdoc;
        use jiff::ToSpan;

        use crate::prelude::*;

        #[test]
        fn caller_location_is_as_expected() {
            assert_caller_location!(assert_that!(0.seconds()), is_positive());
        }

        #[test]
        fn succeeds_when_positive() {
            assert_that!(2.hours().minutes(30)).is_positive();
            assert_that!(assert_that!(0.seconds()).capture(SpanAssertions::is_positive))
                .has_length(1);
        }

        #[test]
        fn panics_when_negative() {
            assert_that!(|| {
                assert_that!((-2).hours().minutes(30))
                    .with_location(false)
                    .is_positive();
            })
            .panics()
            .has_message()
            .is_equal_to(formatdoc! {r"
                -------- assertr --------
                Expression: `(-2).hours().minutes(30)`

                Actual: 2h 30m ago

                is not positive
                -------- assertr --------
            "});
        }
    }
}
