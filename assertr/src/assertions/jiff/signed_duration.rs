use crate::AssertThat;
use crate::mode::Mode;
use crate::tracking::AssertionTracking;
use indoc::writedoc;
use jiff::SignedDuration;
use std::fmt::Write;

pub trait SignedDurationAssertions {
    fn is_zero(self) -> Self;
    fn be_zero(self) -> Self
    where
        Self: Sized,
    {
        self.is_zero()
    }

    fn is_negative(self) -> Self;
    fn be_negative(self) -> Self
    where
        Self: Sized,
    {
        self.is_negative()
    }

    fn is_positive(self) -> Self;
    fn be_positive(self) -> Self
    where
        Self: Sized,
    {
        self.is_positive()
    }

    fn is_close_to(self, expected: SignedDuration, allowed_deviation: SignedDuration) -> Self;
    fn be_close_to(self, expected: SignedDuration, allowed_deviation: SignedDuration) -> Self
    where
        Self: Sized,
    {
        self.is_close_to(expected, allowed_deviation)
    }
}

impl<M: Mode> SignedDurationAssertions for AssertThat<'_, SignedDuration, M> {
    #[track_caller]
    fn is_zero(self) -> Self {
        self.track_assertion();

        if !self.actual().is_zero() {
            self.add_detail_message("Actual was not zero.");

            let expected = SignedDuration::ZERO;
            self.fail(|w: &mut String| {
                writedoc! {w, r#"
                    Expected: {expected:#?}

                      Actual: {actual:#?}
                "#, actual = self.actual()}
            });
        }

        self
    }

    #[track_caller]
    fn is_negative(self) -> Self {
        self.track_assertion();

        if !self.actual().is_negative() {
            self.fail(|w: &mut String| {
                writedoc! {w, r#"
                    Expected: {actual:#?} to be negative,

                      Actual: {actual:#?},
                "#, actual = self.actual()}
            });
        }

        self
    }

    #[track_caller]
    fn is_positive(self) -> Self {
        self.track_assertion();

        if !self.actual().is_positive() {
            self.fail(|w: &mut String| {
                writedoc! {w, r#"
                    Expected: {actual:#?} to be negative,

                      Actual: {actual:#?},
                "#, actual = self.actual()}
            });
        }

        self
    }

    #[track_caller]
    fn is_close_to(self, expected: SignedDuration, allowed_deviation: SignedDuration) -> Self {
        self.track_assertion();

        let actual = *self.actual();
        let min = expected - allowed_deviation;
        let max = expected + allowed_deviation;
        if !(actual >= min && actual <= max) {
            self.fail(|w: &mut String| {
                writedoc! {w, r#"
                    Expected value to be close to: {expected:?},
                     with allowed deviation being: {allowed_deviation:?},
                      but value was outside range: [{min:?}, {max:?}]

                      Actual: {actual:?}
                "#, actual = self.actual()}
            });
        }

        self
    }
}

#[cfg(test)]
mod tests {
    mod is_zero {
        use crate::prelude::*;
        use indoc::formatdoc;
        use jiff::SignedDuration;

        #[test]
        fn succeeds_when_zero() {
            SignedDuration::ZERO.must().be_zero();
        }

        #[test]
        fn panics_when_not_zero() {
            let duration: SignedDuration = "2h 30m".parse().unwrap();

            assert_that_panic_by(|| {
                duration.assert().with_location(false).is_zero();
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {r#"
                    -------- assertr --------
                    Expected: 0s

                      Actual: 9000s

                    Details: [
                        Actual was not zero.,
                    ]
                    -------- assertr --------
                "#});
        }
    }

    mod is_close_to {
        use crate::prelude::*;
        use indoc::formatdoc;
        use jiff::SignedDuration;

        #[test]
        fn panics_when_below_allowed_range() {
            assert_that_panic_by(|| {
                SignedDuration::from_secs_f32(0.3319)
                    .assert()
                    .with_location(false)
                    .is_close_to(
                        SignedDuration::from_secs_f32(0.333),
                        SignedDuration::from_secs_f32(0.001),
                    );
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {r#"
                    -------- assertr --------
                    Expected value to be close to: 333ms,
                     with allowed deviation being: 1ms,
                      but value was outside range: [332ms, 334ms]

                      Actual: 331ms 900µs
                    -------- assertr --------
                "#});
        }

        #[test]
        fn succeeds_when_actual_is_in_allowed_range() {
            SignedDuration::from_secs_f32(0.332).must().be_close_to(
                SignedDuration::from_secs_f32(0.333),
                SignedDuration::from_secs_f32(0.001),
            );
            SignedDuration::from_secs_f32(0.333).must().be_close_to(
                SignedDuration::from_secs_f32(0.333),
                SignedDuration::from_secs_f32(0.001),
            );
            SignedDuration::from_secs_f32(0.334).must().be_close_to(
                SignedDuration::from_secs_f32(0.333),
                SignedDuration::from_secs_f32(0.001),
            );
        }

        #[test]
        fn panics_when_above_allowed_range() {
            assert_that_panic_by(|| {
                SignedDuration::from_secs_f32(0.3341)
                    .assert()
                    .with_location(false)
                    .is_close_to(
                        SignedDuration::from_secs_f32(0.333),
                        SignedDuration::from_secs_f32(0.001),
                    );
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {r#"
                    -------- assertr --------
                    Expected value to be close to: 333ms,
                     with allowed deviation being: 1ms,
                      but value was outside range: [332ms, 334ms]

                      Actual: 334ms 100µs
                    -------- assertr --------
                "#});
        }
    }
}
