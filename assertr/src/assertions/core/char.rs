use crate::{
    AssertThat, Mode,
    expectation::AssertionContext,
    expectation::Expectation,
    failure::{FailureBuilder, FailureKind},
    renderer::DebugRenderer,
    renderer::ValueRenderer,
};

/// Compares characters under ASCII case folding.
#[derive(Debug, Clone, Copy)]
pub struct EqualToIgnoringAsciiCase(char);

impl EqualToIgnoringAsciiCase {
    /// Owns the expected character.
    #[must_use]
    pub const fn new(expected: char) -> Self {
        Self(expected)
    }
}

impl<R: ValueRenderer<char>> Expectation<char, R> for EqualToIgnoringAsciiCase {
    type Success<'a> = ();
    type Rejection<'a> = ();
    fn evaluate<'a>(&'a self, actual: &'a char, _: &AssertionContext<'_, R>) -> Result<(), ()> {
        if actual.eq_ignore_ascii_case(&self.0) {
            Ok(())
        } else {
            Err(())
        }
    }

    const KIND: FailureKind = FailureKind::Equality;
    fn explain(
        &self,
        rejected: Option<(&char, ())>,
        failure: FailureBuilder,
        context: &AssertionContext<'_, R>,
    ) -> FailureBuilder {
        let render = context.render();
        failure
            .relations(
                rejected.map(|(actual, ())| render.value(actual)),
                "is equal to ignoring ASCII case",
                "is not equal to ignoring ASCII case",
            )
            .expected(render.value(&self.0))
    }
}

property_expectation! {
    /// Checks the Unicode `Lowercase` property.
    pub struct IsLowercase for char;
    kind Predicate;
    check |actual| actual.is_lowercase();
    relations "is lowercase", "is not lowercase";
}

property_expectation! {
    /// Checks the Unicode `Uppercase` property.
    pub struct IsUppercase for char;
    kind Predicate;
    check |actual| actual.is_uppercase();
    relations "is uppercase", "is not uppercase";
}

property_expectation! {
    /// Checks for an ASCII lowercase letter.
    pub struct IsAsciiLowercase for char;
    kind Predicate;
    check |actual| actual.is_ascii_lowercase();
    relations "is an ASCII lowercase letter", "is not an ASCII lowercase letter";
}

property_expectation! {
    /// Checks for an ASCII uppercase letter.
    pub struct IsAsciiUppercase for char;
    kind Predicate;
    check |actual| actual.is_ascii_uppercase();
    relations "is an ASCII uppercase letter", "is not an ASCII uppercase letter";
}

/// Assertions for character values.
#[allow(clippy::return_self_not_must_use)]
#[cfg_attr(feature = "fluent", assertr_macros::fluent_aliases)]
pub trait CharAssertions<R = DebugRenderer> {
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
}

impl<M: Mode, R> CharAssertions<R> for AssertThat<'_, char, M, R> {
    #[track_caller]
    fn is_equal_to_ignoring_ascii_case(self, expected: char) -> Self
    where
        R: ValueRenderer<char>,
    {
        self.matches(EqualToIgnoringAsciiCase::new(expected))
    }

    #[track_caller]
    fn is_lowercase(self) -> Self
    where
        R: ValueRenderer<char>,
    {
        self.matches(IsLowercase)
    }

    #[track_caller]
    fn is_uppercase(self) -> Self
    where
        R: ValueRenderer<char>,
    {
        self.matches(IsUppercase)
    }

    #[track_caller]
    fn is_ascii_lowercase(self) -> Self
    where
        R: ValueRenderer<char>,
    {
        self.matches(IsAsciiLowercase)
    }

    #[track_caller]
    fn is_ascii_uppercase(self) -> Self
    where
        R: ValueRenderer<char>,
    {
        self.matches(IsAsciiUppercase)
    }
}

#[cfg(test)]
mod tests {
    #[cfg(feature = "fluent")]
    mod fluent_aliases {
        use crate::prelude::*;

        #[test]
        fn are_as_expected() {
            'a'.must().be_equal_to_ignoring_ascii_case('A');
            'a'.must().be_lowercase();
            'A'.must().be_uppercase();
            'a'.must().be_ascii_lowercase();
            'A'.must().be_ascii_uppercase();
        }
    }

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

            assert_that!(failures[0].to_string()).contains(SENTINEL);
        }
    }

    mod is_equal_to_ignoring_ascii_case {
        use crate::prelude::*;
        use indoc::formatdoc;

        #[test]
        fn caller_location_is_as_expected() {
            assert_caller_location!(assert_that!('a'), is_equal_to_ignoring_ascii_case('B'));
        }

        #[test]
        fn succeeds_when_equal_ignoring_ascii_case() {
            assert_that!('a').is_equal_to_ignoring_ascii_case('A');
        }

        #[test]
        fn panics_when_not_equal_to_ignoring_ascii_case() {
            assert_that!(|| {
                assert_that!('a')
                    .with_location(false)
                    .is_equal_to_ignoring_ascii_case('B')
            })
            .panics()
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
        fn caller_location_is_as_expected() {
            assert_caller_location!(assert_that!('A'), is_lowercase());
        }

        #[test]
        fn succeeds_when_lowercase() {
            assert_that!('a').is_lowercase();
        }

        #[test]
        fn panics_when_not_lowercase() {
            assert_that!(|| assert_that!('A').with_location(false).is_lowercase())
                .panics()
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

        #[test]
        fn caller_location_is_as_expected() {
            assert_caller_location!(assert_that!('a'), is_uppercase());
        }

        #[test]
        fn succeeds_when_uppercase() {
            assert_that!('A').is_uppercase();
        }

        #[test]
        fn reports_the_negated_relation() {
            let failures = assert_that!('a').capture(CharAssertions::is_uppercase);
            assert_that!(failures[0].relation.as_deref()).is_equal_to(Some("is not uppercase"));
        }
    }

    mod is_ascii_lowercase {
        use crate::prelude::*;

        #[test]
        fn caller_location_is_as_expected() {
            assert_caller_location!(assert_that!('A'), is_ascii_lowercase());
        }

        #[test]
        fn succeeds_when_ascii_lowercase() {
            assert_that!('a').is_ascii_lowercase();
        }

        #[test]
        fn reports_the_negated_relation() {
            let failures = assert_that!('A').capture(CharAssertions::is_ascii_lowercase);
            assert_that!(failures[0].relation.as_deref())
                .is_equal_to(Some("is not an ASCII lowercase letter"));
        }
    }

    mod is_ascii_uppercase {
        use crate::prelude::*;

        #[test]
        fn caller_location_is_as_expected() {
            assert_caller_location!(assert_that!('a'), is_ascii_uppercase());
        }

        #[test]
        fn succeeds_when_ascii_uppercase() {
            assert_that!('A').is_ascii_uppercase();
        }

        #[test]
        fn reports_the_negated_relation() {
            let failures = assert_that!('a').capture(CharAssertions::is_ascii_uppercase);
            assert_that!(failures[0].relation.as_deref())
                .is_equal_to(Some("is not an ASCII uppercase letter"));
        }
    }
}
