use crate::{failure::GenericFailure, AssertThat, Mode};
use std::{cmp::Ordering, fmt::Debug};

/// Comparable
impl<'t, T: PartialOrd, M: Mode> AssertThat<'t, T, M> {
    #[track_caller]
    pub fn is_less_than(self, expected: T) -> Self
    where
        T: Debug,
    {
        let actual = self.actual();
        let expected = &expected;

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
    pub fn is_greater_than(self, expected: T) -> Self
    where
        T: Debug,
    {
        let actual = self.actual();
        let expected = &expected;

        if matches!(
            actual.partial_cmp(expected),
            Some(Ordering::Less) | Some(Ordering::Equal)
        ) {
            self.fail(GenericFailure {
                arguments: format_args!(
                    "Actual: {actual:#?}\n\nis not greater than\n\nExpected: {expected:#?}"
                ),
            });
        }
        self
    }

    #[track_caller]
    pub fn is_less_or_equal_to(self, expected: T) -> Self
    where
        T: Debug,
    {
        let actual = self.actual();
        let expected = &expected;

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
    pub fn is_greater_or_equal_to(self, expected: T) -> Self
    where
        T: Debug,
    {
        let actual = self.actual();
        let expected = &expected;

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
    }

    #[test]
    fn is_greater_than_succeeds_when_greater() {
        assert_that(7).is_greater_than(6);
    }

    #[test]
    fn is_less_or_equal_to_than_succeeds_when_less() {
        assert_that(3).is_less_or_equal_to(4);
    }

    #[test]
    fn is_less_or_equal_to_than_succeeds_when_equal() {
        assert_that(3).is_less_or_equal_to(3);
    }

    #[test]
    fn is_greater_or_equal_to_succeeds_when_greater() {
        assert_that(7).is_greater_or_equal_to(6);
    }

    #[test]
    fn is_greater_or_equal_to_succeeds_when_equal() {
        assert_that(7).is_greater_or_equal_to(7);
    }
}
