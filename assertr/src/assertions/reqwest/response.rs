//! Assertions for `reqwest::Response`.
//!
//! Assertions cover status codes and headers. Projections expose one header, the text body, or a
//! JSON body. Failures include the request URL.
//!
//! Reading a body consumes the response. `get_text()` and `get_json()` are async and require
//! `assert_that_owned!` or `.must_owned()`.
#![cfg_attr(
    feature = "http",
    doc = "\nWith the `http` feature enabled, the value extracted by `get_header` composes with
[`HttpHeaderValueAssertions`](crate::prelude::HttpHeaderValueAssertions): `reqwest` re-exports
`http`'s header types, so the two integrations meet on the same `HeaderValue`."
)]
#![cfg_attr(
    not(feature = "http"),
    doc = "\nWith the `http` feature enabled, the value extracted by `get_header` composes with
`HttpHeaderValueAssertions`: `reqwest` re-exports `http`'s header types, so the two integrations
meet on the same `HeaderValue`."
)]

use crate::{
    AssertThat, DebugRenderer, ValueRenderer,
    failure::{Fact, FailureKind},
    mode::{Mode, Panic},
    renderer::{GroupStyle, IntoRendered, Rendered, RenderingContext, SensitiveValuePolicy},
};
use crate::{AssertionContext, Expectation, ExpectationDiagnostics, failure::FailureBuilder};
use alloc::{borrow::ToOwned, string::String, vec::Vec};
use reqwest::header::HeaderValue;

/// Compares the observed response status code.
pub struct HasStatusCode(reqwest::StatusCode);
impl<R> Expectation<reqwest::Response, R> for HasStatusCode {
    type Success<'a>
        = ()
    where
        Self: 'a,
        reqwest::Response: 'a;
    type Rejection<'a>
        = reqwest::StatusCode
    where
        Self: 'a,
        reqwest::Response: 'a;
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
}
impl<R> ExpectationDiagnostics<reqwest::Response, R> for HasStatusCode
where
    R: ValueRenderer<reqwest::StatusCode> + ValueRenderer<str>,
{
    const KIND: FailureKind = FailureKind::Equality;
    fn explain<'a, Target>(
        &'a self,
        rejected: Option<(&'a reqwest::Response, Self::Rejection<'a>)>,
        failure: FailureBuilder<Target>,
        context: &AssertionContext<'_, R>,
    ) -> FailureBuilder<Target> {
        let render = context.render();
        match rejected {
            None => failure
                .relation("has status code")
                .expected(render.value(&self.0)),
            Some((actual, status)) => failure
                .actual(render.value(&status))
                .expected(render.value(&self.0))
                .fact(Fact::labelled(URL, render.value(actual.url().as_str()))),
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
        pub struct $name;
        impl<R> Expectation<reqwest::Response, R> for $name {
            type Success<'a>
                = ()
            where
                Self: 'a,
                reqwest::Response: 'a;
            type Rejection<'a>
                = reqwest::StatusCode
            where
                Self: 'a,
                reqwest::Response: 'a;
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
        }
        impl<R> ExpectationDiagnostics<reqwest::Response, R> for $name
        where
            R: ValueRenderer<reqwest::StatusCode> + ValueRenderer<str>,
        {
            const KIND: FailureKind = FailureKind::Other;
            fn explain<'a, Target>(
                &'a self,
                rejected: Option<(&'a reqwest::Response, Self::Rejection<'a>)>,
                failure: FailureBuilder<Target>,
                context: &AssertionContext<'_, R>,
            ) -> FailureBuilder<Target> {
                let render = context.render();
                match rejected {
                    None => failure.relation($met).expected($class),
                    Some((actual, status)) => failure
                        .actual(render.value(&status))
                        .relation($unmet)
                        .expected($class)
                        .fact(Fact::labelled(URL, render.value(actual.url().as_str()))),
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

/// Checks that the response contains a header, returning its first value.
///
/// Rejection retains the looked-up name. Diagnostics list the present header names.
pub struct HasHeader<E>(E);
impl<E, R> Expectation<reqwest::Response, R> for HasHeader<E>
where
    E: AsRef<str>,
{
    type Success<'a>
        = &'a HeaderValue
    where
        Self: 'a,
        reqwest::Response: 'a;
    type Rejection<'a>
        = &'a str
    where
        Self: 'a,
        reqwest::Response: 'a;
    fn evaluate<'a>(
        &'a self,
        actual: &'a reqwest::Response,
        _context: &AssertionContext<'_, R>,
    ) -> Result<Self::Success<'a>, Self::Rejection<'a>> {
        let name = self.0.as_ref();
        match actual.headers().get(name) {
            Some(value) => Ok(value),
            None => Err(name),
        }
    }
}
impl<E, R> ExpectationDiagnostics<reqwest::Response, R> for HasHeader<E>
where
    E: AsRef<str>,
    R: ValueRenderer<str>,
{
    const KIND: FailureKind = FailureKind::Membership;
    fn explain<'a, Target>(
        &'a self,
        rejected: Option<(&'a reqwest::Response, Self::Rejection<'a>)>,
        failure: FailureBuilder<Target>,
        context: &AssertionContext<'_, R>,
    ) -> FailureBuilder<Target> {
        let render = context.render();
        match rejected {
            None => failure
                .relation("contains the header")
                .expected(render.value(self.0.as_ref())),
            Some((actual, name)) => failure
                .actual(render.borrowed_values::<str, _>(&header_names(actual), GroupStyle::List))
                .relation("does not contain the header")
                .expected(render.value(name))
                .fact(Fact::labelled(URL, render.value(actual.url().as_str()))),
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
/// Rejection retains the looked-up name and the header's first value.
pub struct DoesNotHaveHeader<E>(E);
impl<E, R> Expectation<reqwest::Response, R> for DoesNotHaveHeader<E>
where
    E: AsRef<str>,
{
    type Success<'a>
        = ()
    where
        Self: 'a,
        reqwest::Response: 'a;
    type Rejection<'a>
        = (&'a str, &'a HeaderValue)
    where
        Self: 'a,
        reqwest::Response: 'a;
    fn evaluate<'a>(
        &'a self,
        actual: &'a reqwest::Response,
        _context: &AssertionContext<'_, R>,
    ) -> Result<Self::Success<'a>, Self::Rejection<'a>> {
        let name = self.0.as_ref();
        match actual.headers().get(name) {
            Some(value) => Err((name, value)),
            None => Ok(()),
        }
    }
}
impl<E, R> ExpectationDiagnostics<reqwest::Response, R> for DoesNotHaveHeader<E>
where
    E: AsRef<str>,
    R: ValueRenderer<str> + ValueRenderer<HeaderValue>,
{
    const KIND: FailureKind = FailureKind::Membership;
    fn explain<'a, Target>(
        &'a self,
        rejected: Option<(&'a reqwest::Response, Self::Rejection<'a>)>,
        failure: FailureBuilder<Target>,
        context: &AssertionContext<'_, R>,
    ) -> FailureBuilder<Target> {
        let render = context.render();
        match rejected {
            None => failure
                .relation("does not contain the header")
                .unexpected(render.value(self.0.as_ref())),
            Some((actual, (name, value))) => failure
                .actual(render.borrowed_values::<str, _>(&header_names(actual), GroupStyle::List))
                .relation("contains the header")
                .unexpected(render.value(name))
                .fact(Fact::labelled(URL, render.value(actual.url().as_str())))
                .fact(Fact::labelled("Value", render_header(render, value))),
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
pub struct HasHeaderValue<N, E> {
    name: N,
    expected: E,
}
impl<N, E, R> Expectation<reqwest::Response, R> for HasHeaderValue<N, E>
where
    N: AsRef<str>,
    E: AsRef<str>,
{
    type Success<'a>
        = ()
    where
        Self: 'a,
        reqwest::Response: 'a;
    type Rejection<'a>
        = (&'a str, &'a str, Option<&'a HeaderValue>)
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
        match actual.headers().get(name) {
            Some(value) if value.as_bytes() == expected.as_bytes() => Ok(()),
            value => Err((name, expected, value)),
        }
    }
}
impl<N, E, R> ExpectationDiagnostics<reqwest::Response, R> for HasHeaderValue<N, E>
where
    N: AsRef<str>,
    E: AsRef<str>,
    R: ValueRenderer<str> + ValueRenderer<HeaderValue>,
{
    const KIND: FailureKind = FailureKind::Equality;
    fn explain<'a, Target>(
        &'a self,
        rejected: Option<(&'a reqwest::Response, Self::Rejection<'a>)>,
        failure: FailureBuilder<Target>,
        context: &AssertionContext<'_, R>,
    ) -> FailureBuilder<Target> {
        let render = context.render();
        match rejected {
            None => failure
                .relation("contains the header")
                .expected(render.value(self.name.as_ref()))
                .fact(Fact::labelled(
                    "Expected value",
                    render.value(self.expected.as_ref()),
                )),
            Some((actual, (name, expected, value))) => {
                let failure =
                    failure.fact(Fact::labelled(URL, render.value(actual.url().as_str())));
                match value {
                    None => failure
                        .actual(
                            render
                                .borrowed_values::<str, _>(&header_names(actual), GroupStyle::List),
                        )
                        .relation("does not contain the header")
                        .expected(render.value(name))
                        .fact(Fact::labelled("Expected value", render.value(expected))),
                    Some(value) => failure
                        .actual(render_header(render, value))
                        .expected(render.value(expected))
                        .fact(Fact::labelled("Header", render.value(name))),
                }
            }
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
        self.apply_assertion(HasStatusCode::new(expected))
    }

    #[track_caller]
    fn is_informational(self) -> Self
    where
        R: ValueRenderer<reqwest::StatusCode> + ValueRenderer<str>,
    {
        self.apply_assertion(IsInformational)
    }

    #[track_caller]
    fn is_success(self) -> Self
    where
        R: ValueRenderer<reqwest::StatusCode> + ValueRenderer<str>,
    {
        self.apply_assertion(IsSuccess)
    }

    #[track_caller]
    fn is_redirection(self) -> Self
    where
        R: ValueRenderer<reqwest::StatusCode> + ValueRenderer<str>,
    {
        self.apply_assertion(IsRedirection)
    }

    #[track_caller]
    fn is_client_error(self) -> Self
    where
        R: ValueRenderer<reqwest::StatusCode> + ValueRenderer<str>,
    {
        self.apply_assertion(IsClientError)
    }

    #[track_caller]
    fn is_server_error(self) -> Self
    where
        R: ValueRenderer<reqwest::StatusCode> + ValueRenderer<str>,
    {
        self.apply_assertion(IsServerError)
    }

    #[track_caller]
    fn has_header(self, name: impl AsRef<str>) -> Self
    where
        R: ValueRenderer<str>,
    {
        self.apply_assertion(HasHeader::new(name))
    }

    #[track_caller]
    fn does_not_have_header(self, name: impl AsRef<str>) -> Self
    where
        R: ValueRenderer<HeaderValue> + ValueRenderer<str>,
    {
        self.apply_assertion(DoesNotHaveHeader::new(name))
    }

    #[track_caller]
    fn has_header_value(self, name: impl AsRef<str>, expected: impl AsRef<str>) -> Self
    where
        R: ValueRenderer<HeaderValue> + ValueRenderer<str>,
    {
        self.apply_assertion(HasHeaderValue::new(name, expected))
    }
}

/// Panic-mode projections from [`reqwest::Response`].
///
/// Only available in `Panic` mode. Each projection can fail to produce a value, and a captured
/// failure has no value to continue the chain with.
#[cfg_attr(feature = "fluent", assertr_macros::fluent_aliases)]
pub trait ReqwestResponseExtractAssertions<'t, R = DebugRenderer> {
    /// Asserts that the header is present, then continues the chain on a clone of its first value.
    #[cfg_attr(
        feature = "http",
        doc = "\nWith the `http` feature enabled, the extracted `HeaderValue` is the subject of
[`HttpHeaderValueAssertions`](crate::prelude::HttpHeaderValueAssertions), so
`.get_header(\"content-type\").is_ascii_satisfying(..)` works across both integrations."
    )]
    #[cfg_attr(
        not(feature = "http"),
        doc = "\nWith the `http` feature enabled, the extracted `HeaderValue` is the subject of
`HttpHeaderValueAssertions`, so `.get_header(\"content-type\").is_ascii_satisfying(..)` works
across both integrations."
    )]
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
        let value = self
            .test_assertion(&definition)
            .expect("Panic mode raises missing headers")
            .clone();
        self.map(|_| value.into())
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
        async move { get_text_at(self, location, &url).await }
    }

    #[track_caller]
    #[cfg(feature = "serde-json")]
    fn get_json<T>(self) -> impl Future<Output = AssertThat<'t, T, Panic, R>>
    where
        T: serde::de::DeserializeOwned + 't,
        R: ValueRenderer<str> + ValueRenderer<reqwest::Error> + ValueRenderer<serde_json::Error>,
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

            let this = get_text_at(self, location, &url).await;

            let definition = JsonBody::<T> {
                url: &url,
                output: core::marker::PhantomData,
            };
            let parsed = this
                .test_observation_after_tracking(this.actual(), &definition, location)
                .expect("Panic mode raises invalid JSON");
            this.map(|_| Actual::Owned(parsed))
        }
    }
}

// A body read is awaited once by the adapter. Its original result and URL remain available
// until explanation finishes, without requiring a response or body renderer.
struct ReadableBody<'u> {
    url: &'u str,
}

impl<R> Expectation<Result<String, reqwest::Error>, R> for ReadableBody<'_> {
    type Success<'a>
        = &'a String
    where
        Self: 'a;
    type Rejection<'a>
        = &'a reqwest::Error
    where
        Self: 'a;

    fn evaluate<'a>(
        &'a self,
        actual: &'a Result<String, reqwest::Error>,
        context: &AssertionContext<'_, R>,
    ) -> Result<Self::Success<'a>, Self::Rejection<'a>> {
        crate::assertions::core::result::IsOk.evaluate(actual, context)
    }
}

impl<R> ExpectationDiagnostics<Result<String, reqwest::Error>, R> for ReadableBody<'_>
where
    R: ValueRenderer<str> + ValueRenderer<reqwest::Error>,
{
    const KIND: FailureKind = FailureKind::Other;

    fn explain<'a, Target>(
        &'a self,
        rejected: Option<(&'a Result<String, reqwest::Error>, Self::Rejection<'a>)>,
        failure: FailureBuilder<Target>,
        context: &AssertionContext<'_, R>,
    ) -> FailureBuilder<Target> {
        let render = context.render();
        let failure = failure.fact(Fact::labelled(URL, render.value(self.url)));
        match rejected {
            None => failure.relation("has a readable body"),
            Some((_, error)) => failure
                .relation("has a body that could not be read")
                .fact(Fact::labelled("Error", render.value(error))),
        }
    }
}

#[cfg(feature = "serde-json")]
struct JsonBody<'u, T> {
    url: &'u str,
    output: core::marker::PhantomData<fn() -> T>,
}

#[cfg(feature = "serde-json")]
impl<T: serde::de::DeserializeOwned, R> Expectation<String, R> for JsonBody<'_, T> {
    type Success<'a>
        = T
    where
        Self: 'a;
    type Rejection<'a>
        = serde_json::Error
    where
        Self: 'a;

    fn evaluate<'a>(
        &'a self,
        actual: &'a String,
        _: &AssertionContext<'_, R>,
    ) -> Result<Self::Success<'a>, Self::Rejection<'a>> {
        serde_json::from_str(actual.as_str())
    }
}

#[cfg(feature = "serde-json")]
impl<T, R> ExpectationDiagnostics<String, R> for JsonBody<'_, T>
where
    T: serde::de::DeserializeOwned,
    R: ValueRenderer<str> + ValueRenderer<serde_json::Error>,
{
    const KIND: FailureKind = FailureKind::Other;

    fn explain<'a, Target>(
        &'a self,
        rejected: Option<(&'a String, Self::Rejection<'a>)>,
        failure: FailureBuilder<Target>,
        context: &AssertionContext<'_, R>,
    ) -> FailureBuilder<Target> {
        let render = context.render();
        let failure = failure
            .fact(Fact::labelled(URL, render.value(self.url)))
            .fact(Fact::labelled("Expected type", core::any::type_name::<T>()));
        match rejected {
            None => failure.relation("is valid JSON for the expected type"),
            Some((actual, error)) => failure
                .actual(render.value(actual.as_str()))
                .relation("is not valid JSON for the expected type")
                .fact(Fact::labelled("Error", render.value(&error))),
        }
    }
}

async fn get_text_at<'t, R>(
    assertion: AssertThat<'t, reqwest::Response, Panic, R>,
    location: &'static core::panic::Location<'static>,
    url: &str,
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

    this.apply_assertion_after_tracking_at(ReadableBody { url }, location)
        .map(|it| Actual::Owned(it.unwrap_owned().expect("already checked")))
}

/// The label of the fact carrying the request URL, the one piece of evidence that tells two
/// responses apart.
const URL: &str = "URL";

/// The names of all present headers in their iteration order.
fn header_names(actual: &reqwest::Response) -> Vec<&str> {
    actual
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

#[cfg(test)]
mod tests {
    use crate::prelude::*;
    mod renderer_contract {
        use super::response;
        use crate::{
            prelude::*,
            test_support::{NoRenderer, assert_trait_impl},
        };

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

    use core::{
        pin::Pin,
        task::{Context, Poll},
    };

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

    /// Redacts strings and sensitive header values, showing other header values.
    struct SensitivityAwareRenderer;

    struct RevealingRenderer<'a> {
        original: &'a reqwest::header::HeaderValue,
        calls: &'a core::cell::Cell<usize>,
    }

    impl ValueRenderer<reqwest::header::HeaderValue> for RevealingRenderer<'_> {
        fn fmt(
            &self,
            value: &reqwest::header::HeaderValue,
            f: &mut core::fmt::Formatter<'_>,
        ) -> core::fmt::Result {
            assert_that!(value.is_sensitive()).is_false();
            assert_that!(value.as_bytes()).is_equal_to(self.original.as_bytes());
            assert_that!(core::ptr::eq(value, self.original))
                .is_equal_to(!self.original.is_sensitive());
            self.calls.set(self.calls.get() + 1);
            write!(f, "revealed({value:?})")
        }

        fn sensitive_value_policy(&self) -> crate::renderer::SensitiveValuePolicy {
            crate::renderer::SensitiveValuePolicy::Reveal
        }
    }

    impl ValueRenderer<str> for RevealingRenderer<'_> {
        fn fmt(&self, value: &str, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            ValueRenderer::fmt(&DebugRenderer, value, f)
        }
    }

    /// Renders only strings, so tests can show which failures need no other renderer.
    struct TextOnly;

    impl ValueRenderer<str> for TextOnly {
        fn fmt(&self, _: &str, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            f.write_str("<redacted text>")
        }
    }

    impl ValueRenderer<reqwest::header::HeaderValue> for SensitivityAwareRenderer {
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

    impl ValueRenderer<str> for SensitivityAwareRenderer {
        fn fmt(&self, value: &str, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            ValueRenderer::fmt(&TextOnly, value, f)
        }
    }

    /// Drives a future that must run outside an async test, for example inside a panic probe.
    fn block_on<F: Future>(future: F) -> F::Output {
        tokio::runtime::Builder::new_current_thread()
            .build()
            .expect("runtime")
            .block_on(future)
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
        fn caller_location_is_as_expected() {
            assert_caller_location!(
                assert_that!(response(404, &[], "")),
                has_status_code(reqwest::StatusCode::OK)
            );
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
            assert_that!(failures).contains_exactly_satisfying([
                |element: AssertThat<AssertionFailure, Capture>| {
                    element.derive(|item| item).has_text_report(formatdoc! {r#"
                -------- assertr --------
                Expression: `subject`

                Expected: custom(200)

                  Actual: custom(404)

                Details:
                  - URL: custom("http://localhost/hello")
                -------- assertr --------
            "#});

                    assert_custom_value(
                        element.actual().actual.as_ref().unwrap(),
                        &subject.status(),
                    );
                    assert_custom_value(&element.actual().facts[0].value, subject.url().as_str());
                    assert_custom_value(
                        element.actual().expected.as_ref().unwrap(),
                        &reqwest::StatusCode::OK,
                    );
                },
            ]);
            let failures = assert_that!(subject)
                .with_renderer(RedactingRenderer)
                .with_location(false)
                .capture(|it| it.has_status_code(reqwest::StatusCode::OK));
            assert_that!(failures).contains_exactly_satisfying([
                |element: AssertThat<AssertionFailure, Capture>| {
                    element.derive(|value| value).has_text_report(formatdoc! {r"
                -------- assertr --------
                Expression: `subject`

                Expected: <redacted>

                  Actual: <redacted>

                Details:
                  - URL: <redacted>
                -------- assertr --------
            "});

                    assert_redacted(element.actual(), &["localhost/hello", "404"]);
                },
            ]);
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

        #[test]
        #[cfg(feature = "fluent")]
        fn fluent_alias_is_as_expected() {
            response(100, &[], "").must().be_informational();
        }

        #[test]
        fn caller_location_is_as_expected() {
            assert_caller_location!(assert_that!(response(200, &[], "")), is_informational());
        }
    }

    mod is_success {
        use super::response;
        use crate::prelude::*;

        #[test]
        #[cfg(feature = "fluent")]
        fn fluent_alias_is_as_expected() {
            response(200, &[], "").must().be_success();
        }

        #[test]
        fn caller_location_is_as_expected() {
            assert_caller_location!(assert_that!(response(500, &[], "")), is_success());
        }
    }

    mod is_redirection {
        use super::response;
        use crate::prelude::*;

        #[test]
        #[cfg(feature = "fluent")]
        fn fluent_alias_is_as_expected() {
            response(301, &[], "").must().be_redirection();
        }

        #[test]
        fn caller_location_is_as_expected() {
            assert_caller_location!(assert_that!(response(200, &[], "")), is_redirection());
        }
    }

    mod is_client_error {
        use super::response;
        use crate::prelude::*;

        #[test]
        #[cfg(feature = "fluent")]
        fn fluent_alias_is_as_expected() {
            response(404, &[], "").must().be_client_error();
        }

        #[test]
        fn caller_location_is_as_expected() {
            assert_caller_location!(assert_that!(response(500, &[], "")), is_client_error());
        }
    }

    mod is_server_error {
        use super::response;
        use crate::prelude::*;

        #[test]
        #[cfg(feature = "fluent")]
        fn fluent_alias_is_as_expected() {
            response(500, &[], "").must().be_server_error();
        }

        #[test]
        fn caller_location_is_as_expected() {
            assert_caller_location!(assert_that!(response(404, &[], "")), is_server_error());
        }
    }

    mod status_classes {
        use super::response;
        use crate::prelude::*;
        use crate::test_support::{
            CustomValueRenderer, RedactingRenderer, assert_custom_value, assert_redacted,
        };
        use crate::{Mode, ValueRenderer};
        use indoc::formatdoc;

        #[derive(Clone, Copy)]
        enum Class {
            Informational,
            Success,
            Redirection,
            ClientError,
            ServerError,
        }

        impl Class {
            fn check<M: Mode, R>(
                self,
                it: AssertThat<'_, reqwest::Response, M, R>,
            ) -> AssertThat<'_, reqwest::Response, M, R>
            where
                R: ValueRenderer<reqwest::StatusCode> + ValueRenderer<str>,
            {
                match self {
                    Self::Informational => it.is_informational(),
                    Self::Success => it.is_success(),
                    Self::Redirection => it.is_redirection(),
                    Self::ClientError => it.is_client_error(),
                    Self::ServerError => it.is_server_error(),
                }
            }
        }

        struct Case {
            class: Class,
            members: &'static [u16],
            outsider: u16,
            relation: &'static str,
            label: &'static str,
        }

        const CASES: [Case; 5] = [
            Case {
                class: Class::Informational,
                members: &[100, 103],
                outsider: 200,
                relation: "is not informational",
                label: "1xx",
            },
            Case {
                class: Class::Success,
                members: &[200, 204, 299],
                outsider: 500,
                relation: "is not a success",
                label: "2xx",
            },
            Case {
                class: Class::Redirection,
                members: &[301, 308],
                outsider: 200,
                relation: "is not a redirection",
                label: "3xx",
            },
            Case {
                class: Class::ClientError,
                members: &[400, 451],
                outsider: 500,
                relation: "is not a client error",
                label: "4xx",
            },
            Case {
                class: Class::ServerError,
                members: &[500, 503],
                outsider: 404,
                relation: "is not a server error",
                label: "5xx",
            },
        ];

        #[test]
        fn accept_the_class_and_report_other_statuses_with_the_url() {
            for case in CASES {
                for status in case.members {
                    case.class.check(assert_that!(response(*status, &[], "")));
                }

                let subject = response(case.outsider, &[], "");
                let status = subject.status();
                let failures = assert_that!(subject).with_location(false).capture(|it| {
                    // The chain continues after the failed class check.
                    case.class.check(it).has_status_code(status)
                });
                assert_that!(failures).contains_exactly_satisfying([
                    |it: AssertThat<AssertionFailure, Capture>| {
                        it.has_text_report(formatdoc! {r#"
                            -------- assertr --------
                            Expression: `subject`

                            Actual: {outsider}

                            {relation}

                            Expected: {label}

                            Details:
                              - URL: "http://localhost/hello"
                            -------- assertr --------
                        "#, outsider = case.outsider, relation = case.relation, label = case.label});
                    },
                ]);
            }
        }

        #[test]
        fn render_status_and_url_evidence_through_the_active_renderer() {
            for case in CASES {
                let subject = response(case.outsider, &[], "");
                let failures = assert_that!(subject)
                    .with_renderer(CustomValueRenderer)
                    .with_location(false)
                    .capture(|it| case.class.check(it));
                assert_that!(failures).contains_exactly_satisfying([
                    |element: AssertThat<AssertionFailure, Capture>| {
                        element.derive(|item| item).has_text_report(formatdoc! {r#"
                            -------- assertr --------
                            Expression: `subject`

                            Actual: custom({outsider})

                            {relation}

                            Expected: {label}

                            Details:
                              - URL: custom("http://localhost/hello")
                            -------- assertr --------
                        "#, outsider = case.outsider, relation = case.relation, label = case.label});
                        assert_custom_value(
                            element.actual().actual.as_ref().unwrap(),
                            &subject.status(),
                        );
                        assert_custom_value(
                            &element.actual().facts[0].value,
                            subject.url().as_str(),
                        );
                    },
                ]);

                let failures = assert_that!(subject)
                    .with_renderer(RedactingRenderer)
                    .with_location(false)
                    .capture(|it| case.class.check(it));
                assert_that!(failures).contains_exactly_satisfying([
                    |element: AssertThat<AssertionFailure, Capture>| {
                        element.derive(|value| value).has_text_report(formatdoc! {r"
                            -------- assertr --------
                            Expression: `subject`

                            Actual: <redacted>

                            {relation}

                            Expected: {label}

                            Details:
                              - URL: <redacted>
                            -------- assertr --------
                        ", relation = case.relation, label = case.label});
                        assert_redacted(
                            element.actual(),
                            &["localhost/hello", &case.outsider.to_string()],
                        );
                    },
                ]);
            }
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
        fn caller_location_is_as_expected() {
            assert_caller_location!(
                assert_that!(response(200, &[("x-api-key", "1234")], "")),
                has_header("content-type")
            );
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

            assert_that!(failures).contains_exactly_satisfying([
                |element: AssertThat<AssertionFailure, Capture>| {
                    element.derive(|value| value).has_text_report(formatdoc! {r"
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
                },
            ]);
        }

        #[test]
        fn applies_the_rendering_budget_to_header_names_and_strings() {
            let response = response(200, &[("x-first", "one"), ("x-second", "two")], "");
            let failures = assert_that!(response)
                .with_renderer(TextOnly)
                .with_location(false)
                .with_rendering_budget(
                    RenderingBudget::default()
                        .with_max_items(1)
                        .with_max_leaf_characters(4),
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
        use super::{
            RevealingRenderer, SensitivityAwareRenderer, header_response, ok_response, response,
        };
        use crate::prelude::*;
        use indoc::formatdoc;

        #[test]
        #[cfg(feature = "fluent")]
        fn fluent_alias_is_as_expected() {
            ok_response().must().not_have_header("x-api-key");
        }

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
        fn shows_sensitive_header_contents_by_default() {
            let response = header_response(b"secret-\xff", true);
            let failures = assert_that!(response)
                .with_location(false)
                .capture(|it| it.does_not_have_header("x-api-key"));

            assert_that!(failures).contains_exactly_satisfying([
                |element: AssertThat<AssertionFailure, Capture>| {
                    element
                        .derive(|value| value)
                        .has_text_report(formatdoc! {r#"
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
                },
            ]);
            assert_that!(response.headers()["x-api-key"].is_sensitive()).is_true();
        }

        #[test]
        fn respects_custom_redaction_and_preserves_header_metadata() {
            let response = header_response(b"secret-\xff", true);
            let failures = assert_that!(response)
                .with_renderer(SensitivityAwareRenderer)
                .with_location(false)
                .capture(|it| it.does_not_have_header("x-api-key"));

            assert_that!(failures).contains_exactly_satisfying([
                |element: AssertThat<AssertionFailure, Capture>| {
                    element.derive(|value| value).has_text_report(formatdoc! {r"
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
                },
            ]);
            let value = &failures[0].facts[1].value;
            assert_that!(value.type_name)
                .is_equal_to(Some(core::any::type_name::<reqwest::header::HeaderValue>()));
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

            assert_that!(failures).contains_exactly_satisfying([
                |element: AssertThat<AssertionFailure, Capture>| {
                    element
                        .derive_owned(|item| rendered_text(&item.facts[1].value))
                        .is_equal_to(r#"revealed("secret-\xff")"#);
                },
            ]);
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
        use super::{
            RevealingRenderer, SensitivityAwareRenderer, header_response, ok_response, response,
        };
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
        fn caller_location_is_as_expected() {
            assert_caller_location!(
                assert_that!(ok_response()),
                has_header_value("content-type", "application/json")
            );
        }

        #[test]
        fn succeeds_when_the_value_matches() {
            assert_that!(ok_response()).has_header_value("content-type", "text/plain");
        }

        #[test]
        fn sensitivity_does_not_change_the_comparison() {
            let response = header_response(b"secret", true);
            assert_that!(response)
                .with_renderer(SensitivityAwareRenderer)
                .has_header_value("x-api-key", "secret");
        }

        #[test]
        fn compares_raw_bytes_and_escapes_sensitive_contents_by_default() {
            let response = header_response(b"secret-\xff", true);
            let failures = assert_that!(response)
                .with_location(false)
                .capture(|it| it.has_header_value("x-api-key", "secret-�"));

            assert_that!(failures).contains_exactly_satisfying([
                |element: AssertThat<AssertionFailure, Capture>| {
                    element
                        .derive(|value| value)
                        .has_text_report(formatdoc! {r#"
                -------- assertr --------
                Expression: `response`

                Expected: "secret-�"

                  Actual: "secret-\xff"

                Details:
                  - URL: "http://localhost/hello"
                  - Header: "x-api-key"
                -------- assertr --------
            "#});
                },
            ]);
            assert_that!(response.headers()["x-api-key"].is_sensitive()).is_true();
        }

        #[test]
        fn respects_custom_redaction_and_preserves_header_metadata() {
            let response = header_response(b"secret-\xff", true);
            let failures = assert_that!(response)
                .with_renderer(SensitivityAwareRenderer)
                .with_location(false)
                .capture(|it| it.has_header_value("x-api-key", "another secret"));

            assert_that!(failures).contains_exactly_satisfying([
                |element: AssertThat<AssertionFailure, Capture>| {
                    element.derive(|value| value).has_text_report(formatdoc! {r"
                -------- assertr --------
                Expression: `response`

                Expected: <redacted text>

                  Actual: <redacted header>

                Details:
                  - URL: <redacted text>
                  - Header: <redacted text>
                -------- assertr --------
            "});
                    element
                        .derive_owned(|value| {
                            value.actual.as_ref().expect("actual value").type_name
                        })
                        .is_equal_to(Some(core::any::type_name::<reqwest::header::HeaderValue>()));
                },
            ]);
            assert_that!(response.headers()["x-api-key"].is_sensitive()).is_true();
        }

        #[test]
        fn renders_expected_values_when_the_header_is_missing() {
            let response = header_response(b"secret", true);
            let failures = assert_that!(response)
                .with_renderer(SensitivityAwareRenderer)
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
                .with_renderer(SensitivityAwareRenderer)
                .capture(|it| it.has_header_value("x-api-key", "other"));

            let actual = failures[0].actual.as_ref().expect("actual value");
            assert_that!(rendered_text(actual)).is_equal_to(r#"header("visible-\xff")"#);
        }

        #[test]
        fn applies_the_rendering_budget_to_custom_header_output() {
            let response = header_response(b"secret", true);
            let failures = assert_that!(response)
                .with_renderer(SensitivityAwareRenderer)
                .with_rendering_budget(RenderingBudget::default().with_max_leaf_characters(4))
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

                assert_that!(failures).contains_exactly_satisfying([
                    |failure: AssertThat<AssertionFailure, Capture>| {
                        failure
                            .derive(|failure| &failure.actual)
                            .is_some_satisfying(|actual| {
                                actual
                                    .derive_owned(rendered_text)
                                    .is_equal_to(r#"revealed("secret-\xff")"#);
                                actual.derive(|actual| &actual.type_name).is_equal_to(Some(
                                    core::any::type_name::<reqwest::header::HeaderValue>(),
                                ));
                            });
                    },
                ]);
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
                .with_rendering_budget(RenderingBudget::default().with_max_leaf_characters(4))
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
        fn caller_location_is_as_expected() {
            assert_caller_location!(assert_that!(ok_response()), get_header("missing-header"));
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

            assert_that!(assertion.actual().as_bytes()).is_equal_to(b"text/plain".as_slice());
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

            assert_that!(assertion.state.records.assertion_count()).is_equal_to(1);
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
        use super::{block_on, failing_response, ok_response, response};
        use crate::prelude::*;

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
        fn caller_location_is_as_expected() {
            assert_caller_location!(async assert_that_owned!(failing_response()), get_text());
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

            assert_that!(assertion.state.records.assertion_count()).is_equal_to(1);
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
        use super::{block_on, response};
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
        fn caller_location_is_as_expected() {
            assert_caller_location!(async assert_that_owned!(json_response("not json")), get_json::<Person>());
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
            assert_that!(assertion.state.records.assertion_count()).is_equal_to(1);
            assert_that!(CALLS.load(Ordering::Relaxed)).is_equal_to(1);

            assert_that_panic_by(|| {
                block_on(assert_that_owned!(json_response("0")).get_json::<Decoded>());
            })
            .has_type::<String>()
            .contains("rejected zero");
            assert_that!(CALLS.load(Ordering::Relaxed)).is_equal_to(2);
        }

        #[test]
        fn body_errors_use_typed_renderers_and_can_be_redacted() {
            use indoc::formatdoc;

            use crate::test_support::{CustomValueRenderer, RedactingRenderer};
            let body = r#"{"name":"private-body-name","age":"private-body-age"}"#;
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

                Actual: custom("{{\"name\":\"private-body-name\",\"age\":\"private-body-age\"}}")

                is not valid JSON for the expected type

                Details:
                  - URL: custom("http://localhost/hello")
                  - Expected type: {expected_type}
                  - Error: custom(Error("invalid type: string \"private-body-age\", expected u32", line: 1, column: 52))
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

            assert_that!(assertion.state.records.assertion_count()).is_equal_to(1);
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
