use crate::{AssertThat, Mode, actual::Actual, mode::Panic, tracking::AssertionTracking};
use core::fmt::Debug;
use core::option::Option;

/// Data-extracting assertion for `Option` values.
/// Only available in Panic mode, as the extracted `T` cannot be produced when the value is `None`.
#[cfg_attr(feature = "fluent", assertr_derive::fluent_aliases)]
pub trait OptionExtractAssertions<'t, T> {
    /// Test if this option is of the `Some` variant.
    /// This is a terminal operation on the contained `Option`,
    /// as there is nothing meaningful to do with the option if its variant was ensured.
    /// This allows you to chain additional expectations on the contained success value.
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
            self.fail(format_args!(
                "Actual: {actual:#?}\n\nis not of expected variant: Option:Some\n",
                actual = self.actual()
            ));
        }

        self.map(|actual| match actual {
            Actual::Owned(o) => Actual::Owned(o.unwrap()),
            Actual::Borrowed(b) => Actual::Borrowed(b.as_ref().unwrap()),
        })
    }
}

/// Non-extracting assertions for `Option` values.
/// These work in any mode (Panic or Capture).
#[cfg_attr(feature = "fluent", assertr_derive::fluent_aliases)]
pub trait OptionAssertions<'t, T, M: Mode> {
    /// Test if this option is of the `None` variant.
    /// This is a terminal operation on the contained `Option`,
    /// as there is nothing meaningful to do with the option after its variant was ensured.
    fn is_none(self) -> AssertThat<'t, (), M>
    where
        T: Debug;
}

impl<'t, T, M: Mode> OptionAssertions<'t, T, M> for AssertThat<'t, Option<T>, M> {
    #[track_caller]
    fn is_none(self) -> AssertThat<'t, (), M>
    where
        T: Debug,
    {
        self.track_assertion();

        if !self.actual().is_none() {
            self.fail(format_args!(
                "Actual: {actual:#?}\n\nis not of expected variant: Option:None\n",
                actual = self.actual()
            ));
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

                is not of expected variant: Option:Some
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

                is not of expected variant: Option:None
                -------- assertr --------
            "});
        }
    }
}
