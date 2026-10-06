use crate::assertions::HasLength;
use crate::failure::FailureKind;
use crate::mode::{Mode, Panic};
use crate::{AssertThat, DebugRenderer, ValueRenderer};
use crate::{AssertionContext, Expectation, ExpectationDiagnostics, failure::FailureBuilder};
use alloc::borrow::ToOwned;
use alloc::string::String;

/// Checks printable ASCII and horizontal tabs, returning the accepted header string.
#[derive(Debug, Clone, Copy)]
pub struct IsAscii;
impl<R> Expectation<http::HeaderValue, R> for IsAscii {
    type Success<'a>
        = &'a str
    where
        Self: 'a,
        http::HeaderValue: 'a;
    type Rejection<'a>
        = ()
    where
        Self: 'a,
        http::HeaderValue: 'a;
    fn evaluate<'a>(
        &'a self,
        actual: &'a http::HeaderValue,
        _context: &AssertionContext<'_, R>,
    ) -> Result<Self::Success<'a>, Self::Rejection<'a>> {
        actual.to_str().map_err(|_| ())
    }
}
impl<R> ExpectationDiagnostics<http::HeaderValue, R> for IsAscii
where
    R: ValueRenderer<http::HeaderValue>,
{
    const KIND: FailureKind = FailureKind::Predicate;
    fn explain<'a, Target>(
        &'a self,
        rejected: Option<(&'a http::HeaderValue, Self::Rejection<'a>)>,
        failure: FailureBuilder<Target>,
        context: &AssertionContext<'_, R>,
    ) -> FailureBuilder<Target> {
        let render = context.render();
        match rejected {
            None => failure.relation("is ASCII"),
            Some((actual, ())) => failure
                .actual(render.value(actual))
                .relation("is not ASCII"),
        }
    }
}

/// Generates a header-value sensitivity expectation.
macro_rules! sensitivity_expectation {
    ($(#[$meta:meta])* $name:ident, sensitive: $sensitive:literal, $met:literal, $unmet:literal) => {
        $(#[$meta])*
        #[derive(Debug, Clone, Copy)]
        pub struct $name;
        impl<R> Expectation<http::HeaderValue, R> for $name {
            type Success<'a>
                = ()
            where
                Self: 'a,
                http::HeaderValue: 'a;
            type Rejection<'a>
                = ()
            where
                Self: 'a,
                http::HeaderValue: 'a;
            fn evaluate<'a>(
                &'a self,
                actual: &'a http::HeaderValue,
                _context: &AssertionContext<'_, R>,
            ) -> Result<Self::Success<'a>, Self::Rejection<'a>> {
                if actual.is_sensitive() == $sensitive {
                    Ok(())
                } else {
                    Err(())
                }
            }
        }
        impl<R> ExpectationDiagnostics<http::HeaderValue, R> for $name
        where
            R: ValueRenderer<http::HeaderValue>,
        {
            const KIND: FailureKind = FailureKind::Other;
            fn explain<'a, Target>(
                &'a self,
                rejected: Option<(&'a http::HeaderValue, Self::Rejection<'a>)>,
                failure: FailureBuilder<Target>,
                context: &AssertionContext<'_, R>,
            ) -> FailureBuilder<Target> {
                match rejected {
                    None => failure.relation($met),
                    Some((actual, ())) => failure
                        .actual(context.render().value(actual))
                        .relation($unmet),
                }
            }
        }
    };
}

sensitivity_expectation!(
    /// Checks whether a header value is marked sensitive.
    IsSensitive,
    sensitive: true,
    "is sensitive",
    "is not sensitive"
);

sensitivity_expectation!(
    /// Checks whether a header value is not marked sensitive.
    IsInsensitive,
    sensitive: false,
    "is insensitive",
    "is sensitive"
);

/// The header value's length in bytes, enabling
/// [`LengthAssertions`](crate::assertions::core::length::LengthAssertions).
impl HasLength for http::HeaderValue {
    fn length(&self) -> usize {
        self.len()
    }

    fn is_empty(&self) -> bool {
        http::HeaderValue::is_empty(self)
    }
}

/// Non-extracting assertions for [`http::HeaderValue`].
///
/// Length assertions such as `is_empty`, `is_not_empty`, and `has_length` come from
/// [`LengthAssertions`](crate::assertions::core::length::LengthAssertions) and count bytes.
#[allow(clippy::return_self_not_must_use)]
#[cfg_attr(feature = "fluent", assertr_macros::fluent_aliases)]
pub trait HttpHeaderValueAssertions<M: Mode, R = DebugRenderer> {
    /// Asserts that the header value is marked sensitive.
    fn is_sensitive(self) -> Self
    where
        R: ValueRenderer<http::HeaderValue>;

    /// Asserts that the header value is not marked sensitive.
    fn is_insensitive(self) -> Self
    where
        R: ValueRenderer<http::HeaderValue>;

    /// Asserts that [`HeaderValue::to_str`](http::HeaderValue::to_str) accepts the value.
    ///
    /// This permits printable ASCII and horizontal tabs, but rejects opaque bytes. The subject
    /// stays the full `HeaderValue`, so further assertions can be chained in any mode. Use
    /// [`HttpHeaderValueExtractAssertions::get_ascii`] to extract a `String` in panic mode, or
    /// [`HttpHeaderValueAssertions::is_ascii_satisfying`] to assert on it in any mode.
    fn is_ascii(self) -> Self
    where
        R: ValueRenderer<http::header::HeaderValue>;

    /// Asserts that [`HeaderValue::to_str`](http::HeaderValue::to_str) accepts the value, then runs
    /// additional assertions on the resulting string.
    ///
    /// The closure receives `AssertThat<&str>`. The projection target `str` is unsized, so the
    /// string assertion traits operate on the reference itself.
    fn is_ascii_satisfying<A>(self, assertions: A) -> Self
    where
        A: for<'a> FnOnce(AssertThat<'a, &'a str, M, R>),
        R: ValueRenderer<http::header::HeaderValue> + Clone;
}

impl<M: Mode, R> HttpHeaderValueAssertions<M, R>
    for AssertThat<'_, http::header::HeaderValue, M, R>
{
    #[track_caller]
    fn is_sensitive(self) -> Self
    where
        R: ValueRenderer<http::HeaderValue>,
    {
        self.apply_assertion(IsSensitive)
    }

    #[track_caller]
    fn is_insensitive(self) -> Self
    where
        R: ValueRenderer<http::HeaderValue>,
    {
        self.apply_assertion(IsInsensitive)
    }

    #[track_caller]
    fn is_ascii(self) -> Self
    where
        R: ValueRenderer<http::header::HeaderValue>,
    {
        self.apply_assertion(IsAscii)
    }

    #[track_caller]
    fn is_ascii_satisfying<A>(self, assertions: A) -> Self
    where
        A: for<'a> FnOnce(AssertThat<'a, &'a str, M, R>),
        R: ValueRenderer<http::header::HeaderValue> + Clone,
    {
        if let Some(value) = self.test_assertion(&IsAscii) {
            assertions(self.derive_owned(|_| value));
        }
        self
    }
}

/// Panic-mode string extraction from [`HeaderValue`](http::HeaderValue) subjects.
///
/// A rejected value cannot produce the requested `String`.
#[cfg_attr(feature = "fluent", assertr_macros::fluent_aliases)]
pub trait HttpHeaderValueExtractAssertions<'t, R = DebugRenderer> {
    /// Asserts that [`HeaderValue::to_str`](http::HeaderValue::to_str) accepts the value, then
    /// extracts it as an owned `String`.
    ///
    /// Use [`HttpHeaderValueAssertions::is_ascii_satisfying`] for capture mode, or
    /// [`HttpHeaderValueAssertions::is_ascii`] when the text is irrelevant.
    fn get_ascii(self) -> AssertThat<'t, String, Panic, R>
    where
        R: ValueRenderer<http::header::HeaderValue>;
}

impl<'t, R> HttpHeaderValueExtractAssertions<'t, R>
    for AssertThat<'t, http::header::HeaderValue, Panic, R>
{
    #[track_caller]
    fn get_ascii(self) -> AssertThat<'t, String, Panic, R>
    where
        R: ValueRenderer<http::header::HeaderValue>,
    {
        let value = self
            .test_assertion(&IsAscii)
            .expect("Panic mode raises invalid ASCII")
            .to_owned();
        self.map(|_| value.into())
    }
}

#[cfg(test)]
mod tests {
    mod renderer_contract {
        use crate::prelude::*;
        use crate::test_support::{NoRenderer, SENTINEL, SentinelRenderer, assert_trait_impl};
        use http::HeaderValue;

        #[test]
        fn traits_are_implemented_without_renderer_support() {
            assert_trait_impl!(
                AssertThat<'static, HeaderValue, Panic, NoRenderer>
                    => HttpHeaderValueAssertions<Panic, NoRenderer>
            );
            assert_trait_impl!(
                AssertThat<'static, HeaderValue, Panic, NoRenderer>
                    => HttpHeaderValueExtractAssertions<'static, NoRenderer>
            );
        }

        #[test]
        fn direct_and_projected_failures_use_the_active_renderer() {
            let opaque = HeaderValue::from_bytes(b"\xFF").expect("valid opaque header bytes");
            let opaque_failures = assert_that!(opaque)
                .with_renderer(SentinelRenderer)
                .with_location(false)
                .capture(HttpHeaderValueAssertions::is_ascii);
            assert_that!(ToHumanReadableText.render(&opaque_failures[0])).contains(SENTINEL);

            let visible = HeaderValue::from_static("visible");
            let projected_failures = assert_that!(visible)
                .with_renderer(SentinelRenderer)
                .with_location(false)
                .capture(HttpHeaderValueAssertions::is_sensitive);
            assert_that!(ToHumanReadableText.render(&projected_failures[0])).contains(SENTINEL);
        }
    }

    mod length {
        use crate::prelude::*;
        use http::HeaderValue;
        use indoc::formatdoc;

        #[test]
        fn length_assertions_count_header_bytes() {
            assert_that!(HeaderValue::from_static("")).is_empty();
            assert_that!(HeaderValue::from_static("http/1.1"))
                .is_not_empty()
                .has_length(8);
            assert_that!(HeaderValue::from_bytes(b"\xFF").expect("valid opaque header bytes"))
                .has_length(1);
        }

        #[test]
        fn panics_when_not_empty() {
            let actual = HeaderValue::from_static("http/1.1");

            assert_that_panic_by(|| assert_that!(actual).with_location(false).is_empty())
                .has_type::<String>()
                .is_equal_to(formatdoc! {r#"
                    -------- assertr --------
                    Expression: `actual`

                    Actual: HeaderValue "http/1.1"

                    is not empty
                    -------- assertr --------
                "#});
        }
    }

    mod is_sensitive {
        use crate::prelude::*;
        use http::HeaderValue;
        use indoc::formatdoc;

        #[test]
        #[cfg(feature = "fluent")]
        fn fluent_alias_is_as_expected() {
            let mut actual = HeaderValue::from_str("http/1.1").expect("valid header value");
            actual.set_sensitive(true);
            actual.must().be_sensitive();
        }

        #[test]
        fn caller_location_is_as_expected() {
            let mut actual = HeaderValue::from_str("http/1.1").expect("valid header value");
            actual.set_sensitive(false);
            assert_caller_location!(assert_that!(actual), is_sensitive());
        }

        #[test]
        fn succeeds_when_sensitive() {
            let mut actual = HeaderValue::from_str("http/1.1").expect("valid header value");
            actual.set_sensitive(true);

            assert_that!(actual).is_sensitive();
        }

        #[test]
        fn panics_when_insensitive() {
            let mut actual = HeaderValue::from_str("http/1.1").expect("valid header value");
            actual.set_sensitive(false);

            assert_that_panic_by(|| assert_that!(actual).with_location(false).is_sensitive())
                .has_type::<String>()
                .is_equal_to(formatdoc! {r#"
                    -------- assertr --------
                    Expression: `actual`

                    Actual: "http/1.1"

                    is not sensitive
                    -------- assertr --------
                "#});
        }
    }

    mod is_insensitive {
        use crate::prelude::*;
        use http::HeaderValue;
        use indoc::formatdoc;

        #[test]
        #[cfg(feature = "fluent")]
        fn fluent_alias_is_as_expected() {
            let actual = HeaderValue::from_str("http/1.1").expect("valid header value");
            actual.must().be_insensitive();
        }

        #[test]
        fn caller_location_is_as_expected() {
            let mut actual = HeaderValue::from_str("http/1.1").expect("valid header value");
            actual.set_sensitive(true);
            assert_caller_location!(assert_that!(actual), is_insensitive());
        }

        #[test]
        fn not_sensitive_by_default() {
            let actual = HeaderValue::from_str("http/1.1").expect("valid header value");

            assert_that!(actual).is_insensitive();
        }

        #[test]
        fn succeeds_when_insensitive() {
            let mut actual = HeaderValue::from_str("http/1.1").expect("valid header value");
            actual.set_sensitive(false);

            assert_that!(actual).is_insensitive();
        }

        #[test]
        fn panics_when_sensitive() {
            let mut actual = HeaderValue::from_str("http/1.1").expect("valid header value");
            actual.set_sensitive(true);

            assert_that_panic_by(|| assert_that!(actual).with_location(false).is_insensitive())
                .has_type::<String>()
                .is_equal_to(formatdoc! {r"
                    -------- assertr --------
                    Expression: `actual`

                    Actual: Sensitive

                    is sensitive
                    -------- assertr --------
                "});
        }
    }

    mod is_ascii {
        use crate::prelude::*;
        use http::header::HeaderValue;
        use indoc::formatdoc;

        #[test]
        #[cfg(feature = "fluent")]
        fn fluent_alias_is_as_expected() {
            let actual = HeaderValue::from_str("http/1.1").expect("valid header value");
            actual.must().be_ascii();
        }

        #[test]
        fn caller_location_is_as_expected() {
            let actual = HeaderValue::from_bytes(&[32, 33, 255]).expect("valid header value");
            assert_caller_location!(assert_that!(actual), is_ascii());
        }

        #[test]
        fn succeeds_when_ascii_and_retains_the_subject() {
            let actual = HeaderValue::from_str("http/1.1").expect("valid header value");

            assert_that!(actual).is_ascii().is_not_empty();
        }

        #[test]
        fn panics_when_not_ascii() {
            let actual = HeaderValue::from_bytes(&[32, 33, 255]).expect("valid header value");

            assert_that_panic_by(|| assert_that!(actual).with_location(false).is_ascii())
                .has_type::<String>()
                .is_equal_to(formatdoc! {r#"
                    -------- assertr --------
                    Expression: `actual`

                    Actual: " !\xff"

                    is not ASCII
                    -------- assertr --------
                "#});
        }

        #[test]
        fn works_in_capture_mode() {
            let actual = HeaderValue::from_bytes(&[32, 33, 255]).expect("valid header value");

            let failures = assert_that!(actual)
                .with_location(false)
                .capture(|it| it.is_ascii().is_not_empty());

            assert_that!(&failures).contains_exactly_satisfying([
                |element: AssertThat<AssertionFailure, Capture>| {
                    element
                        .derive_owned(|value| ToHumanReadableText.render(value))
                        .contains("is not ASCII");
                },
            ]);
        }
    }

    mod get_ascii {
        use crate::prelude::*;
        use http::header::HeaderValue;
        use indoc::formatdoc;

        #[test]
        #[cfg(feature = "fluent")]
        fn fluent_alias_is_as_expected() {
            let actual = HeaderValue::from_static("http/1.1");
            actual.must().get_ascii().is_equal_to("http/1.1");
        }

        #[test]
        fn caller_location_is_as_expected() {
            let actual = HeaderValue::from_str("\u{c4}").expect("valid header value");
            assert_caller_location!(assert_that!(actual), get_ascii());
        }

        #[test]
        fn extracts_the_value_when_constructed_from_visible_ascii_characters_through_str() {
            let actual = HeaderValue::from_str("http/1.1").expect("valid header value");

            assert_that!(actual).get_ascii().is_equal_to("http/1.1");
        }

        #[test]
        fn extracts_the_value_when_constructed_from_visible_ascii_characters_through_bytes() {
            let actual = HeaderValue::from_bytes(&[32, 33, 34]).expect("valid header value");

            assert_that!(actual).get_ascii().is_equal_to(" !\"");
        }

        #[test]
        fn panics_when_constructed_from_non_ascii_characters_through_str() {
            let actual = HeaderValue::from_str("\u{c4}").expect("valid header value");

            assert_that_panic_by(|| assert_that!(actual).with_location(false).get_ascii())
                .has_type::<String>()
                .is_equal_to(formatdoc! {r#"
                    -------- assertr --------
                    Expression: `actual`

                    Actual: "\xc3\x84"

                    is not ASCII
                    -------- assertr --------
                "#});
        }

        #[test]
        fn panics_when_constructed_from_non_ascii_characters_through_bytes() {
            let actual = HeaderValue::from_bytes(&[32, 33, 255]).expect("valid header value");

            assert_that_panic_by(|| assert_that!(actual).with_location(false).get_ascii())
                .has_type::<String>()
                .is_equal_to(formatdoc! {r#"
                    -------- assertr --------
                    Expression: `actual`

                    Actual: " !\xff"

                    is not ASCII
                    -------- assertr --------
                "#});
        }
    }

    mod is_ascii_satisfying {
        use crate::prelude::*;
        use http::header::HeaderValue;
        use indoc::formatdoc;

        #[test]
        #[cfg(feature = "fluent")]
        fn fluent_alias_is_as_expected() {
            let actual = HeaderValue::from_str("http/1.1").expect("valid header value");
            actual.must().be_ascii_satisfying(|s| {
                s.starts_with("http");
            });
        }

        #[test]
        fn caller_location_is_as_expected() {
            let actual = HeaderValue::from_bytes(&[32, 33, 255]).expect("valid header value");
            assert_caller_location!(
                assert_that!(actual),
                is_ascii_satisfying(|s| {
                    s.starts_with("http");
                })
            );
        }

        #[test]
        fn succeeds_when_ascii_and_assertions_pass() {
            let actual = HeaderValue::from_str("http/1.1").expect("valid header value");

            assert_that!(actual).is_ascii_satisfying(|s| {
                s.starts_with("http");
            });
        }

        #[test]
        fn collects_failure_in_capture_mode_when_ascii_but_assertion_fails() {
            let actual = HeaderValue::from_str("http/1.1").expect("valid header value");

            let failures = assert_that!(actual).with_location(false).capture(|it| {
                it.is_ascii_satisfying(|s| {
                    s.starts_with("ftp");
                })
            });
            assert_that!(failures).has_length(1);
        }

        #[test]
        fn collects_failure_in_capture_mode_when_not_ascii() {
            let actual = HeaderValue::from_bytes(&[32, 33, 255]).expect("valid header value");

            let failures = assert_that!(actual).with_location(false).capture(|it| {
                it.is_ascii_satisfying(|s| {
                    s.starts_with("http");
                })
            });
            assert_that!(failures).contains_exactly_satisfying([
                |failure: AssertThat<AssertionFailure, Capture>| {
                    failure
                        .derive_owned(|failure| ToHumanReadableText.render(failure))
                        .contains("is not ASCII");
                },
            ]);
        }

        #[test]
        fn panics_when_not_ascii_in_panic_mode() {
            let actual = HeaderValue::from_bytes(&[32, 33, 255]).expect("valid header value");

            assert_that_panic_by(|| {
                assert_that!(actual)
                    .with_location(false)
                    .is_ascii_satisfying(|s| {
                        s.starts_with("http");
                    })
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {r#"
                -------- assertr --------
                Expression: `actual`

                Actual: " !\xff"

                is not ASCII
                -------- assertr --------
            "#});
        }
    }
}
