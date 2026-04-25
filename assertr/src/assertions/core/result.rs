use crate::{
    AssertThat,
    actual::Actual,
    mode::{Mode, Panic},
    tracking::AssertionTracking,
};
use alloc::string::String;
use core::fmt::{Debug, Write};
use indoc::writedoc;

/// Data-extracting assertions for `Result` values.
/// These change the assertion subject type and are only available in Panic mode,
/// as they cannot produce a valid value of the extracted type when the assertion fails.
/// Use `ResultAssertions::is_ok_satisfying` / `is_err_satisfying` for capture mode.
#[cfg_attr(feature = "fluent", assertr_derive::fluent_aliases)]
pub trait ResultExtractAssertions<'t, T, E> {
    fn is_ok(self) -> AssertThat<'t, T, Panic>
    where
        T: Debug,
        E: Debug;

    fn is_err(self) -> AssertThat<'t, E, Panic>
    where
        T: Debug,
        E: Debug;
}

impl<'t, T, E> ResultExtractAssertions<'t, T, E> for AssertThat<'t, Result<T, E>, Panic> {
    /// This is a terminal operation on the contained `Result`,
    /// as there is little meaningful to do with the result if its variant was ensured.
    /// This allows you to chain additional expectations on the contained success value.
    #[track_caller]
    fn is_ok(self) -> AssertThat<'t, T, Panic>
    where
        T: Debug,
        E: Debug,
    {
        self.track_assertion();

        if self.actual().is_err() {
            let actual = self.actual();
            self.fail(|w: &mut String| {
                writedoc! {w, r"
                    Actual: {actual:#?}

                    is not of expected variant: Result:Ok
                "}
            });
        }

        // Calling `unwrap` is safe here, as we would have seen a panic when the error is not present!
        self.map(|it| match it {
            Actual::Owned(o) => Actual::Owned(o.unwrap()),
            Actual::Borrowed(b) => Actual::Borrowed(b.as_ref().unwrap()),
        })
    }

    /// This is a terminal operation on the contained `Result`,
    /// as there is little meaningful to do with the result if its variant was ensured.
    /// This allows you to chain additional expectations on the contained error value.
    #[track_caller]
    fn is_err(self) -> AssertThat<'t, E, Panic>
    where
        T: Debug,
        E: Debug,
    {
        self.track_assertion();

        if self.actual().is_ok() {
            let actual = self.actual();
            self.fail(|w: &mut String| {
                writedoc! {w, r"
                    Actual: {actual:#?}

                    is not of expected variant: Result:Err
                "}
            });
        }

        // Calling `unwrap_err` is safe here, as we would have seen a panic when the error is not present!
        self.map(|it| match it {
            Actual::Owned(o) => Actual::Owned(o.unwrap_err()),
            Actual::Borrowed(b) => Actual::Borrowed(b.as_ref().unwrap_err()),
        })
    }
}

/// Non-extracting assertions for `Result` values.
/// These work in any mode (Panic or Capture).
#[allow(clippy::return_self_not_must_use)]
#[cfg_attr(feature = "fluent", assertr_derive::fluent_aliases)]
pub trait ResultAssertions<'t, M: Mode, T, E> {
    fn is_ok_satisfying<A>(self, assertions: A) -> Self
    where
        T: Debug,
        E: Debug,
        A: for<'a> FnOnce(AssertThat<'a, &'a T, M>);

    fn is_err_satisfying<A>(self, assertions: A) -> Self
    where
        T: Debug,
        E: Debug,
        A: for<'a> FnOnce(AssertThat<'a, &'a E, M>);
}

impl<'t, M: Mode, T, E> ResultAssertions<'t, M, T, E> for AssertThat<'t, Result<T, E>, M> {
    #[track_caller]
    fn is_ok_satisfying<A>(self, assertions: A) -> Self
    where
        T: Debug,
        E: Debug,
        A: for<'a> FnOnce(AssertThat<'a, &'a T, M>),
    {
        self.track_assertion();

        if self.actual().is_ok() {
            self.satisfies_ref(|it| it.as_ref().unwrap(), assertions)
        } else {
            let actual = self.actual();
            self.fail(|w: &mut String| {
                writedoc! {w, r"
                    Actual: {actual:#?}

                    is not of expected variant: Result:Ok
                "}
            });
            self
        }
    }

    #[track_caller]
    fn is_err_satisfying<A>(self, assertions: A) -> Self
    where
        T: Debug,
        E: Debug,
        A: for<'a> FnOnce(AssertThat<'a, &'a E, M>),
    {
        self.track_assertion();

        if self.actual().is_err() {
            self.satisfies_ref(|it| it.as_ref().unwrap_err(), assertions)
        } else {
            let actual = self.actual();
            self.fail(|w: &mut String| {
                writedoc! {w, r"
                    Actual: {actual:#?}

                    is not of expected variant: Result:Err
                "}
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
        assert_that!(Result::<(), ()>::Ok(())).is_ok();
    }

    #[test]
    fn is_ok_panics_when_error() {
        assert_that_panic_by(|| {
            assert_that!(Result::<i32, String>::Err("someError".to_owned()))
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
        assert_that!(Result::<(), ()>::Err(())).is_err();
    }

    #[test]
    fn is_err_panics_when_ok() {
        assert_that_panic_by(|| {
            assert_that!(Result::<i32, String>::Ok(42))
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
        let failures = assert_that!(Result::<i32, ()>::Ok(42))
            .with_capture()
            .with_location(false)
            .is_ok_satisfying(|ok_value| {
                ok_value.is_greater_than(&9000);
            })
            .capture_failures();
        assert_that!(failures).contains_exactly::<String>([formatdoc! {"
                -------- assertr --------
                Actual: 42

                is not greater than

                Expected: 9000
                -------- assertr --------
            "}]);
    }
}
