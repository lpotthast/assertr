use core::fmt::Debug;

use crate::{tracking::AssertionTracking, AssertThat, Mode};

/// Assertions for `&str` (str slices).
pub trait StrSliceAssertions {
    fn contains<E: AsRef<str> + Debug>(self, expected: E) -> Self;

    fn starts_with<E: AsRef<str> + Debug>(self, expected: E) -> Self;

    fn ends_with<E: AsRef<str> + Debug>(self, expected: E) -> Self;
}

impl<'t, M: Mode> StrSliceAssertions for AssertThat<'t, &str, M> {
    #[track_caller]
    fn contains<E: AsRef<str> + Debug>(self, expected: E) -> Self {
        self.track_assertion();
        if !self.actual().contains(expected.as_ref()) {
            self.fail(format_args!(
                "Actual: {actual:?}\n\ndoes not contain\n\nExpected: {expected:?}\n",
                actual = self.actual(),
                expected = &expected,
            ));
        }
        self
    }

    #[track_caller]
    fn starts_with<E: AsRef<str> + Debug>(self, expected: E) -> Self {
        self.track_assertion();
        if !self.actual().starts_with(expected.as_ref()) {
            self.fail(format_args!(
                "Actual: {actual:?}\n\ndoes not start with\n\nExpected: {expected:?}\n",
                actual = self.actual(),
                expected = &expected,
            ));
        }
        self
    }

    #[track_caller]
    fn ends_with<E: AsRef<str> + Debug>(self, expected: E) -> Self {
        self.track_assertion();
        if !self.actual().ends_with(expected.as_ref()) {
            self.fail(format_args!(
                "Actual: {actual:?}\n\ndoes not end with\n\nExpected: {expected:?}\n",
                actual = self.actual(),
                expected = &expected,
            ));
        }
        self
    }
}

#[cfg(test)]
mod tests {
    mod contains {
        use crate::prelude::*;
        use indoc::formatdoc;

        #[test]
        fn succeeds_when_expected_is_contained() {
            assert_that("foobar").contains("foo");
            assert_that("foobar").contains("bar");
            assert_that("foobar").contains("oob");
        }

        #[test]
        fn panics_when_expected_is_not_contained() {
            assert_that_panic_by(|| {
                assert_that("foo bar baz")
                    .with_location(false)
                    .contains("42");
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {r#"
                -------- assertr --------
                Actual: "foo bar baz"

                does not contain

                Expected: "42"
                -------- assertr --------
            "#});
        }
    }

    mod starts_with {
        use crate::prelude::*;
        use indoc::formatdoc;

        #[test]
        fn succeeds_when_start_matches() {
            assert_that("foo bar baz").starts_with("foo b");
        }

        #[test]
        fn panics_when_start_is_different() {
            assert_that_panic_by(|| {
                assert_that("foo bar baz")
                    .with_location(false)
                    .starts_with("oo");
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {r#"
                -------- assertr --------
                Actual: "foo bar baz"

                does not start with

                Expected: "oo"
                -------- assertr --------
            "#});
        }
    }

    mod ends_with {
        use crate::prelude::*;
        use indoc::formatdoc;

        #[test]
        fn succeeds_when_start_matches() {
            assert_that("foo bar baz").ends_with("r baz");
        }

        #[test]
        fn panics_when_start_is_different() {
            assert_that_panic_by(|| {
                assert_that("foo bar baz")
                    .with_location(false)
                    .ends_with("raz");
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {r#"
                -------- assertr --------
                Actual: "foo bar baz"

                does not end with

                Expected: "raz"
                -------- assertr --------
            "#});
        }
    }
}
