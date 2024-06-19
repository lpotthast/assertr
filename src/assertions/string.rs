use std::fmt::Debug;

use crate::{AssertThat, Mode};

// Assertions for Strings.
impl<'t, M: Mode> AssertThat<'t, String, M> {
    #[track_caller]
    pub fn is_empty(self) -> Self {
        self.derive(|actual| actual.as_str()).is_empty();
        self
    }

    #[track_caller]
    pub fn contains<E: AsRef<str> + Debug>(self, expected: E) -> Self {
        self.derive(|actual| actual.as_str()).contains(expected);
        self
    }

    #[track_caller]
    pub fn starts_with<E: AsRef<str> + Debug>(self, expected: E) -> Self {
        self.derive(|actual| actual.as_str()).starts_with(expected);
        self
    }

    #[track_caller]
    pub fn ends_with<E: AsRef<str> + Debug>(self, expected: E) -> Self {
        self.derive(|actual| actual.as_str()).ends_with(expected);
        self
    }
}

#[cfg(test)]
mod tests {
    use indoc::formatdoc;

    use crate::prelude::*;

    #[test]
    fn is_empty_succeeds_when_empty() {
        assert_that(String::from("")).is_empty();
    }

    #[test]
    fn is_empty_panics_when_not_empty() {
        assert_that_panic_by(|| {
            assert_that(String::from("foo"))
                .with_location(false)
                .is_empty();
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
        assert_that(String::from("foobar")).contains("foo");
        assert_that(String::from("foobar")).contains("bar");
        assert_that(String::from("foobar")).contains("oob");
    }

    #[test]
    fn contains_panics_when_expected_is_not_contained() {
        assert_that_panic_by(|| {
            assert_that(String::from("foo bar baz"))
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
        assert_that(String::from("foo bar baz")).starts_with("foo b");
    }

    #[test]
    fn starts_with_panics_when_start_is_different() {
        assert_that_panic_by(|| {
            assert_that(String::from("foo bar baz"))
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
        assert_that(String::from("foo bar baz")).ends_with("r baz");
    }

    #[test]
    fn ends_with_panics_when_start_is_different() {
        assert_that_panic_by(|| {
            assert_that(String::from("foo bar baz"))
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
