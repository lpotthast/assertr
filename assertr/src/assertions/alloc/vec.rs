use alloc::vec::Vec;
use core::fmt::Debug;

use crate::{AssertThat, AssertrPartialEq, Mode, prelude::SliceAssertions};

pub trait VecAssertions<'t, T: Debug> {
    fn contain<E>(self, expected: E) -> Self
    where
        E: Debug,
        T: AssertrPartialEq<E> + Debug;

    fn contains<E>(self, expected: E) -> Self
    where
        E: Debug,
        T: AssertrPartialEq<E> + Debug,
        Self: Sized,
    {
        self.contain(expected)
    }

    fn contain_exactly<E>(self, expected: impl AsRef<[E]>) -> Self
    where
        E: Debug + 't,
        T: AssertrPartialEq<E> + Debug;

    fn contains_exactly<E>(self, expected: impl AsRef<[E]>) -> Self
    where
        E: Debug + 't,
        T: AssertrPartialEq<E> + Debug,
        Self: Sized,
    {
        self.contain_exactly(expected)
    }

    /// [P] - Predicate
    fn contain_exactly_matching_in_any_order<P>(self, expected: impl AsRef<[P]>) -> Self
    where
        P: Fn(&T) -> bool;

    /// [P] - Predicate
    fn contains_exactly_matching_in_any_order<P>(self, expected: impl AsRef<[P]>) -> Self
    where
        P: Fn(&T) -> bool,
        Self: Sized,
    {
        self.contain_exactly_matching_in_any_order(expected)
    }
}

impl<'t, T: Debug, M: Mode> VecAssertions<'t, T> for AssertThat<'t, Vec<T>, M> {
    #[track_caller]
    fn contain<E>(self, expected: E) -> Self
    where
        E: Debug,
        T: AssertrPartialEq<E> + Debug,
    {
        self.derive(|it| it.as_slice()).contains(expected);
        self
    }

    #[track_caller]
    fn contain_exactly<E>(self, expected: impl AsRef<[E]>) -> Self
    where
        E: Debug + 't,
        T: AssertrPartialEq<E> + Debug,
    {
        self.derive(|it| it.as_slice()).contains_exactly(expected);
        self
    }

    #[track_caller]
    fn contain_exactly_matching_in_any_order<P>(self, expected: impl AsRef<[P]>) -> Self
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
