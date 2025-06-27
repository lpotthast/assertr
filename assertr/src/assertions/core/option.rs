use crate::{AssertThat, Mode, actual::Actual, tracking::AssertionTracking};
use core::fmt::Debug;
use core::option::Option;

/// Assertions for generic optional values.
pub trait OptionAssertions<'t, T, M: Mode> {
    /// Test if this option is of the `Some` variant.
    /// This is a terminal operation on the contained `Option`,
    /// as there is nothing meaningful to do with the option if its variant was ensured.
    /// This allows you to chain additional expectations on the contained success value.
    fn is_some(self) -> AssertThat<'t, T, M>
    where
        T: Debug;

    /// Test if this option is of the `Some` variant.
    /// This is a terminal operation on the contained `Option`,
    /// as there is nothing meaningful to do with the option if its variant was ensured.
    /// This allows you to chain additional expectations on the contained success value.
    fn be_some(self) -> AssertThat<'t, T, M>
    where
        T: Debug,
        Self: Sized,
    {
        self.is_some()
    }

    /// Test if this option is of the `None` variant.
    /// This is a terminal operation on the contained `Option`,
    /// as there is nothing meaningful to do with the option after its variant was ensured.
    fn is_none(self) -> AssertThat<'t, (), M>
    where
        T: Debug;

    /// Test if this option is of the `None` variant.
    /// This is a terminal operation on the contained `Option`,
    /// as there is nothing meaningful to do with the option after its variant was ensured.
    fn be_none(self) -> AssertThat<'t, (), M>
    where
        T: Debug,
        Self: Sized,
    {
        self.is_none()
    }
}

impl<'t, T, M: Mode> OptionAssertions<'t, T, M> for AssertThat<'t, Option<T>, M> {
    #[track_caller]
    fn is_some(self) -> AssertThat<'t, T, M>
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
            Option::<i32>::Some(42).must().be_some().be_equal_to(42);
        }

        #[test]
        fn panics_when_none() {
            assert_that_panic_by(|| Option::<i32>::None.assert().with_location(false).is_some())
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
            Option::<i32>::None.must().be_none();
        }

        #[test]
        fn panics_when_some() {
            assert_that_panic_by(|| {
                Option::<i32>::Some(42)
                    .assert()
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
