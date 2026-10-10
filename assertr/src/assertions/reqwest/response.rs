//! Assertions for `reqwest::Response`.
//!
//! Assertions cover status codes and headers. Projections expose one header, the text body, or a
//! JSON body. Failures include the request URL.
//!
//! Reading a body consumes the response. `get_text()` and `get_json()` are async and require
//! `assert_that_owned!` or `.must_owned()`.
//!
//! `reqwest` re-exports `http`'s header types, so the value extracted by `get_header` is an
//! `http::HeaderValue` with `HttpHeaderValueAssertions`. The `reqwest` feature enables `http` for
//! this.

use alloc::{borrow::ToOwned, string::String, vec::Vec};
use core::panic::Location;

use reqwest::header::{HeaderName, HeaderValue};

use crate::{
    AssertThat,
    actual::Actual,
    assert_that::DetachedChain,
    assertions::http::header_value::reveal,
    expectation::{AssertionContext, Expectation},
    failure::{Fact, FailureBuilder, FailureKind},
    mode::{Mode, Panic},
    renderer::{DebugRenderer, Rendered, RenderingContext, RenderingOrder, ValueRenderer},
};

/// Compares the observed response status code.
#[derive(Debug, Clone, Copy)]
pub struct HasStatusCode(reqwest::StatusCode);
impl<R> Expectation<reqwest::Response, R> for HasStatusCode
where
    R: ValueRenderer<reqwest::StatusCode> + ValueRenderer<str>,
{
    type Success<'a> = ();
    type Rejection<'a> = reqwest::StatusCode;
    fn evaluate<'a>(
        &'a self,
        actual: &'a reqwest::Response,
        _context: &AssertionContext<'_, R>,
    ) -> Result<Self::Success<'a>, Self::Rejection<'a>> {
        let status = actual.status();
        if status == self.0 {
            Ok(())
        } else {
            Err(status)
        }
    }

    const KIND: FailureKind = FailureKind::Equality;
    fn explain<'a>(
        &'a self,
        rejected: Option<(&'a reqwest::Response, Self::Rejection<'a>)>,
        failure: FailureBuilder,
        context: &AssertionContext<'_, R>,
    ) -> FailureBuilder {
        let render = context.render();
        match rejected {
            None => failure
                .relation("has status code")
                .expected(render.value(&self.0)),
            Some((actual, status)) => failure
                .actual(render.value(&status))
                .expected(render.value(&self.0))
                .fact(url_fact(render, actual)),
        }
    }
}

impl HasStatusCode {
    /// Expects this status code.
    #[must_use]
    pub const fn new(expected: reqwest::StatusCode) -> Self {
        Self(expected)
    }
}
/// Generates a status-class expectation from its predicate, relations, and class label.
macro_rules! status_class_expectation {
    (
        $(#[$meta:meta])*
        $name:ident,
        $predicate:ident,
        $met:literal,
        $unmet:literal,
        $class:literal $(,)?
    ) => {
        $(#[$meta])*
        #[derive(Debug, Clone, Copy)]
        pub struct $name;
        impl<R> Expectation<reqwest::Response, R> for $name
        where
            R: ValueRenderer<reqwest::StatusCode> + ValueRenderer<str>,
        {
            type Success<'a> = ();
            type Rejection<'a> = reqwest::StatusCode;
            fn evaluate<'a>(
                &'a self,
                actual: &'a reqwest::Response,
                _context: &AssertionContext<'_, R>,
            ) -> Result<Self::Success<'a>, Self::Rejection<'a>> {
                let status = actual.status();
                if status.$predicate() {
                    Ok(())
                } else {
                    Err(status)
                }
            }

            const KIND: FailureKind = FailureKind::Other;
            fn explain<'a>(
                &'a self,
                rejected: Option<(&'a reqwest::Response, Self::Rejection<'a>)>,
                failure: FailureBuilder,
                context: &AssertionContext<'_, R>,
            ) -> FailureBuilder {
                let render = context.render();
                let failure = failure
                    .relations(rejected.map(|(_, status)| render.value(&status)), $met, $unmet)
                    .expected($class);
                match rejected {
                    None => failure,
                    Some((actual, _)) => failure.fact(url_fact(render, actual)),
                }
            }
        }

    };
}

status_class_expectation!(
    /// Checks whether the observed response status is informational.
    IsInformational,
    is_informational,
    "is informational",
    "is not informational",
    "1xx",
);

status_class_expectation!(
    /// Checks whether the observed response status is a success.
    IsSuccess,
    is_success,
    "is a success",
    "is not a success",
    "2xx",
);

status_class_expectation!(
    /// Checks whether the observed response status is a redirection.
    IsRedirection,
    is_redirection,
    "is a redirection",
    "is not a redirection",
    "3xx",
);

status_class_expectation!(
    /// Checks whether the observed response status is a client error.
    IsClientError,
    is_client_error,
    "is a client error",
    "is not a client error",
    "4xx",
);

status_class_expectation!(
    /// Checks whether the observed response status is a server error.
    IsServerError,
    is_server_error,
    "is a server error",
    "is not a server error",
    "5xx",
);

/// Why a response header expectation rejected the response.
///
/// `T` is what the expectation retains for a valid header name, for example the looked-up name
/// of a missing header.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HeaderRejection<'a, T> {
    /// The looked-up name is not a valid HTTP header name, so no header can have it.
    InvalidName(&'a str),
    /// The header did not meet the expectation.
    Rejected(T),
}

/// Checks that the response contains a header, returning its first value.
///
/// Rejection retains the looked-up name. Diagnostics list the present header names. A name that
/// is not a valid HTTP header name is rejected as invalid.
#[derive(Debug, Clone)]
pub struct HasHeader<E>(E);
impl<E, R> Expectation<reqwest::Response, R> for HasHeader<E>
where
    E: AsRef<str>,
    R: ValueRenderer<str>,
{
    type Success<'a>
        = &'a HeaderValue
    where
        Self: 'a,
        reqwest::Response: 'a;
    type Rejection<'a>
        = HeaderRejection<'a, &'a str>
    where
        Self: 'a,
        reqwest::Response: 'a;
    fn evaluate<'a>(
        &'a self,
        actual: &'a reqwest::Response,
        _context: &AssertionContext<'_, R>,
    ) -> Result<Self::Success<'a>, Self::Rejection<'a>> {
        let name = self.0.as_ref();
        first_value(actual, name)?.ok_or(HeaderRejection::Rejected(name))
    }

    const KIND: FailureKind = FailureKind::Membership;
    fn explain<'a>(
        &'a self,
        rejected: Option<(&'a reqwest::Response, Self::Rejection<'a>)>,
        failure: FailureBuilder,
        context: &AssertionContext<'_, R>,
    ) -> FailureBuilder {
        let render = context.render();
        match rejected {
            None => failure
                .relation("contains the header")
                .expected(render.value(self.0.as_ref())),
            Some((actual, HeaderRejection::InvalidName(name))) => {
                explain_invalid_name(failure, render, actual, name)
            }
            Some((actual, HeaderRejection::Rejected(name))) => {
                explain_missing_header(failure, render, actual, name)
            }
        }
    }
}

impl<E> HasHeader<E> {
    /// Expects a header with this name to be present.
    #[must_use]
    pub const fn new(name: E) -> Self {
        Self(name)
    }
}
/// Checks that the response does not contain a header.
///
/// Rejection retains the looked-up name and the header's first value. A name that is not a valid
/// HTTP header name is rejected as invalid instead of passing as absent.
#[derive(Debug, Clone)]
pub struct DoesNotHaveHeader<E>(E);
impl<E, R> Expectation<reqwest::Response, R> for DoesNotHaveHeader<E>
where
    E: AsRef<str>,
    R: ValueRenderer<str> + ValueRenderer<HeaderValue>,
{
    type Success<'a>
        = ()
    where
        Self: 'a,
        reqwest::Response: 'a;
    type Rejection<'a>
        = HeaderRejection<'a, (&'a str, &'a HeaderValue)>
    where
        Self: 'a,
        reqwest::Response: 'a;
    fn evaluate<'a>(
        &'a self,
        actual: &'a reqwest::Response,
        _context: &AssertionContext<'_, R>,
    ) -> Result<Self::Success<'a>, Self::Rejection<'a>> {
        let name = self.0.as_ref();
        match first_value(actual, name)? {
            Some(value) => Err(HeaderRejection::Rejected((name, value))),
            None => Ok(()),
        }
    }

    const KIND: FailureKind = FailureKind::Membership;
    fn explain<'a>(
        &'a self,
        rejected: Option<(&'a reqwest::Response, Self::Rejection<'a>)>,
        failure: FailureBuilder,
        context: &AssertionContext<'_, R>,
    ) -> FailureBuilder {
        let render = context.render();
        match rejected {
            None => failure
                .relation("does not contain the header")
                .unexpected(render.value(self.0.as_ref())),
            Some((actual, HeaderRejection::InvalidName(name))) => {
                explain_invalid_name(failure, render, actual, name)
            }
            Some((actual, HeaderRejection::Rejected((name, value)))) => failure
                .actual(render_header_names(render, actual))
                .relation("contains the header")
                .unexpected(render.value(name))
                .fact(url_fact(render, actual))
                .fact(Fact::labelled("Value", reveal(render, value))),
        }
    }
}

impl<E> DoesNotHaveHeader<E> {
    /// Expects no header with this name.
    #[must_use]
    pub const fn new(name: E) -> Self {
        Self(name)
    }
}
/// Compares the first header value with the expected raw UTF-8 bytes.
///
/// A name that is not a valid HTTP header name is rejected as invalid.
#[derive(Debug, Clone)]
pub struct HasHeaderValue<N, E> {
    name: N,
    expected: E,
}
impl<N, E, R> Expectation<reqwest::Response, R> for HasHeaderValue<N, E>
where
    N: AsRef<str>,
    E: AsRef<str>,
    R: ValueRenderer<str> + ValueRenderer<HeaderValue>,
{
    type Success<'a>
        = ()
    where
        Self: 'a,
        reqwest::Response: 'a;
    type Rejection<'a>
        = HeaderRejection<'a, (&'a str, &'a str, Option<&'a HeaderValue>)>
    where
        Self: 'a,
        reqwest::Response: 'a;
    fn evaluate<'a>(
        &'a self,
        actual: &'a reqwest::Response,
        _context: &AssertionContext<'_, R>,
    ) -> Result<Self::Success<'a>, Self::Rejection<'a>> {
        let name = self.name.as_ref();
        let expected = self.expected.as_ref();
        match first_value(actual, name)? {
            Some(value) if value.as_bytes() == expected.as_bytes() => Ok(()),
            value => Err(HeaderRejection::Rejected((name, expected, value))),
        }
    }

    const KIND: FailureKind = FailureKind::Equality;
    fn explain<'a>(
        &'a self,
        rejected: Option<(&'a reqwest::Response, Self::Rejection<'a>)>,
        failure: FailureBuilder,
        context: &AssertionContext<'_, R>,
    ) -> FailureBuilder {
        let render = context.render();
        match rejected {
            None => failure
                .relation("contains the header")
                .expected(render.value(self.name.as_ref()))
                .fact(Fact::labelled(
                    "Expected value",
                    render.value(self.expected.as_ref()),
                )),
            Some((actual, HeaderRejection::InvalidName(name))) => {
                explain_invalid_name(failure, render, actual, name)
            }
            Some((actual, HeaderRejection::Rejected((name, expected, None)))) => {
                explain_missing_header(failure, render, actual, name)
                    .fact(Fact::labelled("Expected value", render.value(expected)))
            }
            Some((actual, HeaderRejection::Rejected((name, expected, Some(value))))) => failure
                .actual(reveal(render, value))
                .expected(render.value(expected))
                .fact(url_fact(render, actual))
                .fact(Fact::labelled("Header", render.value(name))),
        }
    }
}

impl<N, E> HasHeaderValue<N, E> {
    /// Expects this header and value.
    #[must_use]
    pub const fn new(name: N, expected: E) -> Self {
        Self { name, expected }
    }
}

/// Non-extracting assertions for [`reqwest::Response`].
#[allow(clippy::return_self_not_must_use)]
#[cfg_attr(feature = "fluent", assertr_macros::fluent_aliases)]
pub trait ReqwestResponseAssertions<R = DebugRenderer> {
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
    /// Header names are matched case-insensitively, as HTTP requires. A name that is not a valid
    /// HTTP header name fails with "was given an invalid header name". Failure diagnostics render
    /// header names and the URL through `ValueRenderer<str>`.
    fn has_header(self, name: impl AsRef<str>) -> Self
    where
        R: ValueRenderer<str>;

    /// Asserts that the response has no header with this name.
    ///
    /// A name that is not a valid HTTP header name fails with "was given an invalid header name"
    /// instead of passing, because no header could ever have it. Failure diagnostics show the first
    /// header value. Its contents are displayed even when marked with
    /// [`HeaderValue::set_sensitive(true)`](HeaderValue::set_sensitive), so test
    /// failures expose the value being asserted. The renderer receives an unmarked diagnostic copy
    /// and the response's header stays unchanged. By default, non-ASCII bytes use hexadecimal
    /// escapes. The rendering budget still applies. Header names and the URL use
    /// `ValueRenderer<str>`.
    fn does_not_have_header(self, name: impl AsRef<str>) -> Self
    where
        R: ValueRenderer<HeaderValue> + ValueRenderer<str>;

    /// Asserts that the response's first value for this header equals the expected UTF-8 value.
    ///
    /// The comparison uses raw header bytes. A non-UTF-8 subject value cannot equal the string
    /// expectation. By default, non-ASCII header bytes use hexadecimal escapes on failure. A name
    /// that is not a valid HTTP header name fails with "was given an invalid header name".
    ///
    /// Diagnostics display header contents even when marked with
    /// [`HeaderValue::set_sensitive(true)`](HeaderValue::set_sensitive), so test failures expose
    /// the value being compared. The renderer receives an unmarked diagnostic copy and the
    /// response's header stays unchanged. The expected string, header names, and URL use
    /// `ValueRenderer<str>`.
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
        self.matches(HasStatusCode::new(expected))
    }

    #[track_caller]
    fn is_informational(self) -> Self
    where
        R: ValueRenderer<reqwest::StatusCode> + ValueRenderer<str>,
    {
        self.matches(IsInformational)
    }

    #[track_caller]
    fn is_success(self) -> Self
    where
        R: ValueRenderer<reqwest::StatusCode> + ValueRenderer<str>,
    {
        self.matches(IsSuccess)
    }

    #[track_caller]
    fn is_redirection(self) -> Self
    where
        R: ValueRenderer<reqwest::StatusCode> + ValueRenderer<str>,
    {
        self.matches(IsRedirection)
    }

    #[track_caller]
    fn is_client_error(self) -> Self
    where
        R: ValueRenderer<reqwest::StatusCode> + ValueRenderer<str>,
    {
        self.matches(IsClientError)
    }

    #[track_caller]
    fn is_server_error(self) -> Self
    where
        R: ValueRenderer<reqwest::StatusCode> + ValueRenderer<str>,
    {
        self.matches(IsServerError)
    }

    #[track_caller]
    fn has_header(self, name: impl AsRef<str>) -> Self
    where
        R: ValueRenderer<str>,
    {
        self.matches(HasHeader::new(name))
    }

    #[track_caller]
    fn does_not_have_header(self, name: impl AsRef<str>) -> Self
    where
        R: ValueRenderer<HeaderValue> + ValueRenderer<str>,
    {
        self.matches(DoesNotHaveHeader::new(name))
    }

    #[track_caller]
    fn has_header_value(self, name: impl AsRef<str>, expected: impl AsRef<str>) -> Self
    where
        R: ValueRenderer<HeaderValue> + ValueRenderer<str>,
    {
        self.matches(HasHeaderValue::new(name, expected))
    }
}

/// Panic-mode projections from [`reqwest::Response`].
///
/// Only available in `Panic` mode. Each projection can fail to produce a value, and a captured
/// failure has no value to continue the chain with.
#[cfg_attr(feature = "fluent", assertr_macros::fluent_aliases)]
pub trait ReqwestResponseExtractAssertions<'t, R = DebugRenderer> {
    /// Asserts that the header is present, then continues the chain on a clone of its first value.
    ///
    /// The extracted `HeaderValue` is the subject of `HttpHeaderValueAssertions`, so
    /// `.get_header("content-type").is_ascii_satisfying(..)` works across both integrations. A
    /// name that is not a valid HTTP header name fails like a missing header, with "was given an
    /// invalid header name".
    ///
    /// Missing-header diagnostics render header names and the URL through `ValueRenderer<str>`.
    fn get_header(self, name: impl AsRef<str>) -> AssertThat<'t, HeaderValue, Panic, R>
    where
        R: ValueRenderer<str>;

    /// Reads the response body and continues the chain on it as a `String`.
    ///
    /// Consumes the response, so the assertion has to own it. Create it with
    /// `assert_that_owned!(response)` or `response.must_owned()`.
    ///
    /// The assertion is tracked and the chain detached when this method is called. The returned
    /// future holds no chain records, so it is `Send` when the renderer is. It continues on a new
    /// assertion chain with the original chain's diagnostic settings and messages.
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
    /// Reads the body like [`get_text`](ReqwestResponseExtractAssertions::get_text), then
    /// deserializes it with `serde_json`. Reading and decoding count as one assertion. A
    /// deserialization failure includes the received text.
    ///
    /// Requires the `serde-json` feature in addition to `reqwest`.
    ///
    /// ```
    /// use assertr::prelude::*;
    ///
    /// #[derive(Debug, PartialEq, serde::Deserialize)]
    /// struct Greeting {
    ///     text: String,
    /// }
    ///
    /// # let response = reqwest::Response::from(http::Response::new(r#"{"text":"hello"}"#));
    /// # tokio::runtime::Builder::new_current_thread().build().unwrap().block_on(async {
    /// assert_that_owned!(response)
    ///     .is_success()
    ///     .get_json::<Greeting>()
    ///     .await
    ///     .is_equal_to(Greeting { text: "hello".to_owned() });
    /// # });
    /// ```
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
        R: ValueRenderer<str> + ValueRenderer<reqwest::Error> + ValueRenderer<serde_json::Error>;
}

impl<'t, R> ReqwestResponseExtractAssertions<'t, R>
    for AssertThat<'t, reqwest::Response, Panic, R>
{
    #[track_caller]
    fn get_header(self, name: impl AsRef<str>) -> AssertThat<'t, HeaderValue, Panic, R>
    where
        R: ValueRenderer<str>,
    {
        let definition = HasHeader::new(name);
        let value = self.require(&definition).clone();
        self.map(|_| value.into())
    }

    #[track_caller]
    fn get_text(self) -> impl Future<Output = AssertThat<'t, String, Panic, R>>
    where
        R: ValueRenderer<str> + ValueRenderer<reqwest::Error>,
    {
        let read = read_body(
            self,
            "get_text() consumes the response and can only be called on an owned Response! Create the assertion with `assert_that_owned!(...)` (or `.must_owned()`) instead.",
        );
        async move {
            let body = read.await;
            body.chain.attach(Actual::Owned(body.text))
        }
    }

    #[track_caller]
    #[cfg(feature = "serde-json")]
    fn get_json<T>(self) -> impl Future<Output = AssertThat<'t, T, Panic, R>>
    where
        T: serde::de::DeserializeOwned + 't,
        R: ValueRenderer<str> + ValueRenderer<reqwest::Error> + ValueRenderer<serde_json::Error>,
    {
        let read = read_body(
            self,
            "get_json() consumes the response and can only be called on an owned Response! Create the assertion with `assert_that_owned!(...)` (or `.must_owned()`) instead.",
        );
        async move {
            let ReadBody {
                text,
                chain,
                url,
                location,
            } = read.await;
            match serde_json::from_str::<T>(&text) {
                Ok(value) => chain.attach(Actual::Owned(value)),
                Err(error) => {
                    let render = chain.render();
                    let failure = FailureBuilder::new::<String>(FailureKind::Other)
                        .actual(render.value(text.as_str()))
                        .relation("is not valid JSON for the expected type")
                        .fact(Fact::labelled(URL, render.value(url.as_str())))
                        .fact(Fact::labelled("Expected type", core::any::type_name::<T>()))
                        .fact(Fact::labelled("Error", render.value(&error)));
                    chain.raise_at(failure, location)
                }
            }
        }
    }
}

/// A successfully read response body and its detached chain. JSON decoding also needs the
/// request URL and the caller for its diagnostics.
struct ReadBody<R> {
    text: String,
    chain: DetachedChain<R>,
    #[cfg(feature = "serde-json")]
    url: String,
    #[cfg(feature = "serde-json")]
    location: &'static Location<'static>,
}

/// Tracks a body extraction, rejects a borrowed response with `borrowed_message`, captures the
/// caller and the request URL, and detaches the chain. The returned future reads the body when
/// polled and raises a read failure. It holds no chain records, so it is `Send` when the renderer
/// is.
#[track_caller]
fn read_body<R>(
    assertion: AssertThat<'_, reqwest::Response, Panic, R>,
    borrowed_message: &'static str,
) -> impl Future<Output = ReadBody<R>>
where
    R: ValueRenderer<str> + ValueRenderer<reqwest::Error>,
{
    assertion.track_assertion();
    let location = Location::caller();
    let (actual, chain) = assertion.into_parts();
    let Actual::Owned(response) = actual else {
        std::panic::panic_any(borrowed_message);
    };
    let url = response.url().as_str().to_owned();
    async move {
        match response.text().await {
            Ok(text) => ReadBody {
                text,
                chain,
                #[cfg(feature = "serde-json")]
                url,
                #[cfg(feature = "serde-json")]
                location,
            },
            Err(error) => {
                let render = chain.render();
                let failure = FailureBuilder::new::<reqwest::Response>(FailureKind::Other)
                    .relation("has a body that could not be read")
                    .fact(Fact::labelled(URL, render.value(url.as_str())))
                    .fact(Fact::labelled("Error", render.value(&error)));
                chain.raise_at(failure, location)
            }
        }
    }
}

/// The label of the fact carrying the request URL, the one piece of evidence that tells two
/// responses apart.
const URL: &str = "URL";

/// Renders the request URL as a fact.
fn url_fact<R: ValueRenderer<str>>(
    render: RenderingContext<'_, R>,
    actual: &reqwest::Response,
) -> Fact {
    Fact::labelled(URL, render.value(actual.url().as_str()))
}

/// Renders the names of all present headers in their iteration order.
fn render_header_names<R: ValueRenderer<str>>(
    render: RenderingContext<'_, R>,
    actual: &reqwest::Response,
) -> Rendered {
    let names: Vec<&str> = actual.headers().keys().map(HeaderName::as_str).collect();
    render.borrowed_values::<str, _>(&names, RenderingOrder::PreserveIteration)
}

/// Explains a missing header by listing the present header names.
fn explain_missing_header<R: ValueRenderer<str>>(
    failure: FailureBuilder,
    render: RenderingContext<'_, R>,
    actual: &reqwest::Response,
    name: &str,
) -> FailureBuilder {
    failure
        .actual(render_header_names(render, actual))
        .relation("does not contain the header")
        .expected(render.value(name))
        .fact(url_fact(render, actual))
}

/// Looks up the first value of the header `name`, rejecting a name that no header can have.
///
/// `HeaderMap` lookups treat an invalid name as absent. Checking it first keeps
/// `does_not_have_header` from passing for a name that could never be present.
fn first_value<'a, T>(
    actual: &'a reqwest::Response,
    name: &'a str,
) -> Result<Option<&'a HeaderValue>, HeaderRejection<'a, T>> {
    match HeaderName::from_bytes(name.as_bytes()) {
        Ok(header) => Ok(actual.headers().get(&header)),
        Err(_) => Err(HeaderRejection::InvalidName(name)),
    }
}

/// Explains a header expectation given a name that is not a valid HTTP header name.
fn explain_invalid_name<R: ValueRenderer<str>>(
    failure: FailureBuilder,
    render: RenderingContext<'_, R>,
    actual: &reqwest::Response,
    name: &str,
) -> FailureBuilder {
    failure
        .relation("was given an invalid header name")
        .fact(url_fact(render, actual))
        .fact(Fact::labelled("Header", render.value(name)))
}

#[cfg(test)]
mod tests {
    use core::{
        pin::Pin,
        task::{Context, Poll},
    };

    use reqwest::{ResponseBuilderExt, header::HeaderValue};

    use crate::{prelude::*, test_support::block_on};

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
        let mut value = HeaderValue::from_bytes(bytes).expect("valid header bytes");
        value.set_sensitive(sensitive);
        response.headers_mut().insert("x-api-key", value);
        response
    }

    fn failing_response() -> reqwest::Response {
        let response = http::Response::builder()
            .url("http://localhost/failing".parse().expect("valid url"))
            .body(reqwest::Body::wrap(FailingBody))
            .expect("valid response");
        reqwest::Response::from(response)
    }

    #[cfg(feature = "serde-json")]
    fn json_response(body: &'static str) -> reqwest::Response {
        response(200, &[("content-type", "application/json")], body)
    }

    #[cfg(feature = "serde-json")]
    #[derive(Debug, PartialEq, serde::Deserialize)]
    struct Person {
        name: String,
        age: u32,
    }

    /// Renders only strings, so tests can show which failures need no other renderer.
    struct TextOnly;

    impl ValueRenderer<str> for TextOnly {
        fn fmt(&self, _: &str, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            f.write_str("<redacted text>")
        }
    }

    /// Checks that a header renderer receives an unmarked copy of the original value.
    struct RevealingRenderer<'a> {
        original: &'a HeaderValue,
        calls: &'a core::cell::Cell<usize>,
    }

    impl ValueRenderer<HeaderValue> for RevealingRenderer<'_> {
        fn fmt(&self, value: &HeaderValue, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            assert_that!(value.is_sensitive()).is_false();
            assert_that!(value.as_bytes()).is_equal_to(self.original.as_bytes());
            assert_that!(core::ptr::eq(value, self.original))
                .is_equal_to(!self.original.is_sensitive());
            self.calls.set(self.calls.get() + 1);
            write!(f, "revealed({value:?})")
        }
    }

    impl ValueRenderer<str> for RevealingRenderer<'_> {
        fn fmt(&self, value: &str, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            ValueRenderer::fmt(&DebugRenderer, value, f)
        }
    }

    #[cfg(feature = "fluent")]
    mod fluent_aliases {
        use super::{ok_response, response};
        use crate::{prelude::*, test_support::block_on};

        #[test]
        fn are_as_expected() {
            ok_response()
                .must()
                .have_status_code(reqwest::StatusCode::OK)
                .be_success()
                .have_header("content-type")
                .not_have_header("x-api-key")
                .have_header_value("content-type", "text/plain");
            response(100, &[], "").must().be_informational();
            response(301, &[], "").must().be_redirection();
            response(404, &[], "").must().be_client_error();
            response(500, &[], "").must().be_server_error();
            ok_response()
                .must()
                .get_header("content-type")
                .is_equal_to(reqwest::header::HeaderValue::from_static("text/plain"));
            block_on(ok_response().must_owned().get_text()).is_equal_to("world");
            #[cfg(feature = "serde-json")]
            block_on(super::json_response("42").must_owned().get_json::<u32>()).is_equal_to(42);
        }
    }

    mod renderer_contract {
        use super::{RevealingRenderer, failing_response, header_response, response};
        use crate::{
            prelude::*,
            test_support::{
                CustomValueRenderer, NoRenderer, RedactingRenderer, assert_custom_value,
                assert_redacted, assert_trait_impl, block_on,
            },
        };

        /// Renders everything a response assertion can need, except the response itself.
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
        #[cfg(feature = "serde-json")]
        impl ValueRenderer<serde_json::Error> for EvidenceRenderer {
            fn fmt(
                &self,
                value: &serde_json::Error,
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
        fn assertions_require_no_response_renderer() {
            let failures = assert_that!(response(200, &[], ""))
                .with_renderer(EvidenceRenderer)
                .capture(|it| {
                    it.has_status_code(reqwest::StatusCode::OK)
                        .is_informational()
                        .is_success()
                        .is_redirection()
                        .is_client_error()
                        .is_server_error()
                });
            assert_that!(failures).has_length(4);

            drop(
                assert_that_owned!(response(200, &[], "text"))
                    .with_renderer(EvidenceRenderer)
                    .get_text(),
            );
            #[cfg(feature = "serde-json")]
            drop(
                assert_that_owned!(response(200, &[], "null"))
                    .with_renderer(EvidenceRenderer)
                    .get_json::<u32>(),
            );
        }

        #[test]
        fn successful_header_checks_do_not_render() {
            let response = header_response(b"secret", true);
            assert_that!(response)
                .with_renderer(crate::test_support::PanickingRenderer(
                    "a passing assertion must not render values",
                ))
                .has_header("x-api-key")
                .has_header_value("x-api-key", "secret")
                .does_not_have_header("missing")
                .get_header("x-api-key");
        }

        #[test]
        fn status_evidence_renders_and_redacts_through_the_active_renderer() {
            macro_rules! case {
                ($status:literal, $check:expr) => {{
                    let subject = response($status, &[], "");
                    let failures = assert_that!(subject)
                        .with_renderer(CustomValueRenderer)
                        .capture($check);
                    assert_custom_value(failures[0].actual.as_ref().unwrap(), &subject.status());
                    assert_custom_value(&failures[0].facts[0].value, subject.url().as_str());

                    let failures = assert_that!(subject)
                        .with_renderer(RedactingRenderer)
                        .with_location(false)
                        .capture($check);
                    assert_redacted(&failures[0], &["localhost/hello", stringify!($status)]);
                }};
            }
            case!(404, |it| it.has_status_code(reqwest::StatusCode::OK));
            case!(404, ReqwestResponseAssertions::is_informational);
            case!(404, ReqwestResponseAssertions::is_success);
            case!(404, ReqwestResponseAssertions::is_redirection);
            case!(500, ReqwestResponseAssertions::is_client_error);
            case!(404, ReqwestResponseAssertions::is_server_error);

            let failures = assert_that!(response(404, &[], ""))
                .with_renderer(CustomValueRenderer)
                .capture(|it| it.has_status_code(reqwest::StatusCode::OK));
            assert_custom_value(
                failures[0].expected.as_ref().unwrap(),
                &reqwest::StatusCode::OK,
            );
        }

        #[test]
        fn header_evidence_renders_and_redacts_through_the_active_renderer() {
            let subject = header_response(b"secret-\xff", true);
            let mut revealed = subject.headers()["x-api-key"].clone();
            revealed.set_sensitive(false);

            let failures = assert_that!(subject)
                .with_renderer(CustomValueRenderer)
                .capture(|it| {
                    it.does_not_have_header("x-api-key")
                        .has_header_value("x-api-key", "other")
                });
            assert_custom_value(&failures[0].facts[1].value, &revealed);
            assert_custom_value(failures[1].actual.as_ref().unwrap(), &revealed);

            macro_rules! redacted {
                ($check:expr, $secrets:expr) => {{
                    let failures = assert_that!(subject)
                        .with_renderer(RedactingRenderer)
                        .with_location(false)
                        .capture($check);
                    assert_redacted(&failures[0], $secrets);
                }};
            }
            let secrets = &["secret", "x-api-key", "localhost/hello"];
            redacted!(|it| it.does_not_have_header("x-api-key"), secrets);
            redacted!(|it| it.has_header_value("x-api-key", "other"), secrets);
            redacted!(|it| it.has_header("missing"), &["missing", "x-api-key"]);
            redacted!(
                |it| it.has_header_value("missing", "other"),
                &["missing", "other", "x-api-key"]
            );
            assert_that!(subject.headers()["x-api-key"].is_sensitive()).is_true();
        }

        #[test]
        fn header_renderers_receive_an_unmarked_copy_and_the_budget_applies_after() {
            for sensitive in [true, false] {
                let response = header_response(b"secret-\xff", sensitive);
                let calls = core::cell::Cell::new(0);
                let failures = assert_that!(response)
                    .with_renderer(RevealingRenderer {
                        original: &response.headers()["x-api-key"],
                        calls: &calls,
                    })
                    .capture(|it| {
                        it.does_not_have_header("x-api-key")
                            .has_header_value("x-api-key", "other")
                    });

                let expected = r#"revealed("secret-\xff")"#;
                assert_that!(format!("{:#}", failures[0].facts[1].value)).is_equal_to(expected);
                assert_that!(format!("{:#}", failures[1].actual.as_ref().unwrap()))
                    .is_equal_to(expected);
                assert_that!(calls.get()).is_equal_to(2);
                assert_that!(response.headers()["x-api-key"].is_sensitive()).is_equal_to(sensitive);
            }

            let response = header_response(b"secret", true);
            let calls = core::cell::Cell::new(0);
            let failures = assert_that!(response)
                .with_renderer(RevealingRenderer {
                    original: &response.headers()["x-api-key"],
                    calls: &calls,
                })
                .with_rendering_budget(RenderingBudget::default().with_max_leaf_characters(4))
                .capture(|it| it.has_header_value("x-api-key", "other"));
            assert_that!(failures[0].actual.as_ref().unwrap().body).is_equal_to(
                crate::renderer::RenderedBody::Text {
                    text: "reve".into(),
                    omitted_characters: 14,
                },
            );
        }

        #[test]
        fn body_errors_render_and_redact_through_the_active_renderer() {
            macro_rules! body_panic {
                ($subject:expr, $renderer:expr, $extract:ident $(::<$ty:ty>)?) => {
                    assert_that!(|| {
                        block_on(async {
                            assert_that_owned!($subject)
                                .with_renderer($renderer)
                                .$extract $(::<$ty>)? ()
                                .await;
                        });
                    }).panics()
                    .has_message()
                };
            }
            body_panic!(failing_response(), CustomValueRenderer, get_text)
                .contains(r#"URL: custom("http://localhost/failing")"#)
                .contains("Error: custom(reqwest::Error {");
            body_panic!(failing_response(), RedactingRenderer, get_text)
                .contains("Error: <redacted>")
                .does_not_contain("localhost");

            #[cfg(feature = "serde-json")]
            {
                let body = r#"{"name":"private-name","age":"private-age"}"#;
                body_panic!(
                    super::json_response(body),
                    CustomValueRenderer,
                    get_json::<super::Person>
                )
                .contains(r#"Actual: custom("{\"name\":\"private-name\""#)
                .contains(r#"URL: custom("http://localhost/hello")"#)
                .contains(r#"Error: custom(Error("invalid type: string"#);
                body_panic!(
                    super::json_response(body),
                    RedactingRenderer,
                    get_json::<super::Person>
                )
                .contains("Error: <redacted>")
                .does_not_contain("private")
                .does_not_contain("localhost");
            }
        }
    }

    mod has_status_code {
        use indoc::formatdoc;

        use super::{ok_response, response};
        use crate::prelude::*;

        #[test]
        fn caller_location_is_as_expected() {
            assert_caller_location!(
                assert_that!(response(404, &[], "")),
                has_status_code(reqwest::StatusCode::OK)
            );
        }

        #[test]
        fn succeeds_when_status_code_matches() {
            assert_that!(ok_response()).has_status_code(reqwest::StatusCode::OK);
        }

        #[test]
        fn panics_when_status_code_differs() {
            assert_that!(|| {
                assert_that!(response(404, &[], ""))
                    .with_location(false)
                    .has_status_code(reqwest::StatusCode::OK);
            })
            .panics()
            .has_message()
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
    }

    /// Checks that the status-class assertion `$method` accepts the `$member` statuses and
    /// rejects `$outsider` with `$relation` and the class `$label`, continuing the chain.
    macro_rules! status_class_case {
        ($method:ident, [$($member:literal),+], $outsider:literal, $relation:literal, $label:literal) => {{
            $(assert_that!(super::response($member, &[], "")).$method();)+
            let subject = super::response($outsider, &[], "");
            let failures = assert_that!(subject)
                .with_location(false)
                // The chain continues after the failed class check.
                .capture(|it| it.$method().has_status_code(subject.status()));
            assert_that!(&failures[0]).has_text_report(indoc::formatdoc! {r#"
                -------- assertr --------
                Expression: `subject`

                Actual: {}

                {}

                Expected: {}

                Details:
                  - URL: "http://localhost/hello"
                -------- assertr --------
            "#, $outsider, $relation, $label});
            assert_that!(failures).has_length(1);
        }};
    }

    mod is_informational {
        use super::response;
        use crate::prelude::*;

        #[test]
        fn caller_location_is_as_expected() {
            assert_caller_location!(assert_that!(response(200, &[], "")), is_informational());
        }

        #[test]
        fn accepts_its_class_and_rejects_other_statuses() {
            status_class_case!(
                is_informational,
                [100, 103],
                200,
                "is not informational",
                "1xx"
            );
        }
    }

    mod is_success {
        use super::response;
        use crate::prelude::*;

        #[test]
        fn caller_location_is_as_expected() {
            assert_caller_location!(assert_that!(response(500, &[], "")), is_success());
        }

        #[test]
        fn accepts_its_class_and_rejects_other_statuses() {
            status_class_case!(is_success, [200, 204, 299], 500, "is not a success", "2xx");
        }
    }

    mod is_redirection {
        use super::response;
        use crate::prelude::*;

        #[test]
        fn caller_location_is_as_expected() {
            assert_caller_location!(assert_that!(response(200, &[], "")), is_redirection());
        }

        #[test]
        fn accepts_its_class_and_rejects_other_statuses() {
            status_class_case!(
                is_redirection,
                [301, 308],
                200,
                "is not a redirection",
                "3xx"
            );
        }
    }

    mod is_client_error {
        use super::response;
        use crate::prelude::*;

        #[test]
        fn caller_location_is_as_expected() {
            assert_caller_location!(assert_that!(response(500, &[], "")), is_client_error());
        }

        #[test]
        fn accepts_its_class_and_rejects_other_statuses() {
            status_class_case!(
                is_client_error,
                [400, 451],
                500,
                "is not a client error",
                "4xx"
            );
        }
    }

    mod is_server_error {
        use super::response;
        use crate::prelude::*;

        #[test]
        fn caller_location_is_as_expected() {
            assert_caller_location!(assert_that!(response(404, &[], "")), is_server_error());
        }

        #[test]
        fn accepts_its_class_and_rejects_other_statuses() {
            status_class_case!(
                is_server_error,
                [500, 503],
                404,
                "is not a server error",
                "5xx"
            );
        }
    }

    mod has_header {
        use indoc::formatdoc;

        use super::{TextOnly, ok_response, response};
        use crate::prelude::*;

        #[test]
        fn caller_location_is_as_expected() {
            assert_caller_location!(
                assert_that!(response(200, &[("x-api-key", "1234")], "")),
                has_header("content-type")
            );
        }

        #[test]
        fn matches_the_header_name_case_insensitively() {
            assert_that!(ok_response())
                .has_header("content-type")
                .has_header("Content-Type");
        }

        #[test]
        fn panics_when_the_header_is_absent() {
            assert_that!(|| {
                assert_that!(response(200, &[("x-api-key", "1234")], ""))
                    .with_location(false)
                    .has_header("content-type");
            })
            .panics()
            .has_message()
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
        fn rejects_an_invalid_header_name() {
            let failures = assert_that!(ok_response())
                .with_location(false)
                .capture(|it| it.has_header("content type"));

            assert_that!(failures[0]).has_text_report(formatdoc! {r#"
                -------- assertr --------
                Expression: `ok_response()`

                was given an invalid header name

                Details:
                  - URL: "http://localhost/hello"
                  - Header: "content type"
                -------- assertr --------
            "#});
        }

        #[test]
        fn applies_the_rendering_budget_with_only_a_string_renderer() {
            let response = response(200, &[("x-first", "one"), ("x-second", "two")], "");
            let failures = assert_that!(response)
                .with_renderer(TextOnly)
                .with_location(false)
                .with_rendering_budget(
                    RenderingBudget::default()
                        .with_max_items(1)
                        .with_max_leaf_characters(4),
                )
                .capture(|it| it.has_header("missing").has_header("x-first"));

            assert_that!(failures).has_length(1);
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
    }

    mod does_not_have_header {
        use indoc::formatdoc;

        use super::{header_response, ok_response, response};
        use crate::prelude::*;

        #[test]
        fn caller_location_is_as_expected() {
            assert_caller_location!(
                assert_that!(response(200, &[("x-api-key", "1234")], "")),
                does_not_have_header("x-api-key")
            );
        }

        #[test]
        fn succeeds_when_the_header_is_absent() {
            assert_that!(ok_response()).does_not_have_header("x-api-key");
        }

        #[test]
        fn rejects_an_invalid_header_name_instead_of_passing() {
            let failures =
                assert_that!(ok_response()).capture(|it| it.does_not_have_header("x api key"));
            assert_that!(failures[0].relation.as_deref())
                .is_equal_to(Some("was given an invalid header name"));
        }

        #[test]
        fn shows_sensitive_header_contents_by_default() {
            let response = header_response(b"secret-\xff", true);
            let failures = assert_that!(response)
                .with_location(false)
                .capture(|it| it.does_not_have_header("x-api-key"));

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
        fn applies_the_rendering_budget_to_the_header_value() {
            let response = header_response(b"1234567890", true);
            let failures = assert_that!(response)
                .with_rendering_budget(RenderingBudget::default().with_max_leaf_characters(4))
                .capture(|it| it.does_not_have_header("x-api-key"));

            assert_that!(failures[0].facts[1].value.body).is_equal_to(
                crate::renderer::RenderedBody::Text {
                    text: "\"123".into(),
                    omitted_characters: 8,
                },
            );
        }
    }

    mod has_header_value {
        use indoc::formatdoc;

        use super::{header_response, ok_response, response};
        use crate::{prelude::*, test_support::RedactingRenderer};

        #[test]
        fn caller_location_is_as_expected() {
            assert_caller_location!(
                assert_that!(ok_response()),
                has_header_value("content-type", "application/json")
            );
        }

        #[test]
        fn compares_the_first_value_regardless_of_sensitivity() {
            assert_that!(ok_response()).has_header_value("content-type", "text/plain");
            assert_that!(response(
                200,
                &[("x-mode", "first"), ("x-mode", "second")],
                ""
            ))
            .has_header_value("x-mode", "first");
            assert_that!(header_response(b"secret", true))
                .with_renderer(RedactingRenderer)
                .has_header_value("x-api-key", "secret");
        }

        #[test]
        fn compares_raw_bytes_and_shows_sensitive_contents_by_default() {
            let response = header_response(b"secret-\xff", true);
            let failures = assert_that!(response)
                .with_location(false)
                .capture(|it| it.has_header_value("x-api-key", "secret-�"));

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
        fn rejects_an_invalid_header_name() {
            let failures = assert_that!(ok_response())
                .capture(|it| it.has_header_value("content type", "text/plain"));
            assert_that!(failures[0].relation.as_deref())
                .is_equal_to(Some("was given an invalid header name"));
        }

        #[test]
        fn panics_with_the_expected_value_as_a_detail_when_the_header_is_absent() {
            assert_that!(|| {
                assert_that!(response(200, &[], ""))
                    .with_location(false)
                    .has_header_value("content-type", "application/json");
            })
            .panics()
            .has_message()
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
    }

    mod get_header {
        use super::{TextOnly, ok_response, response};
        use crate::prelude::*;

        #[test]
        fn caller_location_is_as_expected() {
            assert_caller_location!(assert_that!(ok_response()), get_header("missing-header"));
        }

        #[test]
        fn extracts_the_first_value_as_one_assertion() {
            let response = response(200, &[("x-mode", "first"), ("x-mode", "second")], "");
            let assertion = assert_that!(response).get_header("x-mode");

            assert_that!(assertion.state.records.assertion_count()).is_equal_to(1);
            assertion.is_equal_to(reqwest::header::HeaderValue::from_static("first"));
        }

        #[test]
        fn rejects_an_invalid_header_name() {
            assert_that!(|| {
                assert_that!(ok_response()).get_header("content type");
            })
            .panics()
            .has_message()
            .contains("was given an invalid header name");
        }

        #[test]
        fn extraction_requires_only_a_string_renderer_and_preserves_it() {
            let response = ok_response();
            let assertion: AssertThat<'_, reqwest::header::HeaderValue, Panic, TextOnly> =
                assert_that!(response)
                    .with_renderer(TextOnly)
                    .get_header("content-type");

            assert_that!(assertion.actual().as_bytes()).is_equal_to(b"text/plain".as_slice());
        }

        #[test]
        fn does_not_attach_missing_header_detail_to_later_failures() {
            assert_that!(|| {
                assert_that!(ok_response())
                    .with_location(false)
                    .get_header("content-type")
                    .is_ascii_satisfying(|s| {
                        s.is_equal_to("nope");
                    });
            })
            .panics()
            .has_message()
            .is_equal_to(indoc::formatdoc! {r#"
                -------- assertr --------
                Expected: "nope"

                  Actual: "text/plain"
                -------- assertr --------
            "#});
        }
    }

    mod get_text {
        use indoc::formatdoc;

        use super::{block_on, failing_response, ok_response, response};
        use crate::prelude::*;

        #[test]
        fn caller_location_is_as_expected() {
            assert_caller_location!(async assert_that_owned!(failing_response()), get_text());
        }

        #[tokio::test]
        async fn extracts_the_body_as_one_assertion() {
            let parent = assert_that!(());
            parent
                .derive_owned(|()| ok_response())
                .get_text()
                .await
                .is_equal_to("world");
            assert_that!(parent.state.records.assertion_count()).is_equal_to(1);

            assert_that_owned!(response(204, &[], ""))
                .get_text()
                .await
                .is_equal_to("");
        }

        #[test]
        fn future_is_send() {
            fn require_send<F: Future + Send>(future: F) -> F {
                future
            }
            drop(require_send(assert_that_owned!(ok_response()).get_text()));
        }

        #[test]
        fn panics_synchronously_when_the_response_is_only_borrowed() {
            assert_that!(|| {
                let response = ok_response();
                drop(assert_that!(response).get_text());
            }).panics()
            .has_type::<&str>()
            .is_equal_to(
                "get_text() consumes the response and can only be called on an owned Response! Create the assertion with `assert_that_owned!(...)` (or `.must_owned()`) instead.",
            );
        }

        #[test]
        fn panics_with_the_url_and_error_when_the_body_cannot_be_read() {
            assert_that!(|| {
                block_on(async {
                    assert_that_owned!(failing_response())
                        .with_location(false)
                        .get_text()
                        .await;
                });
            })
            .panics()
            .has_message()
            .is_equal_to(formatdoc! {r#"
                -------- assertr --------
                Expression: `failing_response()`

                has a body that could not be read

                Details:
                  - URL: "http://localhost/failing"
                  - Error: reqwest::Error {{
                        kind: Decode,
                        url: "http://localhost/failing",
                        source: reqwest::Error {{
                            kind: Body,
                            source: Custom {{
                                kind: Other,
                                error: "body read failed",
                            }},
                        }},
                    }}
                -------- assertr --------
            "#});
        }
    }

    #[cfg(feature = "serde-json")]
    mod get_json {
        use super::{Person, block_on, json_response};
        use crate::prelude::*;

        #[test]
        fn caller_location_is_as_expected() {
            assert_caller_location!(async assert_that_owned!(json_response("not json")), get_json::<Person>());
        }

        #[tokio::test]
        async fn extracts_the_deserialized_body_as_one_assertion() {
            let parent = assert_that!(());
            parent
                .derive_owned(|()| json_response(r#"{"name":"Bob","age":42}"#))
                .get_json::<Person>()
                .await
                .is_equal_to(Person {
                    name: "Bob".to_owned(),
                    age: 42,
                });
            assert_that!(parent.state.records.assertion_count()).is_equal_to(1);
        }

        #[test]
        fn future_is_send() {
            fn require_send<F: Future + Send>(future: F) -> F {
                future
            }
            drop(require_send(
                assert_that_owned!(json_response("1")).get_json::<u32>(),
            ));
        }

        #[test]
        fn deserialization_runs_once_for_success_and_rejection() {
            use core::sync::atomic::{AtomicUsize, Ordering};

            use serde::de::Error;

            static CALLS: AtomicUsize = AtomicUsize::new(0);
            struct Decoded(u32);

            impl<'de> serde::Deserialize<'de> for Decoded {
                fn deserialize<D: serde::Deserializer<'de>>(
                    deserializer: D,
                ) -> Result<Self, D::Error> {
                    CALLS.fetch_add(1, Ordering::Relaxed);
                    let value = u32::deserialize(deserializer)?;
                    if value == 0 {
                        Err(D::Error::custom("rejected zero"))
                    } else {
                        Ok(Self(value))
                    }
                }
            }

            let assertion = block_on(assert_that_owned!(json_response("7")).get_json::<Decoded>());
            assert_that!(assertion.actual().0).is_equal_to(7);
            assert_that!(CALLS.load(Ordering::Relaxed)).is_equal_to(1);

            assert_that!(|| {
                block_on(assert_that_owned!(json_response("0")).get_json::<Decoded>());
            })
            .panics()
            .has_message()
            .contains("rejected zero");
            assert_that!(CALLS.load(Ordering::Relaxed)).is_equal_to(2);
        }

        #[test]
        fn panics_with_the_body_when_it_is_not_valid_json() {
            let body = "not json";
            let expected_type = core::any::type_name::<Person>();

            assert_that!(|| {
                block_on(async {
                    assert_that_owned!(json_response(body))
                        .with_location(false)
                        .get_json::<Person>()
                        .await;
                });
            })
            .panics()
            .has_message()
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
        fn panics_synchronously_when_the_response_is_only_borrowed() {
            assert_that!(|| {
                let response = json_response(r#"{"name":"Bob","age":42}"#);
                drop(assert_that!(response).get_json::<Person>());
            }).panics()
            .has_type::<&str>()
            .is_equal_to(
                "get_json() consumes the response and can only be called on an owned Response! Create the assertion with `assert_that_owned!(...)` (or `.must_owned()`) instead.",
            );
        }
    }
}
