use alloc::string::String;

use crate::prelude::StrSliceAssertions;
use crate::{AssertThat, Mode};

/// Assertions for heap-allocated, owned [String]s.
pub trait StringAssertions {
    fn contains(self, expected: impl AsRef<str>) -> Self;

    fn does_not_contain(self, unexpected: impl AsRef<str>) -> Self;

    fn starts_with(self, expected: impl AsRef<str>) -> Self;

    fn does_not_start_with(self, unexpected: impl AsRef<str>) -> Self;

    fn ends_with(self, expected: impl AsRef<str>) -> Self;

    fn does_not_end_with(self, unexpected: impl AsRef<str>) -> Self;
}

impl<M: Mode> StringAssertions for AssertThat<'_, String, M> {
    #[track_caller]
    fn contains(self, expected: impl AsRef<str>) -> Self {
        self.derive(|actual| actual.as_str()).contains(expected);
        self
    }

    #[track_caller]
    fn does_not_contain(self, unexpected: impl AsRef<str>) -> Self {
        self.derive(|actual| actual.as_str())
            .does_not_contain(unexpected);
        self
    }

    #[track_caller]
    fn starts_with(self, expected: impl AsRef<str>) -> Self {
        self.derive(|actual| actual.as_str()).starts_with(expected);
        self
    }

    #[track_caller]
    fn does_not_start_with(self, unexpected: impl AsRef<str>) -> Self {
        self.derive(|actual| actual.as_str())
            .does_not_start_with(unexpected);
        self
    }

    #[track_caller]
    fn ends_with(self, expected: impl AsRef<str>) -> Self {
        self.derive(|actual| actual.as_str()).ends_with(expected);
        self
    }

    #[track_caller]
    fn does_not_end_with(self, unexpected: impl AsRef<str>) -> Self {
        self.derive(|actual| actual.as_str())
            .does_not_end_with(unexpected);
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
            assert_that(String::from("foobar")).contains("foo");
            assert_that(String::from("foobar")).contains("bar");
            assert_that(String::from("foobar")).contains("oob");
        }

        #[test]
        fn panics_when_expected_is_not_contained() {
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
    }

    mod does_not_contain {
        use crate::prelude::*;
        use indoc::formatdoc;

        #[test]
        fn succeeds_when_expected_is_not_contained() {
            assert_that(String::from("foobar")).does_not_contain("hello");
        }

        #[test]
        fn panics_when_expected_is_contained() {
            assert_that_panic_by(|| {
                assert_that(String::from("foo bar baz"))
                    .with_location(false)
                    .does_not_contain("ar b");
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {r#"
                -------- assertr --------
                Actual: "foo bar baz"

                contains

                Unexpected: "ar b"
                -------- assertr --------
            "#});
        }
    }

    mod starts_with {
        use crate::prelude::*;
        use indoc::formatdoc;

        #[test]
        fn succeeds_when_start_matches() {
            assert_that(String::from("foo bar baz")).starts_with("foo b");
        }

        #[test]
        fn panics_when_start_does_not_match() {
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
    }

    mod does_not_start_with {
        use crate::prelude::*;
        use indoc::formatdoc;

        #[test]
        fn succeeds_when_start_does_not_match() {
            assert_that(String::from("foo bar baz")).starts_with("fo");
        }

        #[test]
        fn panics_when_start_matches() {
            assert_that_panic_by(|| {
                assert_that(String::from("foo bar baz"))
                    .with_location(false)
                    .does_not_start_with("foo bar ba");
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {r#"
                    -------- assertr --------
                    Actual: "foo bar baz"

                    starts with

                    Unexpected: "foo bar ba"
                    -------- assertr --------
                "#});
        }
    }

    mod ends_with {
        use crate::prelude::*;
        use indoc::formatdoc;

        #[test]
        fn succeeds_when_end_matches() {
            assert_that(String::from("foo bar baz")).ends_with("r baz");
        }

        #[test]
        fn panics_when_end_is_different() {
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

    mod does_not_end_with {
        use crate::prelude::*;
        use indoc::formatdoc;

        #[test]
        fn succeeds_when_end_does_not_matches() {
            assert_that(String::from("foo bar baz")).does_not_end_with("bar");
        }

        #[test]
        fn panics_when_end_matches() {
            assert_that_panic_by(|| {
                assert_that(String::from("foo bar baz"))
                    .with_location(false)
                    .does_not_end_with(" baz");
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {r#"
                    -------- assertr --------
                    Actual: "foo bar baz"

                    ends with

                    Unexpected: " baz"
                    -------- assertr --------
                "#});
        }
    }
}
