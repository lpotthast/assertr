use crate::{AssertThat, Mode, actual::Actual, mode::Panic, tracking::AssertionTracking};
use alloc::string::String;
use core::fmt::{Debug, Write};
use core::option::Option;
use indoc::writedoc;

/// Data-extracting assertion for `Option` values.
#[cfg_attr(feature = "fluent", assertr_derive::fluent_aliases)]
pub trait OptionExtractAssertions<'t, T> {
    /// Test if this option is of the `Some` variant.
    /// This is a terminal operation on the contained `Option`,
    /// as there is nothing meaningful to do with the option if its variant was ensured.
    /// This allows you to chain additional expectations on the contained success value.
    ///
    /// Only available in `Panic` mode, as the extracted `T` cannot be produced when the value is
    /// `None`. Use `OptionAssertions::is_some_satisfying` for capture mode.
    fn is_some(self) -> AssertThat<'t, T, Panic>
    where
        T: Debug;
}

impl<'t, T> OptionExtractAssertions<'t, T> for AssertThat<'t, Option<T>, Panic> {
    #[track_caller]
    fn is_some(self) -> AssertThat<'t, T, Panic>
    where
        T: Debug,
    {
        self.track_assertion();

        if !self.actual().is_some() {
            let actual = self.actual();
            self.fail(|w: &mut String| {
                writedoc! {w, r"
                    Actual: {actual:#?}

                    is not of expected variant: Option::Some
                "}
            });
        }

        self.map(|actual| match actual {
            Actual::Owned(o) => Actual::Owned(o.unwrap()),
            Actual::Borrowed(b) => Actual::Borrowed(b.as_ref().unwrap()),
        })
    }
}

/// Non-extracting assertions for `Option` values.
/// These work in any mode (Panic or Capture).
#[allow(clippy::return_self_not_must_use)]
#[cfg_attr(feature = "fluent", assertr_derive::fluent_aliases)]
pub trait OptionAssertions<'t, T, M: Mode> {
    /// Test if this option is of the `Some` variant, then run additional assertions on the contained value.
    fn is_some_satisfying<A>(self, assertions: A) -> Self
    where
        T: Debug,
        A: for<'a> FnOnce(AssertThat<'a, &'a T, M>);

    /// Test if this option is of the `None` variant.
    /// This is a terminal operation on the contained `Option`,
    /// as there is nothing meaningful to do with the option after its variant was ensured.
    fn is_none(self) -> AssertThat<'t, (), M>
    where
        T: Debug;
}

impl<'t, T, M: Mode> OptionAssertions<'t, T, M> for AssertThat<'t, Option<T>, M> {
    #[track_caller]
    fn is_some_satisfying<A>(self, assertions: A) -> Self
    where
        T: Debug,
        A: for<'a> FnOnce(AssertThat<'a, &'a T, M>),
    {
        self.track_assertion();

        if self.actual().is_some() {
            self.satisfies_ref(|it| it.as_ref().unwrap(), assertions)
        } else {
            let actual = self.actual();
            self.fail(|w: &mut String| {
                writedoc! {w, r"
                    Actual: {actual:#?}

                    is not of expected variant: Option::Some
                "}
            });
            self
        }
    }

    #[track_caller]
    fn is_none(self) -> AssertThat<'t, (), M>
    where
        T: Debug,
    {
        self.track_assertion();

        if !self.actual().is_none() {
            let actual = self.actual();
            self.fail(|w: &mut String| {
                writedoc! {w, r"
                    Actual: {actual:#?}

                    is not of expected variant: Option::None
                "}
            });
        }

        self.map(|_actual| Actual::Owned(()))
    }
}

#[cfg(test)]
mod tests {
    mod is_some {
        use crate::prelude::*;
        use indoc::formatdoc;

        #[test]
        fn succeeds_when_some() {
            assert_that!(Option::<i32>::Some(42))
                .is_some()
                .is_equal_to(42);
        }

        #[test]
        fn panics_when_none() {
            assert_that_panic_by(|| {
                assert_that!(Option::<i32>::None)
                    .with_location(false)
                    .is_some()
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {"
                -------- assertr --------
                Actual: None

                is not of expected variant: Option::Some
                -------- assertr --------
            "});
        }
    }

    mod is_some_satisfying {
        use crate::prelude::*;
        use indoc::formatdoc;

        #[test]
        fn succeeds_when_some_and_assertions_pass() {
            assert_that!(Option::<i32>::Some(42)).is_some_satisfying(|some| {
                some.is_equal_to(&42);
            });
        }

        #[test]
        fn captures_inner_failure_when_some_and_assertion_fails() {
            let failures = assert_that!(Option::<i32>::Some(42))
                .with_capture()
                .with_location(false)
                .is_some_satisfying(|some| {
                    some.is_greater_than(&9000);
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

        #[test]
        fn captures_variant_failure_when_none() {
            let failures = assert_that!(Option::<i32>::None)
                .with_capture()
                .with_location(false)
                .is_some_satisfying(|_| panic!("assertions should not run"))
                .capture_failures();

            assert_that!(failures).contains_exactly::<String>([formatdoc! {"
                -------- assertr --------
                Actual: None

                is not of expected variant: Option::Some
                -------- assertr --------
            "}]);
        }

        #[test]
        fn panics_when_none() {
            assert_that_panic_by(|| {
                assert_that!(Option::<i32>::None)
                    .with_location(false)
                    .is_some_satisfying(|_| panic!("assertions should not run"))
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {"
                -------- assertr --------
                Actual: None

                is not of expected variant: Option::Some
                -------- assertr --------
            "});
        }
    }

    mod is_none {
        use crate::prelude::*;
        use alloc::string::String;
        use indoc::formatdoc;

        #[test]
        fn succeeds_when_none() {
            assert_that!(Option::<i32>::None).is_none();
        }

        #[test]
        fn panics_when_some() {
            assert_that_panic_by(|| {
                assert_that!(Option::<i32>::Some(42))
                    .with_location(false)
                    .is_none()
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {"
                -------- assertr --------
                Actual: Some(
                    42,
                )

                is not of expected variant: Option::None
                -------- assertr --------
            "});
        }
    }
}
