use std::fmt::Debug;

use crate::{
    failure::GenericFailure, tracking::AssertionTracking, AssertThat, AssertrPartialEq, Mode,
};

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
        T: AssertrPartialEq<E>,
    {
        self.track_assertion();
        let expected = expected;
        if !self
            .actual()
            .into_iter()
            .any(|it| AssertrPartialEq::eq(it, &expected, None))
        {
            self.fail(GenericFailure {
                arguments: format_args!(
                    "Actual: ...\n\ndoes not contain expected key: {expected:#?}",
                    //actual = self.actual_ref(),
                ),
            });
        }
        self
    }

    // TODO: Should this exist? Should we create is_empty() impl's for concrete collection types instead?
    #[track_caller]
    fn iterator_is_empty(self) -> Self {
        self.track_assertion();
        if self.actual().into_iter().count() != 0 {
            let actual = self.actual().into_iter().collect::<Vec<_>>();
            self.fail(GenericFailure {
                arguments: format_args!(
                    "Actual: {actual:#?}\n\nIs not empty!",
                    //actual = self.actual_ref(),
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
