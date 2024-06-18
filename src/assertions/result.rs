use crate::{actual::Actual, failure::GenericFailure, AssertThat};
use std::fmt::Debug;

// Assertions for generic result values.
impl<'t, T, E> AssertThat<'t, Result<T, E>> {
    /// This is a terminal operation on the contained `Result`,
    /// as there is little meaningful to do with the result if its variant was ensured.
    /// This allows you to chain additional expectations on the contained success value.
    #[track_caller]
    pub fn is_ok(self) -> AssertThat<'t, T>
    where
        T: Debug,
        E: Debug,
    {
        if !self.actual.borrowed().is_ok() {
            self.fail(GenericFailure {
                arguments: format_args!(
                    "Actual: {actual:#?}\n\nis not of expected variant: Result:Ok",
                    actual = self.actual.borrowed()
                ),
            });
        }

        self.map(|it| match it {
            Actual::Owned(o) => Actual::Owned(o.unwrap()),
            Actual::Borrowed(b) => Actual::Borrowed(b.as_ref().unwrap()),
        })
    }

    /// This is a terminal operation on the contained `Result`,
    /// as there is little meaningful to do with the result if its variant was ensured.
    /// This allows you to chain additional expectations on the contained error value.
    #[track_caller]
    pub fn is_err(self) -> AssertThat<'t, E>
    where
        T: Debug,
        E: Debug,
    {
        if !self.actual.borrowed().is_err() {
            self.fail(GenericFailure {
                arguments: format_args!(
                    "Actual: {actual:#?}\n\nis not of expected variant: Result:Err",
                    actual = self.actual.borrowed()
                ),
            });
        }

        self.map(|it| match it {
            Actual::Owned(o) => Actual::Owned(o.unwrap_err()),
            Actual::Borrowed(b) => Actual::Borrowed(b.as_ref().unwrap_err()),
        })
    }
}

#[cfg(test)]
mod tests {
    use indoc::formatdoc;

    use crate::prelude::*;

    #[test]
    fn is_ok_succeeds_when_ok() {
        assert_that(Result::<(), ()>::Ok(())).is_ok();
    }

    #[test]
    fn is_ok_panics_when_error() {
        assert_that_panic_by(|| {
            assert_that(Result::<i32, String>::Err("someError".to_owned()))
                .with_location(false)
                .is_ok();
        })
        .has_type::<String>()
        .is_equal_to(formatdoc! {r#"
                -------- assertr --------
                Actual: Err(
                    "someError",
                )

                is not of expected variant: Result:Ok
                -------- assertr --------
            "#});
    }

    #[test]
    fn is_err_succeeds_when_error() {
        assert_that(Result::<(), ()>::Err(())).is_err();
    }

    #[test]
    fn is_err_panics_when_ok() {
        assert_that_panic_by(|| {
            assert_that(Result::<i32, String>::Ok(42))
                .with_location(false)
                .is_err();
        })
        .has_type::<String>()
        .is_equal_to(formatdoc! {r#"
                -------- assertr --------
                Actual: Ok(
                    42,
                )

                is not of expected variant: Result:Err
                -------- assertr --------
            "#});
    }
}
