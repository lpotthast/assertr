//! Assertions for numeric identities, signs, tolerances, and floating-point classifications.

#[cfg(any(feature = "std", feature = "libm"))]
use num_traits::Float;
use num_traits::{Num, Signed};

pub use super::distance::{IsCloseTo, NumericDistance};
use crate::{
    AssertThat, Mode,
    borrow_for::BorrowFor,
    renderer::{DebugRenderer, ValueRenderer},
};

property_expectation! {
    /// Checks [`Signed::is_negative`], including the sign bit of floating-point values.
    pub struct IsNegative for<T: Signed> T;
    kind Ordering;
    check |actual| actual.is_negative();
    relations "is negative", "is not negative";
}

property_expectation! {
    /// Checks [`Signed::is_positive`], including the sign bit of floating-point values.
    pub struct IsPositive for<T: Signed> T;
    kind Ordering;
    check |actual| actual.is_positive();
    relations "is positive", "is not positive";
}

#[cfg(any(feature = "std", feature = "libm"))]
property_expectation! {
    /// Checks whether a numeric value is finite.
    pub struct IsFinite for<T: Float> T;
    kind Predicate;
    check |actual| actual.is_finite();
    relations "is finite", "is not finite";
}

#[cfg(any(feature = "std", feature = "libm"))]
property_expectation! {
    /// Checks whether a numeric value is infinite.
    pub struct IsInfinite for<T: Float> T;
    kind Predicate;
    check |actual| actual.is_infinite();
    relations "is infinite", "is not infinite";
}

#[cfg(any(feature = "std", feature = "libm"))]
property_expectation! {
    /// Checks whether a numeric value is normal.
    pub struct IsNormal for<T: Float> T;
    kind Predicate;
    check |actual| actual.is_normal();
    relations "is normal", "is not normal";
}

#[cfg(any(feature = "std", feature = "libm"))]
property_expectation! {
    /// Checks whether a numeric value is subnormal.
    pub struct IsSubnormal for<T: Float> T;
    kind Predicate;
    check |actual| actual.is_subnormal();
    relations "is subnormal", "is not subnormal";
}

property_expectation! {
    /// Checks whether a numeric value is zero.
    pub struct IsZero for<T: Num> T;
    kind Equality;
    check |actual| actual.is_zero();
    expected T::zero(), "is zero";
}

property_expectation! {
    /// Checks whether a numeric value is one.
    pub struct IsOne for<T: Num> T;
    kind Equality;
    check |actual| actual.is_one();
    expected T::one(), "is one";
}

#[cfg(any(feature = "std", feature = "libm"))]
property_expectation! {
    /// Checks whether a numeric value is NaN.
    pub struct IsNan for<T: Float> T;
    kind Predicate;
    check |actual| actual.is_nan();
    relations "is NaN", "is not NaN";
}

/// Assertions for numeric values not already handled by [`crate::prelude::PartialEqAssertions`] and
/// [`crate::prelude::PartialOrdAssertions`].
#[allow(clippy::return_self_not_must_use)]
#[cfg_attr(feature = "fluent", assertr_macros::fluent_aliases)]
pub trait NumAssertions<T: Num, R = DebugRenderer> {
    /// Asserts that the subject equals the additive identity, zero.
    fn is_zero(self) -> Self
    where
        R: ValueRenderer<T>;

    /// Alias of [`NumAssertions::is_zero`].
    fn is_additive_identity(self) -> Self
    where
        R: ValueRenderer<T>;

    /// Asserts that the subject equals the multiplicative identity, one.
    fn is_one(self) -> Self
    where
        R: ValueRenderer<T>;

    /// Alias of [`NumAssertions::is_one`].
    fn is_multiplicative_identity(self) -> Self
    where
        R: ValueRenderer<T>;

    /// Asserts that [`Signed::is_negative`] returns true for the subject.
    ///
    /// For floating-point values this tests the sign bit, so `-0.0` and a negative-sign NaN are
    /// considered negative.
    fn is_negative(self) -> Self
    where
        T: Signed,
        R: ValueRenderer<T>;

    /// Asserts that [`Signed::is_positive`] returns true for the subject.
    ///
    /// For floating-point values this tests the sign bit, so `0.0` and a positive-sign NaN are
    /// considered positive.
    fn is_positive(self) -> Self
    where
        T: Signed,
        R: ValueRenderer<T>;

    /// Asserts that the subject is within `allowed_deviation` of `expected`.
    ///
    /// Compares the distance from [`NumericDistance::checked_distance`] to `allowed_deviation`,
    /// inclusively. Integer distances use checked arithmetic. Floating-point distances use the
    /// rounded result of `(actual - expected).abs()`. For example, `0.334_f64` is outside a
    /// deviation of `0.001` from `0.333_f64`, because their floating-point distance is slightly
    /// greater.
    ///
    /// A negative or NaN deviation fails the assertion, as do incomparable (including NaN) values.
    /// Equal infinities have zero distance. Positive-infinite deviation accepts every comparable
    /// non-NaN value, including when finite subtraction overflows to infinity.
    ///
    /// Custom numeric types must implement [`NumericDistance`]. Neither `Clone` nor floating-point
    /// math features (`std` or `libm`) are required. A type from another crate cannot implement
    /// it. Assert on a supported projection, use a predicate, or wrap it in a local newtype, as
    /// shown in [foreign numeric types](NumericDistance#foreign-numeric-types).
    fn is_close_to<E: BorrowFor<T, View = T>, D: BorrowFor<T, View = T>>(
        self,
        expected: E,
        allowed_deviation: D,
    ) -> Self
    where
        T: NumericDistance,
        R: ValueRenderer<T>;

    /// Asserts that the subject is NaN.
    #[cfg(any(feature = "std", feature = "libm"))]
    fn is_nan(self) -> Self
    where
        T: Float,
        R: ValueRenderer<T>;

    /// Asserts that the subject is finite.
    #[cfg(any(feature = "std", feature = "libm"))]
    fn is_finite(self) -> Self
    where
        T: Float,
        R: ValueRenderer<T>;

    /// Asserts that the subject is positive or negative infinity.
    #[cfg(any(feature = "std", feature = "libm"))]
    fn is_infinite(self) -> Self
    where
        T: Float,
        R: ValueRenderer<T>;

    /// Asserts that the subject is a normal floating-point value.
    #[cfg(any(feature = "std", feature = "libm"))]
    fn is_normal(self) -> Self
    where
        T: Float,
        R: ValueRenderer<T>;

    /// Asserts that the subject is a subnormal floating-point value.
    #[cfg(any(feature = "std", feature = "libm"))]
    fn is_subnormal(self) -> Self
    where
        T: Float,
        R: ValueRenderer<T>;
}

impl<T: Num, M: Mode, R> NumAssertions<T, R> for AssertThat<'_, T, M, R> {
    #[track_caller]
    fn is_zero(self) -> Self
    where
        R: ValueRenderer<T>,
    {
        self.matches(IsZero)
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
        self.matches(IsOne)
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
        self.matches(IsNegative)
    }

    #[track_caller]
    fn is_positive(self) -> Self
    where
        T: Signed,
        R: ValueRenderer<T>,
    {
        self.matches(IsPositive)
    }

    #[track_caller]
    fn is_close_to<E: BorrowFor<T, View = T>, D: BorrowFor<T, View = T>>(
        self,
        expected: E,
        allowed_deviation: D,
    ) -> Self
    where
        T: NumericDistance,
        R: ValueRenderer<T>,
    {
        self.matches(IsCloseTo::new(expected, allowed_deviation))
    }

    #[track_caller]
    #[cfg(any(feature = "std", feature = "libm"))]
    fn is_nan(self) -> Self
    where
        T: Float,
        R: ValueRenderer<T>,
    {
        self.matches(IsNan)
    }

    #[track_caller]
    #[cfg(any(feature = "std", feature = "libm"))]
    fn is_finite(self) -> Self
    where
        T: Float,
        R: ValueRenderer<T>,
    {
        self.matches(IsFinite)
    }

    #[track_caller]
    #[cfg(any(feature = "std", feature = "libm"))]
    fn is_infinite(self) -> Self
    where
        T: Float,
        R: ValueRenderer<T>,
    {
        self.matches(IsInfinite)
    }

    #[track_caller]
    #[cfg(any(feature = "std", feature = "libm"))]
    fn is_normal(self) -> Self
    where
        T: Float,
        R: ValueRenderer<T>,
    {
        self.matches(IsNormal)
    }

    #[track_caller]
    #[cfg(any(feature = "std", feature = "libm"))]
    fn is_subnormal(self) -> Self
    where
        T: Float,
        R: ValueRenderer<T>,
    {
        self.matches(IsSubnormal)
    }
}

#[cfg(test)]
mod tests {
    #[cfg(feature = "fluent")]
    mod fluent_aliases {
        use crate::prelude::*;

        #[test]
        fn are_as_expected() {
            0_i32.must().be_zero().be_additive_identity();
            1_i32.must().be_one().be_multiplicative_identity();
            (-0.01_f64).must().be_negative();
            0.01_f64.must().be_positive();
            0.333_f64.must().be_close_to(0.333, 0.001);
        }

        #[test]
        #[cfg(any(feature = "std", feature = "libm"))]
        fn float_classifications_are_as_expected() {
            f32::NAN.must().be_nan();
            0.3_f32.must().be_finite().be_normal();
            f32::INFINITY.must().be_infinite();
            f32::from_bits(1).must().be_subnormal();
        }
    }

    mod renderer_contract {
        use crate::{
            prelude::*,
            test_support::{NoRenderer, assert_trait_impl},
        };

        #[test]
        fn trait_is_implemented_without_renderer_support() {
            assert_trait_impl!(
                AssertThat<'static, i32, Panic, NoRenderer> => NumAssertions<i32, NoRenderer>
            );
            assert_trait_impl!(
                AssertThat<'static, f64, Panic, NoRenderer> => NumAssertions<f64, NoRenderer>
            );
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

        use ::num_traits::Float;

        use crate::prelude::*;

        fn assert_classification<T: Float + Debug>(nan: T, finite: T, infinite: T) {
            assert_that!(nan).is_nan();
            assert_that!(finite).is_finite();
            assert_that!(infinite).is_infinite();
        }

        assert_classification(f32::nan(), 1.0f32, f32::infinity());
        assert_classification(f64::nan(), 1.0f64, f64::infinity());
    }

    mod is_zero {
        use indoc::formatdoc;

        use crate::prelude::*;

        #[test]
        fn caller_location_is_as_expected() {
            assert_caller_location!(assert_that!(3), is_zero());
        }

        #[test]
        fn succeeds_when_zero() {
            assert_that!(0).is_zero();
        }

        #[test]
        fn panics_when_not_zero() {
            assert_that!(|| assert_that!(3).with_location(false).is_zero())
                .panics()
                .has_type::<String>()
                .is_equal_to(formatdoc! {r"
                    -------- assertr --------
                    Expression: `3`

                    Expected: 0

                      Actual: 3
                    -------- assertr --------
                "});
        }
    }

    /// Synonym of `is_zero`. The caller location is pinned here. The behavior is covered by that
    /// module.
    mod is_additive_identity {
        use crate::prelude::*;

        #[test]
        fn caller_location_is_as_expected() {
            assert_caller_location!(assert_that!(3), is_additive_identity());
        }
    }

    mod is_one {
        use crate::prelude::*;

        #[test]
        fn caller_location_is_as_expected() {
            assert_caller_location!(assert_that!(3), is_one());
        }

        #[test]
        fn accepts_only_one() {
            assert_that!(1).is_one();
            let failures = assert_that!(3).capture(NumAssertions::is_one);
            assert_that!(failures).has_length(1);
        }
    }

    /// Synonym of `is_one`. The caller location is pinned here. The behavior is covered by that
    /// module.
    mod is_multiplicative_identity {
        use crate::prelude::*;

        #[test]
        fn caller_location_is_as_expected() {
            assert_caller_location!(assert_that!(3), is_multiplicative_identity());
        }
    }

    mod is_negative {
        use indoc::formatdoc;

        use crate::prelude::*;

        #[test]
        fn caller_location_is_as_expected() {
            assert_caller_location!(assert_that!(0.0), is_negative());
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
        fn panics_when_zero_or_positive() {
            let failures = assert_that!(1.23).capture(NumAssertions::is_negative);
            assert_that!(failures).has_length(1);
            assert_that!(|| assert_that!(0.0).with_location(false).is_negative())
                .panics()
                .has_type::<String>()
                .is_equal_to(formatdoc! {r"
                    -------- assertr --------
                    Expression: `0.0`

                    Actual: 0.0

                    is not negative
                    -------- assertr --------
                "});
        }
    }

    mod is_positive {
        use crate::prelude::*;

        #[test]
        fn caller_location_is_as_expected() {
            assert_caller_location!(assert_that!(-1.23), is_positive());
        }

        #[test]
        fn uses_the_float_sign_bit() {
            assert_that!(0.01).is_positive();
            assert_that!(0.0).is_positive();
            assert_that!(f64::NAN).is_positive();
            let failures = assert_that!(-0.0).capture(NumAssertions::is_positive);
            assert_that!(failures).has_length(1);
        }
    }

    mod is_close_to {
        // The NumericDistance tests own the primitive arithmetic and special-value matrices.
        // These tests cover tolerance handling, diagnostics, and integration.
        use indoc::formatdoc;

        use crate::prelude::*;

        #[test]
        fn caller_location_is_as_expected() {
            assert_caller_location!(assert_that!(1_i32), is_close_to(3, 1));
        }

        #[test]
        fn description_labels_the_allowed_deviation() {
            use super::super::IsCloseTo;
            use crate::failure::Fact;

            let description =
                AssertionContext::default().describe::<i32, _>(&IsCloseTo::new(42, 2));
            assert_that!(description.relation.as_deref()).is_equal_to(Some("is close to"));
            assert_that!(description.children).is_empty();
            assert_that!(description.facts).contains_exactly([Fact::labelled(
                "Allowed deviation",
                AssertionContext::default().render().value(&2),
            )]);
        }

        #[test]
        fn succeeds_when_actual_is_in_allowed_range() {
            assert_that!(0.25).is_close_to(0.5, 0.25);
            assert_that!(0.5).is_close_to(0.5, 0.25);
            assert_that!(0.75).is_close_to(0.5, 0.25);
            assert_that!(0_u8).is_close_to(0, 1);
        }

        #[test]
        fn panics_when_outside_allowed_range() {
            assert_that!(|| {
                assert_that!(0.3319)
                    .with_location(false)
                    .is_close_to(0.333, 0.001)
            })
            .panics()
            .has_type::<String>()
            .is_equal_to(formatdoc! {r"
                    -------- assertr --------
                    Expression: `0.3319`

                    Actual: 0.3319

                    is not close to

                    Expected: 0.333

                    Details:
                      - Allowed deviation: 0.001
                    -------- assertr --------
                "});
        }

        #[test]
        fn rejects_distances_just_above_deviation() {
            for (actual, expected, deviation) in [
                (0.3341, 0.333, 0.001),
                (0.334_f64, 0.333_f64, 0.001),
                (0.333, 0.334, 0.001),
                (9_007_199_254_740_992_f64, 9_007_199_254_740_994_f64, 1.0),
            ] {
                let failures =
                    assert_that!(actual).capture(|it| it.is_close_to(expected, deviation));
                assert_that!(failures).has_length(1);
            }
        }

        #[test]
        fn handles_infinities_and_nan() {
            assert_that!(f64::NEG_INFINITY).is_close_to(f64::NEG_INFINITY, 0.0);
            // Positive-infinite deviation is unbounded for comparable values only.
            assert_that!(-f64::MAX).is_close_to(f64::MAX, f64::INFINITY);
            assert_that!(f64::NEG_INFINITY).is_close_to(f64::INFINITY, f64::INFINITY);
            let failures = assert_that!(f64::NAN).capture(|it| {
                it.is_close_to(1.0, f64::INFINITY)
                    .is_close_to(f64::NAN, f64::INFINITY)
            });
            assert_that!(failures).has_length(2);
        }

        #[test]
        fn reports_extreme_signed_values_without_overflowing() {
            let failures = assert_that!(i8::MIN).capture(|it| it.is_close_to(i8::MAX, i8::MAX));
            assert_that!(failures).has_length(1);
        }

        #[test]
        fn rejects_negative_or_nan_deviation() {
            let failures = assert_that!(1.0).capture(|it| it.is_close_to(1.0, f64::NAN));
            assert_that!(failures[0].relation.as_deref())
                .is_equal_to(Some("was given an invalid allowed deviation"));
            assert_that!(|| {
                assert_that!(1_i8).with_location(false).is_close_to(1, -1);
            })
            .panics()
            .has_type::<String>()
            .is_equal_to(formatdoc! {r"
                -------- assertr --------
                Expression: `1_i8`

                was given an invalid allowed deviation

                Details:
                  - Allowed deviation: -1
                  - The allowed deviation must be zero or positive.
                -------- assertr --------
            "});
        }
    }

    #[cfg(any(feature = "std", feature = "libm"))]
    mod is_nan {
        use ::num_traits::Float;
        use indoc::formatdoc;

        use crate::prelude::*;

        #[test]
        fn caller_location_is_as_expected() {
            assert_caller_location!(assert_that!(1.23), is_nan());
        }

        #[test]
        fn succeeds_when_nan() {
            assert_that!(f32::nan()).is_nan();
        }

        #[test]
        fn panics_when_not_nan() {
            assert_that!(|| assert_that!(1.23).with_location(false).is_nan())
                .panics()
                .has_type::<String>()
                .is_equal_to(formatdoc! {r"
                    -------- assertr --------
                    Expression: `1.23`

                    Actual: 1.23

                    is not NaN
                    -------- assertr --------
                "});
        }
    }

    #[cfg(any(feature = "std", feature = "libm"))]
    mod is_finite {
        use indoc::formatdoc;

        use crate::prelude::*;

        #[test]
        fn caller_location_is_as_expected() {
            assert_caller_location!(assert_that!(f32::INFINITY), is_finite());
        }

        #[test]
        fn rejects_both_infinities() {
            assert_that!(0.3f32).is_finite();
            let failures = assert_that!(f32::NEG_INFINITY).capture(NumAssertions::is_finite);
            assert_that!(failures).has_length(1);
            assert_that!(|| {
                assert_that!(f32::INFINITY).with_location(false).is_finite();
            })
            .panics()
            .has_type::<String>()
            .is_equal_to(formatdoc! {r"
                    -------- assertr --------
                    Expression: `f32::INFINITY`

                    Actual: inf

                    is not finite
                    -------- assertr --------
                "});
        }
    }

    #[cfg(any(feature = "std", feature = "libm"))]
    mod is_infinite {
        use crate::prelude::*;

        #[test]
        fn caller_location_is_as_expected() {
            assert_caller_location!(assert_that!(1.23), is_infinite());
        }

        #[test]
        fn accepts_both_infinities() {
            assert_that!(f32::INFINITY).is_infinite();
            assert_that!(f32::NEG_INFINITY).is_infinite();
            let failures = assert_that!(1.23).capture(NumAssertions::is_infinite);
            assert_that!(failures).has_length(1);
        }
    }

    #[cfg(any(feature = "std", feature = "libm"))]
    mod is_normal {
        use crate::prelude::*;

        #[test]
        fn caller_location_is_as_expected() {
            let subnormal = f32::from_bits(1);
            assert_caller_location!(assert_that!(subnormal), is_normal());
        }

        #[test]
        fn rejects_subnormal_values() {
            assert_that!(f32::MIN_POSITIVE).is_normal();
            let failures = assert_that!(f32::from_bits(1)).capture(NumAssertions::is_normal);
            assert_that!(failures).has_length(1);
        }
    }

    #[cfg(any(feature = "std", feature = "libm"))]
    mod is_subnormal {
        use crate::prelude::*;

        #[test]
        fn caller_location_is_as_expected() {
            assert_caller_location!(assert_that!(1.0_f32), is_subnormal());
        }

        #[test]
        fn rejects_normal_values() {
            assert_that!(f32::from_bits(1)).is_subnormal();
            let failures = assert_that!(1.0_f32).capture(NumAssertions::is_subnormal);
            assert_that!(failures).has_length(1);
        }
    }

    mod borrowed_tolerances {
        use core::cell::Cell;

        use super::super::IsCloseTo;
        use crate::{prelude::*, test_support::BorrowSpy};

        #[test]
        fn expected_and_deviation_can_be_borrowed_independently() {
            let expected = 10;
            let deviation = 2;
            assert_that!(11)
                .is_close_to(expected, deviation)
                .is_close_to(&expected, deviation)
                .is_close_to(expected, &deviation)
                .is_close_to(&expected, &deviation);
            let matcher = IsCloseTo::new(&expected, &deviation);
            assert_that!(11).matches(&matcher);
            assert_that!(9).matches(&matcher);
        }

        #[test]
        fn borrows_both_operands_once_even_for_invalid_deviation() {
            for deviation in [-1, 1, 10] {
                let calls = Cell::new(0);
                let operand = |value| BorrowSpy {
                    value,
                    observe: || calls.set(calls.get() + 1),
                };
                let failures =
                    assert_that!(5).capture(|it| it.is_close_to(operand(10), operand(deviation)));
                assert_that!(calls.get()).is_equal_to(2);
                assert_that!(failures).has_length(usize::from(deviation != 10));
            }
        }
    }
}
