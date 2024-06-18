use crate::{failure::GenericFailure, AssertThat};
use std::{borrow::Borrow, fmt::Debug};

pub trait IntoIteratorAssertions<'t, T: PartialEq + Debug> {
    fn contains<E: Borrow<T>>(self, expected: E) -> Self;
    fn iterator_is_empty(self) -> Self;
}

impl<'t, T: PartialEq + Debug, I> IntoIteratorAssertions<'t, T> for AssertThat<'t, I>
where
    for<'any> &'any I: IntoIterator<Item = &'any T>,
{
    #[track_caller]
    fn contains<E: Borrow<T>>(self, expected: E) -> Self {
        let expected = expected.borrow();
        if !self.actual.borrowed().into_iter().any(|it| it == expected) {
            self.fail(GenericFailure {
                arguments: format_args!(
                    "Actual: ...\n\ndoes not contain expected key: {expected:#?}",
                    //actual = self.actual.borrowed(),
                ),
            });
        }
        self
    }

    // TODO: Should this exist? Should we create is_empty() impl's for concrete collection types instead?
    #[track_caller]
    fn iterator_is_empty(self) -> Self {
        let count = self.actual.borrowed().into_iter().count();

        let actual = self.actual.borrowed().into_iter().collect::<Vec<_>>();

        if count != 0 {
            self.fail(GenericFailure {
                arguments: format_args!(
                    "Actual: {actual:#?}\n\nIs not empty!",
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
