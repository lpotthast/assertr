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

    /// Fails if actual is not equal to the additive identity.
    fn be_zero(self) -> Self
    where
        Self: Sized,
    {
        self.is_zero()
    }

    fn is_additive_identity(self) -> Self;
    fn be_additive_identity(self) -> Self
    where
        Self: Sized,
    {
        self.is_additive_identity()
    }

    /// Fails if actual is not equal to the multiplicative identity.
    fn is_one(self) -> Self;

    /// Fails if actual is not equal to the multiplicative identity.
    fn be_one(self) -> Self
    where
        Self: Sized,
    {
        self.is_one()
    }

    fn is_multiplicative_identity(self) -> Self;
    fn be_multiplicative_identity(self) -> Self
    where
        Self: Sized,
    {
        self.is_multiplicative_identity()
    }

    fn is_negative(self) -> Self
    where
        T: Signed;

    fn be_negative(self) -> Self
    where
        T: Signed,
        Self: Sized,
    {
        self.is_negative()
    }

    fn is_positive(self) -> Self
    where
        T: Signed;

    fn be_positive(self) -> Self
    where
        T: Signed,
        Self: Sized,
    {
        self.is_positive()
    }

    /// Fails if actual is not in the range
    /// `[expected - allowed_deviation, expected + allowed_deviation]`.
    fn is_close_to(self, expected: T, allowed_deviation: T) -> Self
    where
        T: PartialOrd,
        T: Clone;

    /// Fails if actual is not in the range
    /// `[expected - allowed_deviation, expected + allowed_deviation]`.
    fn be_close_to(self, expected: T, allowed_deviation: T) -> Self
    where
        T: PartialOrd,
        T: Clone,
        Self: Sized,
    {
        self.is_close_to(expected, allowed_deviation)
    }

    #[cfg(any(feature = "std", feature = "libm"))]
    fn is_nan(self) -> Self
    where
        T: Float;
    #[cfg(any(feature = "std", feature = "libm"))]
    fn be_nan(self) -> Self
    where
        T: Float,
        Self: Sized,
    {
        self.is_nan()
    }

    #[cfg(any(feature = "std", feature = "libm"))]
    fn is_finite(self) -> Self
    where
        T: Float;
    #[cfg(any(feature = "std", feature = "libm"))]
    fn be_finite(self) -> Self
    where
        T: Float,
        Self: Sized,
    {
        self.is_finite()
    }

    #[cfg(any(feature = "std", feature = "libm"))]
    fn is_infinite(self) -> Self
    where
        T: Float;
    #[cfg(any(feature = "std", feature = "libm"))]
    fn be_infinite(self) -> Self
    where
        T: Float,
        Self: Sized,
    {
        self.is_infinite()
    }

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
    fn is_negative(self) -> Self
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
    fn is_positive(self) -> Self
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

        0u8.must().be_zero();
        0i8.must().be_zero();
        0u16.must().be_zero();
        0i16.must().be_zero();
        0u32.must().be_zero();
        0i32.must().be_zero();
        0u64.must().be_zero();
        0i64.must().be_zero();
        0u128.must().be_zero();
        0i128.must().be_zero();
        0.0f32.must().be_zero();
        0.0f64.must().be_zero();

        1u8.must().be_one();
        1i8.must().be_one();
        1u16.must().be_one();
        1i16.must().be_one();
        1u32.must().be_one();
        1i32.must().be_one();
        1u64.must().be_one();
        1i64.must().be_one();
        1u128.must().be_one();
        1i128.must().be_one();
        1.0f32.must().be_one();
        1.0f64.must().be_one();

        42u8.must().be_close_to(42, 0);
        42i8.must().be_close_to(42, 0);
        42u16.must().be_close_to(42, 0);
        42i16.must().be_close_to(42, 0);
        42u32.must().be_close_to(42, 0);
        42i32.must().be_close_to(42, 0);
        42u64.must().be_close_to(42, 0);
        42i64.must().be_close_to(42, 0);
        42u128.must().be_close_to(42, 0);
        42i128.must().be_close_to(42, 0);
        (0.2f32 + 0.1f32).must().be_close_to(0.3, 0.0001);
        (0.2f64 + 0.1f64).must().be_close_to(0.3, 0.0001);

        f32::nan().must().be_nan();
        f64::nan().must().be_nan();

        f32::infinity().must().be_infinite();
        f64::infinity().must().be_infinite();
    }

    mod is_zero {
        use crate::prelude::*;
        use indoc::formatdoc;

        #[test]
        fn succeeds_when_zero() {
            0.must().be_zero();
        }

        #[test]
        fn panics_when_not_zero() {
            assert_that_panic_by(|| 3.must().with_location(false).be_zero())
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
            1.must().be_one();
        }

        #[test]
        fn panics_when_not_one() {
            assert_that_panic_by(|| 3.must().with_location(false).be_one())
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
            (-0.01).must().be_negative();
        }

        #[test]
        fn panics_when_zero() {
            assert_that_panic_by(|| 0.0.must().with_location(false).be_negative())
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
            assert_that_panic_by(|| 1.23.must().with_location(false).be_negative())
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
            0.01.must().be_positive();
        }

        #[test]
        fn succeeds_when_zero() {
            0.0.must().be_positive();
        }

        #[test]
        fn panics_when_negative() {
            assert_that_panic_by(|| (-1.23).must().with_location(false).be_positive())
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
            assert_that_panic_by(|| 0.3319.must().with_location(false).be_close_to(0.333, 0.001))
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
            0.332.must().be_close_to(0.333, 0.001);
            0.333.must().be_close_to(0.333, 0.001);
            0.334.must().be_close_to(0.333, 0.001);
        }

        #[test]
        fn panics_when_above_allowed_range() {
            assert_that_panic_by(|| 0.3341.must().with_location(false).be_close_to(0.333, 0.001))
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
            f32::nan().must().be_nan();
        }

        #[test]
        fn panics_when_not_nan() {
            assert_that_panic_by(|| 1.23.must().with_location(false).be_nan())
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
            0.3f32.must().be_finite();
        }

        #[test]
        fn panics_when_positive_infinity() {
            assert_that_panic_by(|| {
                f32::infinity().must().with_location(false).be_finite();
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
                f32::neg_infinity().must().with_location(false).be_finite();
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
            f32::infinity().must().be_infinite();
        }

        #[test]
        fn succeeds_when_negative_infinity() {
            f32::neg_infinity().must().be_infinite();
        }

        #[test]
        fn panics_when_not_infinity() {
            assert_that_panic_by(|| 1.23.must().with_location(false).be_infinite())
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
