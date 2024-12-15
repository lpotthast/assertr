use alloc::vec::Vec;
use core::fmt::Debug;

use crate::{prelude::SliceAssertions, AssertThat, AssertrPartialEq, Mode};

pub trait VecAssertions<'t, T: Debug> {
    fn contains<E>(self, expected: E) -> Self
    where
        E: Debug,
        T: AssertrPartialEq<E> + Debug;

    fn contains_exactly<E>(self, expected: impl AsRef<[E]>) -> Self
    where
        E: Debug + 't,
        T: AssertrPartialEq<E> + Debug;

    /// [P] - Predicate
    fn contains_exactly_matching_in_any_order<P>(self, expected: impl AsRef<[P]>) -> Self
    where
        P: Fn(&T) -> bool;
}

impl<'t, T: Debug, M: Mode> VecAssertions<'t, T> for AssertThat<'t, Vec<T>, M> {
    #[track_caller]
    fn contains<E>(self, expected: E) -> Self
    where
        E: Debug,
        T: AssertrPartialEq<E> + Debug,
    {
        self.derive(|it| it.as_slice()).contains(expected);
        self
    }

    #[track_caller]
    fn contains_exactly<E>(self, expected: impl AsRef<[E]>) -> Self
    where
        E: Debug + 't,
        T: AssertrPartialEq<E> + Debug,
    {
        self.derive(|it| it.as_slice()).contains_exactly(expected);
        self
    }

    #[track_caller]
    fn contains_exactly_matching_in_any_order<P>(self, expected: impl AsRef<[P]>) -> Self
    where
        P: Fn(&T) -> bool, // predicate
    {
        self.derive(|it| it.as_slice())
            .contains_exactly_matching_in_any_order(expected);
        self
    }
}

// TODO: Tests

#[cfg(test)]
mod tests {
    use crate::prelude::*;

    #[test]
    fn assert_vec_contains_exactly() {
        let vec = vec![1, 2, 3];

        assert_that(vec).into_iter_contains_exactly([1, 2, 3]);
    }
}

