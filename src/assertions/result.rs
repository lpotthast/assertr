use crate::{
    actual::Actual, failure::GenericFailure, AssertThat, tracking::AssertionTracking, Capture, Panic,
};
use std::fmt::Debug;

// Assertions for generic result values.
impl<'t, T, E> AssertThat<'t, Result<T, E>, Panic> {
    /// This is a terminal operation on the contained `Result`,
    /// as there is little meaningful to do with the result if its variant was ensured.
    /// This allows you to chain additional expectations on the contained success value.
    #[track_caller]
    pub fn is_ok(self) -> AssertThat<'t, T, Panic>
    where
        T: Debug,
        E: Debug,
    {
        self.track_assertion();

        if self.actual().is_err() {
            self.fail(GenericFailure {
                arguments: format_args!(
                    "Actual: {actual:#?}\n\nis not of expected variant: Result:Ok",
                    actual = self.actual()
                ),
            });
        }

        // Calling `unwrap` is safe here, as we would have seen a panic when the the error is not present!
        self.map(|it| match it {
            Actual::Owned(o) => Actual::Owned(o.unwrap()),
            Actual::Borrowed(b) => Actual::Borrowed(b.as_ref().unwrap()),
        })
    }

    /// This is a terminal operation on the contained `Result`,
    /// as there is little meaningful to do with the result if its variant was ensured.
    /// This allows you to chain additional expectations on the contained error value.
    #[track_caller]
    pub fn is_err(self) -> AssertThat<'t, E, Panic>
    where
        T: Debug,
        E: Debug,
    {
        self.track_assertion();

        if self.actual().is_ok() {
            self.fail(GenericFailure {
                arguments: format_args!(
                    "Actual: {actual:#?}\n\nis not of expected variant: Result:Err",
                    actual = self.actual()
                ),
            });
        }

        // Calling `unwrap_err` is safe here, as we would have seen a panic when the the error is not present!
        self.map(|it| match it {
            Actual::Owned(o) => Actual::Owned(o.unwrap_err()),
            Actual::Borrowed(b) => Actual::Borrowed(b.as_ref().unwrap_err()),
        })
    }
}

impl<'t, T, E> AssertThat<'t, Result<T, E>, Capture> {
    #[track_caller]
    pub fn is_ok_satisfying<A>(self, assertions: A) -> Self
    where
        T: Debug,
        E: Debug,
        A: for<'a> FnOnce(AssertThat<'a, &'a T, Capture>),
    {
        self.track_assertion();

        if self.actual().is_ok() {
            self.satisfies_ref(|it| it.as_ref().unwrap(), assertions)
        } else {
            self.fail(GenericFailure {
                arguments: format_args!(
                    "Actual: {actual:#?}\n\nis not of expected variant: Result:Ok",
                    actual = self.actual()
                ),
            });
            self
        }
    }

    #[track_caller]
    pub fn is_err_satisfying<A>(self, assertions: A) -> Self
    where
        T: Debug,
        E: Debug,
        A: for<'a> FnOnce(AssertThat<'a, &'a E, Capture>),
    {
        self.track_assertion();

        if self.actual().is_err() {
            self.satisfies_ref(|it| it.as_ref().unwrap_err(), assertions)
        } else {
            self.fail(GenericFailure {
                arguments: format_args!(
                    "Actual: {actual:#?}\n\nis not of expected variant: Result:Err",
                    actual = self.actual()
                ),
            });
            self
        }
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

    #[test]
    fn is_ok_satisfying_succeeds_when_ok() {
        assert_that(Result::<i32, ()>::Ok(42))
            .with_location(false)
            .with_capture()
            .is_ok_satisfying(|ok_value| {
                ok_value.is_greater_than(&9000);
            })
            .capture_failures()
            .assert_that_it()
            .contains_exactly::<String>([formatdoc! {"
                -------- assertr --------
                Actual: 42

                is not greater than

                Expected: 9000
                -------- assertr --------
            "}]);
    }
}
