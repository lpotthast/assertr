use crate::{actual::Actual, failure::GenericFailure, AssertThat};
use std::fmt::Debug;

// Assertions for generic optional values.
impl<'t, T> AssertThat<'t, Option<T>> {
    /// This is a terminal operation on the contained `Option`,
    /// as there is little meaningful to do with the option if its variant was ensured.
    /// This allows you to chain additional expectations on the contained success value.
    #[track_caller]
    pub fn is_some(self) -> AssertThat<'t, T>
    where
        T: Debug,
    {
        if !self.actual.borrowed().is_some() {
            self.fail_with(GenericFailure {
                arguments: format_args!(
                    "Actual: {actual:#?}\n\nis not of expected variant: Option:Some",
                    actual = self.actual.borrowed()
                ),
            });
        }

        self.map(|actual| match actual {
            Actual::Owned(o) => Actual::Owned(o.unwrap()),
            Actual::Borrowed(b) => Actual::Borrowed(b.as_ref().unwrap()),
        })
    }

    /// This is a terminal operation on the contained `Option`,
    /// as there is little meaningful to do with the option if its variant was ensured.
    /// This allows you to chain additional expectations on the contained error value.
    #[track_caller]
    pub fn is_none(self) -> AssertThat<'t, ()>
    where
        T: Debug,
    {
        if !self.actual.borrowed().is_none() {
            self.fail_with(GenericFailure {
                arguments: format_args!(
                    "Actual: {actual:#?}\n\nis not of expected variant: Option:None",
                    actual = self.actual.borrowed()
                ),
            });
        }

        self.map(|_actual| Actual::Owned(()))
    }
}

#[cfg(test)]
mod tests {
    use indoc::formatdoc;

    use crate::{assert_that, assert_that_panic_by};

    #[test]
    fn test_option_is_some() {
        assert_that(Option::<i32>::Some(42)).is_some();
    }

    #[test]
    fn option_is_none_succeeds_when_none() {
        assert_that(Option::<i32>::None).is_none();
    }

    #[test]
    fn option_is_none_panics_when_some() {
        assert_that_panic_by(|| {
            assert_that(Option::<i32>::Some(42))
                .with_location(false)
                .is_none()
        })
        .has_box_type::<String>()
        .has_debug_value(formatdoc! {"
                -------- assertr --------
                Actual: Some(
                    42,
                )

                is not of expected variant: Option:None
                -------- assertr --------
            "});
    }
}
