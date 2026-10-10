use crate::{
    AssertThat, Mode,
    expectation::Expectation,
    renderer::{DebugRenderer, ValueRenderer},
};

/// Defines a string expectation evaluated on the subject's [`AsRef<str>`] view.
///
/// A property (`pub struct IsBlank;`) checks the view alone. An operand expectation
/// (`pub struct Contains<E>;`) owns an [`AsRef<str>`] operand and compares both views. `role`
/// selects whether diagnostics report the operand as the expected or unexpected value. Rejections
/// retain the views, and failures render the subject's string view.
macro_rules! str_expectation {
    (
        $(#[$attr:meta])*
        pub struct $name:ident;
        kind $kind:ident;
        check |$actual:ident| $check:expr;
        relations $relation:literal, $negated:literal;
    ) => {
        $(#[$attr])*
        #[derive(Debug, Clone, Copy)]
        pub struct $name;

        impl<T: AsRef<str> + ?Sized, R: ValueRenderer<str>> Expectation<T, R> for $name {
            type Success<'a>
                = ()
            where
                Self: 'a,
                T: 'a;
            type Rejection<'a>
                = &'a str
            where
                Self: 'a,
                T: 'a;

            fn evaluate<'a>(
                &'a self,
                actual: &'a T,
                _: &$crate::expectation::AssertionContext<'_, R>,
            ) -> Result<(), &'a str> {
                let $actual = actual.as_ref();
                if $check { Ok(()) } else { Err($actual) }
            }

            const KIND: $crate::failure::FailureKind = $crate::failure::FailureKind::$kind;

            fn explain<'a>(
                &'a self,
                rejected: Option<(&'a T, &'a str)>,
                failure: $crate::failure::FailureBuilder,
                context: &$crate::expectation::AssertionContext<'_, R>,
            ) -> $crate::failure::FailureBuilder {
                let render = context.render();
                failure.relations(
                    rejected.map(|(_, actual)| render.value(actual)),
                    $relation,
                    $negated,
                )
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
        #[derive(Debug, Clone)]
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

        impl<T: AsRef<str> + ?Sized, E: AsRef<str>, R: ValueRenderer<str>>
            Expectation<T, R> for $name<E>
        {
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
                _: &$crate::expectation::AssertionContext<'_, R>,
            ) -> Result<(), Self::Rejection<'a>> {
                let $actual = actual.as_ref();
                let $operand = self.0.as_ref();
                if $check { Ok(()) } else { Err(($actual, $operand)) }
            }

            const KIND: $crate::failure::FailureKind = $crate::failure::FailureKind::$kind;

            fn explain<'a>(
                &'a self,
                rejected: Option<(&'a T, Self::Rejection<'a>)>,
                failure: $crate::failure::FailureBuilder,
                context: &$crate::expectation::AssertionContext<'_, R>,
            ) -> $crate::failure::FailureBuilder {
                let render = context.render();
                let operand = rejected.map_or_else(|| self.0.as_ref(), |(_, (_, operand))| operand);
                failure
                    .relations(
                        rejected.map(|(_, (actual, _))| render.value(actual)),
                        $relation,
                        $negated,
                    )
                    .$role(render.value(operand))
            }
        }

    };
}

str_expectation! {
    /// Checks that a string is empty or contains only Unicode whitespace.
    pub struct IsBlank;
    kind Predicate;
    check |actual| actual.split_whitespace().next().is_none();
    relations "is blank", "is not blank";
}

str_expectation! {
    /// Checks that a string contains a character without the Unicode whitespace property.
    pub struct IsNotBlank;
    kind Predicate;
    check |actual| actual.split_whitespace().next().is_some();
    relations "is not blank", "is unexpectedly blank";
}

str_expectation! {
    /// Checks that a string is empty or contains only ASCII whitespace.
    pub struct IsAsciiBlank;
    kind Predicate;
    check |actual| actual.split_ascii_whitespace().next().is_none();
    relations "is ASCII blank", "is not ASCII blank";
}

str_expectation! {
    /// Compares string views under ASCII case folding.
    pub struct EqualToIgnoringAsciiCase<E>;
    kind Equality;
    check |actual, expected| actual.eq_ignore_ascii_case(expected);
    relations "is equal to ignoring ASCII case", "is not equal to ignoring ASCII case";
    role expected;
}

str_expectation! {
    /// Checks for an expected substring through [`AsRef<str>`].
    pub struct Contains<E>;
    kind Membership;
    check |actual, expected| actual.contains(expected);
    relations "contains", "does not contain";
    role expected;
}

str_expectation! {
    /// Rejects strings containing an unexpected substring.
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
    /// [`AssertionContext`](crate::expectation::AssertionContext). [`StrAssertions::starts_with`] executes this
    /// same definition on an assertion chain.
    ///
    /// ```
    /// use assertr::prelude::*;
    /// use assertr::matchers::string::StartsWith;
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
    pub struct DoesNotStartWith<E>;
    kind Membership;
    check |actual, unexpected| !actual.starts_with(unexpected);
    relations "does not start with", "starts with";
    role unexpected;
}

str_expectation! {
    /// Checks for an expected string suffix through [`AsRef<str>`].
    pub struct EndsWith<E>;
    kind Membership;
    check |actual, expected| actual.ends_with(expected);
    relations "ends with", "does not end with";
    role expected;
}

str_expectation! {
    /// Rejects strings ending with an unexpected suffix.
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
/// Failures render the subject and string operands as `str`.
#[allow(clippy::return_self_not_must_use)]
#[cfg_attr(feature = "fluent", assertr_macros::fluent_aliases)]
pub trait StrAssertions<R = DebugRenderer> {
    /// Asserts that the subject is empty or contains only Unicode `White_Space` characters.
    fn is_blank(self) -> Self
    where
        R: ValueRenderer<str>;

    /// Asserts that the subject contains at least one character without the Unicode `White_Space`
    /// property.
    fn is_not_blank(self) -> Self
    where
        R: ValueRenderer<str>;

    /// Asserts that the subject is empty or contains only ASCII whitespace.
    fn is_ascii_blank(self) -> Self
    where
        R: ValueRenderer<str>;

    /// Asserts that the subject and `expected` are equal under ASCII case folding.
    fn is_equal_to_ignoring_ascii_case(self, expected: impl AsRef<str>) -> Self
    where
        R: ValueRenderer<str>;

    /// Asserts that the subject contains `expected` as a substring.
    fn contains(self, expected: impl AsRef<str>) -> Self
    where
        R: ValueRenderer<str>;

    /// Asserts that the subject does not contain `unexpected` as a substring.
    fn does_not_contain(self, unexpected: impl AsRef<str>) -> Self
    where
        R: ValueRenderer<str>;

    /// Asserts that the subject starts with `expected`.
    fn starts_with(self, expected: impl AsRef<str>) -> Self
    where
        R: ValueRenderer<str>;

    /// Asserts that the subject does not start with `unexpected`.
    fn does_not_start_with(self, unexpected: impl AsRef<str>) -> Self
    where
        R: ValueRenderer<str>;

    /// Asserts that the subject ends with `expected`.
    fn ends_with(self, expected: impl AsRef<str>) -> Self
    where
        R: ValueRenderer<str>;

    /// Asserts that the subject does not end with `unexpected`.
    fn does_not_end_with(self, unexpected: impl AsRef<str>) -> Self
    where
        R: ValueRenderer<str>;
}

impl<S: AsRef<str>, M: Mode, R> StrAssertions<R> for AssertThat<'_, S, M, R> {
    #[track_caller]
    fn is_blank(self) -> Self
    where
        R: ValueRenderer<str>,
    {
        self.matches(IsBlank)
    }

    #[track_caller]
    fn is_not_blank(self) -> Self
    where
        R: ValueRenderer<str>,
    {
        self.matches(IsNotBlank)
    }

    #[track_caller]
    fn is_ascii_blank(self) -> Self
    where
        R: ValueRenderer<str>,
    {
        self.matches(IsAsciiBlank)
    }

    #[track_caller]
    fn is_equal_to_ignoring_ascii_case(self, expected: impl AsRef<str>) -> Self
    where
        R: ValueRenderer<str>,
    {
        self.matches(EqualToIgnoringAsciiCase::new(expected))
    }

    #[track_caller]
    fn contains(self, expected: impl AsRef<str>) -> Self
    where
        R: ValueRenderer<str>,
    {
        self.matches(Contains::new(expected))
    }

    #[track_caller]
    fn does_not_contain(self, unexpected: impl AsRef<str>) -> Self
    where
        R: ValueRenderer<str>,
    {
        self.matches(DoesNotContain::new(unexpected))
    }

    #[track_caller]
    fn starts_with(self, expected: impl AsRef<str>) -> Self
    where
        R: ValueRenderer<str>,
    {
        self.matches(StartsWith::new(expected))
    }

    #[track_caller]
    fn does_not_start_with(self, unexpected: impl AsRef<str>) -> Self
    where
        R: ValueRenderer<str>,
    {
        self.matches(DoesNotStartWith::new(unexpected))
    }

    #[track_caller]
    fn ends_with(self, expected: impl AsRef<str>) -> Self
    where
        R: ValueRenderer<str>,
    {
        self.matches(EndsWith::new(expected))
    }

    #[track_caller]
    fn does_not_end_with(self, unexpected: impl AsRef<str>) -> Self
    where
        R: ValueRenderer<str>,
    {
        self.matches(DoesNotEndWith::new(unexpected))
    }
}

#[cfg(test)]
mod tests {
    #[cfg(feature = "fluent")]
    mod fluent_aliases {
        use crate::prelude::*;

        #[test]
        fn are_as_expected() {
            "".must().be_blank();
            "".must().be_ascii_blank();
            "a".must().not_be_blank();
            "FoObAr".must().be_equal_to_ignoring_ascii_case("fOoBaR");
            "foobar".must().contain("foo");
            "foobar".must().not_contain("baz");
            "foo bar baz".must().start_with("foo b");
            "foo bar baz".must().not_start_with("oo");
            "foo bar baz".must().end_with("r baz");
            "foo bar baz".must().not_end_with("y");
        }
    }

    mod evaluation {
        use core::cell::Cell;

        use crate::{prelude::*, test_support::SentinelRenderer};

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
                        let it = it.is_blank().is_not_blank().is_ascii_blank();
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
    }

    mod renderer_contract {
        use crate::{
            prelude::*,
            test_support::{NoRenderer, assert_trait_impl},
        };

        #[test]
        fn trait_is_implemented_without_renderer_support() {
            assert_trait_impl!(
                AssertThat<'static, &'static str, Panic, NoRenderer>
                    => StrAssertions<NoRenderer>
            );
        }

        #[test]
        fn operand_methods_render_and_redact_the_string_views() {
            use indoc::formatdoc;

            use crate::test_support::{
                CustomValueRenderer, RedactingRenderer, assert_custom_value, assert_redacted,
            };

            let subject = String::from("private-value");
            macro_rules! case {
                ($method:ident, $operand:literal, $relation:literal, $role:ident, $label:literal) => {{
                    let failures = assert_that!(subject.as_str())
                        .with_renderer(CustomValueRenderer)
                        .with_location(false)
                        .capture(|it| it.$method($operand));
                    assert_that!(&failures[0]).has_text_report(formatdoc! {r#"
                        -------- assertr --------
                        Expression: `subject.as_str()`

                        Actual: custom("private-value")

                        {}

                        {}: custom("{}")
                        -------- assertr --------
                    "#, $relation, $label, $operand});
                    assert_custom_value(failures[0].actual.as_ref().unwrap(), subject.as_str());
                    assert_custom_value(failures[0].$role.as_ref().unwrap(), $operand);

                    let failures = assert_that!(subject.as_str())
                        .with_renderer(RedactingRenderer)
                        .capture(|it| it.$method($operand));
                    assert_redacted(&failures[0], &[subject.as_str(), $operand]);
                }};
            }
            case!(
                is_equal_to_ignoring_ascii_case,
                "other-value",
                "is not equal to ignoring ASCII case",
                expected,
                "Expected"
            );
            case!(contains, "other", "does not contain", expected, "Expected");
            case!(
                does_not_contain,
                "private",
                "contains",
                unexpected,
                "Unexpected"
            );
            case!(
                starts_with,
                "other",
                "does not start with",
                expected,
                "Expected"
            );
            case!(
                does_not_start_with,
                "private",
                "starts with",
                unexpected,
                "Unexpected"
            );
            case!(
                ends_with,
                "other",
                "does not end with",
                expected,
                "Expected"
            );
            case!(
                does_not_end_with,
                "value",
                "ends with",
                unexpected,
                "Unexpected"
            );
        }
    }

    mod is_blank {
        use indoc::formatdoc;

        use crate::prelude::*;

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
            assert_that!(|| {
                assert_that!("a").with_location(false).is_blank();
            })
            .panics()
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

    mod is_ascii_blank {
        use crate::prelude::*;

        #[test]
        fn caller_location_is_as_expected() {
            assert_caller_location!(assert_that!("a"), is_ascii_blank());
        }

        #[test]
        fn succeeds_when_blank() {
            assert_that!("").is_ascii_blank();
            assert_that!(" ").is_ascii_blank();
            assert_that!("\t \n").is_ascii_blank();
            assert_that!(String::from("\t \n")).is_ascii_blank();
        }

        #[test]
        fn identifies_unicode_whitespace_as_non_ascii_whitespace() {
            let failures = assert_that!("\u{a0}").capture(StrAssertions::is_ascii_blank);
            assert_that!(failures[0].relation.as_deref()).is_equal_to(Some("is not ASCII blank"));
        }
    }

    mod is_not_blank {
        use crate::prelude::*;

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
        fn reports_the_negated_relation() {
            let failures = assert_that!("\t \n").capture(StrAssertions::is_not_blank);
            assert_that!(failures[0].relation.as_deref())
                .is_equal_to(Some("is unexpectedly blank"));
        }
    }

    mod is_equal_to_ignoring_ascii_case {
        use crate::prelude::*;

        #[test]
        fn caller_location_is_as_expected() {
            assert_caller_location!(assert_that!("foo"), is_equal_to_ignoring_ascii_case("bar"));
        }

        #[test]
        fn succeeds_when_equal_ignoring_ascii_case() {
            assert_that!("FoObAr").is_equal_to_ignoring_ascii_case("fOoBaR");
            assert_that!(String::from("FoObAr")).is_equal_to_ignoring_ascii_case("fOoBaR");
        }

        #[test]
        fn does_not_fold_non_ascii_case_differences() {
            assert_that!(|| {
                assert_that!("straße")
                    .with_location(false)
                    .is_equal_to_ignoring_ascii_case("STRAẞE");
            })
            .panics()
            .has_type::<String>()
            .contains("ignoring ASCII case");
        }
    }

    mod contains {
        use crate::prelude::*;

        #[test]
        fn caller_location_is_as_expected() {
            assert_caller_location!(assert_that!("foo bar baz"), contains("42"));
        }

        #[test]
        fn succeeds_when_expected_is_contained() {
            assert_that!("foobar").contains("foo");
            assert_that!("foobar").contains("bar");
            assert_that!("foobar").contains("oob");
            assert_that!(String::from("foobar")).contains("oob");
        }
    }

    mod does_not_contain {
        use crate::prelude::*;

        #[test]
        fn caller_location_is_as_expected() {
            assert_caller_location!(assert_that!("foo bar baz"), does_not_contain("o b"));
        }

        #[test]
        fn succeeds_when_expected_is_not_contained() {
            assert_that!("foobar").does_not_contain("baz");
            assert_that!(String::from("foobar")).does_not_contain("baz");
        }
    }

    mod starts_with {
        use crate::prelude::*;

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
            use core::{cell::Cell, fmt};

            use crate::assertions::core::string::StartsWith;

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
                assert_that!(format!("{value:#}")).is_equal_to(text);
            }
        }

        #[test]
        fn succeeds_when_start_matches() {
            assert_that!("foo bar baz").starts_with("foo b");
            assert_that!(String::from("foo bar baz")).starts_with("foo b");
        }
    }

    mod does_not_start_with {
        use crate::prelude::*;

        #[test]
        fn caller_location_is_as_expected() {
            assert_caller_location!(assert_that!("foo bar baz"), does_not_start_with("foo"));
        }

        #[test]
        fn succeeds_when_start_does_not_match() {
            assert_that!("foo bar baz").does_not_start_with("oo");
            assert_that!(String::from("foo bar baz")).does_not_start_with("oo");
        }
    }

    mod ends_with {
        use crate::prelude::*;

        #[test]
        fn caller_location_is_as_expected() {
            assert_caller_location!(assert_that!("foo bar baz"), ends_with("raz"));
        }

        #[test]
        fn succeeds_when_end_matches() {
            assert_that!("foo bar baz").ends_with("r baz");
            assert_that!(String::from("foo bar baz")).ends_with("r baz");
        }
    }

    mod does_not_end_with {
        use crate::prelude::*;

        #[test]
        fn caller_location_is_as_expected() {
            assert_caller_location!(assert_that!("foo bar baz"), does_not_end_with("z"));
        }

        #[test]
        fn succeeds_when_end_does_match() {
            assert_that!("foo bar baz").does_not_end_with("y");
            assert_that!(String::from("foo bar baz")).does_not_end_with("y");
        }
    }

    /// One blanket implementation serves every `AsRef<str>` subject, so all string-like types have
    /// to produce the same assertion-specific descriptions for the same content.
    mod every_string_like_type {
        use alloc::borrow::Cow;

        use crate::prelude::*;

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
                failures.iter().map(ToString::to_string).collect::<Vec<_>>()
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
            .is_equal_to(&reference);

            let boxed = String::from("foobar").into_boxed_str();
            assert_that!(rendered(
                assert_that!(boxed)
                    .with_location(false)
                    .capture(|it| it.contains("baz"))
            ))
            .is_equal_to(&reference);

            let cow = Cow::Borrowed("foobar");
            assert_that!(rendered(
                assert_that!(cow)
                    .with_location(false)
                    .capture(|it| it.contains("baz"))
            ))
            .is_equal_to(reference);
        }
    }
}
