use crate::{failure::GenericFailure, AssertThat};
use std::fmt::Debug;

/// Assertions for str slices.
impl<'t> AssertThat<'t, &str> {
    #[track_caller]
    pub fn is_empty(&self) {
        if !self.actual.borrowed().is_empty() {
            self.fail_with(GenericFailure {
                arguments: format_args!(
                    "Actual: {actual:?}\n\nwas expected to be empty, but it is not!",
                    actual = self.actual.borrowed(),
                ),
            });
        }
    }

    #[track_caller]
    pub fn contains<E: AsRef<str> + Debug>(&self, expected: E) {
        if !self.actual.borrowed().contains(expected.as_ref()) {
            self.fail_with(GenericFailure {
                arguments: format_args!(
                    "Actual: {actual:?}\n\ndoes not contain\n\nExpected: {expected:?}",
                    actual = self.actual.borrowed(),
                    expected = &expected,
                ),
            });
        }
    }

    #[track_caller]
    pub fn starts_with<E: AsRef<str> + Debug>(&self, expected: E) {
        if !self.actual.borrowed().starts_with(expected.as_ref()) {
            self.fail_with(GenericFailure {
                arguments: format_args!(
                    "Actual: {actual:?}\n\ndoes not start with\n\nExpected: {expected:?}",
                    actual = self.actual.borrowed(),
                    expected = &expected,
                ),
            });
        }
    }

    #[track_caller]
    pub fn ends_with<E: AsRef<str> + Debug>(&self, expected: E) {
        if !self.actual.borrowed().ends_with(expected.as_ref()) {
            self.fail_with(GenericFailure {
                arguments: format_args!(
                    "Actual: {actual:?}\n\ndoes not end with\n\nExpected: {expected:?}",
                    actual = self.actual.borrowed(),
                    expected = &expected,
                ),
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use indoc::formatdoc;

    use crate::{assert_that, assert_that_panic_by};

    #[test]
    fn is_empty_succeeds_when_empty() {
        assert_that("").is_empty();
    }

    #[test]
    fn is_empty_panics_when_not_empty() {
        assert_that_panic_by(|| {
            assert_that("foo").with_location(false).is_empty();
        })
        .has_box_type::<String>()
        .has_debug_value(formatdoc! {r#"
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
        .has_box_type::<String>()
        .has_debug_value(formatdoc! {r#"
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
        .has_box_type::<String>()
        .has_debug_value(formatdoc! {r#"
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
        .has_box_type::<String>()
        .has_debug_value(formatdoc! {r#"
                -------- assertr --------
                Actual: "foo bar baz"

                does not end with

                Expected: "raz"
                -------- assertr --------
            "#});
    }
}
