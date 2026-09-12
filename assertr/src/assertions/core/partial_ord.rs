use core::{borrow::Borrow, cmp::Ordering, marker::PhantomData};

use crate::{
    AssertThat, AssertionContext, Expectation, ExpectationDiagnostics, Mode, ValueRenderer,
    failure::{FailureBuilder, FailureKind},
};

/// A reusable partial-order strict upper bound using the subject's heterogeneous [`PartialOrd`]
/// implementation. Incomparable values, including comparisons involving `NaN`, are rejected.
///
/// Construct with [`new`](Self::new) and execute through an assertion chain or a supplied
/// [`AssertionContext`]. [`PartialOrdAssertions::is_less_than`] executes this same definition on an
/// assertion chain. Operand storage `B` defaults to `E` for owned expectations and also lets the
/// ordinary method borrow its expected operand without cloning it.
///
/// ```
/// use assertr::prelude::*;
/// use assertr::assertions::core::partial_ord::LessThan;
///
/// let maximum = LessThan::new(65);
/// assert_that!(42).matches(&maximum);
/// ```
pub struct LessThan<E, B = E> {
    expected: B,
    operand: PhantomData<fn() -> E>,
}

/// Matches values less than `expected`. Incomparable values do not match.
///
/// This is a convenience constructor for [`LessThan::new`].
pub fn lt<E>(expected: E) -> LessThan<E> {
    LessThan::new(expected)
}

impl<E> LessThan<E> {
    /// Owns an expected operand. Pass a reference when the subject implements ordering with it.
    #[must_use]
    pub const fn new(expected: E) -> Self {
        Self {
            expected,
            operand: PhantomData,
        }
    }

    fn with_storage<B: Borrow<E>>(expected: B) -> LessThan<E, B> {
        LessThan {
            expected,
            operand: PhantomData,
        }
    }
}

impl<T: ?Sized, E, B, R> Expectation<T, R> for LessThan<E, B>
where
    T: PartialOrd<E>,
    B: Borrow<E>,
{
    type Success<'a>
        = ()
    where
        Self: 'a,
        T: 'a;
    type Rejection<'a>
        = &'a E
    where
        Self: 'a,
        T: 'a;

    fn evaluate<'a>(
        &'a self,
        actual: &'a T,
        _: &AssertionContext<'_, R>,
    ) -> Result<(), Self::Rejection<'a>> {
        let expected = self.expected.borrow();
        if matches!(actual.partial_cmp(expected), Some(Ordering::Less)) {
            Ok(())
        } else {
            Err(expected)
        }
    }
}

impl<T: ?Sized, E, B, R> ExpectationDiagnostics<T, R> for LessThan<E, B>
where
    T: PartialOrd<E>,
    B: Borrow<E>,
    R: ValueRenderer<T> + ValueRenderer<E>,
{
    const KIND: FailureKind = FailureKind::Ordering;

    fn explain<'a, Target>(
        &'a self,
        rejected: Option<(&'a T, Self::Rejection<'a>)>,
        failure: FailureBuilder<Target>,
        context: &AssertionContext<'_, R>,
    ) -> FailureBuilder<Target> {
        let render = context.render();
        let (failure, expected) = match rejected {
            None => (failure.relation("is less than"), self.expected.borrow()),
            Some((actual, expected)) => (
                failure
                    .actual(render.value(actual))
                    .relation("is not less than"),
                expected,
            ),
        };
        failure.expected(render.value(expected))
    }
}

/// A reusable partial-order strict lower bound using the subject's heterogeneous [`PartialOrd`]
/// implementation. Incomparable values, including comparisons involving `NaN`, are rejected.
///
/// Construct with [`new`](Self::new) and execute through an assertion chain or a supplied
/// [`AssertionContext`]. [`PartialOrdAssertions::is_greater_than`] executes this same definition on
/// an assertion chain. Operand storage `B` defaults to `E` for owned expectations and also lets the
/// ordinary method borrow its expected operand without cloning it.
///
/// ```
/// use assertr::prelude::*;
/// use assertr::assertions::core::partial_ord::GreaterThan;
///
/// let minimum = GreaterThan::new(18);
/// assert_that!(42).matches(&minimum);
/// ```
pub struct GreaterThan<E, B = E> {
    expected: B,
    operand: PhantomData<fn() -> E>,
}

/// Matches values greater than `expected`. Incomparable values do not match.
///
/// This is a convenience constructor for [`GreaterThan::new`].
pub fn gt<E>(expected: E) -> GreaterThan<E> {
    GreaterThan::new(expected)
}

impl<E> GreaterThan<E> {
    /// Owns an expected operand. Pass a reference when the subject implements ordering with it.
    #[must_use]
    pub const fn new(expected: E) -> Self {
        Self {
            expected,
            operand: PhantomData,
        }
    }

    fn with_storage<B: Borrow<E>>(expected: B) -> GreaterThan<E, B> {
        GreaterThan {
            expected,
            operand: PhantomData,
        }
    }
}

impl<T: ?Sized, E, B, R> Expectation<T, R> for GreaterThan<E, B>
where
    T: PartialOrd<E>,
    B: Borrow<E>,
{
    type Success<'a>
        = ()
    where
        Self: 'a,
        T: 'a;
    type Rejection<'a>
        = &'a E
    where
        Self: 'a,
        T: 'a;

    fn evaluate<'a>(
        &'a self,
        actual: &'a T,
        _: &AssertionContext<'_, R>,
    ) -> Result<(), Self::Rejection<'a>> {
        let expected = self.expected.borrow();
        if matches!(actual.partial_cmp(expected), Some(Ordering::Greater)) {
            Ok(())
        } else {
            Err(expected)
        }
    }
}

impl<T: ?Sized, E, B, R> ExpectationDiagnostics<T, R> for GreaterThan<E, B>
where
    T: PartialOrd<E>,
    B: Borrow<E>,
    R: ValueRenderer<T> + ValueRenderer<E>,
{
    const KIND: FailureKind = FailureKind::Ordering;

    fn explain<'a, Target>(
        &'a self,
        rejected: Option<(&'a T, Self::Rejection<'a>)>,
        failure: FailureBuilder<Target>,
        context: &AssertionContext<'_, R>,
    ) -> FailureBuilder<Target> {
        let render = context.render();
        let (failure, expected) = match rejected {
            None => (failure.relation("is greater than"), self.expected.borrow()),
            Some((actual, expected)) => (
                failure
                    .actual(render.value(actual))
                    .relation("is not greater than"),
                expected,
            ),
        };
        failure.expected(render.value(expected))
    }
}

/// A reusable partial-order upper bound using the subject's heterogeneous [`PartialOrd`]
/// implementation. Incomparable values, including comparisons involving `NaN`, are rejected.
///
/// Construct with [`new`](Self::new) and execute through an assertion chain or a supplied
/// [`AssertionContext`]. [`PartialOrdAssertions::is_less_or_equal_to`] executes this same
/// definition on an assertion chain. Operand storage `B` defaults to `E` for owned expectations and
/// also lets the ordinary method borrow its expected operand without cloning it.
///
/// ```
/// use assertr::prelude::*;
/// use assertr::assertions::core::partial_ord::LessOrEqual;
///
/// let maximum = LessOrEqual::new(65);
/// assert_that!(42).matches(&maximum);
/// ```
pub struct LessOrEqual<E, B = E> {
    expected: B,
    operand: PhantomData<fn() -> E>,
}

/// Matches values less than or equal to `expected`. Incomparable values do not match.
///
/// This is a convenience constructor for [`LessOrEqual::new`].
pub fn le<E>(expected: E) -> LessOrEqual<E> {
    LessOrEqual::new(expected)
}

impl<E> LessOrEqual<E> {
    /// Owns an expected operand. Pass a reference when the subject implements ordering with it.
    #[must_use]
    pub const fn new(expected: E) -> Self {
        Self {
            expected,
            operand: PhantomData,
        }
    }

    fn with_storage<B: Borrow<E>>(expected: B) -> LessOrEqual<E, B> {
        LessOrEqual {
            expected,
            operand: PhantomData,
        }
    }
}

impl<T: ?Sized, E, B, R> Expectation<T, R> for LessOrEqual<E, B>
where
    T: PartialOrd<E>,
    B: Borrow<E>,
{
    type Success<'a>
        = ()
    where
        Self: 'a,
        T: 'a;
    type Rejection<'a>
        = &'a E
    where
        Self: 'a,
        T: 'a;

    fn evaluate<'a>(
        &'a self,
        actual: &'a T,
        _: &AssertionContext<'_, R>,
    ) -> Result<(), Self::Rejection<'a>> {
        let expected = self.expected.borrow();
        if matches!(
            actual.partial_cmp(expected),
            Some(Ordering::Less | Ordering::Equal)
        ) {
            Ok(())
        } else {
            Err(expected)
        }
    }
}

impl<T: ?Sized, E, B, R> ExpectationDiagnostics<T, R> for LessOrEqual<E, B>
where
    T: PartialOrd<E>,
    B: Borrow<E>,
    R: ValueRenderer<T> + ValueRenderer<E>,
{
    const KIND: FailureKind = FailureKind::Ordering;

    fn explain<'a, Target>(
        &'a self,
        rejected: Option<(&'a T, Self::Rejection<'a>)>,
        failure: FailureBuilder<Target>,
        context: &AssertionContext<'_, R>,
    ) -> FailureBuilder<Target> {
        let render = context.render();
        let (failure, expected) = match rejected {
            None => (
                failure.relation("is less than or equal to"),
                self.expected.borrow(),
            ),
            Some((actual, expected)) => (
                failure
                    .actual(render.value(actual))
                    .relation("is not less or equal to"),
                expected,
            ),
        };
        failure.expected(render.value(expected))
    }
}

/// A reusable partial-order lower bound using the subject's heterogeneous [`PartialOrd`]
/// implementation. Incomparable values, including comparisons involving `NaN`, are rejected.
///
/// Construct with [`new`](Self::new) and execute through an assertion chain or a supplied
/// [`AssertionContext`]. [`PartialOrdAssertions::is_greater_or_equal_to`] executes this same
/// definition on an assertion chain. Operand storage `B` defaults to `E` for owned expectations and
/// also lets the ordinary method borrow its expected operand without cloning it.
///
/// ```
/// use assertr::prelude::*;
/// use assertr::assertions::core::partial_ord::GreaterOrEqual;
///
/// let minimum = GreaterOrEqual::new(18);
/// assert_that!(42).matches(&minimum);
/// ```
pub struct GreaterOrEqual<E, B = E> {
    expected: B,
    operand: PhantomData<fn() -> E>,
}

/// Matches values greater than or equal to `expected`. Incomparable values do not match.
///
/// This is a convenience constructor for [`GreaterOrEqual::new`].
pub fn ge<E>(expected: E) -> GreaterOrEqual<E> {
    GreaterOrEqual::new(expected)
}

impl<E> GreaterOrEqual<E> {
    /// Owns an expected operand. Pass a reference when the subject implements ordering with it.
    #[must_use]
    pub const fn new(expected: E) -> Self {
        Self {
            expected,
            operand: PhantomData,
        }
    }

    fn with_storage<B: Borrow<E>>(expected: B) -> GreaterOrEqual<E, B> {
        GreaterOrEqual {
            expected,
            operand: PhantomData,
        }
    }
}

impl<T: ?Sized, E, B, R> Expectation<T, R> for GreaterOrEqual<E, B>
where
    T: PartialOrd<E>,
    B: Borrow<E>,
{
    type Success<'a>
        = ()
    where
        Self: 'a,
        T: 'a;
    type Rejection<'a>
        = &'a E
    where
        Self: 'a,
        T: 'a;

    fn evaluate<'a>(
        &'a self,
        actual: &'a T,
        _: &AssertionContext<'_, R>,
    ) -> Result<(), Self::Rejection<'a>> {
        let expected = self.expected.borrow();
        if matches!(
            actual.partial_cmp(expected),
            Some(Ordering::Greater | Ordering::Equal)
        ) {
            Ok(())
        } else {
            Err(expected)
        }
    }
}

impl<T: ?Sized, E, B, R> ExpectationDiagnostics<T, R> for GreaterOrEqual<E, B>
where
    T: PartialOrd<E>,
    B: Borrow<E>,
    R: ValueRenderer<T> + ValueRenderer<E>,
{
    const KIND: FailureKind = FailureKind::Ordering;

    fn explain<'a, Target>(
        &'a self,
        rejected: Option<(&'a T, Self::Rejection<'a>)>,
        failure: FailureBuilder<Target>,
        context: &AssertionContext<'_, R>,
    ) -> FailureBuilder<Target> {
        let render = context.render();
        let (failure, expected) = match rejected {
            None => (
                failure.relation("is greater than or equal to"),
                self.expected.borrow(),
            ),
            Some((actual, expected)) => (
                failure
                    .actual(render.value(actual))
                    .relation("is not greater or equal to"),
                expected,
            ),
        };
        failure.expected(render.value(expected))
    }
}

/// Assertions for partially ordered values.
///
/// Each assertion requires the corresponding concrete [`Ordering`] result. Incomparable values
/// therefore fail every ordering assertion. In particular, a floating-point comparison involving
/// `NaN` does not satisfy either strict or inclusive ordering.
#[allow(clippy::return_self_not_must_use)]
#[cfg_attr(feature = "fluent", assertr_macros::fluent_aliases)]
pub trait PartialOrdAssertions<T, R> {
    /// Asserts that the subject is strictly less than `expected`.
    fn is_less_than<E>(self, expected: impl Borrow<E>) -> Self
    where
        R: ValueRenderer<T> + ValueRenderer<E>,
        T: PartialOrd<E>;

    /// Asserts that the subject is strictly greater than `expected`.
    fn is_greater_than<E>(self, expected: impl Borrow<E>) -> Self
    where
        R: ValueRenderer<T> + ValueRenderer<E>,
        T: PartialOrd<E>;

    /// Asserts that the subject is less than or equal to `expected`.
    fn is_less_or_equal_to<E>(self, expected: impl Borrow<E>) -> Self
    where
        R: ValueRenderer<T> + ValueRenderer<E>,
        T: PartialOrd<E>;

    /// Asserts that the subject is greater than or equal to `expected`.
    fn is_greater_or_equal_to<E>(self, expected: impl Borrow<E>) -> Self
    where
        R: ValueRenderer<T> + ValueRenderer<E>,
        T: PartialOrd<E>;
}

impl<T, M: Mode, R> PartialOrdAssertions<T, R> for AssertThat<'_, T, M, R> {
    #[track_caller]
    fn is_less_than<E>(self, expected: impl Borrow<E>) -> Self
    where
        R: ValueRenderer<T> + ValueRenderer<E>,
        T: PartialOrd<E>,
    {
        self.apply_assertion(LessThan::<E>::with_storage(expected))
    }

    #[track_caller]
    fn is_greater_than<E>(self, expected: impl Borrow<E>) -> Self
    where
        R: ValueRenderer<T> + ValueRenderer<E>,
        T: PartialOrd<E>,
    {
        self.apply_assertion(GreaterThan::<E>::with_storage(expected))
    }

    #[track_caller]
    fn is_less_or_equal_to<E>(self, expected: impl Borrow<E>) -> Self
    where
        R: ValueRenderer<T> + ValueRenderer<E>,
        T: PartialOrd<E>,
    {
        self.apply_assertion(LessOrEqual::<E>::with_storage(expected))
    }

    #[track_caller]
    fn is_greater_or_equal_to<E>(self, expected: impl Borrow<E>) -> Self
    where
        R: ValueRenderer<T> + ValueRenderer<E>,
        T: PartialOrd<E>,
    {
        self.apply_assertion(GreaterOrEqual::<E>::with_storage(expected))
    }
}

#[cfg(test)]
mod tests {
    mod renderer_contract {
        use core::{borrow::Borrow, cmp::Ordering, fmt};

        use crate::prelude::*;
        use crate::test_support::{NoRenderer, assert_trait_impl, rendered_text};
        use crate::{
            Expectation,
            assertions::core::partial_ord::{GreaterOrEqual, GreaterThan, LessOrEqual, LessThan},
        };

        #[test]
        fn trait_is_implemented_without_renderer_support() {
            assert_trait_impl!(
                AssertThat<'static, i32, Panic, NoRenderer>
                    => PartialOrdAssertions<i32, NoRenderer>
            );
            assert_trait_impl!(GreaterOrEqual<i32> => Expectation<i32, NoRenderer>);
            assert_trait_impl!(LessThan<i32> => Expectation<i32, NoRenderer>);
            assert_trait_impl!(GreaterThan<i32> => Expectation<i32, NoRenderer>);
            assert_trait_impl!(LessOrEqual<i32> => Expectation<i32, NoRenderer>);
        }

        #[test]
        fn borrowed_heterogeneous_operands_keep_the_original_renderer_bounds() {
            struct Actual;
            struct Expected;
            struct Operand(Expected);
            struct Renderer;

            impl PartialEq<Expected> for Actual {
                fn eq(&self, _: &Expected) -> bool {
                    false
                }
            }

            impl PartialOrd<Expected> for Actual {
                fn partial_cmp(&self, _: &Expected) -> Option<Ordering> {
                    None
                }
            }

            impl Borrow<Expected> for Operand {
                fn borrow(&self) -> &Expected {
                    &self.0
                }
            }

            impl ValueRenderer<Actual> for Renderer {
                fn fmt(&self, _: &Actual, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                    f.write_str("actual")
                }
            }

            impl ValueRenderer<Expected> for Renderer {
                fn fmt(&self, _: &Expected, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                    f.write_str("expected")
                }
            }

            let failures = assert_that!(Actual).with_renderer(Renderer).capture(|it| {
                it.is_less_than(Operand(Expected))
                    .is_greater_than(Operand(Expected))
                    .is_less_or_equal_to(Operand(Expected))
                    .is_greater_or_equal_to(Operand(Expected))
            });
            assert_that!(failures).has_length(4);
            for failure in &failures {
                assert_that!(rendered_text(failure.actual.as_ref().unwrap())).is_equal_to("actual");
                assert_that!(rendered_text(failure.expected.as_ref().unwrap()))
                    .is_equal_to("expected");
            }
        }
    }

    mod operand_observation {
        use crate::prelude::*;
        use core::{borrow::Borrow, cell::Cell, cmp::Ordering};

        #[derive(Debug)]
        struct Subject<'a> {
            value: i32,
            comparisons: &'a Cell<usize>,
        }

        impl PartialEq<i32> for Subject<'_> {
            fn eq(&self, expected: &i32) -> bool {
                self.value == *expected
            }
        }

        impl PartialOrd<i32> for Subject<'_> {
            fn partial_cmp(&self, expected: &i32) -> Option<Ordering> {
                self.comparisons.set(self.comparisons.get() + 1);
                self.value.partial_cmp(expected)
            }
        }

        struct Operand<'a> {
            value: i32,
            calls: &'a Cell<usize>,
        }
        impl Borrow<i32> for Operand<'_> {
            fn borrow(&self) -> &i32 {
                self.calls.set(self.calls.get() + 1);
                &self.value
            }
        }

        #[test]
        fn borrows_and_compares_once_on_both_acceptance_and_rejection() {
            for value in [0, 1, 2] {
                let calls = Cell::new(0);
                let comparisons = Cell::new(0);
                let actual = Subject {
                    value,
                    comparisons: &comparisons,
                };
                let operand = || Operand {
                    value: 1,
                    calls: &calls,
                };
                let failures = assert_that!(actual).capture(|it| {
                    let it = it.is_less_than::<i32>(operand());
                    assert_that!((calls.get(), comparisons.get())).is_equal_to((1, 1));
                    let it = it.is_greater_than::<i32>(operand());
                    assert_that!((calls.get(), comparisons.get())).is_equal_to((2, 2));
                    let it = it.is_less_or_equal_to::<i32>(operand());
                    assert_that!((calls.get(), comparisons.get())).is_equal_to((3, 3));
                    let it = it.is_greater_or_equal_to::<i32>(operand());
                    assert_that!((calls.get(), comparisons.get())).is_equal_to((4, 4));
                    it
                });
                assert_that!(failures).has_length(2);
            }
        }

        #[test]
        #[cfg(feature = "std")]
        fn tracks_before_user_borrow_code_can_panic() {
            struct PanickingOperand;
            impl Borrow<i32> for PanickingOperand {
                fn borrow(&self) -> &i32 {
                    panic!("operand conversion panicked")
                }
            }
            fn check(assertion: impl FnOnce(AssertThat<'_, i32, Capture>)) {
                let failures = assert_that!(0).capture(|it| {
                    let child = it.derive(|value| value);
                    let outcome = std::panic::catch_unwind(core::panic::AssertUnwindSafe(|| {
                        assertion(child);
                    }));
                    assert_that!(outcome).is_err();
                    it
                });
                assert_that!(failures).is_empty();
            }

            check(|it| {
                it.is_less_than::<i32>(PanickingOperand);
            });
            check(|it| {
                it.is_greater_than::<i32>(PanickingOperand);
            });
            check(|it| {
                it.is_less_or_equal_to::<i32>(PanickingOperand);
            });
            check(|it| {
                it.is_greater_or_equal_to::<i32>(PanickingOperand);
            });
        }
    }

    mod is_less_than {
        use crate::prelude::*;
        use indoc::formatdoc;

        #[test]
        #[cfg(feature = "fluent")]
        fn fluent_alias_is_as_expected() {
            3.must().be_less_than(4);
        }

        #[test]
        fn caller_location_is_as_expected() {
            assert_caller_location!(assert_that!(f32::NAN), is_less_than(0.0));
        }

        #[test]
        fn succeeds_when_less() {
            assert_that!(3).is_less_than(4);
        }

        #[test]
        fn rejects_equal_and_greater_values() {
            for actual in [4, 5] {
                let failures = assert_that!(actual).capture(|it| it.is_less_than(4));
                assert_that!(failures).has_length(1);
            }
        }

        #[test]
        fn panics_when_values_are_not_comparable() {
            assert_that_panic_by(|| {
                assert_that!(f32::NAN)
                    .with_location(false)
                    .is_less_than(0.0)
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {r"
                -------- assertr --------
                Expression: `f32::NAN`

                Actual: NaN

                is not less than

                Expected: 0.0
                -------- assertr --------
            "});
        }
    }

    mod is_greater_than {
        use crate::prelude::*;
        use indoc::formatdoc;

        #[test]
        #[cfg(feature = "fluent")]
        fn fluent_alias_is_as_expected() {
            7.must().be_greater_than(6);
        }

        #[test]
        fn caller_location_is_as_expected() {
            assert_caller_location!(assert_that!(f32::NAN), is_greater_than(0.0));
        }

        #[test]
        fn succeeds_when_greater() {
            assert_that!(7).is_greater_than(6);
        }

        #[test]
        fn rejects_equal_and_less_values() {
            for actual in [5, 6] {
                let failures = assert_that!(actual).capture(|it| it.is_greater_than(6));
                assert_that!(failures).has_length(1);
            }
        }

        #[test]
        fn panics_when_values_are_not_comparable() {
            assert_that_panic_by(|| {
                assert_that!(f32::NAN)
                    .with_location(false)
                    .is_greater_than(0.0)
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {r"
                -------- assertr --------
                Expression: `f32::NAN`

                Actual: NaN

                is not greater than

                Expected: 0.0
                -------- assertr --------
            "});
        }
    }

    mod is_less_or_equal_to {
        use crate::prelude::*;
        use indoc::formatdoc;

        #[test]
        #[cfg(feature = "fluent")]
        fn fluent_alias_is_as_expected() {
            3.must().be_less_or_equal_to(3);
        }

        #[test]
        fn caller_location_is_as_expected() {
            assert_caller_location!(assert_that!(f32::NAN), is_less_or_equal_to(0.0));
        }

        #[test]
        fn succeeds_when_less() {
            assert_that!(3).is_less_or_equal_to(4);
        }

        #[test]
        fn succeeds_when_equal() {
            assert_that!(3).is_less_or_equal_to(3);
        }

        #[test]
        fn rejects_greater_values() {
            let failures = assert_that!(4).capture(|it| it.is_less_or_equal_to(3));
            assert_that!(failures).has_length(1);
        }

        #[test]
        fn panics_when_values_are_not_comparable() {
            assert_that_panic_by(|| {
                assert_that!(f32::NAN)
                    .with_location(false)
                    .is_less_or_equal_to(0.0)
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {r"
                -------- assertr --------
                Expression: `f32::NAN`

                Actual: NaN

                is not less or equal to

                Expected: 0.0
                -------- assertr --------
            "});
        }
    }

    mod is_greater_or_equal_to {
        use crate::prelude::*;
        use indoc::formatdoc;

        #[test]
        #[cfg(feature = "fluent")]
        fn fluent_alias_is_as_expected() {
            7.must().be_greater_or_equal_to(7);
        }

        #[test]
        fn caller_location_is_as_expected() {
            assert_caller_location!(assert_that!(f32::NAN), is_greater_or_equal_to(0.0));
        }

        #[test]
        fn succeeds_when_greater() {
            assert_that!(7).is_greater_or_equal_to(6);
        }

        #[test]
        fn succeeds_when_equal() {
            assert_that!(7).is_greater_or_equal_to(7);
        }

        #[test]
        fn panics_when_values_are_not_comparable() {
            assert_that_panic_by(|| {
                assert_that!(f32::NAN)
                    .with_location(false)
                    .is_greater_or_equal_to(0.0)
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {r"
                -------- assertr --------
                Expression: `f32::NAN`

                Actual: NaN

                is not greater or equal to

                Expected: 0.0
                -------- assertr --------
            "});
        }
    }
}
