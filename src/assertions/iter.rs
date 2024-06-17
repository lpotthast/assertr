use crate::{failure::GenericFailure, AssertThat};
use std::{borrow::Borrow, fmt::Debug};

pub trait IntoIteratorAssertions<'t, T: PartialEq + Debug> {
    fn contains<E: Borrow<T>>(self, expected: E) -> Self;
}

impl<'t, T: PartialEq + Debug, I> IntoIteratorAssertions<'t, T> for AssertThat<'t, I>
where
    for<'any> &'any I: IntoIterator<Item = &'any T>,
{
    #[track_caller]
    fn contains<E: Borrow<T>>(self, expected: E) -> Self {
        let expected = expected.borrow();
        if let None = self
            .actual
            .borrowed()
            .into_iter()
            .find(|it| *it == expected)
        {
            self.fail_with(GenericFailure {
                arguments: format_args!(
                    "Actual: ...\n\ndoes not contain expected key: {expected:#?}",
                    //actual = self.actual.borrowed(),
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
}
