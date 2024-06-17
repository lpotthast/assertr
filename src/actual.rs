pub enum Actual<'t, T> {
    /// Borrowed data.
    Borrowed(&'t T),

    /// Owned data.
    Owned(T),
}

impl<'t, T> Actual<'t, T> {
    pub fn borrowed(&'t self) -> &'t T {
        match self {
            Actual::Borrowed(t) => *t,
            Actual::Owned(t) => t,
        }
    }

    pub fn map<U>(self, mapper: impl Fn(Self) -> Actual<'t, U>) -> Actual<'t, U> {
        mapper(self)
    }
}

impl<'t, T> From<T> for Actual<'t, T> {
    fn from(value: T) -> Self {
        Actual::Owned(value)
    }
}

impl<'t, T> From<&'t T> for Actual<'t, T> {
    fn from(value: &'t T) -> Self {
        Actual::Borrowed(value)
    }
}

impl<'t, T> AsRef<T> for Actual<'t, T> {
    fn as_ref(&self) -> &T {
        self.borrowed()
    }
}
