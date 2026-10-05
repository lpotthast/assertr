use crate::{
    AssertThat, DebugRenderer, ExpectationDiagnostics, Mode, ValueRenderer, renderer::IntoRendered,
};

/// Defines a string expectation evaluated on the subject's [`AsRef<str>`] view.
///
/// A property (`pub struct IsBlank;`) checks the view alone. An operand expectation
/// (`pub struct Contains<E>;`) owns an [`AsRef<str>`] operand and compares both views. `role`
/// selects whether diagnostics report the operand as the expected or unexpected value. Rejections
/// retain the views, and failures render the view only when no caller supplied the actual value.
macro_rules! str_expectation {
    (
        $(#[$attr:meta])*
        pub struct $name:ident;
        kind $kind:ident;
        check |$actual:ident| $check:expr;
        relations $relation:literal, $negated:literal;
    ) => {
        $(#[$attr])*
        pub struct $name;

        impl<T: AsRef<str> + ?Sized, R> $crate::Expectation<T, R> for $name {
            type Success<'a>
                = ()
            where
                T: 'a;
            type Rejection<'a>
                = &'a str
            where
                T: 'a;

            fn evaluate<'a>(
                &'a self,
                actual: &'a T,
                _: &$crate::AssertionContext<'_, R>,
            ) -> Result<(), &'a str> {
                let $actual = actual.as_ref();
                if $check { Ok(()) } else { Err($actual) }
            }
        }

        impl<T: AsRef<str> + ?Sized, R: ValueRenderer<str>> ExpectationDiagnostics<T, R> for $name {
            const KIND: $crate::failure::FailureKind = $crate::failure::FailureKind::$kind;

            fn explain<'a, Target>(
                &'a self,
                rejected: Option<(&'a T, &'a str)>,
                failure: $crate::failure::FailureBuilder<Target>,
                context: &$crate::AssertionContext<'_, R>,
            ) -> $crate::failure::FailureBuilder<Target> {
                match rejected {
                    None => failure.relation($relation),
                    Some((_, actual)) => failure
                        .actual_or_else(|| context.render().value(actual).into_rendered())
                        .relation($negated),
                }
            }
        }
    };
    (
        $(#[$attr:meta])*
        pub struct $name:ident<E>;
        kind $kind:ident;
        check |$actual:ident, $operand:ident| $check:expr;
        relations $relation:literal, $negated:literal;
        role $role:ident;
    ) => {
        $(#[$attr])*
        pub struct $name<E>(E);

        impl<E> $name<E> {
            #[doc = concat!(
                "Owns the ", stringify!($role), " operand, which may itself be a borrowed string."
            )]
            #[must_use]
            pub const fn new($role: E) -> Self {
                Self($role)
            }
        }

        impl<T: AsRef<str> + ?Sized, E: AsRef<str>, R> $crate::Expectation<T, R> for $name<E> {
            type Success<'a>
                = ()
            where
                Self: 'a,
                T: 'a;
            type Rejection<'a>
                = (&'a str, &'a str)
            where
                Self: 'a,
                T: 'a;

            fn evaluate<'a>(
                &'a self,
                actual: &'a T,
                _: &$crate::AssertionContext<'_, R>,
            ) -> Result<(), Self::Rejection<'a>> {
                let $actual = actual.as_ref();
                let $operand = self.0.as_ref();
                if $check { Ok(()) } else { Err(($actual, $operand)) }
            }
        }

        impl<T: AsRef<str> + ?Sized, E: AsRef<str>, R: ValueRenderer<str>>
            ExpectationDiagnostics<T, R> for $name<E>
        {
            const KIND: $crate::failure::FailureKind = $crate::failure::FailureKind::$kind;

            fn explain<'a, Target>(
                &'a self,
                rejected: Option<(&'a T, Self::Rejection<'a>)>,
                failure: $crate::failure::FailureBuilder<Target>,
                context: &$crate::AssertionContext<'_, R>,
            ) -> $crate::failure::FailureBuilder<Target> {
                let render = context.render();
                let (failure, operand) = match rejected {
                    None => (failure.relation($relation), self.0.as_ref()),
                    Some((_, (actual, operand))) => (
                        failure
                            .actual_or_else(|| render.value(actual).into_rendered())
                            .relation($negated),
                        operand,
                    ),
                };
                failure.$role(render.value(operand))
            }
        }
    };
}

str_expectation! {
    /// Checks that a string is empty or contains only Unicode whitespace.
    /// Matching renders the string view. Ordinary methods retain the original subject's renderer.
    pub struct IsBlank;
    kind Predicate;
    check |actual| actual.split_whitespace().next().is_none();
    relations "is blank", "is not blank";
}

str_expectation! {
    /// Checks that a string contains a character without the Unicode whitespace property.
    /// Matching renders the string view. Ordinary methods retain the original subject's renderer.
    pub struct IsNotBlank;
    kind Predicate;
    check |actual| actual.split_whitespace().next().is_some();
    relations "is not blank", "is unexpectedly blank";
}

str_expectation! {
    /// Checks that a string is empty or contains only ASCII whitespace.
    /// Matching renders the string view. Ordinary methods retain the original subject's renderer.
    pub struct IsBlankAscii;
    kind Predicate;
    check |actual| actual.split_ascii_whitespace().next().is_none();
    relations "is ASCII blank", "is not ASCII blank";
}

str_expectation! {
    /// Compares string views under ASCII case folding.
    /// Matching renders the string view. Ordinary methods retain the original subject's renderer.
    pub struct EqualToIgnoringAsciiCase<E>;
    kind Equality;
    check |actual, expected| actual.eq_ignore_ascii_case(expected);
    relations "is equal to ignoring ASCII case", "is not equal to ignoring ASCII case";
    role expected;
}

str_expectation! {
    /// Checks for an expected substring through [`AsRef<str>`].
    /// Matching renders the string view. Ordinary methods retain the original subject's renderer.
    pub struct Contains<E>;
    kind Membership;
    check |actual, expected| actual.contains(expected);
    relations "contains", "does not contain";
    role expected;
}

str_expectation! {
    /// Rejects strings containing an unexpected substring.
    /// Matching renders the string view. Ordinary methods retain the original subject's renderer.
    pub struct DoesNotContain<E>;
    kind Membership;
    check |actual, unexpected| !actual.contains(unexpected);
    relations "does not contain", "contains";
    role unexpected;
}

str_expectation! {
    /// A reusable string prefix assertion accepting [`AsRef<str>`] subjects and expected operands.
    ///
    /// Construct with [`new`](Self::new) and execute through an assertion chain or a supplied
    /// [`AssertionContext`](crate::AssertionContext). [`StrAssertions::starts_with`] executes this
    /// same definition on an assertion chain. Matching renders the subject's string view. The
    /// ordinary method supplies its original subject for rendering, preserving that subject's type
    /// and custom renderer.
    ///
    /// ```
    /// use assertr::prelude::*;
    /// use assertr::assertions::core::string::StartsWith;
    ///
    /// let prefix = StartsWith::new(String::from("hel"));
    /// assert_that!("hello").matches(&prefix);
    /// ```
    pub struct StartsWith<E>;
    kind Membership;
    check |actual, expected| actual.starts_with(expected);
    relations "starts with", "does not start with";
    role expected;
}

str_expectation! {
    /// Rejects strings starting with an unexpected prefix.
    /// Matching renders the string view. Ordinary methods retain the original subject's renderer.
    pub struct DoesNotStartWith<E>;
    kind Membership;
    check |actual, unexpected| !actual.starts_with(unexpected);
    relations "does not start with", "starts with";
    role unexpected;
}

str_expectation! {
    /// Checks for an expected string suffix through [`AsRef<str>`].
    /// Matching renders the string view. Ordinary methods retain the original subject's renderer.
    pub struct EndsWith<E>;
    kind Membership;
    check |actual, expected| actual.ends_with(expected);
    relations "ends with", "does not end with";
    role expected;
}

str_expectation! {
    /// Rejects strings ending with an unexpected suffix.
    /// Matching renders the string view. Ordinary methods retain the original subject's renderer.
    pub struct DoesNotEndWith<E>;
    kind Membership;
    check |actual, unexpected| !actual.ends_with(unexpected);
    relations "does not end with", "ends with";
    role unexpected;
}

/// String-specific assertions.
///
/// Blanket-implemented for every subject that is `AsRef<str>`, so `&str`, `String`, `&String`,
/// `Box<str>`, and `Cow<str>` all share one implementation and one set of failure messages.
/// Failures render the original subject `S` and string operands as `str`.
#[allow(clippy::return_self_not_must_use)]
#[cfg_attr(feature = "fluent", assertr_macros::fluent_aliases)]
pub trait StrAssertions<S, R = DebugRenderer> {
    /// Asserts that the subject is empty or contains only Unicode `White_Space` characters.
    fn is_blank(self) -> Self
    where
        R: ValueRenderer<S> + ValueRenderer<str>;

    /// Asserts that the subject contains at least one character without the Unicode `White_Space`
    /// property.
    fn is_not_blank(self) -> Self
    where
        R: ValueRenderer<S> + ValueRenderer<str>;

    /// Asserts that the subject is empty or contains only ASCII whitespace.
    fn is_blank_ascii(self) -> Self
    where
        R: ValueRenderer<S> + ValueRenderer<str>;

    /// Asserts that the subject and `expected` are equal under ASCII case folding.
    fn is_equal_to_ignoring_ascii_case(self, expected: impl AsRef<str>) -> Self
    where
        R: ValueRenderer<S> + ValueRenderer<str>;

    /// Asserts that the subject contains `expected` as a substring.
    fn contains(self, expected: impl AsRef<str>) -> Self
    where
        R: ValueRenderer<S> + ValueRenderer<str>;

    /// Asserts that the subject does not contain `unexpected` as a substring.
    fn does_not_contain(self, unexpected: impl AsRef<str>) -> Self
    where
        R: ValueRenderer<S> + ValueRenderer<str>;

    /// Asserts that the subject starts with `expected`.
    fn starts_with(self, expected: impl AsRef<str>) -> Self
    where
        R: ValueRenderer<S> + ValueRenderer<str>;

    /// Asserts that the subject does not start with `unexpected`.
    fn does_not_start_with(self, unexpected: impl AsRef<str>) -> Self
    where
        R: ValueRenderer<S> + ValueRenderer<str>;

    /// Asserts that the subject ends with `expected`.
    fn ends_with(self, expected: impl AsRef<str>) -> Self
    where
        R: ValueRenderer<S> + ValueRenderer<str>;

    /// Asserts that the subject does not end with `unexpected`.
    fn does_not_end_with(self, unexpected: impl AsRef<str>) -> Self
    where
        R: ValueRenderer<S> + ValueRenderer<str>;
}

impl<S: AsRef<str>, M: Mode, R> AssertThat<'_, S, M, R> {
    /// Applies a string expectation while rendering the original subject, not its string view.
    ///
    /// This tracks once through `apply_assertion_with_failure` and preserves the caller location.
    #[track_caller]
    fn apply_str_expectation<D: ExpectationDiagnostics<S, R>>(self, definition: D) -> Self
    where
        R: ValueRenderer<S>,
    {
        self.apply_assertion_with_failure(definition, |assertion, failure| {
            failure.actual(assertion.render().value(assertion.actual()))
        })
    }
}

impl<S: AsRef<str>, M: Mode, R> StrAssertions<S, R> for AssertThat<'_, S, M, R> {
    #[track_caller]
    fn is_blank(self) -> Self
    where
        R: ValueRenderer<S> + ValueRenderer<str>,
    {
        self.apply_str_expectation(IsBlank)
    }

    #[track_caller]
    fn is_not_blank(self) -> Self
    where
        R: ValueRenderer<S> + ValueRenderer<str>,
    {
        self.apply_str_expectation(IsNotBlank)
    }

    #[track_caller]
    fn is_blank_ascii(self) -> Self
    where
        R: ValueRenderer<S> + ValueRenderer<str>,
    {
        self.apply_str_expectation(IsBlankAscii)
    }

    #[track_caller]
    fn is_equal_to_ignoring_ascii_case(self, expected: impl AsRef<str>) -> Self
    where
        R: ValueRenderer<S> + ValueRenderer<str>,
    {
        self.apply_str_expectation(EqualToIgnoringAsciiCase::new(expected))
    }

    #[track_caller]
    fn contains(self, expected: impl AsRef<str>) -> Self
    where
        R: ValueRenderer<S> + ValueRenderer<str>,
    {
        self.apply_str_expectation(Contains::new(expected))
    }

    #[track_caller]
    fn does_not_contain(self, unexpected: impl AsRef<str>) -> Self
    where
        R: ValueRenderer<S> + ValueRenderer<str>,
    {
        self.apply_str_expectation(DoesNotContain::new(unexpected))
    }

    #[track_caller]
    fn starts_with(self, expected: impl AsRef<str>) -> Self
    where
        R: ValueRenderer<S> + ValueRenderer<str>,
    {
        self.apply_str_expectation(StartsWith::new(expected))
    }

    #[track_caller]
    fn does_not_start_with(self, unexpected: impl AsRef<str>) -> Self
    where
        R: ValueRenderer<S> + ValueRenderer<str>,
    {
        self.apply_str_expectation(DoesNotStartWith::new(unexpected))
    }

    #[track_caller]
    fn ends_with(self, expected: impl AsRef<str>) -> Self
    where
        R: ValueRenderer<S> + ValueRenderer<str>,
    {
        self.apply_str_expectation(EndsWith::new(expected))
    }

    #[track_caller]
    fn does_not_end_with(self, unexpected: impl AsRef<str>) -> Self
    where
        R: ValueRenderer<S> + ValueRenderer<str>,
    {
        self.apply_str_expectation(DoesNotEndWith::new(unexpected))
    }
}

#[cfg(test)]
mod tests {
    mod evaluation {
        use crate::{prelude::*, test_support::SentinelRenderer};
        use core::cell::Cell;

        struct Text<'a> {
            value: &'a str,
            calls: &'a Cell<usize>,
        }
        impl AsRef<str> for Text<'_> {
            fn as_ref(&self) -> &str {
                self.calls.set(self.calls.get() + 1);
                self.value
            }
        }

        #[test]
        fn converts_each_operand_once_for_passing_and_failing_checks() {
            for value in ["abc", " \t"] {
                let actual_calls = Cell::new(0);
                let expected_calls = Cell::new(0);
                let actual = Text {
                    value,
                    calls: &actual_calls,
                };
                let operand = || Text {
                    value: "a",
                    calls: &expected_calls,
                };
                let failures = assert_that!(actual)
                    .with_renderer(SentinelRenderer)
                    .capture(|it| {
                        let it = it.is_blank().is_not_blank().is_blank_ascii();
                        assert_that!(actual_calls.get()).is_equal_to(3);
                        let it = it.is_equal_to_ignoring_ascii_case(operand());
                        assert_that!((actual_calls.get(), expected_calls.get()))
                            .is_equal_to((4, 1));
                        let it = it.contains(operand());
                        assert_that!((actual_calls.get(), expected_calls.get()))
                            .is_equal_to((5, 2));
                        let it = it.does_not_contain(operand());
                        assert_that!((actual_calls.get(), expected_calls.get()))
                            .is_equal_to((6, 3));
                        let it = it.starts_with(operand());
                        assert_that!((actual_calls.get(), expected_calls.get()))
                            .is_equal_to((7, 4));
                        let it = it.does_not_start_with(operand());
                        assert_that!((actual_calls.get(), expected_calls.get()))
                            .is_equal_to((8, 5));
                        let it = it.ends_with(operand());
                        assert_that!((actual_calls.get(), expected_calls.get()))
                            .is_equal_to((9, 6));
                        let it = it.does_not_end_with(operand());
                        assert_that!((actual_calls.get(), expected_calls.get()))
                            .is_equal_to((10, 7));
                        it
                    });
                assert_that!(failures).is_not_empty();
            }
        }

        #[test]
        #[cfg(feature = "std")]
        fn tracks_before_string_conversion_panics() {
            struct PanickingText;
            impl AsRef<str> for PanickingText {
                fn as_ref(&self) -> &str {
                    panic!("conversion panicked")
                }
            }
            let failures = assert_that!("text").capture(|it| {
                let child = it.derive(|value| value);
                let outcome = std::panic::catch_unwind(core::panic::AssertUnwindSafe(|| {
                    child.contains(PanickingText);
                }));
                assert_that!(outcome).is_err();
                it
            });
            assert_that!(failures).is_empty();
        }
    }

    mod renderer_contract {
        use crate::{
            Expectation,
            assertions::core::string::StartsWith,
            prelude::*,
            test_support::{NoRenderer, assert_trait_impl},
        };

        #[test]
        fn trait_is_implemented_without_renderer_support() {
            assert_trait_impl!(
                AssertThat<'static, &'static str, Panic, NoRenderer>
                    => StrAssertions<&'static str, NoRenderer>
            );
            assert_trait_impl!(StartsWith<&'static str> => Expectation<str, NoRenderer>);

            assert_trait_impl!(super::super::IsBlank => crate::Expectation<str, NoRenderer>);
            assert_trait_impl!(super::super::IsNotBlank => crate::Expectation<str, NoRenderer>);
            assert_trait_impl!(super::super::IsBlankAscii => crate::Expectation<str, NoRenderer>);
            assert_trait_impl!(super::super::EqualToIgnoringAsciiCase<&'static str> => crate::Expectation<str, NoRenderer>);
            assert_trait_impl!(super::super::Contains<&'static str> => crate::Expectation<str, NoRenderer>);
            assert_trait_impl!(super::super::DoesNotContain<&'static str> => crate::Expectation<str, NoRenderer>);
            assert_trait_impl!(super::super::DoesNotStartWith<&'static str> => crate::Expectation<str, NoRenderer>);
            assert_trait_impl!(super::super::EndsWith<&'static str> => crate::Expectation<str, NoRenderer>);
            assert_trait_impl!(super::super::DoesNotEndWith<&'static str> => crate::Expectation<str, NoRenderer>);
        }
    }

    mod is_blank {
        use crate::prelude::*;
        use indoc::formatdoc;

        #[test]
        #[cfg(feature = "fluent")]
        fn fluent_alias_is_as_expected() {
            "".must().be_blank();
        }

        #[test]
        fn caller_location_is_as_expected() {
            assert_caller_location!(assert_that!("a"), is_blank());
        }

        #[test]
        fn succeeds_when_expected_is_blank() {
            assert_that!("").is_blank();
            assert_that!(" ").is_blank();
            assert_that!("\t \n").is_blank();
            assert_that!(String::from("\t \n")).is_blank();
        }

        #[test]
        fn panics_when_expected_is_not_blank() {
            assert_that_panic_by(|| {
                assert_that!("a").with_location(false).is_blank();
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {r#"
                -------- assertr --------
                Expression: `"a"`

                Actual: "a"

                is not blank
                -------- assertr --------
            "#});
        }
    }

    mod is_blank_ascii {
        use crate::prelude::*;
        use indoc::formatdoc;

        #[test]
        #[cfg(feature = "fluent")]
        fn fluent_alias_is_as_expected() {
            "".must().be_blank_ascii();
        }

        #[test]
        fn caller_location_is_as_expected() {
            assert_caller_location!(assert_that!("a"), is_blank_ascii());
        }

        #[test]
        fn succeeds_when_blank() {
            assert_that!("").is_blank_ascii();
            assert_that!(" ").is_blank_ascii();
            assert_that!("\t \n").is_blank_ascii();
            assert_that!(String::from("\t \n")).is_blank_ascii();
        }

        #[test]
        fn panics_when_not_ascii_blank() {
            assert_that_panic_by(|| {
                assert_that!("a").with_location(false).is_blank_ascii();
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {r#"
                -------- assertr --------
                Expression: `"a"`

                Actual: "a"

                is not ASCII blank
                -------- assertr --------
            "#});
        }

        #[test]
        fn identifies_unicode_whitespace_as_non_ascii_whitespace() {
            assert_that_panic_by(|| {
                assert_that!("\u{a0}").with_location(false).is_blank_ascii();
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {r#"
                -------- assertr --------
                Expression: `"\u{{a0}}"`

                Actual: "\u{{a0}}"

                is not ASCII blank
                -------- assertr --------
            "#});
        }
    }

    mod is_not_blank {
        use crate::prelude::*;
        use indoc::formatdoc;

        #[test]
        #[cfg(feature = "fluent")]
        fn fluent_alias_is_as_expected() {
            "a".must().not_be_blank();
        }

        #[test]
        fn caller_location_is_as_expected() {
            assert_caller_location!(assert_that!("\t \n"), is_not_blank());
        }

        #[test]
        fn succeeds_when_not_blank() {
            assert_that!("a").is_not_blank();
            assert_that!(" \n a \t").is_not_blank();
            assert_that!(String::from("hello")).is_not_blank();
        }

        #[test]
        fn panics_when_blank() {
            assert_that_panic_by(|| {
                assert_that!("\t \n").with_location(false).is_not_blank();
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {r#"
                -------- assertr --------
                Expression: `"\t \n"`

                Actual: "\t \n"

                is unexpectedly blank
                -------- assertr --------
            "#});
        }
    }

    mod is_equal_to_ignoring_ascii_case {
        use crate::prelude::*;
        use indoc::formatdoc;

        #[test]
        #[cfg(feature = "fluent")]
        fn fluent_alias_is_as_expected() {
            "FoObAr".must().be_equal_to_ignoring_ascii_case("fOoBaR");
        }

        #[test]
        fn caller_location_is_as_expected() {
            assert_caller_location!(assert_that!("foo"), is_equal_to_ignoring_ascii_case("bar"));
        }

        #[test]
        fn renders_original_operands_and_can_redact_them() {
            use indoc::formatdoc;

            use crate::test_support::{
                CustomValueRenderer, RedactingRenderer, assert_custom_value, assert_redacted,
            };
            let subject = String::from("private-value");
            let operand = "other-value";
            let failures = assert_that!(subject)
                .with_renderer(CustomValueRenderer)
                .with_location(false)
                .capture(|it| it.is_equal_to_ignoring_ascii_case(operand));
            assert_that!(failures).contains_exactly_satisfying([
                |element: AssertThat<AssertionFailure, Capture>| {
                    element
                        .derive(|value| value)
                        .has_text_report(formatdoc! {r#"
                -------- assertr --------
                Expression: `subject`

                Actual: custom("private-value")

                is not equal to ignoring ASCII case

                Expected: custom("other-value")
                -------- assertr --------
            "#});

                    assert_custom_value(element.actual().actual.as_ref().unwrap(), &subject);
                    assert_custom_value(element.actual().expected.as_ref().unwrap(), operand);
                },
            ]);
            let failures = assert_that!(subject)
                .with_renderer(RedactingRenderer)
                .with_location(false)
                .capture(|it| it.is_equal_to_ignoring_ascii_case(operand));
            assert_that!(failures).contains_exactly_satisfying([
                |element: AssertThat<AssertionFailure, Capture>| {
                    element.derive(|value| value).has_text_report(formatdoc! {r"
                -------- assertr --------
                Expression: `subject`

                Actual: <redacted>

                is not equal to ignoring ASCII case

                Expected: <redacted>
                -------- assertr --------
            "});

                    assert_redacted(element.actual(), &[subject.as_str(), operand]);
                },
            ]);
        }

        #[test]
        fn succeeds_when_equal_ignoring_ascii_case() {
            assert_that!("FoObAr").is_equal_to_ignoring_ascii_case("fOoBaR");
            assert_that!(String::from("FoObAr")).is_equal_to_ignoring_ascii_case("fOoBaR");
        }

        #[test]
        fn panics_when_not_equal_to_ignoring_ascii_case() {
            assert_that_panic_by(|| {
                assert_that!("foo")
                    .with_location(false)
                    .is_equal_to_ignoring_ascii_case("bar");
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {r#"
                -------- assertr --------
                Expression: `"foo"`

                Actual: "foo"

                is not equal to ignoring ASCII case

                Expected: "bar"
                -------- assertr --------
            "#});
        }

        #[test]
        fn does_not_fold_non_ascii_case_differences() {
            assert_that_panic_by(|| {
                assert_that!("straße")
                    .with_location(false)
                    .is_equal_to_ignoring_ascii_case("STRAẞE");
            })
            .has_type::<String>()
            .contains("ignoring ASCII case");
        }
    }

    mod contains {
        use crate::prelude::*;
        use indoc::formatdoc;

        #[test]
        #[cfg(feature = "fluent")]
        fn fluent_alias_is_as_expected() {
            "foobar".must().contain("foo");
        }

        #[test]
        fn caller_location_is_as_expected() {
            assert_caller_location!(assert_that!("foo bar baz"), contains("42"));
        }

        #[test]
        fn renders_original_operands_and_can_redact_them() {
            use indoc::formatdoc;

            use crate::test_support::{
                CustomValueRenderer, RedactingRenderer, assert_custom_value, assert_redacted,
            };
            let subject = String::from("private-value");
            let operand = "other-value";
            let failures = assert_that!(subject)
                .with_renderer(CustomValueRenderer)
                .with_location(false)
                .capture(|it| it.contains(operand));
            assert_that!(failures).contains_exactly_satisfying([
                |element: AssertThat<AssertionFailure, Capture>| {
                    element
                        .derive(|value| value)
                        .has_text_report(formatdoc! {r#"
                -------- assertr --------
                Expression: `subject`

                Actual: custom("private-value")

                does not contain

                Expected: custom("other-value")
                -------- assertr --------
            "#});

                    assert_custom_value(element.actual().actual.as_ref().unwrap(), &subject);
                    assert_custom_value(element.actual().expected.as_ref().unwrap(), operand);
                },
            ]);
            let failures = assert_that!(subject)
                .with_renderer(RedactingRenderer)
                .with_location(false)
                .capture(|it| it.contains(operand));
            assert_that!(failures).contains_exactly_satisfying([
                |element: AssertThat<AssertionFailure, Capture>| {
                    element.derive(|value| value).has_text_report(formatdoc! {r"
                -------- assertr --------
                Expression: `subject`

                Actual: <redacted>

                does not contain

                Expected: <redacted>
                -------- assertr --------
            "});

                    assert_redacted(element.actual(), &[subject.as_str(), operand]);
                },
            ]);
        }

        #[test]
        fn succeeds_when_expected_is_contained() {
            assert_that!("foobar").contains("foo");
            assert_that!("foobar").contains("bar");
            assert_that!("foobar").contains("oob");
            assert_that!(String::from("foobar")).contains("oob");
        }

        #[test]
        fn panics_when_expected_is_not_contained() {
            assert_that_panic_by(|| {
                assert_that!("foo bar baz")
                    .with_location(false)
                    .contains("42");
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {r#"
                -------- assertr --------
                Expression: `"foo bar baz"`

                Actual: "foo bar baz"

                does not contain

                Expected: "42"
                -------- assertr --------
            "#});
        }

        #[test]
        fn renders_the_string_subject_with_debug_format() {
            assert_that_panic_by(|| {
                assert_that!(String::from("abc"))
                    .with_location(false)
                    .with_renderer(crate::test_support::CustomValueRenderer)
                    .contains("z");
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {r#"
                -------- assertr --------
                Expression: `String::from("abc")`

                Actual: custom("abc")

                does not contain

                Expected: custom("z")
                -------- assertr --------
            "#});
        }
    }

    mod does_not_contain {
        use crate::prelude::*;
        use indoc::formatdoc;

        #[test]
        #[cfg(feature = "fluent")]
        fn fluent_alias_is_as_expected() {
            "foobar".must().not_contain("baz");
        }

        #[test]
        fn caller_location_is_as_expected() {
            assert_caller_location!(assert_that!("foo bar baz"), does_not_contain("o b"));
        }

        #[test]
        fn renders_original_operands_and_can_redact_them() {
            use indoc::formatdoc;

            use crate::test_support::{
                CustomValueRenderer, RedactingRenderer, assert_custom_value, assert_redacted,
            };
            let subject = String::from("private-value");
            let operand = "private";
            let failures = assert_that!(subject)
                .with_renderer(CustomValueRenderer)
                .with_location(false)
                .capture(|it| it.does_not_contain(operand));
            assert_that!(failures).contains_exactly_satisfying([
                |element: AssertThat<AssertionFailure, Capture>| {
                    element
                        .derive(|value| value)
                        .has_text_report(formatdoc! {r#"
                -------- assertr --------
                Expression: `subject`

                Actual: custom("private-value")

                contains

                Unexpected: custom("private")
                -------- assertr --------
            "#});

                    assert_custom_value(element.actual().actual.as_ref().unwrap(), &subject);
                    assert_custom_value(element.actual().unexpected.as_ref().unwrap(), operand);
                },
            ]);
            let failures = assert_that!(subject)
                .with_renderer(RedactingRenderer)
                .with_location(false)
                .capture(|it| it.does_not_contain(operand));
            assert_that!(failures).contains_exactly_satisfying([
                |element: AssertThat<AssertionFailure, Capture>| {
                    element.derive(|value| value).has_text_report(formatdoc! {r"
                -------- assertr --------
                Expression: `subject`

                Actual: <redacted>

                contains

                Unexpected: <redacted>
                -------- assertr --------
            "});

                    assert_redacted(element.actual(), &[subject.as_str(), operand]);
                },
            ]);
        }

        #[test]
        fn succeeds_when_expected_is_not_contained() {
            assert_that!("foobar").does_not_contain("baz");
            assert_that!(String::from("foobar")).does_not_contain("baz");
        }

        #[test]
        fn panics_when_expected_is_contained() {
            assert_that_panic_by(|| {
                assert_that!("foo bar baz")
                    .with_location(false)
                    .does_not_contain("o b");
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {r#"
                -------- assertr --------
                Expression: `"foo bar baz"`

                Actual: "foo bar baz"

                contains

                Unexpected: "o b"
                -------- assertr --------
            "#});
        }
    }

    mod starts_with {
        use crate::prelude::*;
        use indoc::formatdoc;

        #[test]
        #[cfg(feature = "fluent")]
        fn fluent_alias_is_as_expected() {
            "foo bar baz".must().start_with("foo b");
        }

        #[test]
        fn caller_location_is_as_expected() {
            assert_caller_location!(assert_that!("foo bar baz"), starts_with("oo"));
        }

        #[test]
        fn accepts_unsized_strings() {
            use crate::assertions::core::string::StartsWith;

            let matcher = StartsWith::new("hel");
            let mut context = AssertionContext::default();

            assert_that!(context.evaluate("hello", &matcher)).is_true();
        }

        #[test]
        fn matcher_renders_retained_string_views_without_subject_renderer_bounds() {
            use crate::{assertions::core::string::StartsWith, test_support::rendered_text};
            use core::{cell::Cell, fmt};

            struct Text<'a>(&'a str, Cell<usize>);
            impl AsRef<str> for Text<'_> {
                fn as_ref(&self) -> &str {
                    self.1.set(self.1.get() + 1);
                    self.0
                }
            }

            struct StrRenderer;
            impl ValueRenderer<str> for StrRenderer {
                fn fmt(&self, value: &str, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                    f.write_str(value)
                }
            }

            let actual = Text("hello", Cell::new(0));
            let expected = Text("bye", Cell::new(0));
            let failures = assert_that!(actual)
                .with_renderer(StrRenderer)
                .capture(|it| it.matches(StartsWith::new(&expected)));
            assert_that!(actual.1.get()).is_equal_to(1);
            assert_that!(expected.1.get()).is_equal_to(1);
            assert_that!(failures).has_length(1);
            assert_that!(failures[0].children).is_empty();
            for (value, text) in [
                (&failures[0].actual, "hello"),
                (&failures[0].expected, "bye"),
            ] {
                let value = value.as_ref().unwrap();
                assert_that!(value.type_name).is_equal_to(Some("str"));
                assert_that!(rendered_text(value)).is_equal_to(text);
            }
        }

        #[test]
        fn renders_original_operands_and_can_redact_them() {
            use indoc::formatdoc;

            use crate::test_support::{
                CustomValueRenderer, RedactingRenderer, assert_custom_value, assert_redacted,
            };
            let subject = String::from("private-value");
            let operand = "other-value";
            let failures = assert_that!(subject)
                .with_renderer(CustomValueRenderer)
                .with_location(false)
                .capture(|it| it.starts_with(operand));
            assert_that!(failures).contains_exactly_satisfying([
                |element: AssertThat<AssertionFailure, Capture>| {
                    element
                        .derive(|value| value)
                        .has_text_report(formatdoc! {r#"
                -------- assertr --------
                Expression: `subject`

                Actual: custom("private-value")

                does not start with

                Expected: custom("other-value")
                -------- assertr --------
            "#});

                    assert_custom_value(element.actual().actual.as_ref().unwrap(), &subject);
                    assert_custom_value(element.actual().expected.as_ref().unwrap(), operand);
                },
            ]);
            let failures = assert_that!(subject)
                .with_renderer(RedactingRenderer)
                .with_location(false)
                .capture(|it| it.starts_with(operand));
            assert_that!(failures).contains_exactly_satisfying([
                |element: AssertThat<AssertionFailure, Capture>| {
                    element.derive(|value| value).has_text_report(formatdoc! {r"
                -------- assertr --------
                Expression: `subject`

                Actual: <redacted>

                does not start with

                Expected: <redacted>
                -------- assertr --------
            "});

                    assert_redacted(element.actual(), &[subject.as_str(), operand]);
                },
            ]);
        }

        #[test]
        fn succeeds_when_start_matches() {
            assert_that!("foo bar baz").starts_with("foo b");
            assert_that!(String::from("foo bar baz")).starts_with("foo b");
        }

        #[test]
        fn panics_when_start_is_different() {
            assert_that_panic_by(|| {
                assert_that!("foo bar baz")
                    .with_location(false)
                    .starts_with("oo");
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {r#"
                -------- assertr --------
                Expression: `"foo bar baz"`

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
        #[cfg(feature = "fluent")]
        fn fluent_alias_is_as_expected() {
            "foo bar baz".must().not_start_with("oo");
        }

        #[test]
        fn caller_location_is_as_expected() {
            assert_caller_location!(assert_that!("foo bar baz"), does_not_start_with("foo"));
        }

        #[test]
        fn renders_original_operands_and_can_redact_them() {
            use indoc::formatdoc;

            use crate::test_support::{
                CustomValueRenderer, RedactingRenderer, assert_custom_value, assert_redacted,
            };
            let subject = String::from("private-value");
            let operand = "private";
            let failures = assert_that!(subject)
                .with_renderer(CustomValueRenderer)
                .with_location(false)
                .capture(|it| it.does_not_start_with(operand));
            assert_that!(failures).contains_exactly_satisfying([
                |element: AssertThat<AssertionFailure, Capture>| {
                    element
                        .derive(|value| value)
                        .has_text_report(formatdoc! {r#"
                -------- assertr --------
                Expression: `subject`

                Actual: custom("private-value")

                starts with

                Unexpected: custom("private")
                -------- assertr --------
            "#});

                    assert_custom_value(element.actual().actual.as_ref().unwrap(), &subject);
                    assert_custom_value(element.actual().unexpected.as_ref().unwrap(), operand);
                },
            ]);
            let failures = assert_that!(subject)
                .with_renderer(RedactingRenderer)
                .with_location(false)
                .capture(|it| it.does_not_start_with(operand));
            assert_that!(failures).contains_exactly_satisfying([
                |element: AssertThat<AssertionFailure, Capture>| {
                    element.derive(|value| value).has_text_report(formatdoc! {r"
                -------- assertr --------
                Expression: `subject`

                Actual: <redacted>

                starts with

                Unexpected: <redacted>
                -------- assertr --------
            "});

                    assert_redacted(element.actual(), &[subject.as_str(), operand]);
                },
            ]);
        }

        #[test]
        fn succeeds_when_start_does_not_match() {
            assert_that!("foo bar baz").does_not_start_with("oo");
            assert_that!(String::from("foo bar baz")).does_not_start_with("oo");
        }

        #[test]
        fn panics_when_start_matches() {
            assert_that_panic_by(|| {
                assert_that!("foo bar baz")
                    .with_location(false)
                    .does_not_start_with("foo");
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {r#"
                -------- assertr --------
                Expression: `"foo bar baz"`

                Actual: "foo bar baz"

                starts with

                Unexpected: "foo"
                -------- assertr --------
            "#});
        }
    }

    mod ends_with {
        use crate::prelude::*;
        use indoc::formatdoc;

        #[test]
        #[cfg(feature = "fluent")]
        fn fluent_alias_is_as_expected() {
            "foo bar baz".must().end_with("r baz");
        }

        #[test]
        fn caller_location_is_as_expected() {
            assert_caller_location!(assert_that!("foo bar baz"), ends_with("raz"));
        }

        #[test]
        fn renders_original_operands_and_can_redact_them() {
            use indoc::formatdoc;

            use crate::test_support::{
                CustomValueRenderer, RedactingRenderer, assert_custom_value, assert_redacted,
            };
            let subject = String::from("private-value");
            let operand = "other-value";
            let failures = assert_that!(subject)
                .with_renderer(CustomValueRenderer)
                .with_location(false)
                .capture(|it| it.ends_with(operand));
            assert_that!(failures).contains_exactly_satisfying([
                |element: AssertThat<AssertionFailure, Capture>| {
                    element
                        .derive(|value| value)
                        .has_text_report(formatdoc! {r#"
                -------- assertr --------
                Expression: `subject`

                Actual: custom("private-value")

                does not end with

                Expected: custom("other-value")
                -------- assertr --------
            "#});

                    assert_custom_value(element.actual().actual.as_ref().unwrap(), &subject);
                    assert_custom_value(element.actual().expected.as_ref().unwrap(), operand);
                },
            ]);
            let failures = assert_that!(subject)
                .with_renderer(RedactingRenderer)
                .with_location(false)
                .capture(|it| it.ends_with(operand));
            assert_that!(failures).contains_exactly_satisfying([
                |element: AssertThat<AssertionFailure, Capture>| {
                    element.derive(|value| value).has_text_report(formatdoc! {r"
                -------- assertr --------
                Expression: `subject`

                Actual: <redacted>

                does not end with

                Expected: <redacted>
                -------- assertr --------
            "});

                    assert_redacted(element.actual(), &[subject.as_str(), operand]);
                },
            ]);
        }

        #[test]
        fn succeeds_when_end_matches() {
            assert_that!("foo bar baz").ends_with("r baz");
            assert_that!(String::from("foo bar baz")).ends_with("r baz");
        }

        #[test]
        fn panics_when_end_is_different() {
            assert_that_panic_by(|| {
                assert_that!("foo bar baz")
                    .with_location(false)
                    .ends_with("raz");
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {r#"
                -------- assertr --------
                Expression: `"foo bar baz"`

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
        #[cfg(feature = "fluent")]
        fn fluent_alias_is_as_expected() {
            "foo bar baz".must().not_end_with("y");
        }

        #[test]
        fn caller_location_is_as_expected() {
            assert_caller_location!(assert_that!("foo bar baz"), does_not_end_with("z"));
        }

        #[test]
        fn renders_original_operands_and_can_redact_them() {
            use indoc::formatdoc;

            use crate::test_support::{
                CustomValueRenderer, RedactingRenderer, assert_custom_value, assert_redacted,
            };
            let subject = String::from("private-value");
            let operand = "value";
            let failures = assert_that!(subject)
                .with_renderer(CustomValueRenderer)
                .with_location(false)
                .capture(|it| it.does_not_end_with(operand));
            assert_that!(failures).contains_exactly_satisfying([
                |element: AssertThat<AssertionFailure, Capture>| {
                    element
                        .derive(|value| value)
                        .has_text_report(formatdoc! {r#"
                -------- assertr --------
                Expression: `subject`

                Actual: custom("private-value")

                ends with

                Unexpected: custom("value")
                -------- assertr --------
            "#});

                    assert_custom_value(element.actual().actual.as_ref().unwrap(), &subject);
                    assert_custom_value(element.actual().unexpected.as_ref().unwrap(), operand);
                },
            ]);
            let failures = assert_that!(subject)
                .with_renderer(RedactingRenderer)
                .with_location(false)
                .capture(|it| it.does_not_end_with(operand));
            assert_that!(failures).contains_exactly_satisfying([
                |element: AssertThat<AssertionFailure, Capture>| {
                    element.derive(|value| value).has_text_report(formatdoc! {r"
                -------- assertr --------
                Expression: `subject`

                Actual: <redacted>

                ends with

                Unexpected: <redacted>
                -------- assertr --------
            "});

                    assert_redacted(element.actual(), &[subject.as_str(), operand]);
                },
            ]);
        }

        #[test]
        fn succeeds_when_end_does_match() {
            assert_that!("foo bar baz").does_not_end_with("y");
            assert_that!(String::from("foo bar baz")).does_not_end_with("y");
        }

        #[test]
        fn panics_when_end_is_matches() {
            assert_that_panic_by(|| {
                assert_that!("foo bar baz")
                    .with_location(false)
                    .does_not_end_with("z");
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {r#"
                -------- assertr --------
                Expression: `"foo bar baz"`

                Actual: "foo bar baz"

                ends with

                Unexpected: "z"
                -------- assertr --------
            "#});
        }
    }

    /// One blanket implementation serves every `AsRef<str>` subject, so all string-like types have
    /// to produce the same assertion-specific descriptions for the same content.
    mod every_string_like_type {
        use crate::prelude::*;
        use alloc::borrow::Cow;

        #[derive(Clone, Copy)]
        struct StringRenderer;

        impl ValueRenderer<&str> for StringRenderer {
            fn fmt(&self, value: &&str, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                f.write_fmt(format_args!("string({value})"))
            }
        }

        impl ValueRenderer<str> for StringRenderer {
            fn fmt(&self, value: &str, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                write!(f, "string({value})")
            }
        }

        #[test]
        fn all_string_like_subjects_pass_the_same_assertions() {
            assert_that!("foobar").starts_with("foo").ends_with("bar");
            assert_that!(String::from("foobar"))
                .starts_with("foo")
                .ends_with("bar");
            assert_that!(&String::from("foobar"))
                .starts_with("foo")
                .ends_with("bar");
            assert_that!(String::from("foobar").into_boxed_str())
                .starts_with("foo")
                .ends_with("bar");
            assert_that!(Cow::Borrowed("foobar"))
                .starts_with("foo")
                .ends_with("bar");
            assert_that!(Cow::<str>::Owned(String::from("foobar")))
                .starts_with("foo")
                .ends_with("bar");
        }

        #[test]
        fn all_string_like_subjects_produce_identical_descriptions() {
            let rendered = |failures: AssertionFailures| {
                let mut failures = failures.into_vec();
                for failure in &mut failures {
                    failure.expression = None;
                }
                failures
                    .iter()
                    .map(|failure| ToHumanReadableText.render(failure))
                    .collect::<Vec<_>>()
            };

            let reference = rendered(
                assert_that!("foobar")
                    .with_location(false)
                    .capture(|it| it.contains("baz")),
            );
            assert_that!(reference).has_length(1);

            let owned = String::from("foobar");
            assert_that!(rendered(
                assert_that!(owned)
                    .with_location(false)
                    .capture(|it| it.contains("baz"))
            ))
            .is_equal_to(reference.clone());

            let boxed = String::from("foobar").into_boxed_str();
            assert_that!(rendered(
                assert_that!(boxed)
                    .with_location(false)
                    .capture(|it| it.contains("baz"))
            ))
            .is_equal_to(reference.clone());

            let cow = Cow::Borrowed("foobar");
            assert_that!(rendered(
                assert_that!(cow)
                    .with_location(false)
                    .capture(|it| it.contains("baz"))
            ))
            .is_equal_to(reference);
        }

        #[test]
        fn failures_use_the_custom_renderer_for_the_subject() {
            let failures = assert_that!("foobar")
                .with_renderer(StringRenderer)
                .with_location(false)
                .capture(|it| it.contains("baz"));

            assert_that!(failures).contains_exactly_satisfying([
                |it: AssertThat<AssertionFailure, Capture>| {
                    it.has_text_report(indoc::formatdoc! {r#"
                        -------- assertr --------
                        Expression: `"foobar"`

                        Actual: string(foobar)

                        does not contain

                        Expected: string(baz)
                        -------- assertr --------
                    "#});
                },
            ]);
        }
    }
}
