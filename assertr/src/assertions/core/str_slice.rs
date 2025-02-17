use crate::{tracking::AssertionTracking, AssertThat, Mode};
use indoc::writedoc;
use std::fmt::Write;

/// Special assertions for `&str` (string slices) not covered by other general-purpose assertions,
/// like our `PartialEqAssertions`.
pub trait StrSliceAssertions {
    /// Tests whether this string is empty or only containing whitespace characters.
    /// 'Whitespace' is defined according to the terms of the Unicode Derived Core Property
    /// `White_Space`.
    fn is_blank(self) -> Self;

    /// Tests whether this string is empty or only containing ascii-whitespace characters.
    fn is_blank_ascii(self) -> Self;

    fn contains(self, expected: impl AsRef<str>) -> Self;

    fn starts_with(self, expected: impl AsRef<str>) -> Self;

    fn ends_with(self, expected: impl AsRef<str>) -> Self;
}

impl<M: Mode> StrSliceAssertions for AssertThat<'_, &str, M> {
    #[track_caller]
    fn is_blank(self) -> Self {
        self.track_assertion();
        // This iterator will yield no entries if the string is empty or all whitespace!
        if !self.actual().split_whitespace().next().is_none() {
            self.fail(|w: &mut String| {
                writedoc! {w, r#"
                    Actual: {actual:?}
                    
                    contains non-whitespace characters.
                    
                    Expected it to be empty or only containing whitespace.
                "#, actual = self.actual()}
            });
        }
        self
    }

    #[track_caller]
    fn is_blank_ascii(self) -> Self {
        self.track_assertion();
        // This iterator will yield no entries if the string is empty or all whitespace!
        if !self.actual().split_ascii_whitespace().next().is_none() {
            self.fail(|w: &mut String| {
                writedoc! {w, r#"
                    Actual: {actual:?}
                    
                    contains non-whitespace characters.
                    
                    Expected it to be empty or only containing whitespace.
                "#, actual = self.actual()}
            });
        }
        self
    }

    #[track_caller]
    fn contains(self, expected: impl AsRef<str>) -> Self {
        self.track_assertion();
        let expected = expected.as_ref();
        if !self.actual().contains(expected) {
            self.fail(format_args!(
                "Actual: {actual:?}\n\ndoes not contain\n\nExpected: {expected:?}\n",
                actual = self.actual(),
                expected = &expected,
            ));
        }
        self
    }

    #[track_caller]
    fn starts_with(self, expected: impl AsRef<str>) -> Self {
        self.track_assertion();
        let expected = expected.as_ref();
        if !self.actual().starts_with(expected) {
            self.fail(format_args!(
                "Actual: {actual:?}\n\ndoes not start with\n\nExpected: {expected:?}\n",
                actual = self.actual(),
                expected = &expected,
            ));
        }
        self
    }

    #[track_caller]
    fn ends_with(self, expected: impl AsRef<str>) -> Self {
        self.track_assertion();
        let expected = expected.as_ref();
        if !self.actual().ends_with(expected) {
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
    mod is_blank {
        use crate::prelude::*;
        use indoc::formatdoc;

        #[test]
        fn succeeds_when_expected_is_blank() {
            assert_that("").is_blank();
            assert_that(" ").is_blank();
            assert_that("\t \n").is_blank();
        }

        #[test]
        fn panics_when_expected_is_not_blank() {
            assert_that_panic_by(|| {
                assert_that("a").with_location(false).is_blank();
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {r#"
                -------- assertr --------
                Actual: "a"

                contains non-whitespace characters.

                Expected it to be empty or only containing whitespace.
                -------- assertr --------
            "#});
        }
    }

    mod is_blank_ascii {
        use crate::prelude::*;
        use indoc::formatdoc;

        #[test]
        fn succeeds_when_expected_is_blank() {
            assert_that("").is_blank_ascii();
            assert_that(" ").is_blank_ascii();
            assert_that("\t \n").is_blank_ascii();
        }

        #[test]
        fn panics_when_expected_is_not_blank() {
            assert_that_panic_by(|| {
                assert_that("a").with_location(false).is_blank_ascii();
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {r#"
                -------- assertr --------
                Actual: "a"

                contains non-whitespace characters.

                Expected it to be empty or only containing whitespace.
                -------- assertr --------
            "#});
        }
    }

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
