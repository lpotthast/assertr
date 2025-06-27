use core::{borrow::Borrow, cmp::Ordering, fmt::Debug};

use crate::{AssertThat, Mode, tracking::AssertionTracking};

/// Assertions for comparable values.
pub trait PartialOrdAssertions<T> {
    fn is_less_than<E>(self, expected: impl Borrow<E>) -> Self
    where
        E: Debug,
        T: PartialOrd<E>;

    fn be_less_than<E>(self, expected: impl Borrow<E>) -> Self
    where
        E: Debug,
        T: PartialOrd<E>,
        Self: Sized,
    {
        self.is_less_than(expected)
    }

    fn is_greater_than<E>(self, expected: impl Borrow<E>) -> Self
    where
        E: Debug,
        T: PartialOrd<E>;

    fn be_greater_than<E>(self, expected: impl Borrow<E>) -> Self
    where
        E: Debug,
        T: PartialOrd<E>,
        Self: Sized,
    {
        self.is_greater_than(expected)
    }

    fn is_less_or_equal_to<E>(self, expected: impl Borrow<E>) -> Self
    where
        E: Debug,
        T: PartialOrd<E>;

    fn be_less_or_equal_to<E>(self, expected: impl Borrow<E>) -> Self
    where
        E: Debug,
        T: PartialOrd<E>,
        Self: Sized,
    {
        self.is_less_or_equal_to(expected)
    }

    fn is_greater_or_equal_to<E>(self, expected: impl Borrow<E>) -> Self
    where
        E: Debug,
        T: PartialOrd<E>;

    fn be_greater_or_equal_to<E>(self, expected: impl Borrow<E>) -> Self
    where
        E: Debug,
        T: PartialOrd<E>,
        Self: Sized,
    {
        self.is_greater_or_equal_to(expected)
    }
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

        if matches!(
            actual.partial_cmp(expected),
            Some(Ordering::Equal) | Some(Ordering::Greater)
        ) {
            self.fail(format_args!(
                "Actual: {actual:#?}\n\nis not less than\n\nExpected: {expected:#?}\n"
            ));
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
            self.fail(format_args!(
                "Actual: {actual:#?}\n\nis not greater than\n\nExpected: {expected:#?}\n"
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
            self.fail(format_args!(
                "Actual: {actual:#?}\n\nis not less or equal to\n\nExpected: {expected:#?}\n"
            ));
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
            self.fail(format_args!(
                "Actual: {actual:#?}\n\nis not greater or equal to\n\nExpected: {expected:#?}\n"
            ));
        }
        self
    }
}

#[cfg(test)]
mod tests {
    use crate::prelude::*;

    #[test]
    fn is_less_than_succeeds_when_less() {
        3.must().be_less_than(4);
        3.must().be_less_than(&4);
    }

    #[test]
    fn is_greater_than_succeeds_when_greater() {
        7.must().be_greater_than(6);
        7.must().be_greater_than(&6);
    }

    #[test]
    fn is_less_or_equal_to_than_succeeds_when_less() {
        3.must().be_less_or_equal_to(4);
        3.must().be_less_or_equal_to(&4);
    }

    #[test]
    fn is_less_or_equal_to_than_succeeds_when_equal() {
        3.must().be_less_or_equal_to(3);
        3.must().be_less_or_equal_to(&3);
    }

    #[test]
    fn is_greater_or_equal_to_succeeds_when_greater() {
        7.must().be_greater_or_equal_to(6);
        7.must().be_greater_or_equal_to(&6);
    }

    #[test]
    fn is_greater_or_equal_to_succeeds_when_equal() {
        7.must().be_greater_or_equal_to(7);
        7.must().be_greater_or_equal_to(&7);
    }
}
