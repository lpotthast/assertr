use alloc::string::String;
use core::{
    borrow::Borrow,
    cmp::Ordering,
    fmt::{Debug, Write},
};
use indoc::writedoc;

use crate::{AssertThat, Mode, tracking::AssertionTracking};

/// Assertions for comparable values.
#[allow(clippy::return_self_not_must_use)]
#[cfg_attr(feature = "fluent", assertr_derive::fluent_aliases)]
pub trait PartialOrdAssertions<T> {
    fn is_less_than<E>(self, expected: impl Borrow<E>) -> Self
    where
        E: Debug,
        T: PartialOrd<E>;

    fn is_greater_than<E>(self, expected: impl Borrow<E>) -> Self
    where
        E: Debug,
        T: PartialOrd<E>;

    fn is_less_or_equal_to<E>(self, expected: impl Borrow<E>) -> Self
    where
        E: Debug,
        T: PartialOrd<E>;

    fn is_greater_or_equal_to<E>(self, expected: impl Borrow<E>) -> Self
    where
        E: Debug,
        T: PartialOrd<E>;
}

impl<T: Debug, M: Mode> PartialOrdAssertions<T> for AssertThat<'_, T, M> {
    #[track_caller]
    fn is_less_than<E>(self, expected: impl Borrow<E>) -> Self
    where
        E: Debug,
        T: PartialOrd<E>,
    {
        self.track_assertion();

        let actual = self.actual();
        let expected = expected.borrow();

        if !matches!(actual.partial_cmp(expected), Some(Ordering::Less)) {
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
        E: Debug,
        T: PartialOrd<E>,
    {
        self.track_assertion();

        let actual = self.actual();
        let expected = expected.borrow();

        if !matches!(actual.partial_cmp(expected), Some(Ordering::Greater)) {
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
        E: Debug,
        T: PartialOrd<E>,
    {
        self.track_assertion();

        let actual = self.actual();
        let expected = expected.borrow();

        if !matches!(
            actual.partial_cmp(expected),
            Some(Ordering::Less | Ordering::Equal)
        ) {
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
        E: Debug,
        T: PartialOrd<E>,
    {
        self.track_assertion();

        let actual = self.actual();
        let expected = expected.borrow();

        if !matches!(
            actual.partial_cmp(expected),
            Some(Ordering::Greater | Ordering::Equal)
        ) {
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
    use crate::prelude::*;
    use indoc::formatdoc;

    #[test]
    fn is_less_than_succeeds_when_less() {
        assert_that!(3).is_less_than(4);
        assert_that!(3).is_less_than(&4);
    }

    #[test]
    fn is_greater_than_succeeds_when_greater() {
        assert_that!(7).is_greater_than(6);
        assert_that!(7).is_greater_than(&6);
    }

    #[test]
    fn is_less_or_equal_to_than_succeeds_when_less() {
        assert_that!(3).is_less_or_equal_to(4);
        assert_that!(3).is_less_or_equal_to(&4);
    }

    #[test]
    fn is_less_or_equal_to_than_succeeds_when_equal() {
        assert_that!(3).is_less_or_equal_to(3);
        assert_that!(3).is_less_or_equal_to(&3);
    }

    #[test]
    fn is_greater_or_equal_to_succeeds_when_greater() {
        assert_that!(7).is_greater_or_equal_to(6);
        assert_that!(7).is_greater_or_equal_to(&6);
    }

    #[test]
    fn is_greater_or_equal_to_succeeds_when_equal() {
        assert_that!(7).is_greater_or_equal_to(7);
        assert_that!(7).is_greater_or_equal_to(&7);
    }

    #[test]
    fn is_less_than_panics_when_values_are_not_comparable() {
        assert_that_panic_by(|| {
            assert_that!(f32::NAN)
                .with_location(false)
                .is_less_than(0.0)
        })
        .has_type::<String>()
        .is_equal_to(formatdoc! {r#"
            -------- assertr --------
            Actual: NaN

            is not less than

            Expected: 0.0
            -------- assertr --------
        "#});
    }

    #[test]
    fn is_greater_than_panics_when_values_are_not_comparable() {
        assert_that_panic_by(|| {
            assert_that!(f32::NAN)
                .with_location(false)
                .is_greater_than(0.0)
        })
        .has_type::<String>()
        .is_equal_to(formatdoc! {r#"
            -------- assertr --------
            Actual: NaN

            is not greater than

            Expected: 0.0
            -------- assertr --------
        "#});
    }

    #[test]
    fn is_less_or_equal_to_panics_when_values_are_not_comparable() {
        assert_that_panic_by(|| {
            assert_that!(f32::NAN)
                .with_location(false)
                .is_less_or_equal_to(0.0)
        })
        .has_type::<String>()
        .is_equal_to(formatdoc! {r#"
            -------- assertr --------
            Actual: NaN

            is not less or equal to

            Expected: 0.0
            -------- assertr --------
        "#});
    }

    #[test]
    fn is_greater_or_equal_to_panics_when_values_are_not_comparable() {
        assert_that_panic_by(|| {
            assert_that!(f32::NAN)
                .with_location(false)
                .is_greater_or_equal_to(0.0)
        })
        .has_type::<String>()
        .is_equal_to(formatdoc! {r#"
            -------- assertr --------
            Actual: NaN

            is not greater or equal to

            Expected: 0.0
            -------- assertr --------
        "#});
    }
}
