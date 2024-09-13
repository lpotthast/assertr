use alloc::string::String;
use core::fmt::Debug;

use crate::prelude::StrSliceAssertions;
use crate::{AssertThat, Mode};

/// Assertions for heap-allocated, owned [String]s.
pub trait StringAssertions {
    fn has_length(self, expected: usize) -> Self;
    fn is_empty(self) -> Self;
    fn contains<E: AsRef<str> + Debug>(self, expected: E) -> Self;
    fn starts_with<E: AsRef<str> + Debug>(self, expected: E) -> Self;
    fn ends_with<E: AsRef<str> + Debug>(self, expected: E) -> Self;
}

// Assertions for Strings.
impl<'t, M: Mode> StringAssertions for AssertThat<'t, String, M> {
    #[track_caller]
    fn has_length(self, expected: usize) -> Self {
        self.derive(|actual| actual.as_str()).has_length(expected);
        self
    }

    #[track_caller]
    fn is_empty(self) -> Self {
        self.derive(|actual| actual.as_str()).is_empty();
        self
    }

    #[track_caller]
    fn contains<E: AsRef<str> + Debug>(self, expected: E) -> Self {
        self.derive(|actual| actual.as_str()).contains(expected);
        self
    }

    #[track_caller]
    fn starts_with<E: AsRef<str> + Debug>(self, expected: E) -> Self {
        self.derive(|actual| actual.as_str()).starts_with(expected);
        self
    }

    #[track_caller]
    fn ends_with<E: AsRef<str> + Debug>(self, expected: E) -> Self {
        self.derive(|actual| actual.as_str()).ends_with(expected);
        self
    }
}

#[cfg(test)]
mod tests {
    mod has_length {
        use crate::prelude::*;
        use indoc::formatdoc;

        #[test]
        fn succeeds_when_expected_length_matches() {
            assert_that(String::from("foo bar")).has_length(7);
        }

        #[test]
        fn panics_when_expected_length_does_not_match() {
            assert_that_panic_by(|| {
                assert_that(String::from("foo bar"))
                    .with_location(false)
                    .has_length(42);
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
    }

    mod is_empty {
        use crate::prelude::*;
        use indoc::formatdoc;

        #[test]
        fn succeeds_when_empty() {
            assert_that(String::from("")).is_empty();
        }

        #[test]
        fn panics_when_not_empty() {
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
    }

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

    mod starts_with {
        use crate::prelude::*;
        use indoc::formatdoc;

        #[test]
        fn succeeds_when_start_matches() {
            assert_that(String::from("foo bar baz")).starts_with("foo b");
        }

        #[test]
        fn panics_when_start_is_different() {
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

    mod ends_with {
        use crate::prelude::*;
        use indoc::formatdoc;

        #[test]
        fn succeeds_when_start_matches() {
            assert_that(String::from("foo bar baz")).ends_with("r baz");
        }

        #[test]
        fn panics_when_start_is_different() {
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
}
