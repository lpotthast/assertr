use crate::failure::FailureKind;
use crate::mode::Mode;
use crate::{AssertThat, ValueRenderer};
use crate::{AssertionContext, Expectation, ExpectationDiagnostics, failure::FailureBuilder};
use jiff::Span;

/// Checks whether a `Span` is zero.
pub struct IsZero;
impl<R> Expectation<Span, R> for IsZero {
    type Success<'a>
        = ()
    where
        Self: 'a,
        Span: 'a;
    type Rejection<'a>
        = ()
    where
        Self: 'a,
        Span: 'a;
    fn evaluate<'a>(
        &'a self,
        actual: &'a Span,
        _context: &AssertionContext<'_, R>,
    ) -> Result<Self::Success<'a>, Self::Rejection<'a>> {
        if actual.is_zero() { Ok(()) } else { Err(()) }
    }
}
impl<R> ExpectationDiagnostics<Span, R> for IsZero
where
    R: ValueRenderer<Span>,
{
    const KIND: FailureKind = FailureKind::Equality;
    fn explain<'a, Target>(
        &'a self,
        rejected: Option<(&'a Span, Self::Rejection<'a>)>,
        failure: FailureBuilder<Target>,
        context: &AssertionContext<'_, R>,
    ) -> FailureBuilder<Target> {
        let render = context.render();
        match rejected {
            None => failure
                .relation("is zero")
                .expected(render.value(&Span::new())),
            Some((actual, ())) => failure
                .actual(render.value(actual))
                .expected(render.value(&Span::new())),
        }
    }
}
/// Checks whether a `Span` is negative.
pub struct IsNegative;
impl<R> Expectation<Span, R> for IsNegative {
    type Success<'a>
        = ()
    where
        Self: 'a,
        Span: 'a;
    type Rejection<'a>
        = ()
    where
        Self: 'a,
        Span: 'a;
    fn evaluate<'a>(
        &'a self,
        actual: &'a Span,
        _context: &AssertionContext<'_, R>,
    ) -> Result<Self::Success<'a>, Self::Rejection<'a>> {
        if actual.is_negative() {
            Ok(())
        } else {
            Err(())
        }
    }
}
impl<R> ExpectationDiagnostics<Span, R> for IsNegative
where
    R: ValueRenderer<Span>,
{
    const KIND: FailureKind = FailureKind::Ordering;
    fn explain<'a, Target>(
        &'a self,
        rejected: Option<(&'a Span, Self::Rejection<'a>)>,
        failure: FailureBuilder<Target>,
        context: &AssertionContext<'_, R>,
    ) -> FailureBuilder<Target> {
        let render = context.render();
        match rejected {
            None => failure.relation("is negative"),
            Some((actual, ())) => failure
                .actual(render.value(actual))
                .relation("is not negative"),
        }
    }
}
/// Checks whether a `Span` is positive.
pub struct IsPositive;
impl<R> Expectation<Span, R> for IsPositive {
    type Success<'a>
        = ()
    where
        Self: 'a,
        Span: 'a;
    type Rejection<'a>
        = ()
    where
        Self: 'a,
        Span: 'a;
    fn evaluate<'a>(
        &'a self,
        actual: &'a Span,
        _context: &AssertionContext<'_, R>,
    ) -> Result<Self::Success<'a>, Self::Rejection<'a>> {
        if actual.is_positive() {
            Ok(())
        } else {
            Err(())
        }
    }
}
impl<R> ExpectationDiagnostics<Span, R> for IsPositive
where
    R: ValueRenderer<Span>,
{
    const KIND: FailureKind = FailureKind::Ordering;
    fn explain<'a, Target>(
        &'a self,
        rejected: Option<(&'a Span, Self::Rejection<'a>)>,
        failure: FailureBuilder<Target>,
        context: &AssertionContext<'_, R>,
    ) -> FailureBuilder<Target> {
        let render = context.render();
        match rejected {
            None => failure.relation("is positive"),
            Some((actual, ())) => failure
                .actual(render.value(actual))
                .relation("is not positive"),
        }
    }
}

/// Assertions for [`Span`].
#[allow(clippy::return_self_not_must_use)]
#[cfg_attr(feature = "fluent", assertr_macros::fluent_aliases)]
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
        self.apply_assertion(IsZero)
    }

    #[track_caller]
    fn is_negative(self) -> Self
    where
        R: ValueRenderer<Span>,
    {
        self.apply_assertion(IsNegative)
    }

    #[track_caller]
    fn is_positive(self) -> Self
    where
        R: ValueRenderer<Span>,
    {
        self.apply_assertion(IsPositive)
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

            assert_that!(ToHumanReadableText.render(&failures[0])).contains(SENTINEL);
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

            assert_that_panic_by(|| assert_that!(duration).with_location(false).is_zero())
                .has_type::<String>()
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
        use crate::prelude::*;
        use indoc::formatdoc;
        use jiff::ToSpan;

        #[test]
        #[cfg(feature = "fluent")]
        fn fluent_alias_is_as_expected() {
            (-2).hours().minutes(30).must().be_negative();
        }

        #[test]
        fn caller_location_is_as_expected() {
            assert_caller_location!(assert_that!(0.seconds()), is_negative());
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

                    Actual: 0s

                    is not negative
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

                    Actual: 2h 30m

                    is not negative
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
        fn caller_location_is_as_expected() {
            assert_caller_location!(assert_that!(0.seconds()), is_positive());
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

                    Actual: 0s

                    is not positive
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

                    Actual: 2h 30m ago

                    is not positive
                    -------- assertr --------
                "});
        }
    }
}
