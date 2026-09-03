use crate::mode::Mode;
use crate::{AssertThat, ValueRenderer};
use indoc::writedoc;
use jiff::SignedDuration;
use std::fmt::Write;

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
            let details = [String::from("Actual was not zero.")];

            let expected = SignedDuration::ZERO;
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
        R: ValueRenderer<SignedDuration>,
    {
        self.track_assertion();

        if !self.actual().is_negative() {
            let actual = self.render().value(self.actual());
            self.fail(|w: &mut String| {
                writedoc! {w, r"
                    Expected: {actual:#?} to be negative,

                      Actual: {actual:#?},
                "}
            });
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
            let actual = self.render().value(self.actual());
            self.fail(|w: &mut String| {
                writedoc! {w, r"
                    Expected: {actual:#?} to be positive,

                      Actual: {actual:#?},
                "}
            });
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
            let actual = self.render().value(&actual);
            let expected = self.render().value(&expected);
            let allowed_deviation = self.render().value(&allowed_deviation);
            let min = self.render().value(&min);
            let max = self.render().value(&max);
            self.fail(|w: &mut String| {
                writedoc! {w, r"
                    Expected value to be close to: {expected:?},
                     with allowed deviation being: {allowed_deviation:?},
                      but value was outside range: [{min:?}, {max:?}]

                      Actual: {actual:?}
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

            assert_that!(failures[0].description.as_str()).contains(SENTINEL);
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

                      Actual: 9000s

                    Details:
                      - Actual was not zero.
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

                    Expected: 0s to be negative,

                      Actual: 0s,
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

                    Expected: 5s to be negative,

                      Actual: 5s,
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

                    Expected: 0s to be positive,

                      Actual: 0s,
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

                    Expected: -5s to be positive,

                      Actual: -5s,
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

                    Expected value to be close to: 333ms,
                     with allowed deviation being: 1ms,
                      but value was outside range: [332ms, 334ms]

                      Actual: 331ms 900µs
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

                    Expected value to be close to: 333ms,
                     with allowed deviation being: 1ms,
                      but value was outside range: [332ms, 334ms]

                      Actual: 334ms 100µs
                    -------- assertr --------
                "});
        }
    }
}
