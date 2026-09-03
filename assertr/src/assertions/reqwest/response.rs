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

use crate::AssertThat;
use crate::mode::{Mode, Panic};
use alloc::borrow::ToOwned;
use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;
use core::fmt::Write;
use indoc::writedoc;
use reqwest::header::HeaderValue;

/// Non-extracting assertions for [`reqwest::Response`].
#[allow(clippy::return_self_not_must_use)]
#[cfg_attr(feature = "fluent", assertr_derive::fluent_aliases)]
pub trait ReqwestResponseAssertions {
    /// Asserts that the response has exactly this status code.
    fn has_status_code(self, expected: reqwest::StatusCode) -> Self;

    /// Asserts that the status code is informational (`1xx`).
    fn is_informational(self) -> Self;

    /// Asserts that the status code indicates success (`2xx`).
    fn is_success(self) -> Self;

    /// Asserts that the status code indicates a redirection (`3xx`).
    fn is_redirection(self) -> Self;

    /// Asserts that the status code indicates a client error (`4xx`).
    fn is_client_error(self) -> Self;

    /// Asserts that the status code indicates a server error (`5xx`).
    fn is_server_error(self) -> Self;

    /// Asserts that the response has a header with this name, regardless of its value.
    ///
    /// Header names are matched case-insensitively, as HTTP requires.
    fn has_header(self, name: impl AsRef<str>) -> Self;

    /// Asserts that the response has no header with this name.
    fn does_not_have_header(self, name: impl AsRef<str>) -> Self;

    /// Asserts that the response's first value for this header equals the expected UTF-8 value.
    ///
    /// The comparison uses raw header bytes. A non-UTF-8 subject value cannot equal the string
    /// expectation and is rendered lossily on failure.
    fn has_header_value(self, name: impl AsRef<str>, expected: impl AsRef<str>) -> Self;
}

impl<M: Mode, R> ReqwestResponseAssertions for AssertThat<'_, reqwest::Response, M, R> {
    #[track_caller]
    fn has_status_code(self, expected: reqwest::StatusCode) -> Self {
        self.track_assertion();

        let actual = self.actual().status();
        if actual != expected {
            self.fail_with_details(url_detail(&self), |w: &mut String| {
                writedoc! {w, r"
                    Actual: {actual}

                    is not the expected status code

                    Expected: {expected}
                "}
            });
        }

        self
    }

    #[track_caller]
    fn is_informational(self) -> Self {
        let actual = self.actual().status();
        assert_status_class(&self, actual.is_informational(), "informational (1xx)");
        self
    }

    #[track_caller]
    fn is_success(self) -> Self {
        let actual = self.actual().status();
        assert_status_class(&self, actual.is_success(), "a success (2xx)");
        self
    }

    #[track_caller]
    fn is_redirection(self) -> Self {
        let actual = self.actual().status();
        assert_status_class(&self, actual.is_redirection(), "a redirection (3xx)");
        self
    }

    #[track_caller]
    fn is_client_error(self) -> Self {
        let actual = self.actual().status();
        assert_status_class(&self, actual.is_client_error(), "a client error (4xx)");
        self
    }

    #[track_caller]
    fn is_server_error(self) -> Self {
        let actual = self.actual().status();
        assert_status_class(&self, actual.is_server_error(), "a server error (5xx)");
        self
    }

    #[track_caller]
    fn has_header(self, name: impl AsRef<str>) -> Self {
        self.track_assertion();
        assert_header_present(&self, name.as_ref());
        self
    }

    #[track_caller]
    fn does_not_have_header(self, name: impl AsRef<str>) -> Self {
        self.track_assertion();

        let name = name.as_ref();
        if let Some(value) = self.actual().headers().get(name) {
            let actual = render_value(value);
            self.fail_with_details(url_detail(&self), |w: &mut String| {
                writedoc! {w, r"
                    Actual: {actual:#?}

                    is the value of header {name:?}, which was not expected to be present
                "}
            });
        }

        self
    }

    #[track_caller]
    fn has_header_value(self, name: impl AsRef<str>, expected: impl AsRef<str>) -> Self {
        self.track_assertion();

        let name = name.as_ref();
        let expected = expected.as_ref();

        match self.actual().headers().get(name) {
            None => {
                let actual = header_names(&self);
                let mut details = url_detail(&self);
                details.push(format!("Expected value: {expected:?}"));
                self.fail_with_details(details, |w: &mut String| {
                    writedoc! {w, r"
                        Actual: {actual:#?}

                        does not contain the expected header

                        Expected: {name:?}
                    "}
                });
            }
            Some(value) if value.as_bytes() != expected.as_bytes() => {
                let actual = render_value(value);
                self.fail_with_details(url_detail(&self), |w: &mut String| {
                    writedoc! {w, r"
                        Actual: {actual:#?}

                        is not the expected value of header {name:?}

                        Expected: {expected:?}
                    "}
                });
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
#[cfg_attr(feature = "fluent", assertr_derive::fluent_aliases)]
pub trait ReqwestResponseExtractAssertions<'t, R> {
    /// Asserts that the header is present, then continues the chain on a clone of its first value.
    ///
    /// With the `http` feature enabled, the extracted `HeaderValue` is the subject of
    /// [`HttpHeaderValueAssertions`](crate::prelude::HttpHeaderValueAssertions), so
    /// `.get_header("content-type").is_ascii_satisfying(..)` works across both integrations.
    fn get_header(self, name: impl AsRef<str>) -> AssertThat<'t, HeaderValue, Panic, R>;

    /// Reads the response body and continues the chain on it as a `String`.
    ///
    /// Consumes the response, so the assertion has to own it: create it with
    /// `assert_that_owned!(response)` or `response.must_owned()`.
    ///
    /// # Panics
    ///
    /// Panics when the assertion only borrows its subject, and when the body cannot be read.
    /// Ownership is checked when this method is called, before it returns the future.
    fn get_text(self) -> impl Future<Output = AssertThat<'t, String, Panic, R>>;

    /// Reads the response body, deserializes it into `T`, and continues the chain on the value.
    ///
    /// Reads the body with [`get_text`](ReqwestResponseExtractAssertions::get_text), then
    /// deserializes it with `serde_json`. A deserialization failure includes the received text.
    ///
    /// Requires the `serde-json` feature in addition to `reqwest`.
    ///
    /// # Panics
    ///
    /// Panics when the assertion only borrows its subject, when the body cannot be read, and
    /// when the body is not valid JSON for `T`. Ownership is checked when this method is called,
    /// before it returns the future.
    #[cfg(feature = "serde-json")]
    fn get_json<T>(self) -> impl Future<Output = AssertThat<'t, T, Panic, R>>
    where
        T: serde::de::DeserializeOwned + 't,
        R: crate::ValueRenderer<String>;
}

impl<'t, R> ReqwestResponseExtractAssertions<'t, R>
    for AssertThat<'t, reqwest::Response, Panic, R>
{
    #[track_caller]
    fn get_header(self, name: impl AsRef<str>) -> AssertThat<'t, HeaderValue, Panic, R> {
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
    fn get_text(self) -> impl Future<Output = AssertThat<'t, String, Panic, R>> {
        self.track_assertion();
        if matches!(&self.actual, crate::actual::Actual::Borrowed(_)) {
            panic!(
                "get_text() consumes the response and can only be called on an owned Response! Create the assertion with `assert_that_owned!(...)` (or `.must_owned()`) instead."
            );
        }

        let location = core::panic::Location::caller();
        let url = format!("URL: {}", self.actual().url());
        get_text_at(self, location, url)
    }

    #[track_caller]
    #[cfg(feature = "serde-json")]
    fn get_json<T>(self) -> impl Future<Output = AssertThat<'t, T, Panic, R>>
    where
        T: serde::de::DeserializeOwned + 't,
        R: crate::ValueRenderer<String>,
    {
        self.track_assertion();
        if matches!(&self.actual, crate::actual::Actual::Borrowed(_)) {
            panic!(
                "get_json() consumes the response and can only be called on an owned Response! Create the assertion with `assert_that_owned!(...)` (or `.must_owned()`) instead."
            );
        }

        let location = core::panic::Location::caller();
        let url = format!("URL: {}", self.actual().url());
        async move {
            use crate::actual::Actual;

            let this = get_text_at(self, location, url.clone()).await;

            let parsed = serde_json::from_str::<T>(this.actual().as_str());

            if let Err(error) = &parsed {
                let actual = this.render().value(this.actual());
                let expected_type = core::any::type_name::<T>();
                this.fail_with_details_at(
                    location,
                    [url, format!("Error: {error}")],
                    |w: &mut String| {
                        writedoc! {w, r"
                        Actual: {actual:#?}

                        is not valid JSON for the expected type: {expected_type}
                    "}
                    },
                );
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
) -> AssertThat<'t, String, Panic, R> {
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

    // The read failure gets its own message and its own details rather than a staged detail
    // message: a staged one would also be attached to every later failure of the chain, and
    // claim the body could not be read long after it was read successfully.
    if let Err(error) = this.actual() {
        this.fail_with_details_at(
            location,
            [url, format!("Error: {error}")],
            |w: &mut String| {
                writedoc! {w, r"
                Expected: Response body to be readable.

                  Actual: Reading the response body failed!
            "}
            },
        );
    }

    this.map(|it| Actual::Owned(it.unwrap_owned().expect("already checked")))
}

/// The request URL, as the one piece of evidence that tells two responses apart.
fn url_detail<M: Mode, R>(this: &AssertThat<'_, reqwest::Response, M, R>) -> Vec<String> {
    alloc::vec![format!("URL: {}", this.actual().url())]
}

/// Fails with the missing-header diagnostic shared by assertions and projections.
#[track_caller]
fn assert_header_present<M: Mode, R>(this: &AssertThat<'_, reqwest::Response, M, R>, name: &str) {
    if this.actual().headers().get(name).is_none() {
        let actual = header_names(this);
        this.fail_with_details(url_detail(this), |w: &mut String| {
            writedoc! {w, r"
                Actual: {actual:#?}

                does not contain the expected header

                Expected: {name:?}
            "}
        });
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

/// Header values are bytes. Rendering one lossily keeps a non-UTF-8 value diagnosable instead of
/// replacing the whole message with a conversion error.
fn render_value(value: &HeaderValue) -> String {
    String::from_utf8_lossy(value.as_bytes()).into_owned()
}

#[track_caller]
fn assert_status_class<M: Mode, R>(
    this: &AssertThat<'_, reqwest::Response, M, R>,
    holds: bool,
    class: &str,
) {
    this.track_assertion();

    if !holds {
        let actual = this.actual().status();
        this.fail_with_details(url_detail(this), |w: &mut String| {
            writedoc! {w, r"
                Actual: {actual}

                is not {class} status code
            "}
        });
    }
}

#[cfg(test)]
mod tests {
    mod renderer_contract {
        use super::response;
        use crate::prelude::*;
        use crate::test_support::{NoRenderer, assert_trait_impl};

        /// Renders a response by its status code only. It implements no other `ValueRenderer`,
        /// so it proves which renderer capability each assertion actually requires.
        struct StatusOnly;

        impl ValueRenderer<reqwest::Response> for StatusOnly {
            fn fmt(
                &self,
                value: &reqwest::Response,
                f: &mut core::fmt::Formatter<'_>,
            ) -> core::fmt::Result {
                write!(f, "<{}>", value.status().as_u16())
            }
        }

        #[test]
        fn traits_are_implemented_without_response_renderer_support() {
            assert_trait_impl!(
                AssertThat<'static, reqwest::Response, Panic, NoRenderer>
                    => ReqwestResponseAssertions
            );
            assert_trait_impl!(
                AssertThat<'static, reqwest::Response, Panic, NoRenderer>
                    => ReqwestResponseExtractAssertions<'static, NoRenderer>
            );
        }

        #[test]
        fn checking_assertions_require_only_a_response_renderer() {
            assert_that!(response(200, &[("content-type", "text/plain")], ""))
                .with_renderer(StatusOnly)
                .has_status_code(reqwest::StatusCode::OK)
                .is_success()
                .has_header("content-type")
                .does_not_have_header("x-api-key")
                .has_header_value("content-type", "text/plain");

            assert_that!(response(100, &[], ""))
                .with_renderer(StatusOnly)
                .is_informational();
            assert_that!(response(301, &[], ""))
                .with_renderer(StatusOnly)
                .is_redirection();
            assert_that!(response(404, &[], ""))
                .with_renderer(StatusOnly)
                .is_client_error();
            assert_that!(response(500, &[], ""))
                .with_renderer(StatusOnly)
                .is_server_error();
        }

        #[test]
        fn body_extractors_require_only_the_renderers_their_failure_paths_use() {
            let text = assert_that_owned!(response(200, &[], "text"))
                .with_renderer(StatusOnly)
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

                Actual: 404 Not Found

                is not the expected status code

                Expected: 200 OK

                Details:
                  - URL: http://localhost/hello
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
                    it.has_display_value(formatdoc! {r#"
                        -------- assertr --------
                        Expression: `response(404, &[], "")`

                        Actual: 404 Not Found

                        is not the expected status code

                        Expected: 200 OK

                        Details:
                          - URL: http://localhost/hello
                        -------- assertr --------
                    "#});
                },
                |it: AssertThat<AssertionFailure, Capture>| {
                    it.has_display_value(formatdoc! {r#"
                        -------- assertr --------
                        Expression: `response(404, &[], "")`

                        Actual: 404 Not Found

                        is not a success (2xx) status code

                        Details:
                          - URL: http://localhost/hello
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

                Actual: 200 OK

                is not informational (1xx) status code

                Details:
                  - URL: http://localhost/hello
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
                    it.has_display_value(formatdoc! {r#"
                        -------- assertr --------
                        Expression: `response(200, &[], "")`

                        Actual: 200 OK

                        is not informational (1xx) status code

                        Details:
                          - URL: http://localhost/hello
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

                Actual: 500 Internal Server Error

                is not a success (2xx) status code

                Details:
                  - URL: http://localhost/hello
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
                    it.has_display_value(formatdoc! {r#"
                        -------- assertr --------
                        Expression: `response(500, &[], "")`

                        Actual: 500 Internal Server Error

                        is not a success (2xx) status code

                        Details:
                          - URL: http://localhost/hello
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

                Actual: 200 OK

                is not a redirection (3xx) status code

                Details:
                  - URL: http://localhost/hello
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
                    it.has_display_value(formatdoc! {r#"
                        -------- assertr --------
                        Expression: `response(200, &[], "")`

                        Actual: 200 OK

                        is not a redirection (3xx) status code

                        Details:
                          - URL: http://localhost/hello
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

                Actual: 500 Internal Server Error

                is not a client error (4xx) status code

                Details:
                  - URL: http://localhost/hello
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
                    it.has_display_value(formatdoc! {r#"
                        -------- assertr --------
                        Expression: `response(500, &[], "")`

                        Actual: 500 Internal Server Error

                        is not a client error (4xx) status code

                        Details:
                          - URL: http://localhost/hello
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

                Actual: 404 Not Found

                is not a server error (5xx) status code

                Details:
                  - URL: http://localhost/hello
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
                    it.has_display_value(formatdoc! {r#"
                        -------- assertr --------
                        Expression: `response(404, &[], "")`

                        Actual: 404 Not Found

                        is not a server error (5xx) status code

                        Details:
                          - URL: http://localhost/hello
                        -------- assertr --------
                    "#});
                },
            ]);
        }
    }

    mod has_header {
        use super::{ok_response, response};
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

                does not contain the expected header

                Expected: "content-type"

                Details:
                  - URL: http://localhost/hello
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
                    it.has_display_value(formatdoc! {r#"
                        -------- assertr --------
                        Expression: `response(200, &[("x-api-key", "1234")], "")`

                        Actual: [
                            "x-api-key",
                        ]

                        does not contain the expected header

                        Expected: "content-type"

                        Details:
                          - URL: http://localhost/hello
                        -------- assertr --------
                    "#});
                },
            ]);
        }
    }

    mod does_not_have_header {
        use super::{ok_response, response};
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

                Actual: "1234"

                is the value of header "x-api-key", which was not expected to be present

                Details:
                  - URL: http://localhost/hello
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
                    it.has_display_value(formatdoc! {r#"
                        -------- assertr --------
                        Expression: `response(200, &[("x-api-key", "1234")], "")`

                        Actual: "1234"

                        is the value of header "x-api-key", which was not expected to be present

                        Details:
                          - URL: http://localhost/hello
                        -------- assertr --------
                    "#});
                },
            ]);
        }
    }

    mod has_header_value {
        use super::{ok_response, response};
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

                Actual: "text/plain"

                is not the expected value of header "content-type"

                Expected: "application/json"

                Details:
                  - URL: http://localhost/hello
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

                does not contain the expected header

                Expected: "content-type"

                Details:
                  - URL: http://localhost/hello
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
                    it.has_display_value(formatdoc! {r#"
                        -------- assertr --------
                        Expression: `ok_response()`

                        Actual: "text/plain"

                        is not the expected value of header "content-type"

                        Expected: "application/json"

                        Details:
                          - URL: http://localhost/hello
                        -------- assertr --------
                    "#});
                },
            ]);
        }
    }

    mod get_header {
        use super::{ok_response, response};
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

                does not contain the expected header

                Expected: "content-type"

                Details:
                  - URL: http://localhost/hello
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
            .contains("URL: http://localhost/failing");
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
        /// `UnwindSafe`. A `reqwest::Response` is not, and neither is any future holding one
        /// across an await, so the async form (`assert_that_panic_by_async`) cannot express them.
        /// Building everything inside the closure keeps the closure's own captures unwind-safe.
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
            .is_equal_to(indoc::formatdoc! {r"
                -------- assertr --------
                Expression: `json_response(body)`

                Actual: {body:?}

                is not valid JSON for the expected type: {expected_type}

                Details:
                  - URL: http://localhost/hello
                  - Error: expected ident at line 1 column 2
                -------- assertr --------
            "});
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

                is not valid JSON for the expected type: {expected_type}

                Details:
                  - URL: http://localhost/hello
                  - Error: invalid type: string "old", expected u32 at line 1 column 25
                -------- assertr --------
            "#});
        }
    }
}
