use crate::AssertThat;
use crate::AssertionRenderer;
use crate::mode::Mode;
use crate::tracking::AssertionTracking;
use alloc::format;
use alloc::string::String;
use core::cmp::Ordering;
use core::fmt::Write;
use indoc::writedoc;
#[cfg(any(feature = "std", feature = "libm"))]
use num::Float;
use num::{Num, Signed};

/// Overflow-safe check of `hi - lo <= deviation`, given `lo <= hi` and `deviation >= 0`.
///
/// The difference of same-signed values never overflows. When the values cross zero, the
/// distance is compared piecewise so that neither `hi - lo` nor `-lo` is ever materialized
/// (`lo` may be the minimum of a signed type).
fn distance_at_most<T>(lo: &T, hi: &T, deviation: &T) -> bool
where
    T: Num + PartialOrd + Clone,
{
    let zero = T::zero();
    if hi < &zero {
        // Both negative: their difference never overflows.
        hi.clone() - lo.clone() <= deviation.clone()
    } else if lo >= &zero {
        // Both non-negative: `lo >= 0` is trivially within a lower bound `hi - deviation <= 0`;
        // otherwise that bound is safe to materialize.
        deviation >= hi || lo >= &(hi.clone() - deviation.clone())
    } else if deviation < hi {
        // Crossing zero: the distance is `hi + |lo| > hi`, which already exceeds the deviation.
        false
    } else {
        // `lo < 0 <= hi`: compare `lo` against `-(deviation - hi)` without negating `lo`.
        lo >= &(zero - (deviation.clone() - hi.clone()))
    }
}

/// Assertions for numeric values not already handled by
/// [`crate::prelude::PartialEqAssertions`] and [`crate::prelude::PartialOrdAssertions`].
#[allow(clippy::return_self_not_must_use)]
#[cfg_attr(feature = "fluent", assertr_derive::fluent_aliases)]
pub trait NumAssertions<T: Num> {
    /// Fails if actual is not equal to the additive identity.
    fn is_zero(self) -> Self;

    fn is_additive_identity(self) -> Self;

    /// Fails if actual is not equal to the multiplicative identity.
    fn is_one(self) -> Self;

    fn is_multiplicative_identity(self) -> Self;

    fn is_negative(self) -> Self
    where
        T: Signed;

    fn is_positive(self) -> Self
    where
        T: Signed;

    /// Fails if actual is not in the range
    /// `[expected - allowed_deviation, expected + allowed_deviation]`.
    ///
    /// The allowed deviation must be a non-negative number. Boundary calculations avoid
    /// overflowing the numeric type. Positive-infinite deviation accepts every comparable
    /// non-NaN value.
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

impl<T: Num, M: Mode, R> NumAssertions<T> for AssertThat<'_, T, M, R>
where
    R: AssertionRenderer<T>,
{
    #[track_caller]
    fn is_zero(self) -> Self {
        self.track_assertion();
        let actual = self.actual();
        if !actual.is_zero() {
            let details = [format!(
                "Expecting additive identity of type '{}'",
                core::any::type_name::<T>()
            )];
            let expected = T::zero();
            let expected = self.render_value(&expected);
            let actual = self.render_value(actual);
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
    fn is_additive_identity(self) -> Self {
        self.is_zero()
    }

    #[track_caller]
    fn is_one(self) -> Self {
        self.track_assertion();
        let actual = self.actual();
        if !actual.is_one() {
            let details = [format!(
                "Expecting multiplicative identity of type '{}'",
                core::any::type_name::<T>()
            )];
            let expected = T::one();
            let expected = self.render_value(&expected);
            let actual = self.render_value(actual);
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
            let actual = self.render_value(actual);
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
        T: Signed,
    {
        self.track_assertion();
        let actual = self.actual();
        if !actual.is_positive() {
            let actual = self.render_value(actual);
            self.fail(|w: &mut String| {
                writedoc! {w, r"
                    Expected value to be positive. But was

                      Actual: {actual:#?}
                "}
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
        let zero = T::zero();

        // Rejects both negative deviations and a NaN deviation, which is incomparable to zero.
        let deviation_is_valid = matches!(
            allowed_deviation.partial_cmp(&zero),
            Some(Ordering::Greater | Ordering::Equal)
        );
        if !deviation_is_valid {
            let allowed_deviation = self.render_value(&allowed_deviation);
            self.fail(|w: &mut String| {
                writedoc! {w, r"
                    Allowed deviation must be a non-negative number.

                    Actual deviation: {allowed_deviation:#?}
                "}
            });
            return self;
        }

        // An infinite deviation is detectable without a `Float` bound: `inf - inf` is NaN and
        // therefore incomparable to zero. NaN deviations were already rejected above.
        let deviation_is_unbounded = (allowed_deviation.clone() - allowed_deviation.clone())
            .partial_cmp(&zero)
            .is_none();

        // Checked before `partial_cmp` so equal infinities are accepted without computing their
        // (NaN) distance.
        let within_allowed_deviation = if actual == &expected {
            true
        } else {
            match actual.partial_cmp(&expected) {
                None => false,
                Some(_) if deviation_is_unbounded => true,
                Some(Ordering::Less) => distance_at_most(actual, &expected, &allowed_deviation),
                Some(_) => distance_at_most(&expected, actual, &allowed_deviation),
            }
        };

        if !within_allowed_deviation {
            let actual = self.render_value(actual);
            let expected = self.render_value(&expected);
            let allowed_deviation = self.render_value(&allowed_deviation);
            self.fail(|w: &mut String| {
                writedoc! {w, r"
                    Expected value to be close to: {expected:#?},
                     with allowed deviation being: {allowed_deviation:#?},
                      but value was outside the allowed deviation.

                      Actual: {actual:#?}
                "}
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
            let nan = self.render_value(&nan);
            let actual = self.render_value(actual);
            self.fail(|w: &mut String| {
                writedoc! {w, r"
                    Expected: {nan:#?}

                      Actual: {actual:#?}
                "}
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
            let actual = self.render_value(actual);
            self.fail(|w: &mut String| {
                writedoc! {w, r"
                    Expected a finite value, but was

                      Actual: {actual:#?}
                "}
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
            let inf = self.render_value(&inf);
            let actual = self.render_value(actual);
            self.fail(|w: &mut String| {
                writedoc! {w, r"
                    Expected: +/- {inf:#?}

                      Actual: {actual:#?}
                "}
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

        assert_that!(0u8).is_zero();
        assert_that!(0i8).is_zero();
        assert_that!(0u16).is_zero();
        assert_that!(0i16).is_zero();
        assert_that!(0u32).is_zero();
        assert_that!(0i32).is_zero();
        assert_that!(0u64).is_zero();
        assert_that!(0i64).is_zero();
        assert_that!(0u128).is_zero();
        assert_that!(0i128).is_zero();
        assert_that!(0.0f32).is_zero();
        assert_that!(0.0f64).is_zero();

        assert_that!(1u8).is_one();
        assert_that!(1i8).is_one();
        assert_that!(1u16).is_one();
        assert_that!(1i16).is_one();
        assert_that!(1u32).is_one();
        assert_that!(1i32).is_one();
        assert_that!(1u64).is_one();
        assert_that!(1i64).is_one();
        assert_that!(1u128).is_one();
        assert_that!(1i128).is_one();
        assert_that!(1.0f32).is_one();
        assert_that!(1.0f64).is_one();

        assert_that!(42u8).is_close_to(42, 0);
        assert_that!(42i8).is_close_to(42, 0);
        assert_that!(42u16).is_close_to(42, 0);
        assert_that!(42i16).is_close_to(42, 0);
        assert_that!(42u32).is_close_to(42, 0);
        assert_that!(42i32).is_close_to(42, 0);
        assert_that!(42u64).is_close_to(42, 0);
        assert_that!(42i64).is_close_to(42, 0);
        assert_that!(42u128).is_close_to(42, 0);
        assert_that!(42i128).is_close_to(42, 0);
        assert_that!(0.2f32 + 0.1f32).is_close_to(0.3, 0.0001);
        assert_that!(0.2f64 + 0.1f64).is_close_to(0.3, 0.0001);

        assert_that!(f32::nan()).is_nan();
        assert_that!(f64::nan()).is_nan();

        assert_that!(f32::infinity()).is_infinite();
        assert_that!(f64::infinity()).is_infinite();
    }

    mod is_zero {
        use crate::prelude::*;
        use indoc::formatdoc;

        #[test]
        fn succeeds_when_zero() {
            assert_that!(0).is_zero();
        }

        #[test]
        fn panics_when_not_zero() {
            assert_that_panic_by(|| assert_that!(3).with_location(false).is_zero())
                .has_type::<String>()
                .is_equal_to(formatdoc! {r"
                    -------- assertr --------
                    Expected: 0

                      Actual: 3
                    
                    Details: [
                        Expecting additive identity of type 'i32',
                    ]
                    -------- assertr --------
                "});
        }
    }

    mod is_one {
        use crate::prelude::*;
        use indoc::formatdoc;

        #[test]
        fn succeeds_when_one() {
            assert_that!(1).is_one();
        }

        #[test]
        fn panics_when_not_one() {
            assert_that_panic_by(|| assert_that!(3).with_location(false).is_one())
                .has_type::<String>()
                .is_equal_to(formatdoc! {r"
                    -------- assertr --------
                    Expected: 1

                      Actual: 3
                    
                    Details: [
                        Expecting multiplicative identity of type 'i32',
                    ]
                    -------- assertr --------
                "});
        }
    }

    mod is_negative {
        use crate::prelude::*;
        use indoc::formatdoc;

        #[test]
        fn succeeds_when_zero() {
            assert_that!(-0.01).is_negative();
        }

        #[test]
        fn panics_when_zero() {
            assert_that_panic_by(|| assert_that!(0.0).with_location(false).is_negative())
                .has_type::<String>()
                .is_equal_to(formatdoc! {r"
                    -------- assertr --------
                    Expected value to be negative. But was

                      Actual: 0.0
                    -------- assertr --------
                "});
        }

        #[test]
        fn panics_when_positive() {
            assert_that_panic_by(|| assert_that!(1.23).with_location(false).is_negative())
                .has_type::<String>()
                .is_equal_to(formatdoc! {r"
                    -------- assertr --------
                    Expected value to be negative. But was

                      Actual: 1.23
                    -------- assertr --------
                "});
        }
    }

    mod is_positive {
        use crate::prelude::*;
        use indoc::formatdoc;

        #[test]
        fn succeeds_when_positive() {
            assert_that!(0.01).is_positive();
        }

        #[test]
        fn succeeds_when_zero() {
            assert_that!(0.0).is_positive();
        }

        #[test]
        fn panics_when_negative() {
            assert_that_panic_by(|| assert_that!(-1.23).with_location(false).is_positive())
                .has_type::<String>()
                .is_equal_to(formatdoc! {r"
                    -------- assertr --------
                    Expected value to be positive. But was

                      Actual: -1.23
                    -------- assertr --------
                "});
        }
    }

    mod is_close_to {
        use crate::prelude::*;
        use indoc::formatdoc;

        #[test]
        fn panics_when_below_allowed_range() {
            assert_that_panic_by(|| {
                assert_that!(0.3319)
                    .with_location(false)
                    .is_close_to(0.333, 0.001)
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {r"
                    -------- assertr --------
                    Expected value to be close to: 0.333,
                     with allowed deviation being: 0.001,
                      but value was outside the allowed deviation.

                      Actual: 0.3319
                    -------- assertr --------
                "});
        }

        #[test]
        fn succeeds_when_actual_is_in_allowed_range() {
            assert_that!(0.332).is_close_to(0.333, 0.001);
            assert_that!(0.333).is_close_to(0.333, 0.001);
            assert_that!(0.334).is_close_to(0.333, 0.001);
            assert_that!(0_u8).is_close_to(0, 1);
            assert_that!(-100_i8).is_close_to(20, 120);
            assert_that!(i8::MIN).is_close_to(-1, i8::MAX);
        }

        #[test]
        fn succeeds_for_equal_negative_infinity() {
            assert_that!(f64::NEG_INFINITY).is_close_to(f64::NEG_INFINITY, 0.0);
        }

        #[test]
        fn positive_infinite_deviation_is_unbounded_for_comparable_values() {
            let deviation = f64::INFINITY;

            assert_that!(-1.0).is_close_to(f64::INFINITY, deviation);
            assert_that!(1.0).is_close_to(f64::NEG_INFINITY, deviation);
            assert_that!(f64::INFINITY).is_close_to(-1.0, deviation);
            assert_that!(f64::NEG_INFINITY).is_close_to(1.0, deviation);
            assert_that!(f64::INFINITY).is_close_to(f64::NEG_INFINITY, deviation);
            assert_that!(f64::NEG_INFINITY).is_close_to(f64::INFINITY, deviation);
        }

        #[test]
        fn positive_infinite_deviation_does_not_accept_nan_values() {
            assert_that_panic_by(|| {
                assert_that!(f64::NAN)
                    .with_location(false)
                    .is_close_to(1.0, f64::INFINITY);
            })
            .has_type::<String>()
            .contains("outside the allowed deviation");

            assert_that_panic_by(|| {
                assert_that!(1.0)
                    .with_location(false)
                    .is_close_to(f64::NAN, f64::INFINITY);
            })
            .has_type::<String>()
            .contains("outside the allowed deviation");
        }

        #[test]
        fn rejects_nan_deviation() {
            assert_that_panic_by(|| {
                assert_that!(1.0)
                    .with_location(false)
                    .is_close_to(1.0, f64::NAN);
            })
            .has_type::<String>()
            .contains("Allowed deviation must be a non-negative number");
        }

        #[test]
        fn reports_extreme_signed_values_without_overflowing() {
            assert_that_panic_by(|| {
                assert_that!(i8::MIN)
                    .with_location(false)
                    .is_close_to(i8::MAX, i8::MAX);
            })
            .has_type::<String>()
            .contains("outside the allowed deviation");
        }

        #[test]
        fn rejects_negative_deviation() {
            assert_that_panic_by(|| {
                assert_that!(1_i8).with_location(false).is_close_to(1, -1);
            })
            .has_type::<String>()
            .contains("Allowed deviation must be a non-negative number");
        }

        #[test]
        fn panics_when_above_allowed_range() {
            assert_that_panic_by(|| {
                assert_that!(0.3341)
                    .with_location(false)
                    .is_close_to(0.333, 0.001)
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {r"
                    -------- assertr --------
                    Expected value to be close to: 0.333,
                     with allowed deviation being: 0.001,
                      but value was outside the allowed deviation.

                      Actual: 0.3341
                    -------- assertr --------
                "});
        }
    }

    mod is_nan {
        use crate::prelude::*;
        use ::num::Float;
        use indoc::formatdoc;

        #[test]
        fn succeeds_when_nan() {
            assert_that!(f32::nan()).is_nan();
        }

        #[test]
        fn panics_when_not_nan() {
            assert_that_panic_by(|| assert_that!(1.23).with_location(false).is_nan())
                .has_type::<String>()
                .is_equal_to(formatdoc! {r"
                    -------- assertr --------
                    Expected: NaN

                      Actual: 1.23
                    -------- assertr --------
                "});
        }
    }

    mod is_finite {
        use crate::prelude::*;
        use indoc::formatdoc;
        use num::Float;

        #[test]
        fn succeeds_when_finite() {
            assert_that!(0.3f32).is_finite();
        }

        #[test]
        fn panics_when_positive_infinity() {
            assert_that_panic_by(|| {
                assert_that!(f32::infinity())
                    .with_location(false)
                    .is_finite();
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {r"
                    -------- assertr --------
                    Expected a finite value, but was

                      Actual: inf
                    -------- assertr --------
                "});
        }

        #[test]
        fn panics_when_negative_infinity() {
            assert_that_panic_by(|| {
                assert_that!(f32::neg_infinity())
                    .with_location(false)
                    .is_finite();
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {r"
                    -------- assertr --------
                    Expected a finite value, but was

                      Actual: -inf
                    -------- assertr --------
                "});
        }
    }

    mod is_infinite {
        use crate::prelude::*;
        use ::num::Float;
        use indoc::formatdoc;

        #[test]
        fn succeeds_when_positive_infinity() {
            assert_that!(f32::infinity()).is_infinite();
        }

        #[test]
        fn succeeds_when_negative_infinity() {
            assert_that!(f32::neg_infinity()).is_infinite();
        }

        #[test]
        fn panics_when_not_infinity() {
            assert_that_panic_by(|| assert_that!(1.23).with_location(false).is_infinite())
                .has_type::<String>()
                .is_equal_to(formatdoc! {r"
                    -------- assertr --------
                    Expected: +/- inf

                      Actual: 1.23
                    -------- assertr --------
                "});
        }
    }
}
