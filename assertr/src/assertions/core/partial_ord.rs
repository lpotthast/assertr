use core::{borrow::Borrow, cmp::Ordering, fmt::Debug};

use crate::{failure::GenericFailure, tracking::AssertionTracking, AssertThat, Mode};

/// Assertions for comparable values.
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

impl<'t, T: Debug, M: Mode> PartialOrdAssertions<T> for AssertThat<'t, T, M> {
    #[track_caller]
    fn is_less_than<E>(self, expected: impl Borrow<E>) -> Self
    where
        E: Debug,
        T: PartialOrd<E>,
    {
        self.track_assertion();

        let actual = self.actual();
        let expected = expected.borrow();

        if matches!(
            actual.partial_cmp(expected),
            Some(Ordering::Equal) | Some(Ordering::Greater)
        ) {
            self.fail(GenericFailure {
                arguments: format_args!(
                    "Actual: {actual:#?}\n\nis not less than\n\nExpected: {expected:#?}"
                ),
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

        if matches!(
            actual.partial_cmp(expected),
            Some(Ordering::Less) | Some(Ordering::Equal)
        ) {
            self.fail_with_arguments(format_args!(
                "Actual: {actual:#?}\n\nis not greater than\n\nExpected: {expected:#?}"
            ));
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

        if matches!(actual.partial_cmp(expected), Some(Ordering::Greater)) {
            self.fail(GenericFailure {
                arguments: format_args!(
                    "Actual: {actual:#?}\n\nis not less or equal to\n\nExpected: {expected:#?}"
                ),
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

        if matches!(actual.partial_cmp(expected), Some(Ordering::Less)) {
            self.fail(GenericFailure {
                arguments: format_args!(
                    "Actual: {actual:#?}\n\nis not greater or equal to\n\nExpected: {expected:#?}"
                ),
            });
        }
        self
    }
}

#[cfg(test)]
mod tests {
    use crate::prelude::*;

    #[test]
    fn is_less_than_succeeds_when_less() {
        assert_that(3).is_less_than(4);
        assert_that(3).is_less_than(&4);
    }

    #[test]
    fn is_greater_than_succeeds_when_greater() {
        assert_that(7).is_greater_than(6);
        assert_that(7).is_greater_than(&6);
    }

    #[test]
    fn is_less_or_equal_to_than_succeeds_when_less() {
        assert_that(3).is_less_or_equal_to(4);
        assert_that(3).is_less_or_equal_to(&4);
    }

    #[test]
    fn is_less_or_equal_to_than_succeeds_when_equal() {
        assert_that(3).is_less_or_equal_to(3);
        assert_that(3).is_less_or_equal_to(&3);
    }

    #[test]
    fn is_greater_or_equal_to_succeeds_when_greater() {
        assert_that(7).is_greater_or_equal_to(6);
        assert_that(7).is_greater_or_equal_to(&6);
    }

    #[test]
    fn is_greater_or_equal_to_succeeds_when_equal() {
        assert_that(7).is_greater_or_equal_to(7);
        assert_that(7).is_greater_or_equal_to(&7);
    }
}
