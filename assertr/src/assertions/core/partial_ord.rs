use alloc::string::String;
use core::{borrow::Borrow, cmp::Ordering, fmt::Write};
use indoc::writedoc;

use crate::{AssertThat, Mode, ValueRenderer};

/// Assertions for partially ordered values.
///
/// Each assertion requires the corresponding concrete [`Ordering`] result. Incomparable values
/// therefore fail every ordering assertion. In particular, a floating-point comparison involving
/// `NaN` does not satisfy either strict or inclusive ordering.
#[allow(clippy::return_self_not_must_use)]
#[cfg_attr(feature = "fluent", assertr_derive::fluent_aliases)]
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
        self.track_assertion();

        let actual = self.actual();
        let expected = expected.borrow();

        if !matches!(actual.partial_cmp(expected), Some(Ordering::Less)) {
            let actual = self.render().value(actual);
            let expected = self.render().value(expected);
            self.fail(|w: &mut String| {
                writedoc! {w, r"
                    Actual: {actual:#?}

                    is not less than

                    Expected: {expected:#?}
                "}
            });
        }
        self
    }

    #[track_caller]
    fn is_greater_than<E>(self, expected: impl Borrow<E>) -> Self
    where
        R: ValueRenderer<T> + ValueRenderer<E>,
        T: PartialOrd<E>,
    {
        self.track_assertion();

        let actual = self.actual();
        let expected = expected.borrow();

        if !matches!(actual.partial_cmp(expected), Some(Ordering::Greater)) {
            let actual = self.render().value(actual);
            let expected = self.render().value(expected);
            self.fail(|w: &mut String| {
                writedoc! {w, r"
                    Actual: {actual:#?}

                    is not greater than

                    Expected: {expected:#?}
                "}
            });
        }
        self
    }

    #[track_caller]
    fn is_less_or_equal_to<E>(self, expected: impl Borrow<E>) -> Self
    where
        R: ValueRenderer<T> + ValueRenderer<E>,
        T: PartialOrd<E>,
    {
        self.track_assertion();

        let actual = self.actual();
        let expected = expected.borrow();

        if !matches!(
            actual.partial_cmp(expected),
            Some(Ordering::Less | Ordering::Equal)
        ) {
            let actual = self.render().value(actual);
            let expected = self.render().value(expected);
            self.fail(|w: &mut String| {
                writedoc! {w, r"
                    Actual: {actual:#?}

                    is not less or equal to

                    Expected: {expected:#?}
                "}
            });
        }
        self
    }

    #[track_caller]
    fn is_greater_or_equal_to<E>(self, expected: impl Borrow<E>) -> Self
    where
        R: ValueRenderer<T> + ValueRenderer<E>,
        T: PartialOrd<E>,
    {
        self.track_assertion();

        let actual = self.actual();
        let expected = expected.borrow();

        if !matches!(
            actual.partial_cmp(expected),
            Some(Ordering::Greater | Ordering::Equal)
        ) {
            let actual = self.render().value(actual);
            let expected = self.render().value(expected);
            self.fail(|w: &mut String| {
                writedoc! {w, r"
                    Actual: {actual:#?}

                    is not greater or equal to

                    Expected: {expected:#?}
                "}
            });
        }
        self
    }
}

#[cfg(test)]
mod tests {
    mod renderer_contract {
        use crate::prelude::*;
        use crate::test_support::{NoRenderer, assert_trait_impl};

        #[test]
        fn trait_is_implemented_without_renderer_support() {
            assert_trait_impl!(
                AssertThat<'static, i32, Panic, NoRenderer>
                    => PartialOrdAssertions<i32, NoRenderer>
            );
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
        fn succeeds_when_less() {
            assert_that!(3).is_less_than(4);
            assert_that!(3).is_less_than(4);
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
        fn succeeds_when_greater() {
            assert_that!(7).is_greater_than(6);
            assert_that!(7).is_greater_than(6);
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
        fn succeeds_when_less() {
            assert_that!(3).is_less_or_equal_to(4);
            assert_that!(3).is_less_or_equal_to(4);
        }

        #[test]
        fn succeeds_when_equal() {
            assert_that!(3).is_less_or_equal_to(3);
            assert_that!(3).is_less_or_equal_to(3);
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
        fn succeeds_when_greater() {
            assert_that!(7).is_greater_or_equal_to(6);
            assert_that!(7).is_greater_or_equal_to(6);
        }

        #[test]
        fn succeeds_when_equal() {
            assert_that!(7).is_greater_or_equal_to(7);
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
