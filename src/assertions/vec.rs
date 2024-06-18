use crate::AssertThat;
use std::fmt::Debug;

impl<'t, T: Debug> AssertThat<'t, Vec<T>> {
    #[track_caller]
    pub fn is_empty(self) -> Self {
        self.derive(|it| it.as_slice()).is_empty();
        self
    }

    #[track_caller]
    pub fn contains_exactly<E, EE>(self, expected: EE) -> Self
    where
        E: Debug + 't,
        EE: AsRef<[E]>,
        T: PartialEq<E> + Debug,
    {
        self.derive(|it| it.as_slice()).contains_exactly(expected);
        self
    }
}

// TODO: Tests
