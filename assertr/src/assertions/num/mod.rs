//! Assertions for numeric identities, signs, tolerances, and floating-point classifications.

use crate::AssertThat;
use crate::ValueRenderer;
use crate::mode::Mode;
use alloc::format;
use alloc::string::String;
use core::cmp::Ordering;
use core::fmt::Write;
use indoc::writedoc;
#[cfg(any(feature = "std", feature = "libm"))]
use num_traits::Float;
use num_traits::{Num, Signed};

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
    /// The renderer carried by the assertion chain.
    type Renderer;

    /// Asserts that the subject equals the additive identity, zero.
    fn is_zero(self) -> Self
    where
        Self::Renderer: ValueRenderer<T>;

    /// Alias of [`NumAssertions::is_zero`].
    fn is_additive_identity(self) -> Self
    where
        Self::Renderer: ValueRenderer<T>;

    /// Asserts that the subject equals the multiplicative identity, one.
    fn is_one(self) -> Self
    where
        Self::Renderer: ValueRenderer<T>;

    /// Alias of [`NumAssertions::is_one`].
    fn is_multiplicative_identity(self) -> Self
    where
        Self::Renderer: ValueRenderer<T>;

    /// Asserts that [`Signed::is_negative`] returns true for the subject.
    ///
    /// For floating-point values this tests the sign bit, so `-0.0` and a negative-sign NaN are
    /// considered negative.
    fn is_negative(self) -> Self
    where
        T: Signed,
        Self::Renderer: ValueRenderer<T>;

    /// Asserts that [`Signed::is_positive`] returns true for the subject.
    ///
    /// For floating-point values this tests the sign bit, so `0.0` and a positive-sign NaN are
    /// considered positive.
    fn is_positive(self) -> Self
    where
        T: Signed,
        Self::Renderer: ValueRenderer<T>;

    /// Asserts that the subject is within `allowed_deviation` of `expected`.
    ///
    /// A negative or NaN deviation fails the assertion. Boundary calculations avoid
    /// overflowing the numeric type. Positive-infinite deviation accepts every comparable
    /// non-NaN value.
    fn is_close_to(self, expected: T, allowed_deviation: T) -> Self
    where
        T: PartialOrd,
        T: Clone,
        Self::Renderer: ValueRenderer<T>;

    /// Asserts that the subject is NaN.
    #[cfg(any(feature = "std", feature = "libm"))]
    fn is_nan(self) -> Self
    where
        T: Float,
        Self::Renderer: ValueRenderer<T>;

    /// Asserts that the subject is finite.
    #[cfg(any(feature = "std", feature = "libm"))]
    fn is_finite(self) -> Self
    where
        T: Float,
        Self::Renderer: ValueRenderer<T>;

    /// Asserts that the subject is positive or negative infinity.
    #[cfg(any(feature = "std", feature = "libm"))]
    fn is_infinite(self) -> Self
    where
        T: Float,
        Self::Renderer: ValueRenderer<T>;

    /// Asserts that the subject is a normal floating-point value.
    #[cfg(any(feature = "std", feature = "libm"))]
    fn is_normal(self) -> Self
    where
        T: Float,
        Self::Renderer: ValueRenderer<T>;

    /// Asserts that the subject is a subnormal floating-point value.
    #[cfg(any(feature = "std", feature = "libm"))]
    fn is_subnormal(self) -> Self
    where
        T: Float,
        Self::Renderer: ValueRenderer<T>;
}

impl<T: Num, M: Mode, R> NumAssertions<T> for AssertThat<'_, T, M, R> {
    type Renderer = R;

    #[track_caller]
    fn is_zero(self) -> Self
    where
        R: ValueRenderer<T>,
    {
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
    fn is_additive_identity(self) -> Self
    where
        R: ValueRenderer<T>,
    {
        self.is_zero()
    }

    #[track_caller]
    fn is_one(self) -> Self
    where
        R: ValueRenderer<T>,
    {
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
    fn is_multiplicative_identity(self) -> Self
    where
        R: ValueRenderer<T>,
    {
        self.is_one()
    }

    #[track_caller]
    fn is_negative(self) -> Self
    where
        T: Signed,
        R: ValueRenderer<T>,
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
        R: ValueRenderer<T>,
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
        R: ValueRenderer<T>,
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
        R: ValueRenderer<T>,
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
        R: ValueRenderer<T>,
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
        R: ValueRenderer<T>,
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

    #[track_caller]
    #[cfg(any(feature = "std", feature = "libm"))]
    fn is_normal(self) -> Self
    where
        T: Float,
        R: ValueRenderer<T>,
    {
        self.track_assertion();
        let actual = self.actual();
        if !actual.is_normal() {
            let actual = self.render_value(actual);
            self.fail(|w: &mut String| {
                writedoc! {w, r"
                    Expected a normal floating-point value, but was

                      Actual: {actual:#?}
                "}
            });
        }
        self
    }

    #[track_caller]
    #[cfg(any(feature = "std", feature = "libm"))]
    fn is_subnormal(self) -> Self
    where
        T: Float,
        R: ValueRenderer<T>,
    {
        self.track_assertion();
        let actual = self.actual();
        if !actual.is_subnormal() {
            let actual = self.render_value(actual);
            self.fail(|w: &mut String| {
                writedoc! {w, r"
                    Expected a subnormal floating-point value, but was

                      Actual: {actual:#?}
                "}
            });
        }
        self
    }
}

#[cfg(test)]
mod tests {
    // These helpers deliberately take `T` by value to model the signature available to downstream
    // generic code, rather than proving the assertion bounds only for `&T`.
    #[allow(clippy::needless_pass_by_value)]
    mod generic_num_traits_bounds {
        use core::fmt::Debug;
        use core::ops::{Add, Div, Mul, Neg, Rem, Sub};

        use num_traits::{Num, One, Signed, Zero};

        use crate::prelude::*;

        #[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
        struct Money(i64);

        impl Add for Money {
            type Output = Self;

            fn add(self, rhs: Self) -> Self {
                Self(self.0 + rhs.0)
            }
        }

        impl Sub for Money {
            type Output = Self;

            fn sub(self, rhs: Self) -> Self {
                Self(self.0 - rhs.0)
            }
        }

        impl Mul for Money {
            type Output = Self;

            fn mul(self, rhs: Self) -> Self {
                Self(self.0 * rhs.0)
            }
        }

        impl Div for Money {
            type Output = Self;

            fn div(self, rhs: Self) -> Self {
                Self(self.0 / rhs.0)
            }
        }

        impl Rem for Money {
            type Output = Self;

            fn rem(self, rhs: Self) -> Self {
                Self(self.0 % rhs.0)
            }
        }

        impl Neg for Money {
            type Output = Self;

            fn neg(self) -> Self {
                Self(-self.0)
            }
        }

        impl Zero for Money {
            fn zero() -> Self {
                Self(0)
            }

            fn is_zero(&self) -> bool {
                self.0 == 0
            }
        }

        impl One for Money {
            fn one() -> Self {
                Self(1)
            }
        }

        impl Num for Money {
            type FromStrRadixErr = <i64 as Num>::FromStrRadixErr;

            fn from_str_radix(str: &str, radix: u32) -> Result<Self, Self::FromStrRadixErr> {
                i64::from_str_radix(str, radix).map(Self)
            }
        }

        impl Signed for Money {
            fn abs(&self) -> Self {
                Self(self.0.abs())
            }

            fn abs_sub(&self, other: &Self) -> Self {
                Self(Signed::abs_sub(&self.0, &other.0))
            }

            fn signum(&self) -> Self {
                Self(self.0.signum())
            }

            fn is_positive(&self) -> bool {
                self.0.is_positive()
            }

            fn is_negative(&self) -> bool {
                self.0.is_negative()
            }
        }

        fn assert_identities<T: Num + Debug>(zero: T, one: T) {
            assert_that!(zero).is_zero().is_additive_identity();
            assert_that!(one).is_one().is_multiplicative_identity();
        }

        fn assert_close_to<T: Num + PartialOrd + Clone + Debug>(
            value: T,
            expected: T,
            deviation: T,
        ) {
            assert_that!(value).is_close_to(expected, deviation);
        }

        fn assert_sign<T: Num + Signed + Debug>(negative: T, positive: T) {
            assert_that!(negative).is_negative();
            assert_that!(positive).is_positive();
        }

        #[test]
        fn a_user_defined_numeric_type_reaches_assertions_through_generic_bounds() {
            assert_identities(Money(0), Money(1));
            assert_close_to(Money(42), Money(40), Money(2));
            assert_sign(Money(-7), Money(7));
        }
    }

    #[test]
    fn quick_type_check() {
        use crate::prelude::*;

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
        assert_that!(0usize).is_zero();
        assert_that!(0isize).is_zero();
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
        assert_that!(1usize).is_one();
        assert_that!(1isize).is_one();
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
        assert_that!(42usize).is_close_to(42, 0);
        assert_that!(42isize).is_close_to(42, 0);
        assert_that!(0.2f32 + 0.1f32).is_close_to(0.3, 0.0001);
        assert_that!(0.2f64 + 0.1f64).is_close_to(0.3, 0.0001);
    }

    // The float classification assertions require floating point math, which `num` only provides
    // with either `std` or `libm` enabled.
    #[test]
    #[cfg(any(feature = "std", feature = "libm"))]
    fn quick_float_type_check() {
        use core::fmt::Debug;

        use crate::prelude::*;
        use ::num_traits::Float;

        fn assert_classification<T: Float + Debug>(nan: T, finite: T, infinite: T) {
            assert_that!(nan).is_nan();
            assert_that!(finite).is_finite();
            assert_that!(infinite).is_infinite();
        }

        assert_classification(f32::nan(), 1.0f32, f32::infinity());
        assert_classification(f64::nan(), 1.0f64, f64::infinity());
    }

    mod is_zero {
        use crate::prelude::*;
        use indoc::formatdoc;

        #[test]
        #[cfg(feature = "fluent")]
        fn fluent_alias_is_as_expected() {
            0_i32.must().be_zero();
        }

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
                    Expression: `3`

                    Expected: 0

                      Actual: 3
                    
                    Details: [
                        Expecting additive identity of type 'i32',
                    ]
                    -------- assertr --------
                "});
        }
    }

    /// Synonym of `is_zero`. Only the fluent name is pinned here. The behavior is covered by
    /// that module.
    mod is_additive_identity {
        #[cfg(feature = "fluent")]
        use crate::prelude::*;

        #[test]
        #[cfg(feature = "fluent")]
        fn fluent_alias_is_as_expected() {
            0_i32.must().be_additive_identity();
        }
    }

    mod is_one {
        use crate::prelude::*;
        use indoc::formatdoc;

        #[test]
        #[cfg(feature = "fluent")]
        fn fluent_alias_is_as_expected() {
            1_i32.must().be_one();
        }

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
                    Expression: `3`

                    Expected: 1

                      Actual: 3
                    
                    Details: [
                        Expecting multiplicative identity of type 'i32',
                    ]
                    -------- assertr --------
                "});
        }
    }

    /// Synonym of `is_one`. Only the fluent name is pinned here. The behavior is covered by
    /// that module.
    mod is_multiplicative_identity {
        #[cfg(feature = "fluent")]
        use crate::prelude::*;

        #[test]
        #[cfg(feature = "fluent")]
        fn fluent_alias_is_as_expected() {
            1_i32.must().be_multiplicative_identity();
        }
    }

    mod is_negative {
        use crate::prelude::*;
        use indoc::formatdoc;

        #[test]
        #[cfg(feature = "fluent")]
        fn fluent_alias_is_as_expected() {
            (-0.01_f64).must().be_negative();
        }

        #[test]
        fn succeeds_when_negative() {
            assert_that!(-0.01).is_negative();
        }

        #[test]
        fn uses_the_float_sign_bit() {
            assert_that!(-0.0).is_negative();
            assert_that!(-f64::NAN).is_negative();
        }

        #[test]
        fn panics_when_zero() {
            assert_that_panic_by(|| assert_that!(0.0).with_location(false).is_negative())
                .has_type::<String>()
                .is_equal_to(formatdoc! {r"
                    -------- assertr --------
                    Expression: `0.0`

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
                    Expression: `1.23`

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
        #[cfg(feature = "fluent")]
        fn fluent_alias_is_as_expected() {
            0.01_f64.must().be_positive();
        }

        #[test]
        fn succeeds_when_positive() {
            assert_that!(0.01).is_positive();
        }

        #[test]
        fn succeeds_for_positive_zero() {
            assert_that!(0.0).is_positive();
        }

        #[test]
        fn uses_the_float_sign_bit() {
            assert_that!(f64::NAN).is_positive();
        }

        #[test]
        fn panics_when_negative() {
            assert_that_panic_by(|| assert_that!(-1.23).with_location(false).is_positive())
                .has_type::<String>()
                .is_equal_to(formatdoc! {r"
                    -------- assertr --------
                    Expression: `-1.23`

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
        #[cfg(feature = "fluent")]
        fn fluent_alias_is_as_expected() {
            0.333_f64.must().be_close_to(0.333, 0.001);
        }

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
                    Expression: `0.3319`

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
                    Expression: `0.3341`

                    Expected value to be close to: 0.333,
                     with allowed deviation being: 0.001,
                      but value was outside the allowed deviation.

                      Actual: 0.3341
                    -------- assertr --------
                "});
        }
    }

    #[cfg(any(feature = "std", feature = "libm"))]
    mod is_nan {
        use crate::prelude::*;
        use ::num_traits::Float;
        use indoc::formatdoc;

        #[test]
        #[cfg(feature = "fluent")]
        fn fluent_alias_is_as_expected() {
            f32::nan().must().be_nan();
        }

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
                    Expression: `1.23`

                    Expected: NaN

                      Actual: 1.23
                    -------- assertr --------
                "});
        }
    }

    #[cfg(any(feature = "std", feature = "libm"))]
    mod is_finite {
        use crate::prelude::*;
        use indoc::formatdoc;
        use num_traits::Float;

        #[test]
        #[cfg(feature = "fluent")]
        fn fluent_alias_is_as_expected() {
            0.3_f32.must().be_finite();
        }

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
                    Expression: `f32::infinity()`

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
                    Expression: `f32::neg_infinity()`

                    Expected a finite value, but was

                      Actual: -inf
                    -------- assertr --------
                "});
        }
    }

    #[cfg(any(feature = "std", feature = "libm"))]
    mod is_infinite {
        use crate::prelude::*;
        use ::num_traits::Float;
        use indoc::formatdoc;

        #[test]
        #[cfg(feature = "fluent")]
        fn fluent_alias_is_as_expected() {
            f32::infinity().must().be_infinite();
        }

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
                    Expression: `1.23`

                    Expected: +/- inf

                      Actual: 1.23
                    -------- assertr --------
                "});
        }
    }

    #[cfg(any(feature = "std", feature = "libm"))]
    mod is_normal {
        use crate::prelude::*;
        use indoc::formatdoc;

        #[test]
        #[cfg(feature = "fluent")]
        fn fluent_alias_is_as_expected() {
            f32::MIN_POSITIVE.must().be_normal();
        }

        #[test]
        fn succeeds_when_normal() {
            assert_that!(f32::MIN_POSITIVE).is_normal();
        }

        #[test]
        fn panics_when_subnormal() {
            let subnormal = f32::from_bits(1);

            assert_that_panic_by(|| {
                assert_that!(subnormal).with_location(false).is_normal();
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {r"
                    -------- assertr --------
                    Expression: `subnormal`

                    Expected a normal floating-point value, but was

                      Actual: 1e-45
                    -------- assertr --------
                "});
        }
    }

    #[cfg(any(feature = "std", feature = "libm"))]
    mod is_subnormal {
        use crate::prelude::*;
        use indoc::formatdoc;

        #[test]
        #[cfg(feature = "fluent")]
        fn fluent_alias_is_as_expected() {
            f32::from_bits(1).must().be_subnormal();
        }

        #[test]
        fn succeeds_when_subnormal() {
            assert_that!(f32::from_bits(1)).is_subnormal();
        }

        #[test]
        fn panics_when_normal() {
            assert_that_panic_by(|| {
                assert_that!(1.0_f32).with_location(false).is_subnormal();
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {r"
                    -------- assertr --------
                    Expression: `1.0_f32`

                    Expected a subnormal floating-point value, but was

                      Actual: 1.0
                    -------- assertr --------
                "});
        }
    }
}
