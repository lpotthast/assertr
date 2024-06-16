use crate::{failure::ExpectedActualFailure, AssertThat};
use std::fmt::Debug;

// General Assertions
impl<'t, T: PartialEq> AssertThat<'t, T> {
    #[track_caller]
    pub fn is_equal_to(self, expected: T) -> Self
    where
        T: Debug,
    {
        let actual = self.actual.borrowed();
        let expected = &expected;

        if actual != expected {
            self.fail_with(ExpectedActualFailure { expected, actual });
        }
        self
    }

    #[track_caller]
    pub fn is_not_equal_to(self, expected: T) -> Self
    where
        T: Debug,
    {
        let actual = self.actual.borrowed();
        let expected = &expected;

        if actual == expected {
            self.fail_with(ExpectedActualFailure { expected, actual });
        }
        self
    }
}

#[cfg(test)]
mod tests {
    use indoc::formatdoc;

    use crate::{assert_that, assert_that_panic_by};

    #[test]
    fn is_equal_to_succeeds_when_equal() {
        assert_that("foo".to_owned()).is_equal_to("foo".to_owned());
        assert_that(&"foo".to_owned()).is_equal_to(&"foo".to_owned());
        assert_that("foo").is_equal_to("foo");
    }

    #[test]
    fn is_equal_to_panics_when_not_equal() {
        assert_that_panic_by(|| assert_that("foo").with_location(false).is_equal_to("bar"))
            .has_box_type::<String>()
            .has_debug_value(formatdoc! {r#"
                -------- assertr --------
                Expected: "bar"

                Actual: "foo"
                -------- assertr --------
            "#});
    }
}
