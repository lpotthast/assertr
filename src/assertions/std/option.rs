use crate::{
    actual::Actual, failure::GenericFailure, tracking::AssertionTracking, AssertThat, Mode,
};
use std::fmt::Debug;

// Assertions for generic optional values.
impl<'t, T, M: Mode> AssertThat<'t, Option<T>, M> {
    /// This is a terminal operation on the contained `Option`,
    /// as there is little meaningful to do with the option if its variant was ensured.
    /// This allows you to chain additional expectations on the contained success value.
    #[track_caller]
    pub fn is_some(self) -> AssertThat<'t, T, M>
    where
        T: Debug,
    {
        self.track_assertion();

        if !self.actual().is_some() {
            self.fail(GenericFailure {
                arguments: format_args!(
                    "Actual: {actual:#?}\n\nis not of expected variant: Option:Some",
                    actual = self.actual()
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
    pub fn is_none(self) -> AssertThat<'t, (), M>
    where
        T: Debug,
    {
        self.track_assertion();

        if !self.actual().is_none() {
            self.fail(GenericFailure {
                arguments: format_args!(
                    "Actual: {actual:#?}\n\nis not of expected variant: Option:None",
                    actual = self.actual()
                ),
            });
        }

        self.map(|_actual| Actual::Owned(()))
    }
}

// TODO: is_ok_satisfying, is_err_satisfying

#[cfg(test)]
mod tests {
    use indoc::formatdoc;

    use crate::prelude::*;

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
        .has_type::<String>()
        .is_equal_to(formatdoc! {"
                -------- assertr --------
                Actual: Some(
                    42,
                )

                is not of expected variant: Option:None
                -------- assertr --------
            "});
    }
}
