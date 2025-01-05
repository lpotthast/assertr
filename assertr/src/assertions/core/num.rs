use core::fmt::Debug;
use core::fmt::Write;
use indoc::writedoc;
use num::{Float, Num};

use crate::mode::Mode;
use crate::tracking::AssertionTracking;
use crate::AssertThat;

/// Assertions for numeric values not already handled by
/// [crate::prelude::PartialEqAssertions] and [crate::prelude::PartialOrdAssertions].
pub trait NumAssertions<T: Num> {
    /// Fails if actual is not equal to the additive identity.
    fn is_zero(self) -> Self;

    /// Fails if actual is not equal to the multiplicative identity.
    fn is_one(self) -> Self;

    /// Fails if actual is not in range `[expected - allowed_deviation, expected + allowed_deviation]`.
    fn is_close_to(self, expected: T, allowed_deviation: T) -> Self
    where
        T: PartialOrd,
        T: Clone;

    #[cfg(any(feature = "std", feature = "libm"))]
    fn is_nan(self) -> Self
    where
        T: Float;

    #[cfg(any(feature = "std", feature = "libm"))]
    fn is_infinite(self) -> Self
    where
        T: Float;

    // TODO: is_normal
    // TODO: is_subnormal
}

impl<T: Num + Debug, M: Mode> NumAssertions<T> for AssertThat<'_, T, M> {
    #[track_caller]
    fn is_zero(self) -> Self {
        self.track_assertion();
        let actual = self.actual();
        if !actual.is_zero() {
            let expected = T::zero();
            self.fail(|w: &mut String| {
                writedoc! {w, r#"
                    Expected: {expected:#?}]

                      Actual: {actual:#?}
                "#}
            });
        }
        self
    }

    #[track_caller]
    fn is_one(self) -> Self {
        self.track_assertion();
        let actual = self.actual();
        if !actual.is_one() {
            let expected = T::one();
            self.fail(|w: &mut String| {
                writedoc! {w, r#"
                    Expected: {expected:#?}]

                      Actual: {actual:#?}
                "#}
            });
        }
        self
    }

    #[track_caller]
    fn is_close_to(self, expected: T, allowed_deviation: T) -> Self
    where
        T: PartialOrd,
        T: Clone,
    {
        self.track_assertion();
        let actual = self.actual();
        let min = expected.clone() - allowed_deviation.clone();
        let max = expected.clone() + allowed_deviation.clone();
        if !(actual >= &min && actual <= &max) {
            self.fail(|w: &mut String| {
                writedoc! {w, r#"
                    Expected value in range [{min:?}, {max:?}]

                      Actual: {actual:#?}
                "#}
            });
        }
        self
    }

    #[track_caller]
    #[cfg(any(feature = "std", feature = "libm"))]
    fn is_nan(self) -> Self
    where
        T: Float,
    {
        self.track_assertion();
        let actual = self.actual();
        if !actual.is_nan() {
            let nan = T::nan();
            self.fail(|w: &mut String| {
                writedoc! {w, r#"
                    Expected: {nan:#?}

                      Actual: {actual:#?}
                "#}
            });
        }
        self
    }

    #[track_caller]
    #[cfg(any(feature = "std", feature = "libm"))]
    fn is_infinite(self) -> Self
    where
        T: Float,
    {
        self.track_assertion();
        let actual = self.actual();
        if !actual.is_infinite() {
            let inf = T::infinity();
            self.fail(|w: &mut String| {
                writedoc! {w, r#"
                    Expected: +/- {inf:#?}
                    
                      Actual: {actual:#?}
                "#}
            });
        }
        self
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn quick_type_check() {
        use crate::prelude::*;
        use ::num::Float;

        assert_that(0u8).is_zero();
        assert_that(0i8).is_zero();
        assert_that(0u16).is_zero();
        assert_that(0i16).is_zero();
        assert_that(0u32).is_zero();
        assert_that(0i32).is_zero();
        assert_that(0u64).is_zero();
        assert_that(0i64).is_zero();
        assert_that(0u128).is_zero();
        assert_that(0i128).is_zero();
        assert_that(0.0f32).is_zero();
        assert_that(0.0f64).is_zero();

        assert_that(1u8).is_one();
        assert_that(1i8).is_one();
        assert_that(1u16).is_one();
        assert_that(1i16).is_one();
        assert_that(1u32).is_one();
        assert_that(1i32).is_one();
        assert_that(1u64).is_one();
        assert_that(1i64).is_one();
        assert_that(1u128).is_one();
        assert_that(1i128).is_one();
        assert_that(1.0f32).is_one();
        assert_that(1.0f64).is_one();

        assert_that(42u8).is_close_to(42, 0);
        assert_that(42i8).is_close_to(42, 0);
        assert_that(42u16).is_close_to(42, 0);
        assert_that(42i16).is_close_to(42, 0);
        assert_that(42u32).is_close_to(42, 0);
        assert_that(42i32).is_close_to(42, 0);
        assert_that(42u64).is_close_to(42, 0);
        assert_that(42i64).is_close_to(42, 0);
        assert_that(42u128).is_close_to(42, 0);
        assert_that(42i128).is_close_to(42, 0);
        assert_that(42.0f32).is_close_to(42.0, 0.0001);
        assert_that(42.0f64).is_close_to(42.0, 0.0001);

        let nan: f32 = Float::nan();
        assert_that(nan).is_nan();
        let nan: f64 = Float::nan();
        assert_that(nan).is_nan();

        let inf: f32 = Float::infinity();
        assert_that(inf).is_infinite();
        let inf: f64 = Float::infinity();
        assert_that(inf).is_infinite();
    }

    mod is_zero {
        use crate::prelude::*;

        #[test]
        fn succeeds_when_actual_is_zero() {
            assert_that(0).is_zero();
        }
    }

    mod is_one {
        use crate::prelude::*;

        #[test]
        fn succeeds_when_actual_is_one() {
            assert_that(1).is_one();
            assert_that(42.0f64).is_close_to(41.999f64, 0.01f64);
        }
    }

    mod is_close_to {
        use crate::prelude::*;

        #[test]
        fn succeeds_when_actual_is_in_allowed_range() {
            assert_that(0.3319)
                .with_capture()
                .is_close_to(0.333, 0.001f64)
                .capture_failures()
                .assert_that_it()
                .has_length(1);
            assert_that(0.332).is_close_to(0.333, 0.001f64);
            assert_that(0.333).is_close_to(0.333, 0.001f64);
            assert_that(0.334).is_close_to(0.333, 0.001f64);
            assert_that(0.3341)
                .with_capture()
                .is_close_to(0.333, 0.001f64)
                .capture_failures()
                .assert_that_it()
                .has_length(1);
        }
    }

    mod is_nan {
        use crate::prelude::*;
        use ::num::Float;
        use indoc::formatdoc;

        #[test]
        fn succeeds_when_nan() {
            let nan: f32 = Float::nan();
            assert_that(nan).is_nan();
        }
        #[test]
        fn panics_when_not_nan() {
            assert_that_panic_by(|| assert_that(1.23).with_location(false).is_nan())
                .has_type::<String>()
                .is_equal_to(formatdoc! {r#"
                    -------- assertr --------
                    Expected: NaN

                      Actual: 1.23
                    -------- assertr --------
                "#});
        }
    }

    mod is_infinite {
        use crate::prelude::*;
        use ::num::Float;
        use indoc::formatdoc;

        #[test]
        fn succeeds_when_positive_infinity() {
            let inf: f32 = Float::infinity();
            assert_that(inf).is_infinite();
        }

        #[test]
        fn succeeds_when_negative_infinity() {
            let inf: f32 = Float::neg_infinity();
            assert_that(inf).is_infinite();
        }

        #[test]
        fn panics_when_not_infinity() {
            assert_that_panic_by(|| assert_that(1.23).with_location(false).is_infinite())
                .has_type::<String>()
                .is_equal_to(formatdoc! {r#"
                    -------- assertr --------
                    Expected: +/- inf

                      Actual: 1.23
                    -------- assertr --------
                "#});
        }
    }
}
