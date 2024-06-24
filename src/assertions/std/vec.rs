use crate::{AssertThat, Mode};
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
        T: PartialEq<E> + Debug,
    {
        self.derive(|it| it.as_slice()).contains_exactly(expected);
        self
    }
}

// TODO: Tests
