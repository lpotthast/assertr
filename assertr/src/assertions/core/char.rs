use crate::failure::FailureKind;
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
            self.failure(FailureKind::Equality)
                .actual(self.render().value(actual))
                .relation("is not equal to ignoring ASCII case")
                .expected(self.render().value(&expected))
                .raise();
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
            self.failure(FailureKind::Predicate)
                .actual(self.render().value(actual))
                .relation("is not lowercase")
                .raise();
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
            self.failure(FailureKind::Predicate)
                .actual(self.render().value(actual))
                .relation("is not uppercase")
                .raise();
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
            self.failure(FailureKind::Predicate)
                .actual(self.render().value(actual))
                .relation("is not an ASCII lowercase letter")
                .raise();
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
            self.failure(FailureKind::Predicate)
                .actual(self.render().value(actual))
                .relation("is not an ASCII uppercase letter")
                .raise();
        }
        self
    }
}

#[cfg(test)]
mod tests {
    mod renderer_contract {
        use crate::prelude::*;
        use crate::test_support::{NoRenderer, SENTINEL, SentinelRenderer, assert_trait_impl};

        #[test]
        fn trait_is_implemented_without_renderer_support() {
            assert_trait_impl!(
                AssertThat<'static, char, Panic, NoRenderer> => CharAssertions<NoRenderer>
            );
        }

        #[test]
        fn failures_use_the_active_renderer() {
            let failures = assert_that!('A')
                .with_renderer(SentinelRenderer)
                .with_location(false)
                .capture(CharAssertions::is_lowercase);

            assert_that!(ToHumanReadableText.render(&failures[0])).contains(SENTINEL);
        }
    }

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
                Expression: `'a'`

                Actual: 'a'

                is not equal to ignoring ASCII case

                Expected: 'B'
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
                    Expression: `'A'`

                    Actual: 'A'

                    is not lowercase
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
                    Expression: `'a'`

                    Actual: 'a'

                    is not uppercase
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
                    Expression: `'A'`

                    Actual: 'A'

                    is not an ASCII lowercase letter
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
                    Expression: `'a'`

                    Actual: 'a'

                    is not an ASCII uppercase letter
                    -------- assertr --------
                "});
        }
    }
}
