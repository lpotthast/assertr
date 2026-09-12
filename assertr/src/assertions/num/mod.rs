//! Assertions for numeric identities, signs, tolerances, and floating-point classifications.

use crate::{
    AssertThat, AssertionContext, Expectation, ExpectationDiagnostics, Fact, Mode, ValueRenderer,
    failure::{FailureBuilder, FailureKind},
};
use core::cmp::Ordering;
#[cfg(any(feature = "std", feature = "libm"))]
use num_traits::Float;
use num_traits::{Num, Signed};

mod numeric_distance;
pub use numeric_distance::NumericDistance;

/// Checks [`Signed::is_negative`], including the sign bit of floating-point values.
pub struct IsNegative;

impl<T, R> Expectation<T, R> for IsNegative
where
    T: Signed,
{
    type Success<'a>
        = ()
    where
        T: 'a;
    type Rejection<'a>
        = ()
    where
        T: 'a;

    fn evaluate<'a>(&'a self, actual: &'a T, _: &AssertionContext<'_, R>) -> Result<(), ()> {
        if actual.is_negative() {
            Ok(())
        } else {
            Err(())
        }
    }
}

impl<T, R> ExpectationDiagnostics<T, R> for IsNegative
where
    R: ValueRenderer<T>,
    T: Signed,
{
    const KIND: FailureKind = FailureKind::Ordering;

    fn explain<Target>(
        &self,
        rejected: Option<(&T, ())>,
        failure: FailureBuilder<Target>,
        context: &AssertionContext<'_, R>,
    ) -> FailureBuilder<Target> {
        let render = context.render();
        match rejected {
            None => failure.relation("is negative"),
            Some((actual, ())) => failure
                .actual(render.value(actual))
                .relation("is not negative"),
        }
    }
}

/// Checks [`Signed::is_positive`], including the sign bit of floating-point values.
pub struct IsPositive;

impl<T, R> Expectation<T, R> for IsPositive
where
    T: Signed,
{
    type Success<'a>
        = ()
    where
        T: 'a;
    type Rejection<'a>
        = ()
    where
        T: 'a;

    fn evaluate<'a>(&'a self, actual: &'a T, _: &AssertionContext<'_, R>) -> Result<(), ()> {
        if actual.is_positive() {
            Ok(())
        } else {
            Err(())
        }
    }
}

impl<T, R> ExpectationDiagnostics<T, R> for IsPositive
where
    R: ValueRenderer<T>,
    T: Signed,
{
    const KIND: FailureKind = FailureKind::Ordering;

    fn explain<Target>(
        &self,
        rejected: Option<(&T, ())>,
        failure: FailureBuilder<Target>,
        context: &AssertionContext<'_, R>,
    ) -> FailureBuilder<Target> {
        let render = context.render();
        match rejected {
            None => failure.relation("is positive"),
            Some((actual, ())) => failure
                .actual(render.value(actual))
                .relation("is not positive"),
        }
    }
}

#[cfg(any(feature = "std", feature = "libm"))]
/// Checks whether a numeric value is finite.
pub struct IsFinite;

#[cfg(any(feature = "std", feature = "libm"))]
impl<T, R> Expectation<T, R> for IsFinite
where
    T: Float,
{
    type Success<'a>
        = ()
    where
        T: 'a;
    type Rejection<'a>
        = ()
    where
        T: 'a;

    fn evaluate<'a>(&'a self, actual: &'a T, _: &AssertionContext<'_, R>) -> Result<(), ()> {
        if actual.is_finite() { Ok(()) } else { Err(()) }
    }
}

#[cfg(any(feature = "std", feature = "libm"))]
impl<T, R> ExpectationDiagnostics<T, R> for IsFinite
where
    R: ValueRenderer<T>,
    T: Float,
{
    const KIND: FailureKind = FailureKind::Other;

    fn explain<Target>(
        &self,
        rejected: Option<(&T, ())>,
        failure: FailureBuilder<Target>,
        context: &AssertionContext<'_, R>,
    ) -> FailureBuilder<Target> {
        let render = context.render();
        match rejected {
            None => failure.relation("is finite"),
            Some((actual, ())) => failure
                .actual(render.value(actual))
                .relation("is not finite"),
        }
    }
}

#[cfg(any(feature = "std", feature = "libm"))]
/// Checks whether a numeric value is infinite.
pub struct IsInfinite;

#[cfg(any(feature = "std", feature = "libm"))]
impl<T, R> Expectation<T, R> for IsInfinite
where
    T: Float,
{
    type Success<'a>
        = ()
    where
        T: 'a;
    type Rejection<'a>
        = ()
    where
        T: 'a;

    fn evaluate<'a>(&'a self, actual: &'a T, _: &AssertionContext<'_, R>) -> Result<(), ()> {
        if actual.is_infinite() {
            Ok(())
        } else {
            Err(())
        }
    }
}

#[cfg(any(feature = "std", feature = "libm"))]
impl<T, R> ExpectationDiagnostics<T, R> for IsInfinite
where
    R: ValueRenderer<T>,
    T: Float,
{
    const KIND: FailureKind = FailureKind::Other;

    fn explain<Target>(
        &self,
        rejected: Option<(&T, ())>,
        failure: FailureBuilder<Target>,
        context: &AssertionContext<'_, R>,
    ) -> FailureBuilder<Target> {
        let render = context.render();
        match rejected {
            None => failure.relation("is infinite"),
            Some((actual, ())) => failure
                .actual(render.value(actual))
                .relation("is not infinite"),
        }
    }
}

#[cfg(any(feature = "std", feature = "libm"))]
/// Checks whether a numeric value is normal.
pub struct IsNormal;

#[cfg(any(feature = "std", feature = "libm"))]
impl<T, R> Expectation<T, R> for IsNormal
where
    T: Float,
{
    type Success<'a>
        = ()
    where
        T: 'a;
    type Rejection<'a>
        = ()
    where
        T: 'a;

    fn evaluate<'a>(&'a self, actual: &'a T, _: &AssertionContext<'_, R>) -> Result<(), ()> {
        if actual.is_normal() { Ok(()) } else { Err(()) }
    }
}

#[cfg(any(feature = "std", feature = "libm"))]
impl<T, R> ExpectationDiagnostics<T, R> for IsNormal
where
    R: ValueRenderer<T>,
    T: Float,
{
    const KIND: FailureKind = FailureKind::Other;

    fn explain<Target>(
        &self,
        rejected: Option<(&T, ())>,
        failure: FailureBuilder<Target>,
        context: &AssertionContext<'_, R>,
    ) -> FailureBuilder<Target> {
        let render = context.render();
        match rejected {
            None => failure.relation("is normal"),
            Some((actual, ())) => failure
                .actual(render.value(actual))
                .relation("is not normal"),
        }
    }
}

#[cfg(any(feature = "std", feature = "libm"))]
/// Checks whether a numeric value is subnormal.
pub struct IsSubnormal;

#[cfg(any(feature = "std", feature = "libm"))]
impl<T, R> Expectation<T, R> for IsSubnormal
where
    T: Float,
{
    type Success<'a>
        = ()
    where
        T: 'a;
    type Rejection<'a>
        = ()
    where
        T: 'a;

    fn evaluate<'a>(&'a self, actual: &'a T, _: &AssertionContext<'_, R>) -> Result<(), ()> {
        if actual.is_subnormal() {
            Ok(())
        } else {
            Err(())
        }
    }
}

#[cfg(any(feature = "std", feature = "libm"))]
impl<T, R> ExpectationDiagnostics<T, R> for IsSubnormal
where
    R: ValueRenderer<T>,
    T: Float,
{
    const KIND: FailureKind = FailureKind::Other;

    fn explain<Target>(
        &self,
        rejected: Option<(&T, ())>,
        failure: FailureBuilder<Target>,
        context: &AssertionContext<'_, R>,
    ) -> FailureBuilder<Target> {
        let render = context.render();
        match rejected {
            None => failure.relation("is subnormal"),
            Some((actual, ())) => failure
                .actual(render.value(actual))
                .relation("is not subnormal"),
        }
    }
}

/// Checks whether a numeric value is zero.
pub struct IsZero;

impl<T: Num, R> Expectation<T, R> for IsZero {
    type Success<'a>
        = ()
    where
        T: 'a;
    type Rejection<'a>
        = ()
    where
        T: 'a;
    fn evaluate<'a>(&'a self, actual: &'a T, _: &AssertionContext<'_, R>) -> Result<(), ()> {
        if actual.is_zero() { Ok(()) } else { Err(()) }
    }
}

impl<T: Num, R: ValueRenderer<T>> ExpectationDiagnostics<T, R> for IsZero {
    const KIND: FailureKind = FailureKind::Equality;
    fn explain<Target>(
        &self,
        rejected: Option<(&T, ())>,
        failure: FailureBuilder<Target>,
        context: &AssertionContext<'_, R>,
    ) -> FailureBuilder<Target> {
        let render = context.render();
        let failure = match rejected {
            None => failure.relation("is zero"),
            Some((actual, ())) => failure.actual(render.value(actual)),
        };
        failure.expected(render.value(&T::zero()))
    }
}

/// Checks whether a numeric value is one.
pub struct IsOne;

impl<T: Num, R> Expectation<T, R> for IsOne {
    type Success<'a>
        = ()
    where
        T: 'a;
    type Rejection<'a>
        = ()
    where
        T: 'a;
    fn evaluate<'a>(&'a self, actual: &'a T, _: &AssertionContext<'_, R>) -> Result<(), ()> {
        if actual.is_one() { Ok(()) } else { Err(()) }
    }
}

impl<T: Num, R: ValueRenderer<T>> ExpectationDiagnostics<T, R> for IsOne {
    const KIND: FailureKind = FailureKind::Equality;
    fn explain<Target>(
        &self,
        rejected: Option<(&T, ())>,
        failure: FailureBuilder<Target>,
        context: &AssertionContext<'_, R>,
    ) -> FailureBuilder<Target> {
        let render = context.render();
        let failure = match rejected {
            None => failure.relation("is one"),
            Some((actual, ())) => failure.actual(render.value(actual)),
        };
        failure.expected(render.value(&T::one()))
    }
}

#[cfg(any(feature = "std", feature = "libm"))]
/// Checks whether a numeric value is NaN.
pub struct IsNan;

#[cfg(any(feature = "std", feature = "libm"))]
impl<T: Float, R> Expectation<T, R> for IsNan {
    type Success<'a>
        = ()
    where
        T: 'a;
    type Rejection<'a>
        = ()
    where
        T: 'a;
    fn evaluate<'a>(&'a self, actual: &'a T, _: &AssertionContext<'_, R>) -> Result<(), ()> {
        if actual.is_nan() { Ok(()) } else { Err(()) }
    }
}

#[cfg(any(feature = "std", feature = "libm"))]
impl<T: Float, R: ValueRenderer<T>> ExpectationDiagnostics<T, R> for IsNan {
    const KIND: FailureKind = FailureKind::Equality;
    fn explain<Target>(
        &self,
        rejected: Option<(&T, ())>,
        failure: FailureBuilder<Target>,
        context: &AssertionContext<'_, R>,
    ) -> FailureBuilder<Target> {
        let render = context.render();
        let failure = match rejected {
            None => failure.relation("is NaN"),
            Some((actual, ())) => failure.actual(render.value(actual)),
        };
        failure.expected(render.value(&T::nan()))
    }
}

/// Checks distance from an expected value with an inclusive, non-negative deviation.
/// Uses [`NumericDistance`] without requiring `Clone` or floating-point math features.
pub struct IsCloseTo<T> {
    expected: T,
    allowed_deviation: T,
}

impl<T> IsCloseTo<T> {
    /// Owns the expected value and allowed deviation.
    #[must_use]
    pub const fn new(expected: T, allowed_deviation: T) -> Self {
        Self {
            expected,
            allowed_deviation,
        }
    }
}

/// The reason a numeric tolerance was rejected.
#[non_exhaustive]
pub enum CloseToRejection {
    /// The deviation was negative or incomparable with zero.
    InvalidDeviation,
    /// The distance exceeded the deviation or could not be computed.
    OutsideDeviation,
}

impl<T: NumericDistance, R> Expectation<T, R> for IsCloseTo<T> {
    type Success<'a>
        = ()
    where
        T: 'a;
    type Rejection<'a>
        = CloseToRejection
    where
        T: 'a;
    fn evaluate<'a>(
        &'a self,
        actual: &'a T,
        _: &AssertionContext<'_, R>,
    ) -> Result<(), CloseToRejection> {
        let zero = T::zero();
        if !matches!(
            self.allowed_deviation.partial_cmp(&zero),
            Some(Ordering::Greater | Ordering::Equal)
        ) {
            return Err(CloseToRejection::InvalidDeviation);
        }
        if actual
            .checked_distance(&self.expected)
            .is_some_and(|distance| distance <= self.allowed_deviation)
        {
            Ok(())
        } else {
            Err(CloseToRejection::OutsideDeviation)
        }
    }
}

impl<T: NumericDistance, R: ValueRenderer<T>> ExpectationDiagnostics<T, R> for IsCloseTo<T> {
    const KIND: FailureKind = FailureKind::Ordering;
    fn explain<Target>(
        &self,
        rejected: Option<(&T, CloseToRejection)>,
        failure: FailureBuilder<Target>,
        context: &AssertionContext<'_, R>,
    ) -> FailureBuilder<Target> {
        let render = context.render();
        match rejected {
            None => {
                let description = failure
                    .relation("is close to")
                    .expected(render.value(&self.expected));
                if render.max_items() == 0 {
                    description.omitted_children(1)
                } else {
                    description.children([FailureBuilder::detached::<()>(FailureKind::Matching)
                        .relation("allows a deviation of")
                        .expected(render.value(&self.allowed_deviation))
                        .build()])
                }
            }
            Some((actual, rejection)) => {
                let allowed_deviation = render.value(&self.allowed_deviation);
                match rejection {
                    CloseToRejection::InvalidDeviation => failure
                        .relation("was given an invalid allowed deviation")
                        .fact(Fact::labelled("Allowed deviation", allowed_deviation))
                        .fact(Fact::note(
                            "The allowed deviation must be a non-negative number.",
                        )),
                    CloseToRejection::OutsideDeviation => failure
                        .actual(render.value(actual))
                        .relation("is not close to")
                        .expected(render.value(&self.expected))
                        .fact(Fact::labelled("Allowed deviation", allowed_deviation)),
                }
            }
        }
    }
}

/// Assertions for numeric values not already handled by [`crate::prelude::PartialEqAssertions`] and
/// [`crate::prelude::PartialOrdAssertions`].
#[allow(clippy::return_self_not_must_use)]
#[cfg_attr(feature = "fluent", assertr_macros::fluent_aliases)]
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
    /// math features (`std` or `libm`) are required.
    fn is_close_to(self, expected: T, allowed_deviation: T) -> Self
    where
        T: NumericDistance,
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
        self.apply_assertion(IsZero)
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
        self.apply_assertion(IsOne)
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
        self.apply_assertion(IsNegative)
    }

    #[track_caller]
    fn is_positive(self) -> Self
    where
        T: Signed,
        R: ValueRenderer<T>,
    {
        self.apply_assertion(IsPositive)
    }

    #[track_caller]
    fn is_close_to(self, expected: T, allowed_deviation: T) -> Self
    where
        T: NumericDistance,
        R: ValueRenderer<T>,
    {
        self.apply_assertion(IsCloseTo::new(expected, allowed_deviation))
    }

    #[track_caller]
    #[cfg(any(feature = "std", feature = "libm"))]
    fn is_nan(self) -> Self
    where
        T: Float,
        R: ValueRenderer<T>,
    {
        self.apply_assertion(IsNan)
    }

    #[track_caller]
    #[cfg(any(feature = "std", feature = "libm"))]
    fn is_finite(self) -> Self
    where
        T: Float,
        R: ValueRenderer<T>,
    {
        self.apply_assertion(IsFinite)
    }

    #[track_caller]
    #[cfg(any(feature = "std", feature = "libm"))]
    fn is_infinite(self) -> Self
    where
        T: Float,
        R: ValueRenderer<T>,
    {
        self.apply_assertion(IsInfinite)
    }

    #[track_caller]
    #[cfg(any(feature = "std", feature = "libm"))]
    fn is_normal(self) -> Self
    where
        T: Float,
        R: ValueRenderer<T>,
    {
        self.apply_assertion(IsNormal)
    }

    #[track_caller]
    #[cfg(any(feature = "std", feature = "libm"))]
    fn is_subnormal(self) -> Self
    where
        T: Float,
        R: ValueRenderer<T>,
    {
        self.apply_assertion(IsSubnormal)
    }
}

#[cfg(test)]
mod tests {
    mod renderer_contract {
        use crate::{
            prelude::*,
            test_support::{NoRenderer, assert_trait_impl},
        };

        #[test]
        fn trait_is_implemented_without_renderer_support() {
            assert_trait_impl!(
                AssertThat<'static, i32, Panic, NoRenderer> => NumAssertions<i32>
            );
            assert_trait_impl!(
                AssertThat<'static, f64, Panic, NoRenderer> => NumAssertions<f64>
            );

            assert_trait_impl!(super::super::IsZero => crate::Expectation<i32, NoRenderer>);
            assert_trait_impl!(super::super::IsOne => crate::Expectation<i32, NoRenderer>);
            assert_trait_impl!(super::super::IsNegative => crate::Expectation<i32, NoRenderer>);
            assert_trait_impl!(super::super::IsPositive => crate::Expectation<i32, NoRenderer>);
            assert_trait_impl!(super::super::IsCloseTo<i32> => crate::Expectation<i32, NoRenderer>);
            #[cfg(any(feature = "std", feature = "libm"))]
            {
                assert_trait_impl!(super::super::IsNan => crate::Expectation<f64, NoRenderer>);
                assert_trait_impl!(super::super::IsFinite => crate::Expectation<f64, NoRenderer>);
                assert_trait_impl!(super::super::IsInfinite => crate::Expectation<f64, NoRenderer>);
                assert_trait_impl!(super::super::IsNormal => crate::Expectation<f64, NoRenderer>);
                assert_trait_impl!(super::super::IsSubnormal => crate::Expectation<f64, NoRenderer>);
            }
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
        fn caller_location_is_as_expected() {
            assert_caller_location!(assert_that!(3), is_zero());
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
                    -------- assertr --------
                "});
        }
    }

    /// Synonym of `is_zero`. The fluent name and caller location are pinned here.
    /// The behavior is covered by that module.
    mod is_additive_identity {
        use crate::prelude::*;

        #[test]
        #[cfg(feature = "fluent")]
        fn fluent_alias_is_as_expected() {
            0_i32.must().be_additive_identity();
        }

        #[test]
        fn caller_location_is_as_expected() {
            assert_caller_location!(assert_that!(3), is_additive_identity());
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
        fn caller_location_is_as_expected() {
            assert_caller_location!(assert_that!(3), is_one());
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
                    -------- assertr --------
                "});
        }
    }

    /// Synonym of `is_one`. The fluent name and caller location are pinned here.
    /// The behavior is covered by that module.
    mod is_multiplicative_identity {
        use crate::prelude::*;

        #[test]
        #[cfg(feature = "fluent")]
        fn fluent_alias_is_as_expected() {
            1_i32.must().be_multiplicative_identity();
        }

        #[test]
        fn caller_location_is_as_expected() {
            assert_caller_location!(assert_that!(3), is_multiplicative_identity());
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
        fn panics_when_zero() {
            assert_that_panic_by(|| assert_that!(0.0).with_location(false).is_negative())
                .has_type::<String>()
                .is_equal_to(formatdoc! {r"
                    -------- assertr --------
                    Expression: `0.0`

                    Actual: 0.0

                    is not negative
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

                    Actual: 1.23

                    is not negative
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
        fn caller_location_is_as_expected() {
            assert_caller_location!(assert_that!(-1.23), is_positive());
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

                    Actual: -1.23

                    is not positive
                    -------- assertr --------
                "});
        }
    }

    mod is_close_to {
        // The NumericDistance tests own the primitive arithmetic and special-value matrices.
        // These tests cover tolerance handling, diagnostics, and integration.
        use crate::prelude::*;
        use indoc::formatdoc;

        #[test]
        #[cfg(feature = "fluent")]
        fn fluent_alias_is_as_expected() {
            0.333_f64.must().be_close_to(0.333, 0.001);
        }

        #[test]
        fn caller_location_is_as_expected() {
            assert_caller_location!(assert_that!(1_i32), is_close_to(3, 1));
        }

        #[test]
        fn missing_tolerance_description_respects_the_item_budget() {
            use super::super::IsCloseTo;

            for limit in [0, 1] {
                let budget = RenderingBudget::default().with_max_items(limit);
                let context = crate::AssertionContext::new(&DebugRenderer, budget);
                let description = context.describe::<i32, _>(&IsCloseTo::new(42, 2));
                assert_that!(description.children.len()).is_equal_to(limit);
                assert_that!(description.omitted_children).is_equal_to(1 - limit);
            }
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

                    Actual: 0.3319

                    is not close to

                    Expected: 0.333

                    Details:
                      - Allowed deviation: 0.001
                    -------- assertr --------
                "});
        }

        #[test]
        fn succeeds_when_actual_is_in_allowed_range() {
            assert_that!(0.25).is_close_to(0.5, 0.25);
            assert_that!(0.5).is_close_to(0.5, 0.25);
            assert_that!(0.75).is_close_to(0.5, 0.25);
            assert_that!(0_u8).is_close_to(0, 1);
        }

        #[test]
        fn reports_distance_beyond_a_rounded_subtraction_boundary() {
            assert_that_panic_by(|| {
                assert_that!(9_007_199_254_740_992_f64)
                    .with_location(false)
                    .is_close_to(9_007_199_254_740_994_f64, 1.0);
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {r"
                -------- assertr --------
                Expression: `9_007_199_254_740_992_f64`

                Actual: 9007199254740992.0

                is not close to

                Expected: 9007199254740994.0

                Details:
                  - Allowed deviation: 1.0
                -------- assertr --------
            "});
        }

        #[test]
        fn rejects_decimal_distance_just_above_deviation() {
            for (actual, expected) in [(0.334_f64, 0.333_f64), (0.333, 0.334)] {
                let failures = assert_that!(actual).capture(|it| it.is_close_to(expected, 0.001));
                assert_that!(failures).has_length(1);
            }
        }

        #[test]
        fn succeeds_for_equal_negative_infinity() {
            assert_that!(f64::NEG_INFINITY).is_close_to(f64::NEG_INFINITY, 0.0);
        }

        #[test]
        fn positive_infinite_deviation_is_unbounded_for_comparable_values() {
            let deviation = f64::INFINITY;

            assert_that!(-f64::MAX).is_close_to(f64::MAX, deviation);
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
            .contains("is not close to");

            assert_that_panic_by(|| {
                assert_that!(1.0)
                    .with_location(false)
                    .is_close_to(f64::NAN, f64::INFINITY);
            })
            .has_type::<String>()
            .contains("is not close to");
        }

        #[test]
        fn rejects_nan_deviation() {
            assert_that_panic_by(|| {
                assert_that!(1.0)
                    .with_location(false)
                    .is_close_to(1.0, f64::NAN);
            })
            .has_type::<String>()
            .contains("was given an invalid allowed deviation");
        }

        #[test]
        fn reports_extreme_signed_values_without_overflowing() {
            assert_that_panic_by(|| {
                assert_that!(i8::MIN)
                    .with_location(false)
                    .is_close_to(i8::MAX, i8::MAX);
            })
            .has_type::<String>()
            .contains("is not close to");
        }

        #[test]
        fn rejects_negative_deviation() {
            assert_that_panic_by(|| {
                assert_that!(1_i8).with_location(false).is_close_to(1, -1);
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {r"
                -------- assertr --------
                Expression: `1_i8`

                was given an invalid allowed deviation

                Details:
                  - Allowed deviation: -1
                  - The allowed deviation must be a non-negative number.
                -------- assertr --------
            "});
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

                    Actual: 0.3341

                    is not close to

                    Expected: 0.333

                    Details:
                      - Allowed deviation: 0.001
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
        fn caller_location_is_as_expected() {
            assert_caller_location!(assert_that!(1.23), is_nan());
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
        fn caller_location_is_as_expected() {
            assert_caller_location!(assert_that!(f32::INFINITY), is_finite());
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

                    Actual: inf

                    is not finite
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

                    Actual: -inf

                    is not finite
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
        fn caller_location_is_as_expected() {
            assert_caller_location!(assert_that!(1.23), is_infinite());
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

                    Actual: 1.23

                    is not infinite
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
        fn caller_location_is_as_expected() {
            let subnormal = f32::from_bits(1);
            assert_caller_location!(assert_that!(subnormal), is_normal());
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

                    Actual: 1e-45

                    is not normal
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
        fn caller_location_is_as_expected() {
            assert_caller_location!(assert_that!(1.0_f32), is_subnormal());
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

                    Actual: 1.0

                    is not subnormal
                    -------- assertr --------
                "});
        }
    }
}
