use core::fmt::Write;
use indoc::writedoc;

use crate::mode::Mode;
use crate::tracking::AssertionTracking;
use crate::AssertThat;

pub trait CharAssertions {
    fn is_equal_to_ignoring_ascii_case(self, expected: char) -> Self;

    fn is_lowercase(self) -> Self;

    fn is_uppercase(self) -> Self;

    fn is_ascii_lowercase(self) -> Self;

    fn is_ascii_uppercase(self) -> Self;

    //fn is_ascii(self) -> Self;
    //fn is_whitespace(self) -> Self;
    //fn is_alphabetic(self) -> Self;
    //fn is_alphanumeric(self) -> Self;
    //fn is_numeric(self) -> Self;
}

impl<'t, M: Mode> CharAssertions for AssertThat<'t, char, M> {
    #[track_caller]
    fn is_equal_to_ignoring_ascii_case(self, expected: char) -> Self {
        self.track_assertion();
        let actual = self.actual();
        if !actual.eq_ignore_ascii_case(&expected) {
            self.add_detail_message("Actual is not equal to expected, even when ignoring casing.");
            self.fail(|w: &mut String| {
                writedoc! {w, r#"
                    Expected: {expected:#?},
                    
                      Actual: {actual:#?},
                "#}
            })
        }
        self
    }

    #[track_caller]
    fn is_lowercase(self) -> Self {
        self.track_assertion();
        let actual = self.actual();
        if !actual.is_lowercase() {
            self.fail(|w: &mut String| {
                writedoc! {w, r#"
                    Expected {actual:#?} to be lowercase, but it is not.
                "#}
            })
        }
        self
    }

    #[track_caller]
    fn is_uppercase(self) -> Self {
        self.track_assertion();
        let actual = self.actual();
        if !actual.is_uppercase() {
            self.fail(|w: &mut String| {
                writedoc! {w, r#"
                    Expected {actual:#?} to be uppercase, but it is not.
                "#}
            })
        }
        self
    }

    #[track_caller]
    fn is_ascii_lowercase(self) -> Self {
        self.track_assertion();
        let actual = self.actual();
        if !actual.is_ascii_lowercase() {
            self.fail(|w: &mut String| {
                writedoc! {w, r#"
                    Expected {actual:#?} to be an ascii-lowercase char, but it is not.
                "#}
            })
        }
        self
    }

    #[track_caller]
    fn is_ascii_uppercase(self) -> Self {
        self.track_assertion();
        let actual = self.actual();
        if !actual.is_ascii_uppercase() {
            self.fail(|w: &mut String| {
                writedoc! {w, r#"
                    Expected {actual:#?} to be an ascii-uppercase char, but it is not.
                "#}
            })
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
        fn succeeds_when_equal_ignoring_ascii_case() {
            assert_that('a').is_equal_to_ignoring_ascii_case('A');
        }

        #[test]
        fn panics_when_not_equal_to_ignoring_ascii_case() {
            assert_that_panic_by(|| {
                assert_that('a')
                    .with_location(false)
                    .is_equal_to_ignoring_ascii_case('B')
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {r#"
                -------- assertr --------
                Expected: 'B',
                
                  Actual: 'a',
                
                Details: [
                    Actual is not equal to expected, even when ignoring casing.,
                ]
                -------- assertr --------
            "#});
        }
    }
}
