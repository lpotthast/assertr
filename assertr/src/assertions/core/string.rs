use crate::{AssertThat, Mode, ValueRenderer, failure::FailureKind};

/// String-specific assertions.
///
/// Blanket-implemented for every subject that is `AsRef<str>`, so `&str`, `String`, `&String`,
/// `Box<str>`, and `Cow<str>` all share one implementation and one set of failure messages.
#[allow(clippy::return_self_not_must_use)]
#[cfg_attr(feature = "fluent", assertr_derive::fluent_aliases)]
pub trait StrAssertions {
    /// The renderer carried by the assertion chain.
    type Renderer;

    /// The string-like assertion subject rendered in failure messages.
    type Subject: AsRef<str>;

    /// Asserts that the subject is empty or contains only Unicode `White_Space` characters.
    fn is_blank(self) -> Self
    where
        Self::Renderer: ValueRenderer<Self::Subject>;

    /// Asserts that the subject contains at least one character without the Unicode `White_Space`
    /// property.
    fn is_not_blank(self) -> Self
    where
        Self::Renderer: ValueRenderer<Self::Subject>;

    /// Asserts that the subject is empty or contains only ASCII whitespace.
    fn is_blank_ascii(self) -> Self
    where
        Self::Renderer: ValueRenderer<Self::Subject>;

    /// Asserts that the subject and `expected` are equal under ASCII case folding.
    fn is_equal_to_ignoring_ascii_case(self, expected: impl AsRef<str>) -> Self
    where
        Self::Renderer: ValueRenderer<Self::Subject>;

    /// Asserts that the subject contains `expected` as a substring.
    fn contains(self, expected: impl AsRef<str>) -> Self
    where
        Self::Renderer: ValueRenderer<Self::Subject>;

    /// Asserts that the subject does not contain `unexpected` as a substring.
    fn does_not_contain(self, unexpected: impl AsRef<str>) -> Self
    where
        Self::Renderer: ValueRenderer<Self::Subject>;

    /// Asserts that the subject starts with `expected`.
    fn starts_with(self, expected: impl AsRef<str>) -> Self
    where
        Self::Renderer: ValueRenderer<Self::Subject>;

    /// Asserts that the subject does not start with `unexpected`.
    fn does_not_start_with(self, unexpected: impl AsRef<str>) -> Self
    where
        Self::Renderer: ValueRenderer<Self::Subject>;

    /// Asserts that the subject ends with `expected`.
    fn ends_with(self, expected: impl AsRef<str>) -> Self
    where
        Self::Renderer: ValueRenderer<Self::Subject>;

    /// Asserts that the subject does not end with `unexpected`.
    fn does_not_end_with(self, unexpected: impl AsRef<str>) -> Self
    where
        Self::Renderer: ValueRenderer<Self::Subject>;
}

impl<S: AsRef<str>, M: Mode, R> StrAssertions for AssertThat<'_, S, M, R> {
    type Renderer = R;
    type Subject = S;

    #[track_caller]
    fn is_blank(self) -> Self
    where
        R: ValueRenderer<S>,
    {
        self.track_assertion();
        let actual = self.actual().as_ref();
        // This iterator will yield no entries if the string is empty or all whitespace!
        if actual.split_whitespace().next().is_some() {
            self.failure(FailureKind::Other)
                .actual(self.render().value(self.actual()))
                .relation("is not blank")
                .raise();
        }
        self
    }

    #[track_caller]
    fn is_not_blank(self) -> Self
    where
        R: ValueRenderer<S>,
    {
        self.track_assertion();
        let actual = self.actual().as_ref();
        if actual.split_whitespace().next().is_none() {
            self.failure(FailureKind::Other)
                .actual(self.render().value(self.actual()))
                .relation("is unexpectedly blank")
                .raise();
        }
        self
    }

    #[track_caller]
    fn is_blank_ascii(self) -> Self
    where
        R: ValueRenderer<S>,
    {
        self.track_assertion();
        let actual = self.actual().as_ref();
        // This iterator will yield no entries if the string is empty or all whitespace!
        if actual.split_ascii_whitespace().next().is_some() {
            self.failure(FailureKind::Other)
                .actual(self.render().value(self.actual()))
                .relation("is not ASCII blank")
                .raise();
        }
        self
    }

    #[track_caller]
    fn is_equal_to_ignoring_ascii_case(self, expected: impl AsRef<str>) -> Self
    where
        R: ValueRenderer<S>,
    {
        self.track_assertion();
        let actual = self.actual().as_ref();
        let expected = expected.as_ref();
        if !actual.eq_ignore_ascii_case(expected) {
            self.failure(FailureKind::Equality)
                .actual(self.render().value(self.actual()))
                .expected(format_args!("{expected:?}"))
                .note("Values differ even when ignoring ASCII case.")
                .raise();
        }
        self
    }

    #[track_caller]
    fn contains(self, expected: impl AsRef<str>) -> Self
    where
        R: ValueRenderer<S>,
    {
        self.track_assertion();
        let actual = self.actual().as_ref();
        let expected = expected.as_ref();
        if !actual.contains(expected) {
            self.failure(FailureKind::Membership)
                .actual(self.render().value(self.actual()))
                .relation("does not contain")
                .expected(format_args!("{expected:?}"))
                .raise();
        }
        self
    }

    #[track_caller]
    fn does_not_contain(self, unexpected: impl AsRef<str>) -> Self
    where
        R: ValueRenderer<S>,
    {
        self.track_assertion();
        let actual = self.actual().as_ref();
        let unexpected = unexpected.as_ref();
        if actual.contains(unexpected) {
            self.failure(FailureKind::Membership)
                .actual(self.render().value(self.actual()))
                .relation("contains")
                .unexpected(format_args!("{unexpected:?}"))
                .raise();
        }
        self
    }

    #[track_caller]
    fn starts_with(self, expected: impl AsRef<str>) -> Self
    where
        R: ValueRenderer<S>,
    {
        self.track_assertion();
        let actual = self.actual().as_ref();
        let expected = expected.as_ref();
        if !actual.starts_with(expected) {
            self.failure(FailureKind::Membership)
                .actual(self.render().value(self.actual()))
                .relation("does not start with")
                .expected(format_args!("{expected:?}"))
                .raise();
        }
        self
    }

    #[track_caller]
    fn does_not_start_with(self, unexpected: impl AsRef<str>) -> Self
    where
        R: ValueRenderer<S>,
    {
        self.track_assertion();
        let actual = self.actual().as_ref();
        let unexpected = unexpected.as_ref();
        if actual.starts_with(unexpected) {
            self.failure(FailureKind::Membership)
                .actual(self.render().value(self.actual()))
                .relation("starts with")
                .unexpected(format_args!("{unexpected:?}"))
                .raise();
        }
        self
    }

    #[track_caller]
    fn ends_with(self, expected: impl AsRef<str>) -> Self
    where
        R: ValueRenderer<S>,
    {
        self.track_assertion();
        let actual = self.actual().as_ref();
        let expected = expected.as_ref();
        if !actual.ends_with(expected) {
            self.failure(FailureKind::Membership)
                .actual(self.render().value(self.actual()))
                .relation("does not end with")
                .expected(format_args!("{expected:?}"))
                .raise();
        }
        self
    }

    #[track_caller]
    fn does_not_end_with(self, unexpected: impl AsRef<str>) -> Self
    where
        R: ValueRenderer<S>,
    {
        self.track_assertion();
        let actual = self.actual().as_ref();
        let unexpected = unexpected.as_ref();
        if actual.ends_with(unexpected) {
            self.failure(FailureKind::Membership)
                .actual(self.render().value(self.actual()))
                .relation("ends with")
                .unexpected(format_args!("{unexpected:?}"))
                .raise();
        }
        self
    }
}

#[cfg(test)]
mod tests {
    mod renderer_contract {
        use crate::prelude::*;
        use crate::test_support::{NoRenderer, assert_trait_impl};

        #[test]
        fn trait_is_implemented_without_renderer_support() {
            assert_trait_impl!(
                AssertThat<'static, &'static str, Panic, NoRenderer> => StrAssertions
            );
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

                Expected: "bar"

                  Actual: "foo"

                Details:
                  - Values differ even when ignoring ASCII case.
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
                    .with_debug_format(|value: &String, f| write!(f, "custom({value})"))
                    .contains("z");
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {r#"
                -------- assertr --------
                Expression: `String::from("abc")`

                Actual: custom(abc)

                does not contain

                Expected: "z"
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
            let rendered = |mut failures: Vec<AssertionFailure>| {
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

                        Expected: "baz"
                        -------- assertr --------
                    "#});
                },
            ]);
        }
    }
}
