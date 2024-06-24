use crate::{failure::GenericFailure, tracking::AssertionTracking, AssertThat, Mode};
use std::fmt::Debug;

/// Assertions for str slices.
impl<'t, M: Mode> AssertThat<'t, &str, M> {
    #[track_caller]
    pub fn has_length(self, expected: usize) -> Self {
        self.track_assertion();
        let actual = self.actual();
        let actual_len = actual.len();
        if actual_len != expected {
            self.fail(GenericFailure {
                arguments: format_args!(
                    "Actual: {actual:?} (length: {actual_len})\n\ndoes not have length\n\nExpected: {expected:?}",
                    actual = self.actual(),
                ),
            });
        }
        self
    }

    #[track_caller]
    pub fn is_empty(self) -> Self {
        self.track_assertion();
        if !self.actual().is_empty() {
            self.fail(GenericFailure {
                arguments: format_args!(
                    "Actual: {actual:?}\n\nwas expected to be empty, but it is not!",
                    actual = self.actual(),
                ),
            });
        }
        self
    }

    #[track_caller]
    pub fn contains<E: AsRef<str> + Debug>(self, expected: E) -> Self {
        self.track_assertion();
        if !self.actual().contains(expected.as_ref()) {
            self.fail(GenericFailure {
                arguments: format_args!(
                    "Actual: {actual:?}\n\ndoes not contain\n\nExpected: {expected:?}",
                    actual = self.actual(),
                    expected = &expected,
                ),
            });
        }
        self
    }

    #[track_caller]
    pub fn starts_with<E: AsRef<str> + Debug>(self, expected: E) -> Self {
        self.track_assertion();
        if !self.actual().starts_with(expected.as_ref()) {
            self.fail(GenericFailure {
                arguments: format_args!(
                    "Actual: {actual:?}\n\ndoes not start with\n\nExpected: {expected:?}",
                    actual = self.actual(),
                    expected = &expected,
                ),
            });
        }
        self
    }

    #[track_caller]
    pub fn ends_with<E: AsRef<str> + Debug>(self, expected: E) -> Self {
        self.track_assertion();
        if !self.actual().ends_with(expected.as_ref()) {
            self.fail(GenericFailure {
                arguments: format_args!(
                    "Actual: {actual:?}\n\ndoes not end with\n\nExpected: {expected:?}",
                    actual = self.actual(),
                    expected = &expected,
                ),
            });
        }
        self
    }
}

#[cfg(test)]
mod tests {
    use indoc::formatdoc;

    use crate::prelude::*;

    #[test]
    fn has_length_succeeds_when_expected_length_matches() {
        assert_that("foo bar").has_length(7);
    }

    #[test]
    fn has_length_panics_when_expected_length_does_not_match() {
        assert_that_panic_by(|| {
            assert_that("foo bar").with_location(false).has_length(42);
        })
        .has_type::<String>()
        .is_equal_to(formatdoc! {r#"
                -------- assertr --------
                Actual: "foo bar" (length: 7)

                does not have length

                Expected: 42
                -------- assertr --------
            "#});
    }

    #[test]
    fn is_empty_succeeds_when_empty() {
        assert_that("").is_empty();
    }

    #[test]
    fn is_empty_panics_when_not_empty() {
        assert_that_panic_by(|| {
            assert_that("foo").with_location(false).is_empty();
        })
        .has_type::<String>()
        .is_equal_to(formatdoc! {r#"
                -------- assertr --------
                Actual: "foo"

                was expected to be empty, but it is not!
                -------- assertr --------
            "#});
    }

    #[test]
    fn contains_succeeds_when_expected_is_contained() {
        assert_that("foobar").contains("foo");
        assert_that("foobar").contains("bar");
        assert_that("foobar").contains("oob");
    }

    #[test]
    fn contains_panics_when_expected_is_not_contained() {
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

    #[test]
    fn starts_with_succeeds_when_start_matches() {
        assert_that("foo bar baz").starts_with("foo b");
    }

    #[test]
    fn starts_with_panics_when_start_is_different() {
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

    #[test]
    fn ends_with_succeeds_when_start_matches() {
        assert_that("foo bar baz").ends_with("r baz");
    }

    #[test]
    fn ends_with_panics_when_start_is_different() {
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
