//! Tolerance checks for values with a computable distance, such as numbers and durations.

use core::num::Wrapping;

use crate::{
    borrow_for::{BorrowFor, borrow_for},
    expectation::{AssertionContext, Expectation},
    failure::{Fact, FailureBuilder, FailureKind},
    renderer::ValueRenderer,
};

/// Numeric values whose distance can be calculated without integer overflow.
///
/// This capability enables [`IsCloseTo`] and the `is_close_to` assertions for numbers and, with
/// the `jiff` feature, `SignedDuration`. Implementations are provided for all primitive integers,
/// `f32`, `f64`, [`Wrapping`] of a primitive integer, and `jiff::SignedDuration`. They need no
/// optional feature apart from the integration they cover.
///
/// Integer implementations subtract the smaller endpoint from the larger with checked arithmetic.
/// `Wrapping` integers use the distance of their inner values and never wrap it. Floating-point
/// implementations use the rounded result of `(self - other).abs()`, treating equal infinities as
/// zero distance. They do not rearrange the comparison into tolerance boundaries.
///
/// Generic helpers calling the numeric `is_close_to` need this bound next to `num_traits::Num`:
///
/// ```
/// # #[cfg(feature = "num")] {
/// use assertr::assertions::NumericDistance;
/// use assertr::prelude::*;
///
/// fn assert_close<T>(actual: T, expected: T, deviation: T)
/// where
///     T: num_traits::Num + NumericDistance + core::fmt::Debug,
/// {
///     assert_that!(actual).is_close_to(expected, deviation);
/// }
///
/// assert_close(0.25_f64, 0.5, 0.25);
/// # }
/// ```
///
/// # Foreign numeric types
///
/// The orphan rule prevents implementing this trait for a type from another crate, such as a
/// big integer or decimal type. For those, either assert on a projection that already supports
/// `is_close_to`, express the tolerance with a predicate, or wrap the value in a local newtype
/// implementing `PartialOrd` and `NumericDistance`. For example,
/// `core::num::Saturating` has no implementation:
///
/// ```
/// # #[cfg(feature = "num")] {
/// use assertr::{matchers::predicate, prelude::*};
/// use core::num::Saturating;
///
/// let actual = Saturating(5_u8);
/// assert_that!(actual).satisfies_owned(|it| it.0, |inner| {
///     inner.is_close_to(4, 1);
/// });
/// assert_that!(actual).matches(
///     predicate(|it: &Saturating<u8>| it.0.abs_diff(4) <= 1).described_as("is within 1 of 4"),
/// );
/// # }
/// ```
pub trait NumericDistance: PartialOrd + Sized {
    /// Returns the zero distance, which a valid allowed deviation must not be less than.
    #[must_use]
    fn zero_distance() -> Self;

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
            fn zero_distance() -> Self {
                0
            }

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
            fn zero_distance() -> Self {
                0.0
            }

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

impl<T: NumericDistance> NumericDistance for Wrapping<T> {
    fn zero_distance() -> Self {
        Wrapping(T::zero_distance())
    }

    fn checked_distance(&self, other: &Self) -> Option<Self> {
        self.0.checked_distance(&other.0).map(Wrapping)
    }
}

/// Checks that the subject lies within an inclusive, non-negative deviation of an expected value.
///
/// Build it with [`close_to`] or [`IsCloseTo::new`]. It uses [`NumericDistance`] without
/// requiring `Clone` or floating-point math features. A negative or NaN deviation rejects every
/// subject with its own report. Values render compactly, for example `10s` for a
/// `jiff::SignedDuration`.
#[derive(Debug, Clone)]
pub struct IsCloseTo<E, D = E> {
    expected: E,
    allowed_deviation: D,
}

impl<E, D> IsCloseTo<E, D> {
    /// Owns the expected value and allowed deviation. [`close_to`] is a shorter spelling.
    #[must_use]
    pub const fn new(expected: E, allowed_deviation: D) -> Self {
        Self {
            expected,
            allowed_deviation,
        }
    }
}

/// Matches a subject within `allowed_deviation` of `expected`, inclusively.
///
/// Both operands may be owned or borrowed. They are compared as the subject's type, which must
/// implement [`NumericDistance`]:
///
/// ```
/// use assertr::{matchers::{close_to, each}, prelude::*};
///
/// assert_that!(0.3_f64).matches(close_to(0.1 + 0.2, 1e-9));
/// assert_that!([9, 10, 11]).matches(each(close_to(10, 1)));
///
/// let failures = assert_that!(13).with_location(false).capture(|it| it.matches(close_to(10, 2)));
/// assert_that!(failures[0].to_string()).contains("is not close to");
/// ```
#[must_use]
pub const fn close_to<E, D>(expected: E, allowed_deviation: D) -> IsCloseTo<E, D> {
    IsCloseTo::new(expected, allowed_deviation)
}

/// Why [`IsCloseTo`] rejected a subject.
///
/// The expectation's rejection is `(&T, &T, CloseToRejection)`: the borrowed expected value, the
/// borrowed allowed deviation, and this reason. Explanation uses the reason to choose between
/// reporting an invalid deviation and reporting the distance, so it never has to compare again.
/// Match on it to tell a misconfigured tolerance from a value that is too far away.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum CloseToRejection {
    /// The deviation was negative or incomparable with zero.
    InvalidDeviation,
    /// The distance exceeded the deviation or could not be computed.
    OutsideDeviation,
}

impl<T: NumericDistance, E: BorrowFor<T, View = T>, D: BorrowFor<T, View = T>, R: ValueRenderer<T>>
    Expectation<T, R> for IsCloseTo<E, D>
{
    type Success<'a>
        = ()
    where
        Self: 'a,
        T: 'a;
    type Rejection<'a>
        = (&'a T, &'a T, CloseToRejection)
    where
        Self: 'a,
        T: 'a;
    fn evaluate<'a>(
        &'a self,
        actual: &'a T,
        _: &AssertionContext<'_, R>,
    ) -> Result<(), Self::Rejection<'a>> {
        let expected = borrow_for::<T, _>(&self.expected);
        let allowed_deviation = borrow_for::<T, _>(&self.allowed_deviation);
        let reject = |reason| Err((expected, allowed_deviation, reason));
        // A NaN deviation is incomparable with zero and therefore invalid, like a negative one.
        if !allowed_deviation.ge(&T::zero_distance()) {
            return reject(CloseToRejection::InvalidDeviation);
        }
        if actual
            .checked_distance(expected)
            .is_some_and(|distance| &distance <= allowed_deviation)
        {
            Ok(())
        } else {
            reject(CloseToRejection::OutsideDeviation)
        }
    }

    const KIND: FailureKind = FailureKind::Ordering;
    fn explain<'a>(
        &'a self,
        rejected: Option<(&'a T, Self::Rejection<'a>)>,
        failure: FailureBuilder,
        context: &AssertionContext<'_, R>,
    ) -> FailureBuilder {
        // Compact leaves keep values such as durations readable. Scalars render the same either
        // way.
        let render = context.render().compact();
        let allowed = |deviation: &T| Fact::labelled("Allowed deviation", render.value(deviation));
        let (actual, expected, allowed_deviation) = match rejected {
            None => (
                None,
                borrow_for::<T, _>(&self.expected),
                borrow_for::<T, _>(&self.allowed_deviation),
            ),
            Some((_, (_, allowed_deviation, CloseToRejection::InvalidDeviation))) => {
                return failure
                    .relation("was given an invalid allowed deviation")
                    .fact(allowed(allowed_deviation))
                    .fact(Fact::note(
                        "The allowed deviation must be zero or positive.",
                    ));
            }
            Some((actual, (expected, allowed_deviation, CloseToRejection::OutsideDeviation))) => {
                (Some(actual), expected, allowed_deviation)
            }
        };
        failure
            .relations(
                actual.map(|actual| render.value(actual)),
                "is close to",
                "is not close to",
            )
            .expected(render.value(expected))
            .fact(allowed(allowed_deviation))
    }
}

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
                assert_that!(actual_distance).is_greater_or_equal_to(T::zero_distance());
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
        ($ty:ident) => {
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
                    for other in [Number::NEG_INFINITY, -1.0, 0.0, Number::MAX, Number::NAN] {
                        assert_distance(Number::NAN, other, None);
                    }
                }
            }
        };
    }

    mod wrapping {
        use core::num::Wrapping;

        use super::assert_distance;
        use crate::prelude::*;

        #[test]
        fn uses_the_unwrapped_distance() {
            assert_distance(Wrapping(5_u8), Wrapping(4), Some(Wrapping(1)));
            assert_distance(Wrapping(-3_i32), Wrapping(4), Some(Wrapping(7)));
            assert_distance(
                Wrapping(u8::MIN),
                Wrapping(u8::MAX),
                Some(Wrapping(u8::MAX)),
            );
        }

        #[test]
        fn unrepresentable_distances_do_not_wrap() {
            assert_distance(Wrapping(i8::MIN), Wrapping(i8::MAX), None);
        }

        #[test]
        fn enables_is_close_to() {
            use super::super::IsCloseTo;

            assert_that!(Wrapping(5)).matches(IsCloseTo::new(Wrapping(4), Wrapping(1)));
            assert_that!(Wrapping(5_u64)).matches(IsCloseTo::new(&Wrapping(7), Wrapping(2)));

            let failures = assert_that!(Wrapping(5))
                .with_location(false)
                .capture(|it| it.matches(IsCloseTo::new(Wrapping(2), Wrapping(1))));
            assert_that!(failures).has_length(1);
        }
    }

    mod is_close_to {
        // The generic tolerance protocol and its reports. Each `NumericDistance` integration
        // covers only its own distances and rendering.
        use core::{
            borrow::Borrow,
            cell::{Cell, RefCell},
        };

        use indoc::formatdoc;

        use super::super::{IsCloseTo, close_to};
        use crate::{borrow_for::BorrowFor, expectation::Expectation, prelude::*};

        #[test]
        fn reports_a_value_outside_the_deviation() {
            let failures = assert_that!(13)
                .with_location(false)
                .capture(|it| it.matches(close_to(10, 2)));
            assert_that!(&failures[0]).has_text_report(formatdoc! {r"
                -------- assertr --------
                Expression: `13`

                Actual: 13

                is not close to

                Expected: 10

                Details:
                  - Allowed deviation: 2
                -------- assertr --------
            "});
        }

        #[test]
        fn reports_an_invalid_deviation_without_comparing() {
            let failures = assert_that!(1)
                .with_location(false)
                .capture(|it| it.matches(close_to(1, -1)).matches(close_to(1, 0)));
            assert_that!(failures).has_length(1);
            assert_that!(&failures[0]).has_text_report(formatdoc! {r"
                -------- assertr --------
                Expression: `1`

                was given an invalid allowed deviation

                Details:
                  - Allowed deviation: -1
                  - The allowed deviation must be zero or positive.
                -------- assertr --------
            "});
        }

        #[test]
        fn resolves_operands_once_in_order_and_reuses_rejected_views() {
            /// Records each borrow. Later borrows return another value, so an explanation that
            /// borrowed again would show it.
            struct Operand<'a> {
                value: i32,
                calls: Cell<usize>,
                name: &'static str,
                events: &'a RefCell<Vec<&'static str>>,
            }
            impl Borrow<i32> for Operand<'_> {
                fn borrow(&self) -> &i32 {
                    self.events.borrow_mut().push(self.name);
                    if self.calls.replace(self.calls.get() + 1) == 0 {
                        &self.value
                    } else {
                        &i32::MAX
                    }
                }
            }
            impl BorrowFor<i32> for Operand<'_> {
                type View = i32;
            }

            for deviation in [-1, 0, 1] {
                let events = RefCell::new(Vec::new());
                let operand = |value, name| Operand {
                    value,
                    calls: Cell::new(0),
                    name,
                    events: &events,
                };
                let matcher =
                    IsCloseTo::new(operand(3, "expected"), operand(deviation, "deviation"));
                assert_that!(&*events.borrow()).is_empty();
                let failures = assert_that!(5)
                    .with_location(false)
                    .capture(|it| it.matches(&matcher));
                assert_that!(&*events.borrow()).contains_exactly(["expected", "deviation"]);
                let plain = assert_that!(5)
                    .with_location(false)
                    .capture(|it| it.matches(close_to(3, deviation)));
                assert_that!(failures).is_equal_to(plain);

                events.borrow_mut().clear();
                let description = AssertionContext::default().describe::<i32, _>(&matcher);
                assert_that!(description.relation.as_deref()).is_equal_to(Some("is close to"));
                assert_that!(&*events.borrow()).contains_exactly(["expected", "deviation"]);

                // Re-evaluation resolves the new views.
                events.borrow_mut().clear();
                assert_that!(matcher.evaluate(&i32::MAX, &AssertionContext::default())).is_ok();
                assert_that!(&*events.borrow()).contains_exactly(["expected", "deviation"]);
            }
        }
    }

    unsigned_integer_tests!(u8, u16, u32, u64, u128, usize);
    signed_integer_tests!(i8, i16, i32, i64, i128, isize);
    float_tests!(f32);
    float_tests!(f64);
}
