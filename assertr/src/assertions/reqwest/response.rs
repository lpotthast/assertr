//! Assertions for `reqwest::Response`.
//!
//! Assertions cover status codes and headers. Projections expose one header, the text body, or a
//! JSON body. Failures include the request URL.
//!
//! Reading a body consumes the response. `get_text()` and `get_json()` are async and require
//! `assert_that_owned!` or `.must_owned()`.
//!
//! With the `http` feature enabled, the value extracted by `get_header` composes with
//! [`HttpHeaderValueAssertions`](crate::prelude::HttpHeaderValueAssertions): `reqwest` re-exports
//! `http`'s header types, so the two integrations meet on the same `HeaderValue`.

use crate::failure::{Fact, FailureKind};
use crate::mode::{Mode, Panic};
use crate::renderer::{GroupStyle, IntoRendered, Rendered, RenderingContext, SensitiveValuePolicy};
use crate::{AssertThat, ValueRenderer};
use alloc::borrow::ToOwned;
use alloc::string::String;
use alloc::vec::Vec;
use reqwest::header::HeaderValue;

/// Non-extracting assertions for [`reqwest::Response`].
#[allow(clippy::return_self_not_must_use)]
#[cfg_attr(feature = "fluent", assertr_macros::fluent_aliases)]
pub trait ReqwestResponseAssertions<R = crate::DebugRenderer> {
    /// Asserts that the response has exactly this status code.
    fn has_status_code(self, expected: reqwest::StatusCode) -> Self
    where
        R: ValueRenderer<reqwest::StatusCode> + ValueRenderer<str>;

    /// Asserts that the status code is informational (`1xx`).
    fn is_informational(self) -> Self
    where
        R: ValueRenderer<reqwest::StatusCode> + ValueRenderer<str>;

    /// Asserts that the status code indicates success (`2xx`).
    fn is_success(self) -> Self
    where
        R: ValueRenderer<reqwest::StatusCode> + ValueRenderer<str>;

    /// Asserts that the status code indicates a redirection (`3xx`).
    fn is_redirection(self) -> Self
    where
        R: ValueRenderer<reqwest::StatusCode> + ValueRenderer<str>;

    /// Asserts that the status code indicates a client error (`4xx`).
    fn is_client_error(self) -> Self
    where
        R: ValueRenderer<reqwest::StatusCode> + ValueRenderer<str>;

    /// Asserts that the status code indicates a server error (`5xx`).
    fn is_server_error(self) -> Self
    where
        R: ValueRenderer<reqwest::StatusCode> + ValueRenderer<str>;

    /// Asserts that the response has a header with this name, regardless of its value.
    ///
    /// Header names are matched case-insensitively, as HTTP requires.
    /// Failure diagnostics render header names and the URL through `ValueRenderer<str>`.
    fn has_header(self, name: impl AsRef<str>) -> Self
    where
        R: ValueRenderer<str>;

    /// Asserts that the response has no header with this name.
    ///
    /// Failure diagnostics show the first header value. By default, its contents are displayed
    /// even when marked with [`HeaderValue::set_sensitive(true)`](HeaderValue::set_sensitive),
    /// so test failures expose the value being asserted. Non-ASCII bytes use hexadecimal escapes.
    /// The rendering budget still applies.
    ///
    /// Custom renderers default to [`SensitiveValuePolicy::Preserve`], receiving the original
    /// header and sensitivity flag. Opting into [`SensitiveValuePolicy::Reveal`] passes an
    /// unmarked diagnostic copy through their normal `fmt` method. The response's header stays
    /// unchanged. Header names and the URL use `ValueRenderer<str>`.
    fn does_not_have_header(self, name: impl AsRef<str>) -> Self
    where
        R: ValueRenderer<HeaderValue> + ValueRenderer<str>;

    /// Asserts that the response's first value for this header equals the expected UTF-8 value.
    ///
    /// The comparison uses raw header bytes. A non-UTF-8 subject value cannot equal the string
    /// expectation. By default, non-ASCII header bytes use hexadecimal escapes on failure.
    ///
    /// By default, diagnostics display header contents even when marked with
    /// [`HeaderValue::set_sensitive(true)`](HeaderValue::set_sensitive), so test failures expose
    /// the value being compared.
    ///
    /// Custom renderers default to [`SensitiveValuePolicy::Preserve`], receiving the original
    /// header and sensitivity flag. Opting into [`SensitiveValuePolicy::Reveal`] passes an
    /// unmarked diagnostic copy through their normal `fmt` method. The response's header stays
    /// unchanged. The expected string, header names, and URL use `ValueRenderer<str>`.
    fn has_header_value(self, name: impl AsRef<str>, expected: impl AsRef<str>) -> Self
    where
        R: ValueRenderer<HeaderValue> + ValueRenderer<str>;
}

impl<M: Mode, R> ReqwestResponseAssertions<R> for AssertThat<'_, reqwest::Response, M, R> {
    #[track_caller]
    fn has_status_code(self, expected: reqwest::StatusCode) -> Self
    where
        R: ValueRenderer<reqwest::StatusCode> + ValueRenderer<str>,
    {
        self.track_assertion();

        let actual = self.actual().status();
        if actual != expected {
            self.failure(FailureKind::Equality)
                .actual(self.render().value(&actual))
                .expected(self.render().value(&expected))
                .fact(Fact::labelled(
                    URL,
                    self.render().value(self.actual().url().as_str()),
                ))
                .raise();
        }

        self
    }

    #[track_caller]
    fn is_informational(self) -> Self
    where
        R: ValueRenderer<reqwest::StatusCode> + ValueRenderer<str>,
    {
        let actual = self.actual().status();
        assert_status_class(
            &self,
            actual.is_informational(),
            "is not informational",
            "1xx",
        );
        self
    }

    #[track_caller]
    fn is_success(self) -> Self
    where
        R: ValueRenderer<reqwest::StatusCode> + ValueRenderer<str>,
    {
        let actual = self.actual().status();
        assert_status_class(&self, actual.is_success(), "is not a success", "2xx");
        self
    }

    #[track_caller]
    fn is_redirection(self) -> Self
    where
        R: ValueRenderer<reqwest::StatusCode> + ValueRenderer<str>,
    {
        let actual = self.actual().status();
        assert_status_class(
            &self,
            actual.is_redirection(),
            "is not a redirection",
            "3xx",
        );
        self
    }

    #[track_caller]
    fn is_client_error(self) -> Self
    where
        R: ValueRenderer<reqwest::StatusCode> + ValueRenderer<str>,
    {
        let actual = self.actual().status();
        assert_status_class(
            &self,
            actual.is_client_error(),
            "is not a client error",
            "4xx",
        );
        self
    }

    #[track_caller]
    fn is_server_error(self) -> Self
    where
        R: ValueRenderer<reqwest::StatusCode> + ValueRenderer<str>,
    {
        let actual = self.actual().status();
        assert_status_class(
            &self,
            actual.is_server_error(),
            "is not a server error",
            "5xx",
        );
        self
    }

    #[track_caller]
    fn has_header(self, name: impl AsRef<str>) -> Self
    where
        R: ValueRenderer<str>,
    {
        self.track_assertion();
        assert_header_present(&self, name.as_ref());
        self
    }

    #[track_caller]
    fn does_not_have_header(self, name: impl AsRef<str>) -> Self
    where
        R: ValueRenderer<HeaderValue> + ValueRenderer<str>,
    {
        self.track_assertion();

        let name = name.as_ref();
        if let Some(value) = self.actual().headers().get(name) {
            self.failure(FailureKind::Membership)
                .actual(
                    self.render()
                        .borrowed_values::<str, _>(&header_names(&self), GroupStyle::List),
                )
                .relation("contains the header")
                .unexpected(self.render().value(name))
                .fact(Fact::labelled(
                    URL,
                    self.render().value(self.actual().url().as_str()),
                ))
                .fact(Fact::labelled("Value", render_header(self.render(), value)))
                .raise();
        }

        self
    }

    #[track_caller]
    fn has_header_value(self, name: impl AsRef<str>, expected: impl AsRef<str>) -> Self
    where
        R: ValueRenderer<HeaderValue> + ValueRenderer<str>,
    {
        self.track_assertion();

        let name = name.as_ref();
        let expected = expected.as_ref();

        match self.actual().headers().get(name) {
            None => {
                self.failure(FailureKind::Equality)
                    .actual(
                        self.render()
                            .borrowed_values::<str, _>(&header_names(&self), GroupStyle::List),
                    )
                    .relation("does not contain the header")
                    .expected(self.render().value(name))
                    .fact(Fact::labelled(
                        URL,
                        self.render().value(self.actual().url().as_str()),
                    ))
                    .fact(Fact::labelled(
                        "Expected value",
                        self.render().value(expected),
                    ))
                    .raise();
            }
            Some(value) if value.as_bytes() != expected.as_bytes() => {
                self.failure(FailureKind::Equality)
                    .actual(render_header(self.render(), value))
                    .expected(self.render().value(expected))
                    .fact(Fact::labelled(
                        URL,
                        self.render().value(self.actual().url().as_str()),
                    ))
                    .fact(Fact::labelled("Header", self.render().value(name)))
                    .raise();
            }
            Some(_) => {}
        }

        self
    }
}

/// Panic-mode projections from [`reqwest::Response`].
///
/// Only available in `Panic` mode. Each projection can fail to produce a value, and a captured
/// failure has no value to continue the chain with.
#[cfg_attr(feature = "fluent", assertr_macros::fluent_aliases)]
pub trait ReqwestResponseExtractAssertions<'t, R> {
    /// Asserts that the header is present, then continues the chain on a clone of its first value.
    ///
    /// With the `http` feature enabled, the extracted `HeaderValue` is the subject of
    /// [`HttpHeaderValueAssertions`](crate::prelude::HttpHeaderValueAssertions), so
    /// `.get_header("content-type").is_ascii_satisfying(..)` works across both integrations.
    ///
    /// Missing-header diagnostics render header names and the URL through `ValueRenderer<str>`.
    fn get_header(self, name: impl AsRef<str>) -> AssertThat<'t, HeaderValue, Panic, R>
    where
        R: ValueRenderer<str>;

    /// Reads the response body and continues the chain on it as a `String`.
    ///
    /// Consumes the response, so the assertion has to own it: create it with
    /// `assert_that_owned!(response)` or `response.must_owned()`.
    ///
    /// # Panics
    ///
    /// Panics when the assertion only borrows its subject, and when the body cannot be read.
    /// Ownership is checked when this method is called, before it returns the future.
    fn get_text(self) -> impl Future<Output = AssertThat<'t, String, Panic, R>>
    where
        R: ValueRenderer<str> + ValueRenderer<reqwest::Error>;

    /// Reads the response body, deserializes it into `T`, and continues the chain on the value.
    ///
    /// Reads the body with [`get_text`](ReqwestResponseExtractAssertions::get_text), then
    /// deserializes it with `serde_json`. A deserialization failure includes the received text.
    ///
    /// Requires the `serde-json` feature in addition to `reqwest`.
    ///
    /// # Panics
    ///
    /// Panics when the assertion only borrows its subject, when the body cannot be read, and when
    /// the body is not valid JSON for `T`. Ownership is checked when this method is called, before
    /// it returns the future.
    #[cfg(feature = "serde-json")]
    fn get_json<T>(self) -> impl Future<Output = AssertThat<'t, T, Panic, R>>
    where
        T: serde::de::DeserializeOwned + 't,
        R: crate::ValueRenderer<String>
            + ValueRenderer<str>
            + ValueRenderer<reqwest::Error>
            + ValueRenderer<serde_json::Error>;
}

impl<'t, R> ReqwestResponseExtractAssertions<'t, R>
    for AssertThat<'t, reqwest::Response, Panic, R>
{
    #[track_caller]
    fn get_header(self, name: impl AsRef<str>) -> AssertThat<'t, HeaderValue, Panic, R>
    where
        R: ValueRenderer<str>,
    {
        self.track_assertion();
        let name = name.as_ref().to_owned();
        assert_header_present(&self, &name);

        self.map(move |it| {
            it.borrowed()
                .headers()
                .get(&name)
                .cloned()
                .expect("already checked")
                .into()
        })
    }

    #[track_caller]
    fn get_text(self) -> impl Future<Output = AssertThat<'t, String, Panic, R>>
    where
        R: ValueRenderer<str> + ValueRenderer<reqwest::Error>,
    {
        self.track_assertion();
        if matches!(&self.actual, crate::actual::Actual::Borrowed(_)) {
            panic!(
                "get_text() consumes the response and can only be called on an owned Response! Create the assertion with `assert_that_owned!(...)` (or `.must_owned()`) instead."
            );
        }

        let location = core::panic::Location::caller();
        let url = self.actual().url().as_str().to_owned();
        get_text_at(self, location, url)
    }

    #[track_caller]
    #[cfg(feature = "serde-json")]
    fn get_json<T>(self) -> impl Future<Output = AssertThat<'t, T, Panic, R>>
    where
        T: serde::de::DeserializeOwned + 't,
        R: crate::ValueRenderer<String>
            + ValueRenderer<str>
            + ValueRenderer<reqwest::Error>
            + ValueRenderer<serde_json::Error>,
    {
        self.track_assertion();
        if matches!(&self.actual, crate::actual::Actual::Borrowed(_)) {
            panic!(
                "get_json() consumes the response and can only be called on an owned Response! Create the assertion with `assert_that_owned!(...)` (or `.must_owned()`) instead."
            );
        }

        let location = core::panic::Location::caller();
        let url = self.actual().url().as_str().to_owned();
        async move {
            use crate::actual::Actual;

            let this = get_text_at(self, location, url.clone()).await;

            let parsed = serde_json::from_str::<T>(this.actual().as_str());

            if let Err(error) = &parsed {
                this.failure_at(FailureKind::Other, location)
                    .actual(this.render().value(this.actual()))
                    .relation("is not valid JSON for the expected type")
                    .fact(Fact::labelled(URL, this.render().value(url.as_str())))
                    .fact(Fact::labelled("Expected type", core::any::type_name::<T>()))
                    .fact(Fact::labelled("Error", this.render().value(error)))
                    .raise();
            }

            // Unreachable when the body did not deserialize: this trait is panic-mode only, so the
            // failure above never returns. Mirrors `OptionExtractAssertions::get_some`.
            this.map(|_| Actual::Owned(parsed.expect("already checked")))
        }
    }
}

async fn get_text_at<'t, R>(
    assertion: AssertThat<'t, reqwest::Response, Panic, R>,
    location: &'static core::panic::Location<'static>,
    url: String,
) -> AssertThat<'t, String, Panic, R>
where
    R: ValueRenderer<str> + ValueRenderer<reqwest::Error>,
{
    use crate::actual::Actual;

    let this = assertion
        .map_async(|it| {
            let response = match it {
                Actual::Borrowed(_) => unreachable!("ownership checked before creating the future"),
                Actual::Owned(response) => response,
            };
            async move { response.text().await }
        })
        .await;

    // The read failure gets its own message and its own facts rather than a staged detail message:
    // a staged one would also be attached to every later failure of the chain, and claim the body
    // could not be read long after it was read successfully.
    if let Err(error) = this.actual() {
        this.failure_at(FailureKind::Other, location)
            .relation("has a body that could not be read")
            .fact(Fact::labelled(URL, this.render().value(url.as_str())))
            .fact(Fact::labelled("Error", this.render().value(error)))
            .raise();
    }

    this.map(|it| Actual::Owned(it.unwrap_owned().expect("already checked")))
}

/// The label of the fact carrying the request URL, the one piece of evidence that tells two
/// responses apart.
const URL: &str = "URL";

/// Fails with the missing-header diagnostic shared by assertions and projections.
#[track_caller]
fn assert_header_present<M: Mode, R>(this: &AssertThat<'_, reqwest::Response, M, R>, name: &str)
where
    R: ValueRenderer<str>,
{
    if this.actual().headers().get(name).is_none() {
        this.failure(FailureKind::Membership)
            .actual(
                this.render()
                    .borrowed_values::<str, _>(&header_names(this), GroupStyle::List),
            )
            .relation("does not contain the header")
            .expected(this.render().value(name))
            .fact(Fact::labelled(
                URL,
                this.render().value(this.actual().url().as_str()),
            ))
            .raise();
    }
}

/// The names of all present headers, in wire order, as the evidence a missing-header failure needs.
fn header_names<'a, M: Mode, R>(this: &'a AssertThat<'_, reqwest::Response, M, R>) -> Vec<&'a str> {
    this.actual()
        .headers()
        .keys()
        .map(reqwest::header::HeaderName::as_str)
        .collect()
}

/// Prepare an unmarked diagnostic copy only when the active renderer requests it.
fn render_header<R: ValueRenderer<HeaderValue>>(
    rendering: RenderingContext<'_, R>,
    value: &HeaderValue,
) -> Rendered {
    match rendering.renderer().sensitive_value_policy() {
        SensitiveValuePolicy::Reveal if value.is_sensitive() => {
            let mut visible = value.clone();
            visible.set_sensitive(false);
            rendering.value(&visible).into_rendered()
        }
        _ => rendering.value(value).into_rendered(),
    }
}

#[track_caller]
fn assert_status_class<M: Mode, R>(
    this: &AssertThat<'_, reqwest::Response, M, R>,
    holds: bool,
    relation: &'static str,
    class: &'static str,
) where
    R: ValueRenderer<reqwest::StatusCode> + ValueRenderer<str>,
{
    this.track_assertion();

    if !holds {
        this.failure(FailureKind::Other)
            .actual(this.render().value(&this.actual().status()))
            .relation(relation)
            .expected(class)
            .fact(Fact::labelled(
                URL,
                this.render().value(this.actual().url().as_str()),
            ))
            .raise();
    }
}

#[cfg(test)]
mod tests {
    mod renderer_contract {
        use super::response;
        use crate::prelude::*;
        use crate::test_support::{NoRenderer, assert_trait_impl};

        struct EvidenceRenderer;
        impl ValueRenderer<str> for EvidenceRenderer {
            fn fmt(&self, value: &str, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                core::fmt::Debug::fmt(value, f)
            }
        }
        impl ValueRenderer<reqwest::StatusCode> for EvidenceRenderer {
            fn fmt(
                &self,
                value: &reqwest::StatusCode,
                f: &mut core::fmt::Formatter<'_>,
            ) -> core::fmt::Result {
                core::fmt::Debug::fmt(value, f)
            }
        }
        impl ValueRenderer<reqwest::Error> for EvidenceRenderer {
            fn fmt(
                &self,
                value: &reqwest::Error,
                f: &mut core::fmt::Formatter<'_>,
            ) -> core::fmt::Result {
                core::fmt::Debug::fmt(value, f)
            }
        }
        #[test]
        fn traits_are_implemented_without_response_renderer_support() {
            assert_trait_impl!(
                AssertThat<'static, reqwest::Response, Panic, NoRenderer>
                    => ReqwestResponseAssertions<NoRenderer>
            );
            assert_trait_impl!(
                AssertThat<'static, reqwest::Response, Panic, NoRenderer>
                    => ReqwestResponseExtractAssertions<'static, NoRenderer>
            );
        }

        #[test]
        fn successful_header_checks_do_not_render_or_consult_the_sensitivity_policy() {
            struct NeverRender;

            impl<T: ?Sized> ValueRenderer<T> for NeverRender {
                fn fmt(&self, _: &T, _: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                    panic!("a passing assertion must not render values")
                }

                fn sensitive_value_policy(&self) -> crate::renderer::SensitiveValuePolicy {
                    panic!("a passing assertion must not query diagnostic policy")
                }
            }

            let response = super::header_response(b"secret", true);
            assert_that!(response)
                .with_renderer(NeverRender)
                .has_header("x-api-key")
                .has_header_value("x-api-key", "secret")
                .does_not_have_header("missing")
                .get_header("x-api-key");
        }

        #[test]
        fn status_checks_do_not_require_a_subject_renderer() {
            assert_that!(response(200, &[("content-type", "text/plain")], ""))
                .with_renderer(EvidenceRenderer)
                .with_location(false)
                .has_status_code(reqwest::StatusCode::OK)
                .is_success();

            assert_that!(response(100, &[], ""))
                .with_renderer(EvidenceRenderer)
                .with_location(false)
                .is_informational();
            assert_that!(response(301, &[], ""))
                .with_renderer(EvidenceRenderer)
                .with_location(false)
                .is_redirection();
            assert_that!(response(404, &[], ""))
                .with_renderer(EvidenceRenderer)
                .with_location(false)
                .is_client_error();
            assert_that!(response(500, &[], ""))
                .with_renderer(EvidenceRenderer)
                .with_location(false)
                .is_server_error();
        }

        #[test]
        fn body_extractors_require_only_the_renderers_their_failure_paths_use() {
            let text = assert_that_owned!(response(200, &[], "text"))
                .with_renderer(EvidenceRenderer)
                .get_text();
            drop(text);

            #[cfg(feature = "serde-json")]
            {
                struct StringRenderer;

                impl ValueRenderer<String> for StringRenderer {
                    fn fmt(
                        &self,
                        value: &String,
                        f: &mut core::fmt::Formatter<'_>,
                    ) -> core::fmt::Result {
                        core::fmt::Debug::fmt(value, f)
                    }
                }

                impl ValueRenderer<str> for StringRenderer {
                    fn fmt(
                        &self,
                        value: &str,
                        f: &mut core::fmt::Formatter<'_>,
                    ) -> core::fmt::Result {
                        core::fmt::Debug::fmt(value, f)
                    }
                }
                impl ValueRenderer<reqwest::Error> for StringRenderer {
                    fn fmt(
                        &self,
                        value: &reqwest::Error,
                        f: &mut core::fmt::Formatter<'_>,
                    ) -> core::fmt::Result {
                        core::fmt::Debug::fmt(value, f)
                    }
                }
                impl ValueRenderer<serde_json::Error> for StringRenderer {
                    fn fmt(
                        &self,
                        value: &serde_json::Error,
                        f: &mut core::fmt::Formatter<'_>,
                    ) -> core::fmt::Result {
                        core::fmt::Debug::fmt(value, f)
                    }
                }
                let json = assert_that_owned!(response(200, &[], "null"))
                    .with_renderer(StringRenderer)
                    .get_json::<serde_json::Value>();
                drop(json);
            }
        }
    }

    use core::pin::Pin;
    use core::task::{Context, Poll};

    use reqwest::ResponseBuilderExt;

    struct FailingBody;

    impl http_body::Body for FailingBody {
        type Data = bytes::Bytes;
        type Error = std::io::Error;

        fn poll_frame(
            self: Pin<&mut Self>,
            _cx: &mut Context<'_>,
        ) -> Poll<Option<Result<http_body::Frame<Self::Data>, Self::Error>>> {
            Poll::Ready(Some(Err(std::io::Error::other("body read failed"))))
        }
    }

    /// Builds a response without a server: `reqwest` converts an `http::Response` directly, and
    /// `ResponseBuilderExt` carries the URL that the failure messages report.
    fn response(status: u16, headers: &[(&str, &str)], body: &'static str) -> reqwest::Response {
        let mut builder = http::Response::builder()
            .status(status)
            .url("http://localhost/hello".parse().expect("valid url"));

        for (name, value) in headers {
            builder = builder.header(*name, *value);
        }

        reqwest::Response::from(builder.body(body).expect("valid response"))
    }

    fn ok_response() -> reqwest::Response {
        response(200, &[("content-type", "text/plain")], "world")
    }

    fn header_response(bytes: &[u8], sensitive: bool) -> reqwest::Response {
        let mut response = response(200, &[], "");
        let mut value =
            reqwest::header::HeaderValue::from_bytes(bytes).expect("valid header bytes");
        value.set_sensitive(sensitive);
        response.headers_mut().insert("x-api-key", value);
        response
    }

    struct RedactingRenderer;

    struct RevealingRenderer<'a> {
        original: &'a reqwest::header::HeaderValue,
        calls: &'a core::cell::Cell<usize>,
    }

    impl crate::ValueRenderer<reqwest::header::HeaderValue> for RevealingRenderer<'_> {
        fn fmt(
            &self,
            value: &reqwest::header::HeaderValue,
            f: &mut core::fmt::Formatter<'_>,
        ) -> core::fmt::Result {
            assert!(!value.is_sensitive());
            assert_eq!(value.as_bytes(), self.original.as_bytes());
            assert_eq!(
                core::ptr::eq(value, self.original),
                !self.original.is_sensitive()
            );
            self.calls.set(self.calls.get() + 1);
            write!(f, "revealed({value:?})")
        }

        fn sensitive_value_policy(&self) -> crate::renderer::SensitiveValuePolicy {
            crate::renderer::SensitiveValuePolicy::Reveal
        }
    }

    impl crate::ValueRenderer<str> for RevealingRenderer<'_> {
        fn fmt(&self, value: &str, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            crate::ValueRenderer::fmt(&crate::DebugRenderer, value, f)
        }
    }

    struct TextOnly;

    impl crate::ValueRenderer<str> for TextOnly {
        fn fmt(&self, _: &str, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            f.write_str("<redacted text>")
        }
    }

    impl crate::ValueRenderer<reqwest::header::HeaderValue> for RedactingRenderer {
        fn fmt(
            &self,
            value: &reqwest::header::HeaderValue,
            f: &mut core::fmt::Formatter<'_>,
        ) -> core::fmt::Result {
            if value.is_sensitive() {
                f.write_str("<redacted header>")
            } else {
                write!(f, "header({value:?})")
            }
        }
    }

    impl crate::ValueRenderer<str> for RedactingRenderer {
        fn fmt(&self, _: &str, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            f.write_str("<redacted text>")
        }
    }

    fn failing_response() -> reqwest::Response {
        let response = http::Response::builder()
            .url("http://localhost/failing".parse().expect("valid url"))
            .body(reqwest::Body::wrap(FailingBody))
            .expect("valid response");
        reqwest::Response::from(response)
    }

    mod has_status_code {
        use super::{ok_response, response};
        use crate::prelude::*;
        use indoc::formatdoc;

        #[test]
        #[cfg(feature = "fluent")]
        fn fluent_alias_is_as_expected() {
            ok_response()
                .must()
                .have_status_code(reqwest::StatusCode::OK);
        }

        #[test]
        fn renders_status_and_url_evidence() {
            use indoc::formatdoc;

            use crate::test_support::{
                CustomValueRenderer, RedactingRenderer, assert_custom_value, assert_redacted,
            };
            let subject = response(404, &[], "");
            let failures = assert_that!(subject)
                .with_renderer(CustomValueRenderer)
                .with_location(false)
                .capture(|it| it.has_status_code(reqwest::StatusCode::OK));
            assert_that!(failures).has_length(1);
            assert_that!(failures[0]).has_text_report(formatdoc! {r#"
                -------- assertr --------
                Expression: `subject`

                Expected: custom(200)

                  Actual: custom(404)

                Details:
                  - URL: custom("http://localhost/hello")
                -------- assertr --------
            "#});

            assert_custom_value(failures[0].actual.as_ref().unwrap(), &subject.status());
            assert_custom_value(&failures[0].facts[0].value, subject.url().as_str());
            assert_custom_value(
                failures[0].expected.as_ref().unwrap(),
                &reqwest::StatusCode::OK,
            );
            let failures = assert_that!(subject)
                .with_renderer(RedactingRenderer)
                .with_location(false)
                .capture(|it| it.has_status_code(reqwest::StatusCode::OK));
            assert_that!(failures).has_length(1);
            assert_that!(failures[0]).has_text_report(formatdoc! {r"
                -------- assertr --------
                Expression: `subject`

                Expected: <redacted>

                  Actual: <redacted>

                Details:
                  - URL: <redacted>
                -------- assertr --------
            "});

            assert_redacted(&failures[0], &["localhost/hello", "404"]);
        }

        #[test]
        fn succeeds_when_status_code_matches() {
            assert_that!(ok_response()).has_status_code(reqwest::StatusCode::OK);
        }

        #[test]
        fn panics_when_status_code_differs() {
            assert_that_panic_by(|| {
                assert_that!(response(404, &[], ""))
                    .with_location(false)
                    .has_status_code(reqwest::StatusCode::OK);
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {r#"
                -------- assertr --------
                Expression: `response(404, &[], "")`

                Expected: 200

                  Actual: 404

                Details:
                  - URL: "http://localhost/hello"
                -------- assertr --------
            "#});
        }

        #[test]
        fn works_in_capture_mode_and_allows_further_chaining() {
            let failures = assert_that!(response(404, &[], ""))
                .with_location(false)
                .capture(|it| it.has_status_code(reqwest::StatusCode::OK).is_success());

            assert_that!(failures).contains_exactly_satisfying([
                |it: AssertThat<AssertionFailure, Capture>| {
                    it.has_text_report(formatdoc! {r#"
                        -------- assertr --------
                        Expression: `response(404, &[], "")`

                        Expected: 200

                          Actual: 404

                        Details:
                          - URL: "http://localhost/hello"
                        -------- assertr --------
                    "#});
                },
                |it: AssertThat<AssertionFailure, Capture>| {
                    it.has_text_report(formatdoc! {r#"
                        -------- assertr --------
                        Expression: `response(404, &[], "")`

                        Actual: 404

                        is not a success

                        Expected: 2xx

                        Details:
                          - URL: "http://localhost/hello"
                        -------- assertr --------
                    "#});
                },
            ]);
        }
    }

    mod is_informational {
        use super::response;
        use crate::prelude::*;
        use indoc::formatdoc;

        #[test]
        #[cfg(feature = "fluent")]
        fn fluent_alias_is_as_expected() {
            response(100, &[], "").must().be_informational();
        }

        #[test]
        fn renders_status_and_url_evidence() {
            use indoc::formatdoc;

            use crate::test_support::{
                CustomValueRenderer, RedactingRenderer, assert_custom_value, assert_redacted,
            };
            let subject = response(200, &[], "");
            let failures = assert_that!(subject)
                .with_renderer(CustomValueRenderer)
                .with_location(false)
                .capture(ReqwestResponseAssertions::is_informational);
            assert_that!(failures).has_length(1);
            assert_that!(failures[0]).has_text_report(formatdoc! {r#"
                -------- assertr --------
                Expression: `subject`

                Actual: custom(200)

                is not informational

                Expected: 1xx

                Details:
                  - URL: custom("http://localhost/hello")
                -------- assertr --------
            "#});

            assert_custom_value(failures[0].actual.as_ref().unwrap(), &subject.status());
            assert_custom_value(&failures[0].facts[0].value, subject.url().as_str());

            let failures = assert_that!(subject)
                .with_renderer(RedactingRenderer)
                .with_location(false)
                .capture(ReqwestResponseAssertions::is_informational);
            assert_that!(failures).has_length(1);
            assert_that!(failures[0]).has_text_report(formatdoc! {r"
                -------- assertr --------
                Expression: `subject`

                Actual: <redacted>

                is not informational

                Expected: 1xx

                Details:
                  - URL: <redacted>
                -------- assertr --------
            "});

            assert_redacted(&failures[0], &["localhost/hello", "200"]);
        }

        #[test]
        fn succeeds_for_any_1xx_status() {
            assert_that!(response(100, &[], "")).is_informational();
            assert_that!(response(103, &[], "")).is_informational();
        }

        #[test]
        fn panics_when_status_is_outside_the_class() {
            assert_that_panic_by(|| {
                assert_that!(response(200, &[], ""))
                    .with_location(false)
                    .is_informational();
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {r#"
                -------- assertr --------
                Expression: `response(200, &[], "")`

                Actual: 200

                is not informational

                Expected: 1xx

                Details:
                  - URL: "http://localhost/hello"
                -------- assertr --------
            "#});
        }

        #[test]
        fn works_in_capture_mode_and_allows_further_chaining() {
            let failures = assert_that!(response(200, &[], ""))
                .with_location(false)
                .capture(|it| it.is_informational().is_success());

            assert_that!(failures).contains_exactly_satisfying([
                |it: AssertThat<AssertionFailure, Capture>| {
                    it.has_text_report(formatdoc! {r#"
                        -------- assertr --------
                        Expression: `response(200, &[], "")`

                        Actual: 200

                        is not informational

                        Expected: 1xx

                        Details:
                          - URL: "http://localhost/hello"
                        -------- assertr --------
                    "#});
                },
            ]);
        }
    }

    mod is_success {
        use super::response;
        use crate::prelude::*;
        use indoc::formatdoc;

        #[test]
        #[cfg(feature = "fluent")]
        fn fluent_alias_is_as_expected() {
            response(200, &[], "").must().be_success();
        }

        #[test]
        fn renders_status_and_url_evidence() {
            use indoc::formatdoc;

            use crate::test_support::{
                CustomValueRenderer, RedactingRenderer, assert_custom_value, assert_redacted,
            };
            let subject = response(404, &[], "");
            let failures = assert_that!(subject)
                .with_renderer(CustomValueRenderer)
                .with_location(false)
                .capture(ReqwestResponseAssertions::is_success);
            assert_that!(failures).has_length(1);
            assert_that!(failures[0]).has_text_report(formatdoc! {r#"
                -------- assertr --------
                Expression: `subject`

                Actual: custom(404)

                is not a success

                Expected: 2xx

                Details:
                  - URL: custom("http://localhost/hello")
                -------- assertr --------
            "#});

            assert_custom_value(failures[0].actual.as_ref().unwrap(), &subject.status());
            assert_custom_value(&failures[0].facts[0].value, subject.url().as_str());

            let failures = assert_that!(subject)
                .with_renderer(RedactingRenderer)
                .with_location(false)
                .capture(ReqwestResponseAssertions::is_success);
            assert_that!(failures).has_length(1);
            assert_that!(failures[0]).has_text_report(formatdoc! {r"
                -------- assertr --------
                Expression: `subject`

                Actual: <redacted>

                is not a success

                Expected: 2xx

                Details:
                  - URL: <redacted>
                -------- assertr --------
            "});

            assert_redacted(&failures[0], &["localhost/hello", "404"]);
        }

        #[test]
        fn succeeds_for_any_2xx_status() {
            assert_that!(response(200, &[], "")).is_success();
            assert_that!(response(204, &[], "")).is_success();
            assert_that!(response(299, &[], "")).is_success();
        }

        #[test]
        fn panics_when_status_is_outside_the_class() {
            assert_that_panic_by(|| {
                assert_that!(response(500, &[], ""))
                    .with_location(false)
                    .is_success();
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {r#"
                -------- assertr --------
                Expression: `response(500, &[], "")`

                Actual: 500

                is not a success

                Expected: 2xx

                Details:
                  - URL: "http://localhost/hello"
                -------- assertr --------
            "#});
        }

        #[test]
        fn works_in_capture_mode_and_allows_further_chaining() {
            let failures = assert_that!(response(500, &[], ""))
                .with_location(false)
                .capture(|it| it.is_success().is_server_error());

            assert_that!(failures).contains_exactly_satisfying([
                |it: AssertThat<AssertionFailure, Capture>| {
                    it.has_text_report(formatdoc! {r#"
                        -------- assertr --------
                        Expression: `response(500, &[], "")`

                        Actual: 500

                        is not a success

                        Expected: 2xx

                        Details:
                          - URL: "http://localhost/hello"
                        -------- assertr --------
                    "#});
                },
            ]);
        }
    }

    mod is_redirection {
        use super::response;
        use crate::prelude::*;
        use indoc::formatdoc;

        #[test]
        #[cfg(feature = "fluent")]
        fn fluent_alias_is_as_expected() {
            response(301, &[], "").must().be_redirection();
        }

        #[test]
        fn renders_status_and_url_evidence() {
            use indoc::formatdoc;

            use crate::test_support::{
                CustomValueRenderer, RedactingRenderer, assert_custom_value, assert_redacted,
            };
            let subject = response(200, &[], "");
            let failures = assert_that!(subject)
                .with_renderer(CustomValueRenderer)
                .with_location(false)
                .capture(ReqwestResponseAssertions::is_redirection);
            assert_that!(failures).has_length(1);
            assert_that!(failures[0]).has_text_report(formatdoc! {r#"
                -------- assertr --------
                Expression: `subject`

                Actual: custom(200)

                is not a redirection

                Expected: 3xx

                Details:
                  - URL: custom("http://localhost/hello")
                -------- assertr --------
            "#});

            assert_custom_value(failures[0].actual.as_ref().unwrap(), &subject.status());
            assert_custom_value(&failures[0].facts[0].value, subject.url().as_str());

            let failures = assert_that!(subject)
                .with_renderer(RedactingRenderer)
                .with_location(false)
                .capture(ReqwestResponseAssertions::is_redirection);
            assert_that!(failures).has_length(1);
            assert_that!(failures[0]).has_text_report(formatdoc! {r"
                -------- assertr --------
                Expression: `subject`

                Actual: <redacted>

                is not a redirection

                Expected: 3xx

                Details:
                  - URL: <redacted>
                -------- assertr --------
            "});

            assert_redacted(&failures[0], &["localhost/hello", "200"]);
        }

        #[test]
        fn succeeds_for_any_3xx_status() {
            assert_that!(response(301, &[], "")).is_redirection();
            assert_that!(response(308, &[], "")).is_redirection();
        }

        #[test]
        fn panics_when_status_is_outside_the_class() {
            assert_that_panic_by(|| {
                assert_that!(response(200, &[], ""))
                    .with_location(false)
                    .is_redirection();
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {r#"
                -------- assertr --------
                Expression: `response(200, &[], "")`

                Actual: 200

                is not a redirection

                Expected: 3xx

                Details:
                  - URL: "http://localhost/hello"
                -------- assertr --------
            "#});
        }

        #[test]
        fn works_in_capture_mode_and_allows_further_chaining() {
            let failures = assert_that!(response(200, &[], ""))
                .with_location(false)
                .capture(|it| it.is_redirection().is_success());

            assert_that!(failures).contains_exactly_satisfying([
                |it: AssertThat<AssertionFailure, Capture>| {
                    it.has_text_report(formatdoc! {r#"
                        -------- assertr --------
                        Expression: `response(200, &[], "")`

                        Actual: 200

                        is not a redirection

                        Expected: 3xx

                        Details:
                          - URL: "http://localhost/hello"
                        -------- assertr --------
                    "#});
                },
            ]);
        }
    }

    mod is_client_error {
        use super::response;
        use crate::prelude::*;
        use indoc::formatdoc;

        #[test]
        #[cfg(feature = "fluent")]
        fn fluent_alias_is_as_expected() {
            response(404, &[], "").must().be_client_error();
        }

        #[test]
        fn renders_status_and_url_evidence() {
            use indoc::formatdoc;

            use crate::test_support::{
                CustomValueRenderer, RedactingRenderer, assert_custom_value, assert_redacted,
            };
            let subject = response(200, &[], "");
            let failures = assert_that!(subject)
                .with_renderer(CustomValueRenderer)
                .with_location(false)
                .capture(ReqwestResponseAssertions::is_client_error);
            assert_that!(failures).has_length(1);
            assert_that!(failures[0]).has_text_report(formatdoc! {r#"
                -------- assertr --------
                Expression: `subject`

                Actual: custom(200)

                is not a client error

                Expected: 4xx

                Details:
                  - URL: custom("http://localhost/hello")
                -------- assertr --------
            "#});

            assert_custom_value(failures[0].actual.as_ref().unwrap(), &subject.status());
            assert_custom_value(&failures[0].facts[0].value, subject.url().as_str());

            let failures = assert_that!(subject)
                .with_renderer(RedactingRenderer)
                .with_location(false)
                .capture(ReqwestResponseAssertions::is_client_error);
            assert_that!(failures).has_length(1);
            assert_that!(failures[0]).has_text_report(formatdoc! {r"
                -------- assertr --------
                Expression: `subject`

                Actual: <redacted>

                is not a client error

                Expected: 4xx

                Details:
                  - URL: <redacted>
                -------- assertr --------
            "});

            assert_redacted(&failures[0], &["localhost/hello", "200"]);
        }

        #[test]
        fn succeeds_for_any_4xx_status() {
            assert_that!(response(400, &[], "")).is_client_error();
            assert_that!(response(451, &[], "")).is_client_error();
        }

        #[test]
        fn panics_when_status_is_outside_the_class() {
            assert_that_panic_by(|| {
                assert_that!(response(500, &[], ""))
                    .with_location(false)
                    .is_client_error();
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {r#"
                -------- assertr --------
                Expression: `response(500, &[], "")`

                Actual: 500

                is not a client error

                Expected: 4xx

                Details:
                  - URL: "http://localhost/hello"
                -------- assertr --------
            "#});
        }

        #[test]
        fn works_in_capture_mode_and_allows_further_chaining() {
            let failures = assert_that!(response(500, &[], ""))
                .with_location(false)
                .capture(|it| it.is_client_error().is_server_error());

            assert_that!(failures).contains_exactly_satisfying([
                |it: AssertThat<AssertionFailure, Capture>| {
                    it.has_text_report(formatdoc! {r#"
                        -------- assertr --------
                        Expression: `response(500, &[], "")`

                        Actual: 500

                        is not a client error

                        Expected: 4xx

                        Details:
                          - URL: "http://localhost/hello"
                        -------- assertr --------
                    "#});
                },
            ]);
        }
    }

    mod is_server_error {
        use super::response;
        use crate::prelude::*;
        use indoc::formatdoc;

        #[test]
        #[cfg(feature = "fluent")]
        fn fluent_alias_is_as_expected() {
            response(500, &[], "").must().be_server_error();
        }

        #[test]
        fn renders_status_and_url_evidence() {
            use indoc::formatdoc;

            use crate::test_support::{
                CustomValueRenderer, RedactingRenderer, assert_custom_value, assert_redacted,
            };
            let subject = response(200, &[], "");
            let failures = assert_that!(subject)
                .with_renderer(CustomValueRenderer)
                .with_location(false)
                .capture(ReqwestResponseAssertions::is_server_error);
            assert_that!(failures).has_length(1);
            assert_that!(failures[0]).has_text_report(formatdoc! {r#"
                -------- assertr --------
                Expression: `subject`

                Actual: custom(200)

                is not a server error

                Expected: 5xx

                Details:
                  - URL: custom("http://localhost/hello")
                -------- assertr --------
            "#});

            assert_custom_value(failures[0].actual.as_ref().unwrap(), &subject.status());
            assert_custom_value(&failures[0].facts[0].value, subject.url().as_str());

            let failures = assert_that!(subject)
                .with_renderer(RedactingRenderer)
                .with_location(false)
                .capture(ReqwestResponseAssertions::is_server_error);
            assert_that!(failures).has_length(1);
            assert_that!(failures[0]).has_text_report(formatdoc! {r"
                -------- assertr --------
                Expression: `subject`

                Actual: <redacted>

                is not a server error

                Expected: 5xx

                Details:
                  - URL: <redacted>
                -------- assertr --------
            "});

            assert_redacted(&failures[0], &["localhost/hello", "200"]);
        }

        #[test]
        fn succeeds_for_any_5xx_status() {
            assert_that!(response(500, &[], "")).is_server_error();
            assert_that!(response(503, &[], "")).is_server_error();
        }

        #[test]
        fn panics_when_status_is_outside_the_class() {
            assert_that_panic_by(|| {
                assert_that!(response(404, &[], ""))
                    .with_location(false)
                    .is_server_error();
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {r#"
                -------- assertr --------
                Expression: `response(404, &[], "")`

                Actual: 404

                is not a server error

                Expected: 5xx

                Details:
                  - URL: "http://localhost/hello"
                -------- assertr --------
            "#});
        }

        #[test]
        fn works_in_capture_mode_and_allows_further_chaining() {
            let failures = assert_that!(response(404, &[], ""))
                .with_location(false)
                .capture(|it| it.is_server_error().is_client_error());

            assert_that!(failures).contains_exactly_satisfying([
                |it: AssertThat<AssertionFailure, Capture>| {
                    it.has_text_report(formatdoc! {r#"
                        -------- assertr --------
                        Expression: `response(404, &[], "")`

                        Actual: 404

                        is not a server error

                        Expected: 5xx

                        Details:
                          - URL: "http://localhost/hello"
                        -------- assertr --------
                    "#});
                },
            ]);
        }
    }

    mod has_header {
        use super::{TextOnly, ok_response, response};
        use crate::prelude::*;
        use indoc::formatdoc;

        #[test]
        #[cfg(feature = "fluent")]
        fn fluent_alias_is_as_expected() {
            ok_response().must().have_header("content-type");
        }

        #[test]
        fn succeeds_when_the_header_is_present() {
            assert_that!(ok_response()).has_header("content-type");
        }

        #[test]
        fn matches_the_header_name_case_insensitively() {
            assert_that!(ok_response()).has_header("Content-Type");
        }

        #[test]
        fn renders_failure_evidence_with_only_a_string_renderer() {
            let response = ok_response();
            let failures = assert_that!(response)
                .with_renderer(TextOnly)
                .with_location(false)
                .capture(|it| it.has_header("missing").has_header("content-type"));

            assert_that!(failures).has_length(1);
            assert_that!(failures[0]).has_text_report(formatdoc! {r"
                -------- assertr --------
                Expression: `response`

                Actual: [
                    <redacted text>,
                ]

                does not contain the header

                Expected: <redacted text>

                Details:
                  - URL: <redacted text>
                -------- assertr --------
            "});
        }

        #[test]
        fn applies_the_rendering_budget_to_header_names_and_strings() {
            let response = response(200, &[("x-first", "one"), ("x-second", "two")], "");
            let failures = assert_that!(response)
                .with_renderer(TextOnly)
                .with_location(false)
                .with_rendering_budget(
                    RenderingBudget::builder()
                        .max_items(1)
                        .max_leaf_characters(4)
                        .build(),
                )
                .capture(|it| it.has_header("missing"));

            assert_that!(failures[0]).has_text_report(formatdoc! {r"
                -------- assertr --------
                Expression: `response`

                Actual: [
                    <red... 11 more characters ...,
                ] (... 1 more element ...)

                does not contain the header

                Expected: <red... 11 more characters ...

                Details:
                  - URL: <red... 11 more characters ...
                -------- assertr --------
            "});
        }

        #[test]
        fn panics_when_the_header_is_absent() {
            assert_that_panic_by(|| {
                assert_that!(response(200, &[("x-api-key", "1234")], ""))
                    .with_location(false)
                    .has_header("content-type");
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {r#"
                -------- assertr --------
                Expression: `response(200, &[("x-api-key", "1234")], "")`

                Actual: [
                    "x-api-key",
                ]

                does not contain the header

                Expected: "content-type"

                Details:
                  - URL: "http://localhost/hello"
                -------- assertr --------
            "#});
        }

        #[test]
        fn works_in_capture_mode_and_allows_further_chaining() {
            let failures = assert_that!(response(200, &[("x-api-key", "1234")], ""))
                .with_location(false)
                .capture(|it| it.has_header("content-type").has_header("x-api-key"));

            assert_that!(failures).contains_exactly_satisfying([
                |it: AssertThat<AssertionFailure, Capture>| {
                    it.has_text_report(formatdoc! {r#"
                        -------- assertr --------
                        Expression: `response(200, &[("x-api-key", "1234")], "")`

                        Actual: [
                            "x-api-key",
                        ]

                        does not contain the header

                        Expected: "content-type"

                        Details:
                          - URL: "http://localhost/hello"
                        -------- assertr --------
                    "#});
                },
            ]);
        }
    }

    mod does_not_have_header {
        use super::{RedactingRenderer, RevealingRenderer, header_response, ok_response, response};
        use crate::prelude::*;
        use indoc::formatdoc;

        #[test]
        #[cfg(feature = "fluent")]
        fn fluent_alias_is_as_expected() {
            ok_response().must().not_have_header("x-api-key");
        }

        #[test]
        fn succeeds_when_the_header_is_absent() {
            assert_that!(ok_response()).does_not_have_header("x-api-key");
        }

        #[test]
        fn shows_sensitive_header_contents_by_default() {
            let response = header_response(b"secret-\xff", true);
            let failures = assert_that!(response)
                .with_location(false)
                .capture(|it| it.does_not_have_header("x-api-key"));

            assert_that!(failures).has_length(1);
            assert_that!(failures[0]).has_text_report(formatdoc! {r#"
                -------- assertr --------
                Expression: `response`

                Actual: [
                    "x-api-key",
                ]

                contains the header

                Unexpected: "x-api-key"

                Details:
                  - URL: "http://localhost/hello"
                  - Value: "secret-\xff"
                -------- assertr --------
            "#});
            assert_that!(response.headers()["x-api-key"].is_sensitive()).is_true();
        }

        #[test]
        fn respects_custom_redaction_and_preserves_header_metadata() {
            let response = header_response(b"secret-\xff", true);
            let failures = assert_that!(response)
                .with_renderer(RedactingRenderer)
                .with_location(false)
                .capture(|it| it.does_not_have_header("x-api-key"));

            assert_that!(failures).has_length(1);
            assert_that!(failures[0]).has_text_report(formatdoc! {r"
                -------- assertr --------
                Expression: `response`

                Actual: [
                    <redacted text>,
                ]

                contains the header

                Unexpected: <redacted text>

                Details:
                  - URL: <redacted text>
                  - Value: <redacted header>
                -------- assertr --------
            "});
            let value = &failures[0].facts[1].value;
            assert_that!(value.type_name)
                .is_equal_to(Some(core::any::type_name::<reqwest::header::HeaderValue>()));
            assert_that!(response.headers()["x-api-key"].is_sensitive()).is_true();
        }

        #[test]
        fn applies_the_rendering_budget_to_the_header_value() {
            let response = header_response(b"1234567890", true);
            let failures = assert_that!(response)
                .with_rendering_budget(RenderingBudget::builder().max_leaf_characters(4).build())
                .capture(|it| it.does_not_have_header("x-api-key"));

            assert_that!(failures[0].facts[1].value.body).is_equal_to(
                crate::renderer::RenderedBody::Text {
                    text: "\"123".into(),
                    omitted_characters: 8,
                },
            );
        }

        #[test]
        fn custom_renderers_can_request_revealed_header_values() {
            let response = header_response(b"secret-\xff", true);
            let calls = core::cell::Cell::new(0);
            let failures = assert_that!(response)
                .with_renderer(RevealingRenderer {
                    original: &response.headers()["x-api-key"],
                    calls: &calls,
                })
                .capture(|it| it.does_not_have_header("x-api-key"));

            assert_that!(failures).has_length(1);
            assert_that!(crate::test_support::rendered_text(
                &failures[0].facts[1].value
            ))
            .is_equal_to(r#"revealed("secret-\xff")"#);
            assert_that!(calls.get()).is_equal_to(1);
            assert_that!(response.headers()["x-api-key"].is_sensitive()).is_true();
        }

        #[test]
        fn panics_when_the_header_is_present() {
            assert_that_panic_by(|| {
                assert_that!(response(200, &[("x-api-key", "1234")], ""))
                    .with_location(false)
                    .does_not_have_header("x-api-key");
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {r#"
                -------- assertr --------
                Expression: `response(200, &[("x-api-key", "1234")], "")`

                Actual: [
                    "x-api-key",
                ]

                contains the header

                Unexpected: "x-api-key"

                Details:
                  - URL: "http://localhost/hello"
                  - Value: "1234"
                -------- assertr --------
            "#});
        }

        #[test]
        fn works_in_capture_mode_and_allows_further_chaining() {
            let failures = assert_that!(response(200, &[("x-api-key", "1234")], ""))
                .with_location(false)
                .capture(|it| it.does_not_have_header("x-api-key").has_header("x-api-key"));

            assert_that!(failures).contains_exactly_satisfying([
                |it: AssertThat<AssertionFailure, Capture>| {
                    it.has_text_report(formatdoc! {r#"
                        -------- assertr --------
                        Expression: `response(200, &[("x-api-key", "1234")], "")`

                        Actual: [
                            "x-api-key",
                        ]

                        contains the header

                        Unexpected: "x-api-key"

                        Details:
                          - URL: "http://localhost/hello"
                          - Value: "1234"
                        -------- assertr --------
                    "#});
                },
            ]);
        }
    }

    mod has_header_value {
        use super::{RedactingRenderer, RevealingRenderer, header_response, ok_response, response};
        use crate::prelude::*;
        use indoc::formatdoc;

        #[test]
        #[cfg(feature = "fluent")]
        fn fluent_alias_is_as_expected() {
            ok_response()
                .must()
                .have_header_value("content-type", "text/plain");
        }

        #[test]
        fn succeeds_when_the_value_matches() {
            assert_that!(ok_response()).has_header_value("content-type", "text/plain");
        }

        #[test]
        fn sensitivity_does_not_change_the_comparison() {
            let response = header_response(b"secret", true);
            assert_that!(response)
                .with_renderer(RedactingRenderer)
                .has_header_value("x-api-key", "secret");
        }

        #[test]
        fn compares_raw_bytes_and_escapes_sensitive_contents_by_default() {
            let response = header_response(b"secret-\xff", true);
            let failures = assert_that!(response)
                .with_location(false)
                .capture(|it| it.has_header_value("x-api-key", "secret-�"));

            assert_that!(failures).has_length(1);
            assert_that!(failures[0]).has_text_report(formatdoc! {r#"
                -------- assertr --------
                Expression: `response`

                Expected: "secret-�"

                  Actual: "secret-\xff"

                Details:
                  - URL: "http://localhost/hello"
                  - Header: "x-api-key"
                -------- assertr --------
            "#});
            assert_that!(response.headers()["x-api-key"].is_sensitive()).is_true();
        }

        #[test]
        fn respects_custom_redaction_and_preserves_header_metadata() {
            let response = header_response(b"secret-\xff", true);
            let failures = assert_that!(response)
                .with_renderer(RedactingRenderer)
                .with_location(false)
                .capture(|it| it.has_header_value("x-api-key", "another secret"));

            assert_that!(failures).has_length(1);
            assert_that!(failures[0]).has_text_report(formatdoc! {r"
                -------- assertr --------
                Expression: `response`

                Expected: <redacted text>

                  Actual: <redacted header>

                Details:
                  - URL: <redacted text>
                  - Header: <redacted text>
                -------- assertr --------
            "});
            assert_that!(failures[0].actual.as_ref().expect("actual value").type_name)
                .is_equal_to(Some(core::any::type_name::<reqwest::header::HeaderValue>()));
            assert_that!(response.headers()["x-api-key"].is_sensitive()).is_true();
        }

        #[test]
        fn renders_expected_values_when_the_header_is_missing() {
            let response = header_response(b"secret", true);
            let failures = assert_that!(response)
                .with_renderer(RedactingRenderer)
                .with_location(false)
                .capture(|it| it.has_header_value("missing", "another secret"));

            assert_that!(failures[0]).has_text_report(formatdoc! {r"
                -------- assertr --------
                Expression: `response`

                Actual: [
                    <redacted text>,
                ]

                does not contain the header

                Expected: <redacted text>

                Details:
                  - URL: <redacted text>
                  - Expected value: <redacted text>
                -------- assertr --------
            "});
        }

        #[test]
        fn passes_insensitive_header_bytes_to_custom_renderers_unchanged() {
            let response = header_response(b"visible-\xff", false);
            let failures = assert_that!(response)
                .with_renderer(RedactingRenderer)
                .capture(|it| it.has_header_value("x-api-key", "other"));

            let actual = failures[0].actual.as_ref().expect("actual value");
            assert_that!(crate::test_support::rendered_text(actual))
                .is_equal_to(r#"header("visible-\xff")"#);
        }

        #[test]
        fn applies_the_rendering_budget_to_custom_header_output() {
            let response = header_response(b"secret", true);
            let failures = assert_that!(response)
                .with_renderer(RedactingRenderer)
                .with_rendering_budget(RenderingBudget::builder().max_leaf_characters(4).build())
                .capture(|it| it.has_header_value("x-api-key", "other"));

            assert_that!(failures[0].actual.as_ref().expect("actual value").body).is_equal_to(
                crate::renderer::RenderedBody::Text {
                    text: "<red".into(),
                    omitted_characters: 13,
                },
            );
        }

        #[test]
        fn custom_renderers_can_request_revealed_header_values() {
            for sensitive in [true, false] {
                let response = header_response(b"secret-\xff", sensitive);
                let calls = core::cell::Cell::new(0);
                let failures = assert_that!(response)
                    .with_renderer(RevealingRenderer {
                        original: &response.headers()["x-api-key"],
                        calls: &calls,
                    })
                    .capture(|it| it.has_header_value("x-api-key", "other"));

                assert_that!(failures).has_length(1);
                let actual = failures[0].actual.as_ref().expect("actual value");
                assert_that!(crate::test_support::rendered_text(actual))
                    .is_equal_to(r#"revealed("secret-\xff")"#);
                assert_that!(actual.type_name)
                    .is_equal_to(Some(core::any::type_name::<reqwest::header::HeaderValue>()));
                assert_that!(calls.get()).is_equal_to(1);
                assert_that!(response.headers()["x-api-key"].is_sensitive()).is_equal_to(sensitive);
            }
        }

        #[test]
        fn applies_the_rendering_budget_after_a_custom_renderer_reveals_the_header() {
            let response = header_response(b"secret", true);
            let calls = core::cell::Cell::new(0);
            let failures = assert_that!(response)
                .with_renderer(RevealingRenderer {
                    original: &response.headers()["x-api-key"],
                    calls: &calls,
                })
                .with_rendering_budget(RenderingBudget::builder().max_leaf_characters(4).build())
                .capture(|it| it.has_header_value("x-api-key", "other"));

            assert_that!(failures[0].actual.as_ref().expect("actual value").body).is_equal_to(
                crate::renderer::RenderedBody::Text {
                    text: "reve".into(),
                    omitted_characters: 14,
                },
            );
            assert_that!(calls.get()).is_equal_to(1);
            assert_that!(response.headers()["x-api-key"].is_sensitive()).is_true();
        }

        #[test]
        fn compares_the_first_value_when_a_header_is_repeated() {
            assert_that!(response(
                200,
                &[("x-mode", "first"), ("x-mode", "second")],
                ""
            ))
            .has_header_value("x-mode", "first");
        }

        #[test]
        fn panics_when_the_value_differs() {
            assert_that_panic_by(|| {
                assert_that!(ok_response())
                    .with_location(false)
                    .has_header_value("content-type", "application/json");
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {r#"
                -------- assertr --------
                Expression: `ok_response()`

                Expected: "application/json"

                  Actual: "text/plain"

                Details:
                  - URL: "http://localhost/hello"
                  - Header: "content-type"
                -------- assertr --------
            "#});
        }

        #[test]
        fn panics_with_the_expected_value_as_a_detail_when_the_header_is_absent() {
            assert_that_panic_by(|| {
                assert_that!(response(200, &[], ""))
                    .with_location(false)
                    .has_header_value("content-type", "application/json");
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {r#"
                -------- assertr --------
                Expression: `response(200, &[], "")`

                Actual: []

                does not contain the header

                Expected: "content-type"

                Details:
                  - URL: "http://localhost/hello"
                  - Expected value: "application/json"
                -------- assertr --------
            "#});
        }

        #[test]
        fn works_in_capture_mode_and_allows_further_chaining() {
            let failures = assert_that!(ok_response())
                .with_location(false)
                .capture(|it| {
                    it.has_header_value("content-type", "application/json")
                        .has_header("content-type")
                });

            assert_that!(failures).contains_exactly_satisfying([
                |it: AssertThat<AssertionFailure, Capture>| {
                    it.has_text_report(formatdoc! {r#"
                        -------- assertr --------
                        Expression: `ok_response()`

                        Expected: "application/json"

                          Actual: "text/plain"

                        Details:
                          - URL: "http://localhost/hello"
                          - Header: "content-type"
                        -------- assertr --------
                    "#});
                },
            ]);
        }
    }

    mod get_header {
        use super::{TextOnly, ok_response, response};
        use crate::prelude::*;
        use indoc::formatdoc;

        #[test]
        #[cfg(feature = "fluent")]
        fn fluent_alias_is_as_expected() {
            ok_response()
                .must()
                .get_header("content-type")
                .is_equal_to(reqwest::header::HeaderValue::from_static("text/plain"));
        }

        #[test]
        fn extracts_the_value_of_a_present_header() {
            assert_that!(ok_response())
                .get_header("content-type")
                .is_equal_to(reqwest::header::HeaderValue::from_static("text/plain"));
        }

        #[test]
        fn extraction_requires_only_a_string_renderer_and_preserves_it() {
            let response = ok_response();
            let assertion: AssertThat<'_, reqwest::header::HeaderValue, Panic, TextOnly> =
                assert_that!(response)
                    .with_renderer(TextOnly)
                    .get_header("content-type");

            assert_that!(assertion.actual().as_bytes()).is_equal_to(b"text/plain");
        }

        #[test]
        fn renders_missing_header_evidence_with_only_a_string_renderer() {
            let response = ok_response();
            assert_that_panic_by(|| {
                assert_that!(response)
                    .with_renderer(TextOnly)
                    .with_location(false)
                    .get_header("missing");
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {r"
                -------- assertr --------
                Expression: `response`

                Actual: [
                    <redacted text>,
                ]

                does not contain the header

                Expected: <redacted text>

                Details:
                  - URL: <redacted text>
                -------- assertr --------
            "});
        }

        #[test]
        fn extracts_the_first_value_when_a_header_is_repeated() {
            assert_that!(response(
                200,
                &[("x-mode", "first"), ("x-mode", "second")],
                ""
            ))
            .get_header("x-mode")
            .is_equal_to(reqwest::header::HeaderValue::from_static("first"));
        }

        #[test]
        fn counts_as_one_assertion() {
            let response = ok_response();
            let assertion = assert_that!(response).get_header("content-type");

            assert_that!(assertion.state.number_of_assertions.borrow().0).is_equal_to(1);
        }

        #[test]
        fn panics_when_the_header_is_absent() {
            assert_that_panic_by(|| {
                assert_that!(response(200, &[("x-api-key", "1234")], ""))
                    .with_location(false)
                    .get_header("content-type");
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {r#"
                -------- assertr --------
                Expression: `response(200, &[("x-api-key", "1234")], "")`

                Actual: [
                    "x-api-key",
                ]

                does not contain the header

                Expected: "content-type"

                Details:
                  - URL: "http://localhost/hello"
                -------- assertr --------
            "#});
        }

        #[test]
        #[cfg(feature = "http")]
        fn does_not_attach_missing_header_detail_to_later_failures() {
            assert_that_panic_by(|| {
                assert_that!(ok_response())
                    .with_location(false)
                    .get_header("content-type")
                    .is_ascii_satisfying(|s| {
                        s.is_equal_to("nope");
                    });
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {r#"
                -------- assertr --------
                Expected: "nope"

                  Actual: "text/plain"
                -------- assertr --------
            "#});
        }
    }

    mod get_text {
        use super::{failing_response, ok_response, response};
        use crate::prelude::*;

        fn block_on<F: Future>(future: F) -> F::Output {
            tokio::runtime::Builder::new_current_thread()
                .build()
                .expect("runtime")
                .block_on(future)
        }

        #[tokio::test]
        #[cfg(feature = "fluent")]
        async fn fluent_alias_is_as_expected() {
            ok_response()
                .must_owned()
                .get_text()
                .await
                .is_equal_to("world");
        }

        #[test]
        fn body_errors_use_typed_renderers_and_can_be_redacted() {
            use indoc::formatdoc;

            use crate::test_support::{CustomValueRenderer, RedactingRenderer};

            assert_that_panic_by(|| {
                block_on(async {
                    assert_that_owned!(failing_response())
                        .with_renderer(CustomValueRenderer)
                        .with_location(false)
                        .get_text()
                        .await;
                });
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {r#"
                -------- assertr --------
                Expression: `failing_response()`

                has a body that could not be read

                Details:
                  - URL: custom("http://localhost/failing")
                  - Error: custom(reqwest::Error {{ kind: Decode, url: "http://localhost/failing", source: reqwest::Error {{ kind: Body, source: Custom {{ kind: Other, error: "body read failed" }} }} }})
                -------- assertr --------
            "#});
            assert_that_panic_by(|| {
                block_on(async {
                    assert_that_owned!(failing_response())
                        .with_renderer(RedactingRenderer)
                        .with_location(false)
                        .get_text()
                        .await;
                });
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {r"
                -------- assertr --------
                Expression: `failing_response()`

                has a body that could not be read

                Details:
                  - URL: <redacted>
                  - Error: <redacted>
                -------- assertr --------
            "});
        }

        #[tokio::test]
        async fn extracts_the_body() {
            assert_that_owned!(ok_response())
                .get_text()
                .await
                .is_equal_to("world");
        }

        #[tokio::test]
        async fn extracts_an_empty_body() {
            assert_that_owned!(response(204, &[], ""))
                .get_text()
                .await
                .is_equal_to("");
        }

        #[tokio::test]
        async fn counts_as_one_assertion() {
            let assertion = assert_that_owned!(ok_response()).get_text().await;

            assert_that!(assertion.state.number_of_assertions.borrow().0).is_equal_to(1);
        }

        #[test]
        fn failure_location_points_at_the_callers_assertion() {
            let expected_line = line!() + 3;
            let panic = assert_that_panic_by(|| {
                block_on(async {
                    assert_that_owned!(failing_response()).get_text().await;
                });
            });

            panic
                .has_type::<String>()
                .contains(format!("Assertion failed at {}:{expected_line}:", file!()));
        }

        #[test]
        fn panics_synchronously_when_the_response_is_only_borrowed() {
            assert_that_panic_by(|| {
                let response = ok_response();
                drop(assert_that!(response).get_text());
            })
            .has_type::<&str>()
            .is_equal_to(
                "get_text() consumes the response and can only be called on an owned Response! Create the assertion with `assert_that_owned!(...)` (or `.must_owned()`) instead.",
            );
        }

        #[test]
        fn body_read_failure_attaches_the_request_url() {
            assert_that_panic_by(|| {
                block_on(async {
                    assert_that_owned!(failing_response())
                        .with_location(false)
                        .get_text()
                        .await;
                });
            })
            .has_type::<String>()
            .contains("has a body that could not be read")
            .contains(r#"URL: "http://localhost/failing""#);
        }
    }

    #[cfg(feature = "serde-json")]
    mod get_json {
        use super::response;
        use crate::prelude::*;

        #[derive(Debug, PartialEq, serde::Deserialize)]
        struct Person {
            name: String,
            age: u32,
        }

        fn json_response(body: &'static str) -> reqwest::Response {
            response(200, &[("content-type", "application/json")], body)
        }

        /// Drives a future to completion from a synchronous test.
        ///
        /// The failure tests below need `assert_that_panic_by`, whose closure has to be
        /// `UnwindSafe`. A `reqwest::Response` is not, and neither is any future holding one across
        /// an await, so the async form (`assert_that_panic_by_async`) cannot express them. Building
        /// everything inside the closure keeps the closure's own captures unwind-safe.
        fn block_on<F: Future>(future: F) -> F::Output {
            tokio::runtime::Builder::new_current_thread()
                .build()
                .expect("runtime")
                .block_on(future)
        }

        #[tokio::test]
        #[cfg(feature = "fluent")]
        async fn fluent_alias_is_as_expected() {
            json_response(r#"{"name":"Bob","age":42}"#)
                .must_owned()
                .get_json::<Person>()
                .await
                .is_equal_to(Person {
                    name: "Bob".to_owned(),
                    age: 42,
                });
        }

        #[test]
        fn body_errors_use_typed_renderers_and_can_be_redacted() {
            use indoc::formatdoc;

            use crate::test_support::{CustomValueRenderer, RedactingRenderer};
            let body = r#"{{"name":"private-body-name","age":"private-body-age"}}"#;
            let expected_type = core::any::type_name::<Person>();
            assert_that_panic_by(|| {
                block_on(async {
                    assert_that_owned!(json_response(body))
                        .with_renderer(CustomValueRenderer)
                        .with_location(false)
                        .get_json::<Person>()
                        .await;
                });
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {r#"
                -------- assertr --------
                Expression: `json_response(body)`

                Actual: custom("{{{{\"name\":\"private-body-name\",\"age\":\"private-body-age\"}}}}")

                is not valid JSON for the expected type

                Details:
                  - URL: custom("http://localhost/hello")
                  - Expected type: {expected_type}
                  - Error: custom(Error("key must be a string", line: 1, column: 2))
                -------- assertr --------
            "#});
            assert_that_panic_by(|| {
                block_on(async {
                    assert_that_owned!(json_response(body))
                        .with_renderer(RedactingRenderer)
                        .with_location(false)
                        .get_json::<Person>()
                        .await;
                });
            })
            .has_type::<String>()
            .is_equal_to(formatdoc! {r"
                -------- assertr --------
                Expression: `json_response(body)`

                Actual: <redacted>

                is not valid JSON for the expected type

                Details:
                  - URL: <redacted>
                  - Expected type: {expected_type}
                  - Error: <redacted>
                -------- assertr --------
            "});
        }

        #[tokio::test]
        async fn extracts_the_deserialized_body() {
            assert_that_owned!(json_response(r#"{"name":"Bob","age":42}"#))
                .get_json::<Person>()
                .await
                .is_equal_to(Person {
                    name: "Bob".to_owned(),
                    age: 42,
                });
        }

        #[tokio::test]
        async fn counts_as_one_assertion() {
            let assertion = assert_that_owned!(json_response(r#"{"name":"Bob","age":42}"#))
                .get_json::<Person>()
                .await;

            assert_that!(assertion.state.number_of_assertions.borrow().0).is_equal_to(1);
        }

        #[tokio::test]
        async fn allows_chaining_on_the_deserialized_value() {
            assert_that_owned!(json_response(r#"{"name":"Bob","age":42}"#))
                .get_json::<Person>()
                .await
                .satisfies(
                    |person| &person.age,
                    |it| {
                        it.is_greater_than(18);
                    },
                );
        }

        #[test]
        fn panics_with_the_body_when_it_is_not_valid_json() {
            let body = "not json";
            let expected_type = core::any::type_name::<Person>();

            assert_that_panic_by(|| {
                block_on(async {
                    assert_that_owned!(json_response(body))
                        .with_location(false)
                        .get_json::<Person>()
                        .await;
                });
            })
            .has_type::<String>()
            .is_equal_to(indoc::formatdoc! {r#"
                -------- assertr --------
                Expression: `json_response(body)`

                Actual: {body:?}

                is not valid JSON for the expected type

                Details:
                  - URL: "http://localhost/hello"
                  - Expected type: {expected_type}
                  - Error: Error("expected ident", line: 1, column: 2)
                -------- assertr --------
            "#});
        }

        #[test]
        fn failure_location_points_at_the_callers_assertion() {
            let expected_line = line!() + 4;
            let panic = assert_that_panic_by(|| {
                block_on(async {
                    assert_that_owned!(json_response("not json"))
                        .get_json::<Person>()
                        .await;
                });
            });

            panic
                .has_type::<String>()
                .contains(format!("Assertion failed at {}:{expected_line}:", file!()));
        }

        #[test]
        fn panics_synchronously_when_the_response_is_only_borrowed() {
            assert_that_panic_by(|| {
                let response = json_response(r#"{"name":"Bob","age":42}"#);
                drop(assert_that!(response).get_json::<Person>());
            })
            .has_type::<&str>()
            .is_equal_to(
                "get_json() consumes the response and can only be called on an owned Response! Create the assertion with `assert_that_owned!(...)` (or `.must_owned()`) instead.",
            );
        }

        #[test]
        fn panics_with_the_body_when_a_field_has_the_wrong_type() {
            let body = r#"{"name":"Bob","age":"old"}"#;
            let expected_type = core::any::type_name::<Person>();

            assert_that_panic_by(|| {
                block_on(async {
                    assert_that_owned!(json_response(body))
                        .with_location(false)
                        .get_json::<Person>()
                        .await;
                });
            })
            .has_type::<String>()
            .is_equal_to(indoc::formatdoc! {r#"
                -------- assertr --------
                Expression: `json_response(body)`

                Actual: {body:?}

                is not valid JSON for the expected type

                Details:
                  - URL: "http://localhost/hello"
                  - Expected type: {expected_type}
                  - Error: Error("invalid type: string \"old\", expected u32", line: 1, column: 25)
                -------- assertr --------
            "#});
        }
    }
}
