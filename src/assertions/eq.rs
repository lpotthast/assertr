use crate::{
    failure::{ExpectedActualFailure, GenericFailure},
    AssertThat,
};
use std::fmt::Debug;

// Assertions usable when T implements PartialEq.
pub trait EqualityAssertions<T: PartialEq + Debug> {
    fn is_equal_to(self, expected: T) -> Self;
    fn is_not_equal_to(self, expected: T) -> Self;
}

impl<'t, T: PartialEq + Debug> EqualityAssertions<T> for AssertThat<'t, T> {
    #[track_caller]
    fn is_equal_to(self, expected: T) -> Self {
        let actual = self.actual.borrowed();
        let expected = &expected;

        if actual != expected {
            self.fail(GenericFailure {
                arguments: format_args!("Expected: {:#?}\n\n  Actual: {:#?}", expected, actual),
            });
        }
        self
    }

    #[track_caller]
    fn is_not_equal_to(self, expected: T) -> Self {
        let actual = self.actual.borrowed();
        let expected = &expected;

        if actual == expected {
            self.fail(ExpectedActualFailure { expected, actual });
        }
        self
    }
}

#[cfg(test)]
mod tests {
    use indoc::formatdoc;

    use crate::prelude::*;

    #[test]
    fn is_equal_to_succeeds_when_equal() {
        assert_that("foo".to_owned()).is_equal_to("foo".to_owned());
        assert_that(&"foo".to_owned()).is_equal_to(&"foo".to_owned());
        assert_that("foo").is_equal_to("foo");
    }

    #[test]
    fn is_equal_to_panics_when_not_equal() {
        assert_that_panic_by(|| assert_that("foo").with_location(false).is_equal_to("bar"))
            .has_type::<String>()
            .is_equal_to(formatdoc! {r#"
                -------- assertr --------
                Expected: "bar"

                  Actual: "foo"
                -------- assertr --------
            "#});
    }
}
