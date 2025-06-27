use alloc::string::String;

use crate::prelude::StrSliceAssertions;
use crate::{AssertThat, Mode};

/// Assertions for heap-allocated, owned [String]s.
pub trait StringAssertions {
    fn contains(self, expected: impl AsRef<str>) -> Self;

    fn contain(self, expected: impl AsRef<str>) -> Self
    where
        Self: Sized,
    {
        self.contains(expected)
    }

    fn starts_with(self, expected: impl AsRef<str>) -> Self;

    fn start_with(self, expected: impl AsRef<str>) -> Self
    where
        Self: Sized,
    {
        self.starts_with(expected)
    }

    fn ends_with(self, expected: impl AsRef<str>) -> Self;

    fn end_with(self, expected: impl AsRef<str>) -> Self
    where
        Self: Sized,
    {
        self.ends_with(expected)
    }
}

impl<M: Mode> StringAssertions for AssertThat<'_, String, M> {
    #[track_caller]
    fn contains(self, expected: impl AsRef<str>) -> Self {
        self.derive(|actual| actual.as_str()).contains(expected);
        self
    }

    #[track_caller]
    fn starts_with(self, expected: impl AsRef<str>) -> Self {
        self.derive(|actual| actual.as_str()).starts_with(expected);
        self
    }

    #[track_caller]
    fn ends_with(self, expected: impl AsRef<str>) -> Self {
        self.derive(|actual| actual.as_str()).ends_with(expected);
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
            String::from("foobar").must().contain("foo");
            String::from("foobar").must().contain("bar");
            String::from("foobar").must().contain("oob");
        }

        #[test]
        fn panics_when_expected_is_not_contained() {
            assert_that_panic_by(|| {
                assert_that!(String::from("foo bar baz"))
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
            String::from("foo bar baz").must().start_with("foo b");
        }

        #[test]
        fn panics_when_start_is_different() {
            assert_that_panic_by(|| {
                assert_that!(String::from("foo bar baz"))
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
            String::from("foo bar baz").must().end_with("r baz");
        }

        #[test]
        fn panics_when_start_is_different() {
            assert_that_panic_by(|| {
                assert_that!(String::from("foo bar baz"))
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
