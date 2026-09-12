//! Owned and borrowed assertion subjects.

/// Either a borrowed or an owned assertion subject.
///
/// Which one it is stays hidden behind `AssertThat<T>`: assertion methods are looked up by `T`
/// alone. Only assertions that consume their subject need the distinction.
pub enum Actual<'t, T> {
    /// Borrowed data.
    Borrowed(&'t T),

    /// Owned data.
    Owned(T),
}

impl<'t, T> Actual<'t, T> {
    /// Unwraps the owned subject.
    ///
    /// # Panics
    ///
    /// Panics if the value is borrowed rather than owned.
    #[track_caller]
    pub fn unwrap_owned(self) -> T {
        match self {
            Actual::Borrowed(_t) => panic!(
                "Cannot unwrap a borrowed value. Create the assertion with `assert_that_owned!(...)` (or `.must_owned()`) instead."
            ),
            Actual::Owned(t) => t,
        }
    }

    /// Borrows the subject, regardless of whether it is stored by value or by reference.
    pub fn borrowed(&self) -> &T {
        match self {
            Actual::Borrowed(t) => t,
            Actual::Owned(t) => t,
        }
    }

    /// Passes this subject to `mapper` exactly once, returning its new owned or borrowed subject.
    pub fn map<U>(self, mapper: impl FnOnce(Self) -> Actual<'t, U>) -> Actual<'t, U> {
        mapper(self)
    }
}

impl<T> From<T> for Actual<'_, T> {
    fn from(value: T) -> Self {
        Actual::Owned(value)
    }
}

impl<'t, T> From<&'t T> for Actual<'t, T> {
    fn from(value: &'t T) -> Self {
        Actual::Borrowed(value)
    }
}

impl<T> AsRef<T> for Actual<'_, T> {
    fn as_ref(&self) -> &T {
        self.borrowed()
    }
}

#[cfg(test)]
mod tests {
    mod map {
        use alloc::string::String;

        use crate::{actual::Actual, prelude::*};

        #[test]
        fn moves_captured_non_clone_value_into_result() {
            struct NonClone(String);

            let captured = NonClone(String::from("captured"));
            let mapped =
                Actual::Owned(42).map(|actual| Actual::Owned((actual.unwrap_owned(), captured)));
            let (subject, captured) = mapped.unwrap_owned();

            assert_that!(subject).is_equal_to(42);
            assert_that!(captured.0).is_equal_to("captured");
        }

        #[test]
        fn maps_owned_subject() {
            let mapped = Actual::Owned(String::from("owned"))
                .map(|actual| Actual::Owned(actual.unwrap_owned().into_bytes()));

            assert_that!(mapped.unwrap_owned()).contains_exactly(*b"owned");
        }

        #[test]
        fn maps_borrowed_subject() {
            let subject = (String::from("borrowed"), 42);
            let mapped = Actual::Borrowed(&subject).map(|actual| match actual {
                Actual::Borrowed(value) => Actual::Borrowed(&value.0),
                Actual::Owned(_) => panic!("Expected a borrowed subject"),
            });

            assert_that!(matches!(mapped, Actual::Borrowed(_))).is_true();
            assert_that!(mapped.borrowed()).is_same_instance_as(&subject.0);
        }
    }
}
