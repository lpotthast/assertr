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

    /// Passes this subject to `mapper`, which returns a new owned or borrowed subject.
    pub fn map<U>(self, mapper: impl Fn(Self) -> Actual<'t, U>) -> Actual<'t, U> {
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
