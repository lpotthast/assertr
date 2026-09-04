use crate::failure::FailureKind;
use crate::mode::Mode;
use crate::{AssertThat, ValueRenderer, renderer::Compact};
use jiff::SignedDuration;

/// Assertions for [`SignedDuration`].
#[allow(clippy::return_self_not_must_use)]
#[cfg_attr(feature = "fluent", assertr_derive::fluent_aliases)]
pub trait SignedDurationAssertions<R = crate::DebugRenderer> {
    /// Asserts that the duration is zero.
    fn is_zero(self) -> Self
    where
        R: ValueRenderer<SignedDuration>;

    /// Asserts that the duration is strictly negative.
    fn is_negative(self) -> Self
    where
        R: ValueRenderer<SignedDuration>;

    /// Asserts that the duration is strictly positive.
    fn is_positive(self) -> Self
    where
        R: ValueRenderer<SignedDuration>;

    /// Asserts that the duration is within `allowed_deviation` of `expected`, including the
    /// endpoints.
    ///
    /// A negative `allowed_deviation` produces an empty range, so the assertion always fails.
    ///
    /// # Panics
    ///
    /// Panics if adding or subtracting `allowed_deviation` from `expected` overflows
    /// [`SignedDuration`].
    fn is_close_to(self, expected: SignedDuration, allowed_deviation: SignedDuration) -> Self
    where
        R: ValueRenderer<SignedDuration>;
}

impl<M: Mode, R> SignedDurationAssertions<R> for AssertThat<'_, SignedDuration, M, R> {
    #[track_caller]
    fn is_zero(self) -> Self
    where
        R: ValueRenderer<SignedDuration>,
    {
        self.track_assertion();

        if !self.actual().is_zero() {
            self.failure(FailureKind::Equality)
                .actual(Compact(self.render().value(self.actual())))
                .expected(Compact(self.render().value(&SignedDuration::ZERO)))
                .raise();
        }

        self
    }

    #[track_caller]
    fn is_negative(self) -> Self
    where
        R: ValueRenderer<SignedDuration>,
    {
        self.track_assertion();

        if !self.actual().is_negative() {
            self.failure(FailureKind::Ordering)
                .actual(Compact(self.render().value(self.actual())))
                .relation("is not negative")
                .raise();
        }

        self
    }

    #[track_caller]
    fn is_positive(self) -> Self
    where
        R: ValueRenderer<SignedDuration>,
    {
        self.track_assertion();

        if !self.actual().is_positive() {
            self.failure(FailureKind::Ordering)
                .actual(Compact(self.render().value(self.actual())))
                .relation("is not positive")
                .raise();
        }

        self
    }

    #[track_caller]
    fn is_close_to(self, expected: SignedDuration, allowed_deviation: SignedDuration) -> Self
    where
        R: ValueRenderer<SignedDuration>,
    {
        self.track_assertion();

        let actual = *self.actual();
        let min = expected - allowed_deviation;
        let max = expected + allowed_deviation;
        if !(actual >= min && actual <= max) {
            self.failure(FailureKind::Ordering)
                .actual(Compact(self.render().value(&actual)))
                .relation("is not close to")
                .expected(Compact(self.render().value(&expected)))
                .fact(
                    "Allowed deviation",
                    format_args!("{:?}", self.render().value(&allowed_deviation)),
                )
                .fact(
                    "Allowed range",
                    format_args!(
                        "[{:?}, {:?}]",
                        self.render().value(&min),
                        self.render().value(&max)
                    ),
                )
                .raise();
        }

        self
    }
}

#[cfg(test)]
mod tests {
    mod renderer_contract {
        use crate::prelude::*;
        use crate::test_support::{NoRenderer, SENTINEL, SentinelRenderer, assert_trait_impl};
        use jiff::SignedDuration;

        #[test]
        fn trait_is_implemented_without_renderer_support() {
            assert_trait_impl!(
                AssertThat<'static, SignedDuration, Panic, NoRenderer>
                    => SignedDurationAssertions<NoRenderer>
            );
        }

        #[test]
        fn failures_use_the_active_renderer() {
            let failures = assert_that!(SignedDuration::from_secs(1))
                .with_renderer(SentinelRenderer)
                .with_location(false)
                .capture(SignedDurationAssertions::is_zero);

            assert_that!(failures[0].description()).contains(SENTINEL);
        }
    }

    mod is_zero {
        use crate::prelude::*;
        use indoc::formatdoc;
        use jiff::SignedDuration;

        #[test]
        #[cfg(feature = "fluent")]
        fn fluent_alias_is_as_expected() {
            SignedDuration::ZERO.must().be_zero();
        }

        #[test]
        fn succeeds_when_zero() {
            assert_that!(SignedDuration::ZERO).is_zero();
        }

        #[test]
        fn panics_when_not_zero() {
            let duration: SignedDuration = "2h 30m".parse().unwrap();

            assert_that_panic_by(|| {
                assert_that!(duration).with_location(false).is_zero();
            })
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
        use jiff::SignedDuration;

        #[test]
        #[cfg(feature = "fluent")]
        fn fluent_alias_is_as_expected() {
            SignedDuration::from_secs(-5).must().be_negative();
        }

        #[test]
        fn succeeds_when_negative() {
            assert_that!(SignedDuration::from_secs(-5)).is_negative();
        }

        #[test]
        fn panics_when_zero() {
            assert_that_panic_by(|| {
                assert_that!(SignedDuration::ZERO)
                    .with_location(false)
                    .is_negative();
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {r"
                    -------- assertr --------
                    Expression: `SignedDuration::ZERO`

                    Actual: 0s

                    is not negative
                    -------- assertr --------
                "});
        }

        #[test]
        fn panics_when_positive() {
            assert_that_panic_by(|| {
                assert_that!(SignedDuration::from_secs(5))
                    .with_location(false)
                    .is_negative();
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {r"
                    -------- assertr --------
                    Expression: `SignedDuration::from_secs(5)`

                    Actual: 5s

                    is not negative
                    -------- assertr --------
                "});
        }
    }

    mod is_positive {
        use crate::prelude::*;
        use indoc::formatdoc;
        use jiff::SignedDuration;

        #[test]
        #[cfg(feature = "fluent")]
        fn fluent_alias_is_as_expected() {
            SignedDuration::from_secs(5).must().be_positive();
        }

        #[test]
        fn succeeds_when_positive() {
            assert_that!(SignedDuration::from_secs(5)).is_positive();
        }

        #[test]
        fn panics_when_zero() {
            assert_that_panic_by(|| {
                assert_that!(SignedDuration::ZERO)
                    .with_location(false)
                    .is_positive();
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {r"
                    -------- assertr --------
                    Expression: `SignedDuration::ZERO`

                    Actual: 0s

                    is not positive
                    -------- assertr --------
                "});
        }

        #[test]
        fn panics_when_negative() {
            assert_that_panic_by(|| {
                assert_that!(SignedDuration::from_secs(-5))
                    .with_location(false)
                    .is_positive();
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {r"
                    -------- assertr --------
                    Expression: `SignedDuration::from_secs(-5)`

                    Actual: 5s ago

                    is not positive
                    -------- assertr --------
                "});
        }
    }

    mod is_close_to {
        use crate::prelude::*;
        use indoc::formatdoc;
        use jiff::SignedDuration;

        #[test]
        #[cfg(feature = "fluent")]
        fn fluent_alias_is_as_expected() {
            SignedDuration::from_secs_f32(0.333).must().be_close_to(
                SignedDuration::from_secs_f32(0.333),
                SignedDuration::from_secs_f32(0.001),
            );
        }

        #[test]
        fn panics_when_below_allowed_range() {
            assert_that_panic_by(|| {
                assert_that!(SignedDuration::from_secs_f32(0.3319))
                    .with_location(false)
                    .is_close_to(
                        SignedDuration::from_secs_f32(0.333),
                        SignedDuration::from_secs_f32(0.001),
                    );
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {r"
                    -------- assertr --------
                    Expression: `SignedDuration::from_secs_f32(0.3319)`

                    Actual: 331ms 900µs

                    is not close to

                    Expected: 333ms

                    Details:
                      - Allowed deviation: 1ms
                      - Allowed range: [332ms, 334ms]
                    -------- assertr --------
                "});
        }

        #[test]
        fn succeeds_when_actual_is_in_allowed_range() {
            assert_that!(SignedDuration::from_secs_f32(0.332)).is_close_to(
                SignedDuration::from_secs_f32(0.333),
                SignedDuration::from_secs_f32(0.001),
            );
            assert_that!(SignedDuration::from_secs_f32(0.333)).is_close_to(
                SignedDuration::from_secs_f32(0.333),
                SignedDuration::from_secs_f32(0.001),
            );
            assert_that!(SignedDuration::from_secs_f32(0.334)).is_close_to(
                SignedDuration::from_secs_f32(0.333),
                SignedDuration::from_secs_f32(0.001),
            );
        }

        #[test]
        fn panics_when_above_allowed_range() {
            assert_that_panic_by(|| {
                assert_that!(SignedDuration::from_secs_f32(0.3341))
                    .with_location(false)
                    .is_close_to(
                        SignedDuration::from_secs_f32(0.333),
                        SignedDuration::from_secs_f32(0.001),
                    );
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {r"
                    -------- assertr --------
                    Expression: `SignedDuration::from_secs_f32(0.3341)`

                    Actual: 334ms 100µs

                    is not close to

                    Expected: 333ms

                    Details:
                      - Allowed deviation: 1ms
                      - Allowed range: [332ms, 334ms]
                    -------- assertr --------
                "});
        }
    }
}
