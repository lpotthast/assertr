use alloc::string::String;
use core::fmt::Write;
use indoc::writedoc;

use crate::mode::Mode;
use crate::{AssertThat, ValueRenderer};

/// Assertions for character values.
#[allow(clippy::return_self_not_must_use)]
#[cfg_attr(feature = "fluent", assertr_derive::fluent_aliases)]
pub trait CharAssertions<R = crate::DebugRenderer> {
    /// Asserts that the subject and `expected` are equal under ASCII case folding.
    fn is_equal_to_ignoring_ascii_case(self, expected: char) -> Self
    where
        R: ValueRenderer<char>;

    /// Asserts that the subject has the Unicode `Lowercase` property.
    fn is_lowercase(self) -> Self
    where
        R: ValueRenderer<char>;

    /// Asserts that the subject has the Unicode `Uppercase` property.
    fn is_uppercase(self) -> Self
    where
        R: ValueRenderer<char>;

    /// Asserts that the subject is an ASCII lowercase letter.
    fn is_ascii_lowercase(self) -> Self
    where
        R: ValueRenderer<char>;

    /// Asserts that the subject is an ASCII uppercase letter.
    fn is_ascii_uppercase(self) -> Self
    where
        R: ValueRenderer<char>;

    //fn is_ascii(self) -> Self;
    //fn is_whitespace(self) -> Self;
    //fn is_alphabetic(self) -> Self;
    //fn is_alphanumeric(self) -> Self;
    //fn is_numeric(self) -> Self;
}

impl<M: Mode, R> CharAssertions<R> for AssertThat<'_, char, M, R> {
    #[track_caller]
    fn is_equal_to_ignoring_ascii_case(self, expected: char) -> Self
    where
        R: ValueRenderer<char>,
    {
        self.track_assertion();
        let actual = self.actual();
        if !actual.eq_ignore_ascii_case(&expected) {
            let details = [String::from(
                "Actual is not equal to expected, even when ignoring casing.",
            )];
            let actual = self.render_value(actual);
            let expected = self.render_value(&expected);
            self.fail_with_details(details, |w: &mut String| {
                writedoc! {w, r"
                    Expected: {expected:#?}

                      Actual: {actual:#?}
                "}
            });
        }
        self
    }

    #[track_caller]
    fn is_lowercase(self) -> Self
    where
        R: ValueRenderer<char>,
    {
        self.track_assertion();
        let actual = self.actual();
        if !actual.is_lowercase() {
            let actual = self.render_value(actual);
            self.fail(|w: &mut String| {
                writedoc! {w, r"
                    Expected {actual:#?} to be lowercase, but it is not.
                "}
            });
        }
        self
    }

    #[track_caller]
    fn is_uppercase(self) -> Self
    where
        R: ValueRenderer<char>,
    {
        self.track_assertion();
        let actual = self.actual();
        if !actual.is_uppercase() {
            let actual = self.render_value(actual);
            self.fail(|w: &mut String| {
                writedoc! {w, r"
                    Expected {actual:#?} to be uppercase, but it is not.
                "}
            });
        }
        self
    }

    #[track_caller]
    fn is_ascii_lowercase(self) -> Self
    where
        R: ValueRenderer<char>,
    {
        self.track_assertion();
        let actual = self.actual();
        if !actual.is_ascii_lowercase() {
            let actual = self.render_value(actual);
            self.fail(|w: &mut String| {
                writedoc! {w, r"
                    Expected {actual:#?} to be an ascii-lowercase char, but it is not.
                "}
            });
        }
        self
    }

    #[track_caller]
    fn is_ascii_uppercase(self) -> Self
    where
        R: ValueRenderer<char>,
    {
        self.track_assertion();
        let actual = self.actual();
        if !actual.is_ascii_uppercase() {
            let actual = self.render_value(actual);
            self.fail(|w: &mut String| {
                writedoc! {w, r"
                    Expected {actual:#?} to be an ascii-uppercase char, but it is not.
                "}
            });
        }
        self
    }
}

#[cfg(test)]
mod tests {

    mod is_equal_to_ignoring_ascii_case {
        use crate::prelude::*;
        use indoc::formatdoc;

        #[test]
        #[cfg(feature = "fluent")]
        fn fluent_alias_is_as_expected() {
            'a'.must().be_equal_to_ignoring_ascii_case('A');
        }

        #[test]
        fn succeeds_when_equal_ignoring_ascii_case() {
            assert_that!('a').is_equal_to_ignoring_ascii_case('A');
        }

        #[test]
        fn panics_when_not_equal_to_ignoring_ascii_case() {
            assert_that_panic_by(|| {
                assert_that!('a')
                    .with_location(false)
                    .is_equal_to_ignoring_ascii_case('B')
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {r"
                -------- assertr --------
                Expected: 'B'
                
                  Actual: 'a'
                
                Details: [
                    Actual is not equal to expected, even when ignoring casing.,
                ]
                -------- assertr --------
            "});
        }
    }

    mod is_lowercase {
        use crate::prelude::*;
        use indoc::formatdoc;

        #[test]
        #[cfg(feature = "fluent")]
        fn fluent_alias_is_as_expected() {
            'a'.must().be_lowercase();
        }

        #[test]
        fn succeeds_when_lowercase() {
            assert_that!('a').is_lowercase();
        }

        #[test]
        fn panics_when_not_lowercase() {
            assert_that_panic_by(|| assert_that!('A').with_location(false).is_lowercase())
                .has_type::<String>()
                .is_equal_to(formatdoc! {r"
                    -------- assertr --------
                    Expected 'A' to be lowercase, but it is not.
                    -------- assertr --------
                "});
        }
    }

    mod is_uppercase {
        use crate::prelude::*;
        use indoc::formatdoc;

        #[test]
        #[cfg(feature = "fluent")]
        fn fluent_alias_is_as_expected() {
            'A'.must().be_uppercase();
        }

        #[test]
        fn succeeds_when_uppercase() {
            assert_that!('A').is_uppercase();
        }

        #[test]
        fn panics_when_not_uppercase() {
            assert_that_panic_by(|| assert_that!('a').with_location(false).is_uppercase())
                .has_type::<String>()
                .is_equal_to(formatdoc! {r"
                    -------- assertr --------
                    Expected 'a' to be uppercase, but it is not.
                    -------- assertr --------
                "});
        }
    }

    mod is_ascii_lowercase {
        use crate::prelude::*;
        use indoc::formatdoc;

        #[test]
        #[cfg(feature = "fluent")]
        fn fluent_alias_is_as_expected() {
            'a'.must().be_ascii_lowercase();
        }

        #[test]
        fn succeeds_when_ascii_lowercase() {
            assert_that!('a').is_ascii_lowercase();
        }

        #[test]
        fn panics_when_not_ascii_lowercase() {
            assert_that_panic_by(|| assert_that!('A').with_location(false).is_ascii_lowercase())
                .has_type::<String>()
                .is_equal_to(formatdoc! {r"
                    -------- assertr --------
                    Expected 'A' to be an ascii-lowercase char, but it is not.
                    -------- assertr --------
                "});
        }
    }

    mod is_ascii_uppercase {
        use crate::prelude::*;
        use indoc::formatdoc;

        #[test]
        #[cfg(feature = "fluent")]
        fn fluent_alias_is_as_expected() {
            'A'.must().be_ascii_uppercase();
        }

        #[test]
        fn succeeds_when_ascii_uppercase() {
            assert_that!('A').is_ascii_uppercase();
        }

        #[test]
        fn panics_when_not_ascii_uppercase() {
            assert_that_panic_by(|| assert_that!('a').with_location(false).is_ascii_uppercase())
                .has_type::<String>()
                .is_equal_to(formatdoc! {r"
                    -------- assertr --------
                    Expected 'a' to be an ascii-uppercase char, but it is not.
                    -------- assertr --------
                "});
        }
    }
}
