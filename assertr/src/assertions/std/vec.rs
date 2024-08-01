use crate::{AssertrPartialEq, AssertThat, Mode};
use std::fmt::Debug;

impl<'t, T: Debug, M: Mode> AssertThat<'t, Vec<T>, M> {
    #[track_caller]
    pub fn is_empty(self) -> Self {
        self.derive(|it| it.as_slice()).is_empty();
        self
    }

    #[track_caller]
    pub fn contains_exactly<E>(self, expected: impl AsRef<[E]>) -> Self
    where
        E: Debug + 't,
        T: AssertrPartialEq<E> + Debug,
    {
        self.derive(|it| it.as_slice()).contains_exactly(expected);
        self
    }

    #[track_caller]
    pub fn contains_exactly_matching_in_any_order<P>(self, expected: impl AsRef<[P]>) -> Self
    where
        P: Fn(&T) -> bool, // predicate
    {
        self.derive(|it| it.as_slice())
            .contains_exactly_matching_in_any_order(expected);
        self
    }
}

// TODO: Tests
