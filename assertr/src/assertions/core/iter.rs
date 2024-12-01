use alloc::vec::Vec;
use core::fmt::Debug;

use crate::actual::Actual;
use crate::{tracking::AssertionTracking, AssertThat, AssertrPartialEq, Mode};

pub trait IteratorAssertions<'t, T: Debug, M: Mode> {
    /// This is a terminal assertion, as it must consume the underlying iterator.
    fn contains<'u, E>(self, expected: E) -> AssertThat<'u, (), M>
    where
        E: Debug,
        T: AssertrPartialEq<E>,
        't: 'u;
}

impl<'t, T, I, M: Mode> IteratorAssertions<'t, T, M> for AssertThat<'t, I, M>
where
    T: Debug,
    I: Iterator<Item = T>,
{
    #[track_caller]
    fn contains<'u, E>(self, expected: E) -> AssertThat<'u, (), M>
    where
        E: Debug,
        T: Debug + AssertrPartialEq<E>,
        't: 'u
    {
        self.track_assertion();
        let (actual, this) = self.replace_actual_with(Actual::Owned(()));

        let actual = actual.unwrap_owned().collect::<Vec<_>>();
        let expected = expected;
        if !actual
            .iter()
            .any(|it| AssertrPartialEq::eq(it, &expected, None))
        {
            this.fail(format_args!(
                "Actual: {actual:#?}\n\ndoes not contain expected: {expected:#?}\n",
            ));
        }
        this
    }
}

pub trait IntoIteratorAssertions<T: Debug> {
    fn contains<E>(self, expected: E) -> Self
    where
        E: Debug,
        T: AssertrPartialEq<E>;

    fn iterator_is_empty(self) -> Self;
}

impl<'t, T, I, M: Mode> IntoIteratorAssertions<T> for AssertThat<'t, I, M>
where
    T: Debug,
    for<'any> &'any I: IntoIterator<Item = &'any T>,
{
    #[track_caller]
    fn contains<E>(self, expected: E) -> Self
    where
        E: Debug,
        T: Debug + AssertrPartialEq<E>,
    {
        self.track_assertion();
        let actual = self.actual().into_iter().collect::<Vec<_>>();
        let expected = expected;
        if !self
            .actual()
            .into_iter()
            .any(|it| AssertrPartialEq::eq(it, &expected, None))
        {
            self.fail(format_args!(
                "Actual: {actual:#?}\n\ndoes not contain expected: {expected:#?}\n",
            ));
        }
        self
    }

    // TODO: Should this exist? Should we create is_empty() impl's for concrete collection types instead?
    #[track_caller]
    fn iterator_is_empty(self) -> Self {
        self.track_assertion();
        if self.actual().into_iter().count() != 0 {
            let actual = self.actual().into_iter().collect::<Vec<_>>();
            self.fail(format_args!(
                "Actual: {actual:#?}\n\nIs not empty!\n",
                //actual = self.actual_ref(),
            ));
        }
        self
    }
}

#[cfg(test)]
mod tests {
    use crate::prelude::*;

    #[test]
    fn contains_succeeds_when_value_is_present() {
        let values = vec![1, 2, 3, 42];
        assert_that(values)
            .contains(1)
            .contains(42)
            .contains(3)
            .contains(2);
    }

    #[test]
    fn compiles_for_comparable_but_different_type() {
        let values = vec!["foo"];
        assert_that(values).contains("foo".to_string());
    }
}
