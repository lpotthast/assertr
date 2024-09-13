use alloc::vec::Vec;
use core::fmt::Debug;

use crate::{prelude::SliceAssertions, AssertThat, AssertrPartialEq, Mode};

pub trait VecAssertions<'t, T: Debug> {
    fn is_empty(self) -> Self;

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
    fn is_empty(self) -> Self {
        self.derive(|it| it.as_slice()).is_empty();
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
    mod is_empty {
        use crate::prelude::*;
        use alloc::format;
        use alloc::string::String;
        use alloc::vec;
        use alloc::vec::Vec;
        use indoc::formatdoc;

        #[test]
        fn with_slice_succeeds_when_empty() {
            let vec = Vec::<i32>::new();
            assert_that(vec).is_empty();
        }

        #[test]
        fn with_slice_panics_when_not_empty() {
            assert_that_panic_by(|| {
                assert_that(vec![42]).with_location(false).is_empty();
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {r#"
                    -------- assertr --------
                    Actual: [42]

                    was expected to be empty, but it is not!
                    -------- assertr --------
                "#});
        }
    }
}
