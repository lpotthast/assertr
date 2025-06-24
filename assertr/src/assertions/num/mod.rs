use crate::AssertThat;
use crate::mode::Mode;
use crate::tracking::AssertionTracking;
use core::fmt::Debug;
use core::fmt::Write;
use indoc::writedoc;
use num::{Float, Num, Signed};

/// Assertions for numeric values not already handled by
/// [crate::prelude::PartialEqAssertions] and [crate::prelude::PartialOrdAssertions].
pub trait NumAssertions<T: Num> {
    /// Fails if actual is not equal to the additive identity.
    fn is_zero(self) -> Self;

    fn is_additive_identity(self) -> Self;

    /// Fails if actual is not equal to the multiplicative identity.
    fn is_one(self) -> Self;

    fn is_multiplicative_identity(self) -> Self;

    fn be_negative(self) -> Self
    where
        T: Signed;

    fn is_negative(self) -> Self
    where
        T: Signed,
        Self: Sized,
    {
        self.be_negative()
    }

    fn be_positive(self) -> Self
    where
        T: Signed;

    fn is_positive(self) -> Self
    where
        T: Signed,
        Self: Sized,
    {
        self.be_positive()
    }

    /// Fails if actual is not in the range
    /// `[expected - allowed_deviation, expected + allowed_deviation]`.
    fn is_close_to(self, expected: T, allowed_deviation: T) -> Self
    where
        T: PartialOrd,
        T: Clone;

    #[cfg(any(feature = "std", feature = "libm"))]
    fn is_nan(self) -> Self
    where
        T: Float;

    #[cfg(any(feature = "std", feature = "libm"))]
    fn is_finite(self) -> Self
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
            self.add_detail_message(format!(
                "Expecting additive identity of type '{}'",
                core::any::type_name::<T>()
            ));
            let expected = T::zero();
            self.fail(|w: &mut String| {
                writedoc! {w, r#"
                    Expected: {expected:#?}

                      Actual: {actual:#?}
                "#}
            });
        }
        self
    }

    #[track_caller]
    fn is_additive_identity(self) -> Self {
        self.is_zero()
    }

    #[track_caller]
    fn is_one(self) -> Self {
        self.track_assertion();
        let actual = self.actual();
        if !actual.is_one() {
            self.add_detail_message(format!(
                "Expecting multiplicative identity of type '{}'",
                core::any::type_name::<T>()
            ));
            let expected = T::one();
            self.fail(|w: &mut String| {
                writedoc! {w, r#"
                    Expected: {expected:#?}

                      Actual: {actual:#?}
                "#}
            });
        }
        self
    }

    #[track_caller]
    fn is_multiplicative_identity(self) -> Self {
        self.is_one()
    }

    #[track_caller]
    fn be_negative(self) -> Self
    where
        T: Signed,
    {
        self.track_assertion();
        let actual = self.actual();
        if !actual.is_negative() {
            self.fail(|w: &mut String| {
                writedoc! {w, r#"
                    Expected value to be negative. But was

                      Actual: {actual:#?}
                "#}
            });
        }
        self
    }

    #[track_caller]
    fn be_positive(self) -> Self
    where
        T: Signed,
    {
        self.track_assertion();
        let actual = self.actual();
        if !actual.is_positive() {
            self.fail(|w: &mut String| {
                writedoc! {w, r#"
                    Expected value to be positive. But was

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
                    Expected value to be close to: {expected:#?},
                     with allowed deviation being: {allowed_deviation:#?},
                      but value was outside range: [{min:?}, {max:?}]

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
    fn is_finite(self) -> Self
    where
        T: Float,
    {
        self.track_assertion();
        let actual = self.actual();
        if !actual.is_finite() {
            self.fail(|w: &mut String| {
                writedoc! {w, r#"
                    Expected a finite value, but was

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
        assert_that(0.2f32 + 0.1f32).is_close_to(0.3, 0.0001);
        assert_that(0.2f64 + 0.1f64).is_close_to(0.3, 0.0001);

        assert_that(f32::nan()).is_nan();
        assert_that(f64::nan()).is_nan();

        assert_that(f32::infinity()).is_infinite();
        assert_that(f64::infinity()).is_infinite();
    }

    mod is_zero {
        use crate::prelude::*;
        use indoc::formatdoc;

        #[test]
        fn succeeds_when_zero() {
            assert_that(0).is_zero();
        }

        #[test]
        fn panics_when_not_zero() {
            assert_that_panic_by(|| assert_that(3).with_location(false).is_zero())
                .has_type::<String>()
                .is_equal_to(formatdoc! {r#"
                    -------- assertr --------
                    Expected: 0

                      Actual: 3
                    
                    Details: [
                        Expecting additive identity of type 'i32',
                    ]
                    -------- assertr --------
                "#});
        }
    }

    mod is_one {
        use crate::prelude::*;
        use indoc::formatdoc;

        #[test]
        fn succeeds_when_one() {
            assert_that(1).is_one();
        }

        #[test]
        fn panics_when_not_one() {
            assert_that_panic_by(|| assert_that(3).with_location(false).is_one())
                .has_type::<String>()
                .is_equal_to(formatdoc! {r#"
                    -------- assertr --------
                    Expected: 1

                      Actual: 3
                    
                    Details: [
                        Expecting multiplicative identity of type 'i32',
                    ]
                    -------- assertr --------
                "#});
        }
    }

    mod is_negative {
        use crate::prelude::*;
        use indoc::formatdoc;

        #[test]
        fn succeeds_when_zero() {
            assert_that(-0.01).is_negative();
        }

        #[test]
        fn panics_when_zero() {
            assert_that_panic_by(|| assert_that(0.0).with_location(false).is_negative())
                .has_type::<String>()
                .is_equal_to(formatdoc! {r#"
                    -------- assertr --------
                    Expected value to be negative. But was

                      Actual: 0.0
                    -------- assertr --------
                "#});
        }

        #[test]
        fn panics_when_positive() {
            assert_that_panic_by(|| assert_that(1.23).with_location(false).is_negative())
                .has_type::<String>()
                .is_equal_to(formatdoc! {r#"
                    -------- assertr --------
                    Expected value to be negative. But was

                      Actual: 1.23
                    -------- assertr --------
                "#});
        }
    }

    mod is_positive {
        use crate::prelude::*;
        use indoc::formatdoc;

        #[test]
        fn succeeds_when_positive() {
            assert_that(0.01).is_positive();
        }

        #[test]
        fn succeeds_when_zero() {
            assert_that(0.0).is_positive();
        }

        #[test]
        fn panics_when_negative() {
            assert_that_panic_by(|| assert_that(-1.23).with_location(false).is_positive())
                .has_type::<String>()
                .is_equal_to(formatdoc! {r#"
                    -------- assertr --------
                    Expected value to be positive. But was

                      Actual: -1.23
                    -------- assertr --------
                "#});
        }
    }

    mod is_close_to {
        use crate::prelude::*;
        use indoc::formatdoc;

        #[test]
        fn panics_when_below_allowed_range() {
            assert_that_panic_by(|| {
                assert_that(0.3319)
                    .with_location(false)
                    .is_close_to(0.333, 0.001)
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {r#"
                    -------- assertr --------
                    Expected value to be close to: 0.333,
                     with allowed deviation being: 0.001,
                      but value was outside range: [0.332, 0.334]

                      Actual: 0.3319
                    -------- assertr --------
                "#});
        }

        #[test]
        fn succeeds_when_actual_is_in_allowed_range() {
            assert_that(0.332).is_close_to(0.333, 0.001);
            assert_that(0.333).is_close_to(0.333, 0.001);
            assert_that(0.334).is_close_to(0.333, 0.001);
        }

        #[test]
        fn panics_when_above_allowed_range() {
            assert_that_panic_by(|| {
                assert_that(0.3341)
                    .with_location(false)
                    .is_close_to(0.333, 0.001)
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {r#"
                    -------- assertr --------
                    Expected value to be close to: 0.333,
                     with allowed deviation being: 0.001,
                      but value was outside range: [0.332, 0.334]

                      Actual: 0.3341
                    -------- assertr --------
                "#});
        }
    }

    mod is_nan {
        use crate::prelude::*;
        use ::num::Float;
        use indoc::formatdoc;

        #[test]
        fn succeeds_when_nan() {
            assert_that(f32::nan()).is_nan();
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

    mod is_finite {
        use crate::prelude::*;
        use indoc::formatdoc;
        use num::Float;

        #[test]
        fn succeeds_when_finite() {
            assert_that(0.3f32).is_finite();
        }

        #[test]
        fn panics_when_positive_infinity() {
            assert_that_panic_by(|| {
                assert_that(f32::infinity())
                    .with_location(false)
                    .is_finite()
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {r#"
                    -------- assertr --------
                    Expected a finite value, but was

                      Actual: inf
                    -------- assertr --------
                "#});
        }

        #[test]
        fn panics_when_negative_infinity() {
            assert_that_panic_by(|| {
                assert_that(f32::neg_infinity())
                    .with_location(false)
                    .is_finite()
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {r#"
                    -------- assertr --------
                    Expected a finite value, but was

                      Actual: -inf
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
            assert_that(f32::infinity()).is_infinite();
        }

        #[test]
        fn succeeds_when_negative_infinity() {
            assert_that(f32::neg_infinity()).is_infinite();
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
