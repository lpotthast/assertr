use num_traits::Num;

/// Numeric values whose distance can be calculated without integer overflow.
///
/// This capability enables [`super::NumAssertions::is_close_to`] for custom numeric types.
/// Implementations are provided for all primitive integers, `f32`, and `f64`, including with only
/// the `num` feature enabled. Other numeric assertions require only their own `num_traits` bounds.
///
/// Integer implementations subtract the smaller endpoint from the larger with checked arithmetic.
/// Floating-point implementations use the rounded result of `(self - other).abs()`, treating equal
/// infinities as zero distance. They do not rearrange the comparison into tolerance boundaries.
///
/// Generic helpers calling `is_close_to` must include this bound:
///
/// ```
/// # #[cfg(feature = "num")] {
/// use assertr::assertions::num::NumericDistance;
/// use assertr::prelude::*;
///
/// fn assert_close<T: NumericDistance + core::fmt::Debug>(actual: T, expected: T, deviation: T) {
///     assert_that!(actual).is_close_to(expected, deviation);
/// }
///
/// assert_close(0.25_f64, 0.5, 0.25);
/// # }
/// ```
pub trait NumericDistance: Num + PartialOrd {
    /// Returns the nonnegative distance between two values, without overflowing integer arithmetic.
    ///
    /// The result must be symmetric. Equal comparable values have zero distance, including equal
    /// infinities. Return `None` for incomparable values (including NaN) or when a nonnegative
    /// distance cannot be represented by this type. In particular, a signed integer distance larger
    /// than `Self::MAX` returns `None`, even if its negative could be represented.
    ///
    /// Types with infinity return positive infinity for unequal infinite endpoints or when finite
    /// subtraction overflows to infinity. Floating-point distances otherwise retain the type's
    /// subtraction rounding. Implementations must not return a negative or NaN distance.
    #[must_use]
    fn checked_distance(&self, other: &Self) -> Option<Self>;
}

macro_rules! integer_distance {
    ($($ty:ty),+ $(,)?) => {$(
        impl NumericDistance for $ty {
            fn checked_distance(&self, other: &Self) -> Option<Self> {
                if self <= other {
                    other.checked_sub(*self)
                } else {
                    self.checked_sub(*other)
                }
            }
        }
    )+};
}

integer_distance!(
    u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize
);

macro_rules! float_distance {
    ($($ty:ty),+ $(,)?) => {$(
        impl NumericDistance for $ty {
            #[allow(clippy::float_cmp)] // Equal infinities require exact equality.
            fn checked_distance(&self, other: &Self) -> Option<Self> {
                // Preserve equal infinities without evaluating their NaN difference.
                if self == other {
                    return Some(0.0);
                }
                let distance = (*self - *other).abs();
                (!distance.is_nan()).then_some(distance)
            }
        }
    )+};
}

float_distance!(f32, f64);

#[cfg(test)]
mod tests {
    use core::fmt::Debug;

    use super::NumericDistance;
    use crate::prelude::*;

    // Every matrix row checks both operand orders and the nonnegative, non-NaN result contract.
    // Expected distances are explicit fixtures, not recomputed using the implementation's formula.
    #[track_caller]
    fn assert_distance<T: NumericDistance + Copy + Debug>(a: T, b: T, expected: Option<T>) {
        for (a, b) in [(a, b), (b, a)] {
            let actual_distance = a.checked_distance(&b);
            assert_that!(actual_distance)
                .with_detail_message(format!("distance between {a:?} and {b:?}"))
                .is_equal_to(expected);
            if let Some(actual_distance) = actual_distance {
                assert_that!(actual_distance).is_greater_or_equal_to(T::zero());
            }
        }
    }

    macro_rules! unsigned_integer_tests {
        ($($ty:ident),+ $(,)?) => {$(
            mod $ty {
                use super::assert_distance;
                type Number = core::primitive::$ty;

                #[test]
                fn equal_values_have_zero_distance() {
                    for value in [0, 1, 17, Number::MAX / 2, Number::MAX - 1, Number::MAX] {
                        assert_distance(value, value, Some(0));
                    }
                }

                #[test]
                fn distinct_values_have_exact_distance() {
                    for (a, b, distance) in [
                        (0, 1, 1),
                        (17, 42, 25),
                        (Number::MAX - 1, Number::MAX, 1),
                        (Number::MAX - 42, Number::MAX - 17, 25),
                        (Number::MAX / 2, Number::MAX, Number::MAX / 2 + 1),
                    ] {
                        assert_distance(a, b, Some(distance));
                    }
                }

                #[test]
                fn full_range_distances_are_representable() {
                    for (a, b, distance) in [
                        (Number::MIN, Number::MAX, Number::MAX),
                        (1, Number::MAX, Number::MAX - 1),
                        (0, Number::MAX - 1, Number::MAX - 1),
                    ] {
                        assert_distance(a, b, Some(distance));
                    }
                }
            }
        )+};
    }

    macro_rules! signed_integer_tests {
        ($($ty:ident),+ $(,)?) => {$(
            mod $ty {
                use super::assert_distance;
                type Number = core::primitive::$ty;

                #[test]
                fn equal_values_have_zero_distance() {
                    for value in [Number::MIN, Number::MIN + 1, -1, 0, 1, Number::MAX - 1, Number::MAX] {
                        assert_distance(value, value, Some(0));
                    }
                }

                #[test]
                fn same_sign_values_have_exact_distance() {
                    for (a, b, distance) in [
                        (Number::MIN, Number::MIN + 1, 1),
                        (Number::MIN, -1, Number::MAX),
                        (-Number::MAX, -1, Number::MAX - 1),
                        (-42, -17, 25),
                        (17, 42, 25),
                        (0, Number::MAX, Number::MAX),
                        (Number::MAX - 1, Number::MAX, 1),
                    ] {
                        assert_distance(a, b, Some(distance));
                    }
                }

                #[test]
                fn crossing_zero_can_have_a_representable_distance() {
                    for (a, b, distance) in [
                        (-1, 1, 2),
                        (-42, 17, 59),
                        (Number::MIN + 1, 0, Number::MAX),
                        (Number::MIN + 2, 1, Number::MAX),
                        (-1, Number::MAX - 1, Number::MAX),
                        (-Number::MAX / 2, Number::MAX / 2, Number::MAX - 1),
                    ] {
                        assert_distance(a, b, Some(distance));
                    }
                }

                #[test]
                fn max_plus_one_distance_is_unrepresentable_even_when_its_negative_fits() {
                    for (a, b) in [(Number::MIN, 0), (Number::MIN + 1, 1), (-1, Number::MAX)] {
                        assert_distance(a, b, None);
                    }
                }

                #[test]
                fn larger_distances_are_unrepresentable() {
                    for (a, b) in [
                        (Number::MIN, 1),
                        (Number::MIN, Number::MAX),
                        (-Number::MAX, Number::MAX),
                        (-2, Number::MAX),
                    ] {
                        assert_distance(a, b, None);
                    }
                }
            }
        )+};
    }

    macro_rules! float_tests {
        ($ty:ident, $precision_boundary:expr) => {
            mod $ty {
                use super::{NumericDistance, assert_distance};
                use crate::prelude::*;
                type Number = core::primitive::$ty;

                #[test]
                fn equal_finite_values_have_zero_distance() {
                    let smallest = Number::from_bits(1);
                    for value in [
                        Number::MIN,
                        -42.5,
                        -Number::MIN_POSITIVE,
                        -smallest,
                        smallest,
                        Number::MIN_POSITIVE,
                        42.5,
                        Number::MAX,
                    ] {
                        assert_distance(value, value, Some(0.0));
                    }
                }

                #[test]
                fn signed_zeros_have_positive_zero_distance() {
                    for a in [0.0, -0.0] {
                        for b in [0.0, -0.0] {
                            assert_distance(a, b, Some(0.0));
                            assert_that!(Number::checked_distance(&a, &b).map(Number::to_bits))
                                .is_equal_to(Some(0));
                        }
                    }
                }

                #[test]
                fn finite_distances_can_be_exact_across_signs_and_at_extrema() {
                    for (a, b, distance) in [
                        (-3.5, -1.25, 2.25),
                        (1.25, 3.5, 2.25),
                        (-1.25, 3.5, 4.75),
                        (0.0, Number::MIN, Number::MAX),
                        (0.0, Number::MAX, Number::MAX),
                        (-Number::MAX / 2.0, Number::MAX / 2.0, Number::MAX),
                    ] {
                        assert_distance(a, b, Some(distance));
                    }
                }

                #[test]
                fn subnormal_distances_are_not_flushed_to_zero() {
                    let smallest = Number::from_bits(1);
                    let largest = Number::from_bits(Number::MIN_POSITIVE.to_bits() - 1);
                    for (a, b, distance) in [
                        (0.0, smallest, smallest),
                        (0.0, -smallest, smallest),
                        (-smallest, smallest, Number::from_bits(2)),
                        (smallest, Number::from_bits(3), Number::from_bits(2)),
                        (largest, Number::MIN_POSITIVE, smallest),
                        (-Number::MIN_POSITIVE, -largest, smallest),
                    ] {
                        assert_distance(a, b, Some(distance));
                    }
                }

                #[test]
                fn adjacent_normal_values_retain_their_spacing() {
                    let one: Number = 1.0;
                    let below = Number::from_bits(one.to_bits() - 1);
                    let above = Number::from_bits(one.to_bits() + 1);
                    assert_distance(below, one, Some(Number::EPSILON / 2.0));
                    assert_distance(one, above, Some(Number::EPSILON));
                    assert_distance(-above, -one, Some(Number::EPSILON));
                }

                #[test]
                fn distances_round_to_nearest_with_ties_to_even() {
                    let boundary: Number = $precision_boundary;
                    for (a, b, distance) in [
                        (boundary, boundary + 2.0, 2.0),
                        (-boundary, -boundary - 2.0, 2.0),
                        (boundary, 1.0, boundary - 1.0),
                        (boundary + 2.0, 1.0, boundary),
                        (-1.0, boundary, boundary),
                        (-boundary, 1.0, boundary),
                        (-3.0, boundary, boundary + 4.0),
                        (-1.0, boundary + 2.0, boundary + 4.0),
                    ] {
                        assert_distance(a, b, Some(distance));
                    }
                }

                #[test]
                fn finite_subtraction_overflow_returns_positive_infinity() {
                    for (a, b) in [
                        (Number::MIN, Number::MAX),
                        (Number::MIN, Number::MAX / 2.0),
                        (Number::MIN / 2.0, Number::MAX),
                    ] {
                        assert_distance(a, b, Some(Number::INFINITY));
                    }
                }

                #[test]
                fn rounding_at_the_overflow_threshold_distinguishes_max_from_infinity() {
                    let previous = Number::from_bits(Number::MAX.to_bits() - 1);
                    // The subtraction is exact and gives the spacing at the largest finite value.
                    let half_ulp = (Number::MAX - previous) / 2.0;
                    let below_half_ulp = Number::from_bits(half_ulp.to_bits() - 1);
                    assert_distance(Number::MAX, -below_half_ulp, Some(Number::MAX));
                    assert_distance(Number::MAX, -half_ulp, Some(Number::INFINITY));
                    assert_distance(Number::MIN, below_half_ulp, Some(Number::MAX));
                    assert_distance(Number::MIN, half_ulp, Some(Number::INFINITY));
                }

                #[test]
                fn equal_infinities_have_zero_distance() {
                    for infinity in [Number::NEG_INFINITY, Number::INFINITY] {
                        assert_distance(infinity, infinity, Some(0.0));
                    }
                }

                #[test]
                fn unequal_infinite_endpoints_have_positive_infinite_distance() {
                    for infinity in [Number::NEG_INFINITY, Number::INFINITY] {
                        for finite in [
                            Number::MIN,
                            -1.0,
                            -Number::from_bits(1),
                            -0.0,
                            0.0,
                            Number::from_bits(1),
                            1.0,
                            Number::MAX,
                        ] {
                            assert_distance(infinity, finite, Some(Number::INFINITY));
                        }
                    }
                    assert_distance(
                        Number::NEG_INFINITY,
                        Number::INFINITY,
                        Some(Number::INFINITY),
                    );
                }

                #[test]
                fn any_nan_endpoint_has_no_distance() {
                    let signaling = Number::from_bits(Number::INFINITY.to_bits() | 1);
                    let payload = Number::from_bits(Number::NAN.to_bits() | 0x123);
                    let nans = [
                        Number::NAN,
                        -Number::NAN,
                        signaling,
                        -signaling,
                        payload,
                        -payload,
                    ];
                    for nan in nans {
                        for other in [
                            Number::NEG_INFINITY,
                            Number::MIN,
                            -1.0,
                            -Number::from_bits(1),
                            -0.0,
                            0.0,
                            Number::from_bits(1),
                            1.0,
                            Number::MAX,
                            Number::INFINITY,
                        ]
                        .into_iter()
                        .chain(nans)
                        {
                            assert_distance(nan, other, None);
                        }
                    }
                }
            }
        };
    }

    unsigned_integer_tests!(u8, u16, u32, u64, u128, usize);
    signed_integer_tests!(i8, i16, i32, i64, i128, isize);
    float_tests!(f32, 16_777_216.0);
    float_tests!(f64, 9_007_199_254_740_992.0);
}
