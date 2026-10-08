use crate::assertions::HasLength;
use crate::failure::FailureKind;
use crate::mode::{Mode, Panic};
use crate::renderer::{Rendered, RenderingContext};
use crate::{AssertThat, renderer::DebugRenderer, renderer::ValueRenderer};
use crate::{expectation::AssertionContext, expectation::Expectation, failure::FailureBuilder};
use alloc::borrow::ToOwned;
use alloc::string::String;

/// Renders a header value with its contents visible, even when it is marked sensitive.
///
/// The `Debug` form of a sensitive `HeaderValue` hides its contents, but a failing header check
/// must show the value it asserted. The active renderer therefore receives an unmarked diagnostic
/// copy, so a renderer that redacts header values still applies. The subject stays unchanged.
fn reveal<R: ValueRenderer<http::HeaderValue>>(
    render: RenderingContext<'_, R>,
    value: &http::HeaderValue,
) -> Rendered {
    if value.is_sensitive() {
        let mut visible = value.clone();
        visible.set_sensitive(false);
        render.value(&visible)
    } else {
        render.value(value)
    }
}

/// Checks printable ASCII and horizontal tabs, returning the accepted header string.
#[derive(Debug, Clone, Copy)]
pub struct IsAscii;
impl<R> Expectation<http::HeaderValue, R> for IsAscii
where
    R: ValueRenderer<http::HeaderValue>,
{
    type Success<'a> = &'a str;
    type Rejection<'a> = ();
    fn evaluate<'a>(
        &'a self,
        actual: &'a http::HeaderValue,
        _context: &AssertionContext<'_, R>,
    ) -> Result<Self::Success<'a>, Self::Rejection<'a>> {
        actual.to_str().map_err(|_| ())
    }

    const KIND: FailureKind = FailureKind::Predicate;
    fn explain<'a>(
        &'a self,
        rejected: Option<(&'a http::HeaderValue, Self::Rejection<'a>)>,
        failure: FailureBuilder,
        context: &AssertionContext<'_, R>,
    ) -> FailureBuilder {
        let render = context.render();
        failure.relations(
            rejected.map(|(actual, ())| reveal(render, actual)),
            "is ASCII",
            "is not ASCII",
        )
    }
}

property_expectation! {
    /// Checks whether a header value is marked sensitive.
    pub struct IsSensitive for http::HeaderValue;
    kind Other;
    check |actual| actual.is_sensitive();
    relations "is sensitive", "is not sensitive";
    present reveal;
}

property_expectation! {
    /// Checks whether a header value is not marked sensitive.
    pub struct IsInsensitive for http::HeaderValue;
    kind Other;
    check |actual| !actual.is_sensitive();
    relations "is insensitive", "is sensitive";
    present reveal;
}

/// The header value's length in bytes, enabling
/// [`LengthAssertions`](crate::assertions::LengthAssertions).
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
/// Failure diagnostics display header contents even when the value is marked sensitive, so test
/// failures expose the value being asserted. The rendering budget still applies.
///
/// Length assertions such as `is_empty`, `is_not_empty`, and `has_length` come from
/// [`LengthAssertions`](crate::assertions::LengthAssertions) and count bytes.
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
        self.matches(IsSensitive)
    }

    #[track_caller]
    fn is_insensitive(self) -> Self
    where
        R: ValueRenderer<http::HeaderValue>,
    {
        self.matches(IsInsensitive)
    }

    #[track_caller]
    fn is_ascii(self) -> Self
    where
        R: ValueRenderer<http::header::HeaderValue>,
    {
        self.matches(IsAscii)
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
        let value = self.require(&IsAscii).to_owned();
        self.map(|_| value.into())
    }
}

#[cfg(test)]
mod tests {
    #[cfg(feature = "fluent")]
    mod fluent_aliases {
        use crate::prelude::*;
        use http::HeaderValue;

        #[test]
        fn are_as_expected() {
            let mut sensitive = HeaderValue::from_static("http/1.1");
            sensitive.set_sensitive(true);
            sensitive.must().be_sensitive();

            let actual = HeaderValue::from_static("http/1.1");
            actual
                .must()
                .be_insensitive()
                .be_ascii()
                .be_ascii_satisfying(|s| {
                    s.starts_with("http");
                });
            actual.must().get_ascii().is_equal_to("http/1.1");
        }
    }

    mod renderer_contract {
        use crate::prelude::*;
        use crate::test_support::{
            NoRenderer, RedactingRenderer, SENTINEL, SentinelRenderer, assert_redacted,
            assert_trait_impl,
        };
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
            assert_that!(opaque_failures[0].to_string()).contains(SENTINEL);

            let visible = HeaderValue::from_static("visible");
            let projected_failures = assert_that!(visible)
                .with_renderer(SentinelRenderer)
                .with_location(false)
                .capture(HttpHeaderValueAssertions::is_sensitive);
            assert_that!(projected_failures[0].to_string()).contains(SENTINEL);
        }

        #[test]
        fn redacting_renderers_still_redact_sensitive_values() {
            let mut actual = HeaderValue::from_static("secret");
            actual.set_sensitive(true);

            let failures = assert_that!(actual)
                .with_renderer(RedactingRenderer)
                .with_location(false)
                .capture(HttpHeaderValueAssertions::is_insensitive);

            assert_redacted(&failures[0], &["secret"]);
        }
    }

    #[test]
    fn length_assertions_count_header_bytes() {
        use crate::prelude::*;
        use http::HeaderValue;

        assert_that!(HeaderValue::from_static("")).is_empty();
        assert_that!(HeaderValue::from_static("http/1.1"))
            .is_not_empty()
            .has_length(8);
        assert_that!(HeaderValue::from_bytes(b"\xFF").expect("valid opaque header bytes"))
            .has_length(1);
    }

    mod is_sensitive {
        use crate::prelude::*;
        use http::HeaderValue;
        use indoc::formatdoc;

        #[test]
        fn caller_location_is_as_expected() {
            let actual = HeaderValue::from_static("http/1.1");
            assert_caller_location!(assert_that!(actual), is_sensitive());
        }

        #[test]
        fn succeeds_when_sensitive() {
            let mut actual = HeaderValue::from_static("http/1.1");
            actual.set_sensitive(true);

            assert_that!(actual).is_sensitive();
        }

        #[test]
        fn panics_when_insensitive() {
            let actual = HeaderValue::from_static("http/1.1");

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
        fn caller_location_is_as_expected() {
            let mut actual = HeaderValue::from_static("http/1.1");
            actual.set_sensitive(true);
            assert_caller_location!(assert_that!(actual), is_insensitive());
        }

        #[test]
        fn succeeds_when_not_marked_sensitive() {
            assert_that!(HeaderValue::from_static("http/1.1")).is_insensitive();
        }

        #[test]
        fn panics_with_the_revealed_value_when_sensitive() {
            let mut actual = HeaderValue::from_static("http/1.1");
            actual.set_sensitive(true);

            assert_that_panic_by(|| assert_that!(actual).with_location(false).is_insensitive())
                .has_type::<String>()
                .is_equal_to(formatdoc! {r#"
                    -------- assertr --------
                    Expression: `actual`

                    Actual: "http/1.1"

                    is sensitive
                    -------- assertr --------
                "#});
        }
    }

    mod is_ascii {
        use crate::prelude::*;
        use http::header::HeaderValue;
        use indoc::formatdoc;

        #[test]
        fn caller_location_is_as_expected() {
            let actual = HeaderValue::from_bytes(&[32, 33, 255]).expect("valid header value");
            assert_caller_location!(assert_that!(actual), is_ascii());
        }

        #[test]
        fn succeeds_when_ascii_and_retains_the_subject() {
            let actual = HeaderValue::from_static("http/1.1");

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
        fn shows_sensitive_contents_and_keeps_the_subject_sensitive() {
            let mut actual = HeaderValue::from_bytes(&[32, 33, 255]).expect("valid header value");
            actual.set_sensitive(true);

            let failures = assert_that!(actual)
                .with_location(false)
                .capture(|it| it.is_ascii().is_not_empty());

            assert_that!(failures).has_length(1);
            assert_that!(failures[0].to_string()).contains(r#"Actual: " !\xff""#);
            assert_that!(actual.is_sensitive()).is_true();
        }
    }

    mod get_ascii {
        use crate::prelude::*;
        use http::header::HeaderValue;

        #[test]
        fn caller_location_is_as_expected() {
            let actual = HeaderValue::from_str("\u{c4}").expect("valid header value");
            assert_caller_location!(assert_that!(actual), get_ascii());
        }

        #[test]
        fn extracts_visible_ascii_values() {
            let actual = HeaderValue::from_static("http/1.1");
            assert_that!(actual).get_ascii().is_equal_to("http/1.1");

            let actual = HeaderValue::from_bytes(&[32, 33, 34]).expect("valid header value");
            assert_that!(actual).get_ascii().is_equal_to(" !\"");
        }

        #[test]
        fn rejects_non_ascii_utf8_values() {
            let actual = HeaderValue::from_str("\u{c4}").expect("valid header value");

            assert_that_panic_by(|| assert_that!(actual).with_location(false).get_ascii())
                .has_type::<String>()
                .contains(r#"Actual: "\xc3\x84""#);
        }
    }

    mod is_ascii_satisfying {
        use crate::prelude::*;
        use http::header::HeaderValue;

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
            let actual = HeaderValue::from_static("http/1.1");

            assert_that!(actual).is_ascii_satisfying(|s| {
                s.starts_with("http");
            });
        }

        #[test]
        fn collects_either_the_ascii_or_the_callback_failure_in_capture_mode() {
            let ascii = HeaderValue::from_static("http/1.1");
            let failures = assert_that!(ascii).with_location(false).capture(|it| {
                it.is_ascii_satisfying(|s| {
                    s.starts_with("ftp");
                })
            });
            assert_that!(failures[0].to_string()).contains("does not start with");

            let opaque = HeaderValue::from_bytes(&[32, 33, 255]).expect("valid header value");
            let failures = assert_that!(opaque).with_location(false).capture(|it| {
                it.is_ascii_satisfying(|s| {
                    s.starts_with("http");
                })
            });
            assert_that!(failures).has_length(1);
            assert_that!(failures[0].to_string()).contains("is not ASCII");
        }
    }
}
