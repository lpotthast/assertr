//! Async browser observations that continue as ordinary value assertions.

use alloc::{format, string::String};
use core::panic::Location;

use thirtyfour::{WebElement, error::WebDriverError, prelude::WebDriverResult};

use super::read;
use crate::{
    AssertThat,
    actual::Actual,
    assertions::{std::eventually::not_observed, support::project_checked},
    expectation::{AssertionContext, Expectation},
    failure::{Fact, FailureBuilder, FailureKind},
    mode::Panic,
    renderer::{DebugRenderer, ValueRenderer},
};

/// Browser-specific reads on a real thirtyfour element, followed by ordinary value assertions.
///
/// Each method reads the element once, then continues the chain on the value it observed, so
/// string, `Option`, and `bool` assertions apply. [`has_attribute`](Self::has_attribute) also
/// checks presence. Other projections keep optional values where the browser can report absence.
/// Protocol errors fail at the read call and never become absence, empty strings, or `false`.
/// Borrowed handles remain available for further reads and actions.
///
/// The reads require panic mode. Each one tracks one assertion at its call site and detaches the
/// chain before awaiting, so its future is `Send` when the renderer is. The continuation is a new
/// assertion chain holding the chain's diagnostic settings and messages, plus a message naming
/// the operation and the remote element. Finish it before the next suspension.
///
/// ```
/// # #[cfg(feature = "thirtyfour")]
/// # async fn example(element: &thirtyfour::WebElement) {
/// use assertr::prelude::*;
///
/// assert_that!(element)
///     .has_attribute("aria-label")
///     .await
///     .starts_with("Open");
/// assert_that!(element).attribute("aria-controls").await.is_none();
/// assert_that!(element).displayed().await.is_true();
/// # }
/// ```
///
/// A read is one snapshot, not a wait. For repeated observations, give
/// [`eventually_ok`](crate::assertions::EventualAssertions::eventually_ok) or
/// [`consistently_ok`](crate::assertions::EventualAssertions::consistently_ok) a fresh read, such
/// as `|| element.attr("aria-label")` or a helper from
/// [`read`](crate::assertions::thirtyfour::read). Call `giving_up_on_any_error()` to end at the
/// first `WebDriver` error.
#[cfg_attr(feature = "fluent", assertr_macros::fluent_aliases)]
pub trait ThirtyfourWebElementAssertions<'t, R = DebugRenderer> {
    /// Asserts presence and continues on the attribute's string, including an empty string.
    ///
    /// A missing attribute fails with "does not have the attribute", naming it.
    fn has_attribute(
        self,
        name: impl Into<String>,
    ) -> impl Future<Output = AssertThat<'t, String, Panic, R>>
    where
        R: ValueRenderer<str> + ValueRenderer<WebDriverError>;

    /// Reads an optional attribute. Use `.is_none()` for absence or `.get_some()` to extract it.
    fn attribute(
        self,
        name: impl Into<String>,
    ) -> impl Future<Output = AssertThat<'t, Option<String>, Panic, R>>
    where
        R: ValueRenderer<str> + ValueRenderer<WebDriverError>;

    /// Reads a DOM property as text.
    ///
    /// Strings are kept, booleans and numbers use their textual form, and `null` is `None`. An
    /// object or array value fails the read. Prefer [`selected`](Self::selected) or
    /// [`enabled`](Self::enabled) for boolean state.
    fn property(
        self,
        name: impl Into<String>,
    ) -> impl Future<Output = AssertThat<'t, Option<String>, Panic, R>>
    where
        R: ValueRenderer<str> + ValueRenderer<WebDriverError>;

    /// Reads `WebDriver` rendered text. This differs from DOM text content and inner text.
    fn text(self) -> impl Future<Output = AssertThat<'t, String, Panic, R>>
    where
        R: ValueRenderer<str> + ValueRenderer<WebDriverError>;

    /// Reads DOM `textContent`, preserving absence.
    fn text_content(self) -> impl Future<Output = AssertThat<'t, Option<String>, Panic, R>>
    where
        R: ValueRenderer<str> + ValueRenderer<WebDriverError>;

    /// Reads trimmed DOM `innerText`. See [`read::inner_text`] for semantics.
    fn inner_text(self) -> impl Future<Output = AssertThat<'t, String, Panic, R>>
    where
        R: ValueRenderer<str> + ValueRenderer<WebDriverError>;

    /// Reads whether `WebDriver` considers the element displayed.
    fn displayed(self) -> impl Future<Output = AssertThat<'t, bool, Panic, R>>
    where
        R: ValueRenderer<str> + ValueRenderer<WebDriverError>;

    /// Reads whether `WebDriver` considers the element enabled.
    fn enabled(self) -> impl Future<Output = AssertThat<'t, bool, Panic, R>>
    where
        R: ValueRenderer<str> + ValueRenderer<WebDriverError>;

    /// Reads the native selected state, for example a checkbox or option.
    fn selected(self) -> impl Future<Output = AssertThat<'t, bool, Panic, R>>
    where
        R: ValueRenderer<str> + ValueRenderer<WebDriverError>;

    /// Reads focus by remote identity, rejecting stale handles.
    fn focused(self) -> impl Future<Output = AssertThat<'t, bool, Panic, R>>
    where
        R: ValueRenderer<str> + ValueRenderer<WebDriverError>;

    /// Reads the browser-computed accessible name for ordinary string assertions.
    fn accessible_name(self) -> impl Future<Output = AssertThat<'t, String, Panic, R>>
    where
        R: ValueRenderer<str> + ValueRenderer<WebDriverError>;

    /// Reads the browser-computed accessible role for ordinary string assertions.
    fn accessible_role(self) -> impl Future<Output = AssertThat<'t, String, Panic, R>>
    where
        R: ValueRenderer<str> + ValueRenderer<WebDriverError>;

    /// Reads the Chromium accessibility description. See [`read::accessible_description`].
    #[cfg(feature = "thirtyfour-cdp")]
    fn accessible_description(self) -> impl Future<Output = AssertThat<'t, String, Panic, R>>
    where
        R: ValueRenderer<str> + ValueRenderer<WebDriverError>;
}

impl<'t, R> ThirtyfourWebElementAssertions<'t, R> for AssertThat<'t, WebElement, Panic, R> {
    #[track_caller]
    fn has_attribute(
        self,
        name: impl Into<String>,
    ) -> impl Future<Output = AssertThat<'t, String, Panic, R>>
    where
        R: ValueRenderer<str> + ValueRenderer<WebDriverError>,
    {
        let location = Location::caller();
        let name = name.into();
        let read = self.attribute(name.clone());
        async move {
            read.await
                .apply_assertion_after_tracking(AttributePresent(name), location)
                .map(|actual| project_checked(actual, |it| it, Option::as_ref))
        }
    }

    #[track_caller]
    fn attribute(
        self,
        name: impl Into<String>,
    ) -> impl Future<Output = AssertThat<'t, Option<String>, Panic, R>>
    where
        R: ValueRenderer<str> + ValueRenderer<WebDriverError>,
    {
        let name = name.into();
        let operation = self.named_operation("attribute", &name);
        observe(self, operation, |element| async move {
            element.attr(name).await
        })
    }

    #[track_caller]
    fn property(
        self,
        name: impl Into<String>,
    ) -> impl Future<Output = AssertThat<'t, Option<String>, Panic, R>>
    where
        R: ValueRenderer<str> + ValueRenderer<WebDriverError>,
    {
        let name = name.into();
        let operation = self.named_operation("property", &name);
        observe(self, operation, |element| async move {
            element.prop(name).await
        })
    }

    #[track_caller]
    fn text(self) -> impl Future<Output = AssertThat<'t, String, Panic, R>>
    where
        R: ValueRenderer<str> + ValueRenderer<WebDriverError>,
    {
        observe(self, "text", |element| async move { element.text().await })
    }

    #[track_caller]
    fn text_content(self) -> impl Future<Output = AssertThat<'t, Option<String>, Panic, R>>
    where
        R: ValueRenderer<str> + ValueRenderer<WebDriverError>,
    {
        observe(self, "text content", |element| async move {
            element.prop("textContent").await
        })
    }

    #[track_caller]
    fn inner_text(self) -> impl Future<Output = AssertThat<'t, String, Panic, R>>
    where
        R: ValueRenderer<str> + ValueRenderer<WebDriverError>,
    {
        observe(self, "inner text", |element| async move {
            read::inner_text(&element).await
        })
    }

    #[track_caller]
    fn displayed(self) -> impl Future<Output = AssertThat<'t, bool, Panic, R>>
    where
        R: ValueRenderer<str> + ValueRenderer<WebDriverError>,
    {
        observe(self, "displayed", |element| async move {
            element.is_displayed().await
        })
    }

    #[track_caller]
    fn enabled(self) -> impl Future<Output = AssertThat<'t, bool, Panic, R>>
    where
        R: ValueRenderer<str> + ValueRenderer<WebDriverError>,
    {
        observe(self, "enabled", |element| async move {
            element.is_enabled().await
        })
    }

    #[track_caller]
    fn selected(self) -> impl Future<Output = AssertThat<'t, bool, Panic, R>>
    where
        R: ValueRenderer<str> + ValueRenderer<WebDriverError>,
    {
        observe(self, "selected", |element| async move {
            element.is_selected().await
        })
    }

    #[track_caller]
    fn focused(self) -> impl Future<Output = AssertThat<'t, bool, Panic, R>>
    where
        R: ValueRenderer<str> + ValueRenderer<WebDriverError>,
    {
        observe(self, "focused", |element| async move {
            read::focused(&element).await
        })
    }

    #[track_caller]
    fn accessible_name(self) -> impl Future<Output = AssertThat<'t, String, Panic, R>>
    where
        R: ValueRenderer<str> + ValueRenderer<WebDriverError>,
    {
        observe(self, "accessible name", |element| async move {
            read::accessible_name(&element).await
        })
    }

    #[track_caller]
    fn accessible_role(self) -> impl Future<Output = AssertThat<'t, String, Panic, R>>
    where
        R: ValueRenderer<str> + ValueRenderer<WebDriverError>,
    {
        observe(self, "accessible role", |element| async move {
            read::accessible_role(&element).await
        })
    }

    #[cfg(feature = "thirtyfour-cdp")]
    #[track_caller]
    fn accessible_description(self) -> impl Future<Output = AssertThat<'t, String, Panic, R>>
    where
        R: ValueRenderer<str> + ValueRenderer<WebDriverError>,
    {
        observe(self, "accessible description", |element| async move {
            read::accessible_description(&element).await
        })
    }
}

impl<R: ValueRenderer<str>> AssertThat<'_, WebElement, Panic, R> {
    /// Describes a read of the attribute or property `name`, rendering the name.
    fn named_operation(&self, kind: &str, name: &str) -> String {
        format!("{kind} {}", self.render().value(name))
    }
}

/// Checks that an attribute read found the attribute, naming it when absent.
struct AttributePresent(String);

impl<R: ValueRenderer<str>> Expectation<Option<String>, R> for AttributePresent {
    type Success<'a> = &'a String;
    type Rejection<'a> = ();
    fn evaluate<'a>(
        &'a self,
        actual: &'a Option<String>,
        _: &AssertionContext<'_, R>,
    ) -> Result<&'a String, ()> {
        actual.as_ref().ok_or(())
    }

    const KIND: FailureKind = FailureKind::Membership;
    fn explain(
        &self,
        rejected: Option<(&Option<String>, ())>,
        failure: FailureBuilder,
        context: &AssertionContext<'_, R>,
    ) -> FailureBuilder {
        let relation = if rejected.is_some() {
            "does not have the attribute"
        } else {
            "has the attribute"
        };
        failure
            .relation(relation)
            .expected(context.render().value(self.0.as_str()))
    }
}

/// Tracks the read as one assertion at the caller and detaches the chain before the browser
/// await, keeping only its transferable diagnostic settings. The continuation names the
/// operation and the element in a message, and a browser error fails with both as facts.
#[track_caller]
fn observe<'t, T: 't, R, Fut>(
    chain: AssertThat<'t, WebElement, Panic, R>,
    operation: impl Into<String>,
    read: impl FnOnce(WebElement) -> Fut,
) -> impl Future<Output = AssertThat<'t, T, Panic, R>>
where
    R: ValueRenderer<str> + ValueRenderer<WebDriverError>,
    Fut: Future<Output = WebDriverResult<T>>,
{
    let operation = operation.into();
    let location = Location::caller();
    chain.track_assertion();
    let element_id = chain.actual().element_id().to_string();
    let element_id = chain.render().value(element_id.as_str());
    let (actual, detached) = chain.into_parts();
    let element = actual.into_owned();
    async move {
        match read(element).await {
            Ok(value) => detached
                .attach(Actual::Owned(value))
                .with_detail_message(format!("Observed {operation} of element {element_id}")),
            Err(error) => {
                let failure = not_observed::<WebElement>()
                    .fact(Fact::labelled("Operation", operation))
                    .fact(Fact::labelled("Element", element_id))
                    .fact(Fact::labelled(
                        "WebDriver error",
                        detached.render().value(&error),
                    ));
                detached.raise_at(failure, location)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use alloc::{collections::VecDeque, sync::Arc};
    use std::sync::Mutex;

    use serde_json::{Value, json};
    use thirtyfour::{
        DesiredCapabilities, WebDriver,
        session::http::{Body, HttpClient},
    };

    use super::*;
    use crate::{prelude::*, test_support::block_on};

    /// A scripted response to one browser read.
    enum Reply {
        /// Responds with HTTP `status` and `value`.
        Respond(u16, Value),
        /// Never responds.
        Pending,
    }

    #[derive(Clone, Default)]
    struct Transport {
        replies: Arc<Mutex<VecDeque<Reply>>>,
        paths: Arc<Mutex<Vec<String>>>,
        commands: Arc<Mutex<Vec<String>>>,
    }

    impl Transport {
        /// The request paths of the browser reads so far.
        fn paths(&self) -> Vec<String> {
            self.paths.lock().unwrap().clone()
        }

        /// The CDP commands sent so far.
        #[cfg(feature = "thirtyfour-cdp")]
        fn commands(&self) -> Vec<String> {
            self.commands.lock().unwrap().clone()
        }
    }

    #[async_trait::async_trait]
    impl HttpClient for Transport {
        async fn send(
            &self,
            request: http::Request<Body<'_>>,
        ) -> WebDriverResult<http::Response<bytes::Bytes>> {
            let path = request.uri().path().to_owned();
            if let Body::Json(body) = request.body()
                && let Some(command) = body.get("cmd").and_then(Value::as_str)
            {
                self.commands.lock().unwrap().push(command.to_owned());
            }
            let (status, value) = if path == "/session" {
                (200, json!({"sessionId": "session", "capabilities": {}}))
            } else if path == "/session/session/timeouts" {
                (200, Value::Null)
            } else {
                self.paths.lock().unwrap().push(path);
                let reply = self.replies.lock().unwrap().pop_front();
                match reply.expect("unexpected browser read") {
                    Reply::Respond(status, value) => (status, value),
                    Reply::Pending => return core::future::pending().await,
                }
            };
            Ok(http::Response::builder()
                .status(status)
                .body(serde_json::to_vec(&json!({"value":value}))?.into())
                .unwrap())
        }

        async fn new(&self) -> Arc<dyn HttpClient> {
            Arc::new(self.clone())
        }
    }

    fn element(replies: impl IntoIterator<Item = Reply>) -> (WebElement, Transport) {
        let transport = Transport::default();
        transport.replies.lock().unwrap().extend(replies);
        let driver = block_on(
            WebDriver::builder("http://localhost:4444", DesiredCapabilities::chrome())
                .client(transport.clone())
                .connect(),
        )
        .unwrap();
        let element = WebElement::from_json(
            json!({"element-6066-11e4-a52e-4f735466cecf":"node"}),
            driver.handle().clone(),
        )
        .unwrap();
        driver.leak().unwrap(); // No real session exists, and teardown must not pollute request counts.
        (element, transport)
    }

    fn ok(value: Value) -> Reply {
        Reply::Respond(200, value)
    }

    fn browser_error(code: &str, message: &str) -> Reply {
        Reply::Respond(
            404,
            json!({"error": code, "message": message, "stacktrace": ""}),
        )
    }

    fn stale() -> Reply {
        browser_error("stale element reference", "detached node")
    }

    fn send<F: Future + Send>(future: F) -> F {
        future
    }

    mod has_attribute {
        use super::*;

        #[test]
        fn caller_location_is_as_expected() {
            let (element, _) = element([ok(Value::Null)]);
            assert_caller_location!(async assert_that!(element), has_attribute("aria-label"));
        }

        #[test]
        fn extracts_once_and_composes_string_assertions() {
            let (element, transport) = element([ok(json!("Open menu")), ok(json!(""))]);
            block_on(send(async {
                assert_that!(&element)
                    .has_attribute("aria-label")
                    .await
                    .starts_with("Open")
                    .contains("menu");
                assert_that_owned!(element.clone())
                    .has_attribute("disabled")
                    .await
                    .is_empty();
            }));
            assert_that!(transport.paths()).is_equal_to([
                "/session/session/element/node/attribute/aria-label",
                "/session/session/element/node/attribute/disabled",
            ]);
        }

        #[test]
        fn unpolled_read_is_lazy() {
            let (element, transport) = element([]);
            drop(assert_that!(element).has_attribute("label"));
            assert_that!(transport.paths()).is_empty();
        }

        #[test]
        fn protocol_error_keeps_the_call_site() {
            let (element, transport) = element([stale()]);
            assert_caller_location!(async assert_that!(element), has_attribute("aria-label"));
            assert_that!(transport.paths()).has_length(1);
        }
    }

    mod attribute {
        use super::*;

        #[test]
        fn caller_location_is_as_expected() {
            let (element, _) = element([stale()]);
            assert_caller_location!(async assert_that!(element), attribute("aria-label"));
        }

        #[test]
        fn absence_and_empty_string_differ() {
            let (element, _) = element([ok(Value::Null), ok(json!(""))]);
            block_on(send(async {
                assert_that!(element).attribute("missing").await.is_none();
                assert_that!(element)
                    .attribute("empty")
                    .await
                    .get_some()
                    .is_empty();
            }));
        }
    }

    mod renderer_contract {
        use super::*;
        use crate::test_support::{NoRenderer, assert_trait_impl};

        #[test]
        fn trait_is_implemented_without_renderer_support() {
            assert_trait_impl!(
                AssertThat<'static, WebElement, Panic, NoRenderer>
                    => ThirtyfourWebElementAssertions<'static, NoRenderer>
            );
        }
    }

    #[cfg(feature = "fluent")]
    mod fluent_aliases {
        use super::*;

        #[test]
        fn are_as_expected() {
            let (element, _) = element([ok(json!("yes"))]);
            block_on(async {
                element
                    .must()
                    .have_attribute("label")
                    .await
                    .is_equal_to("yes");
            });
        }
    }

    /// The structured failure that `action` raises with the presentation it installs.
    fn failure(action: impl FnOnce(crate::test_support::LocationRecorder)) -> AssertionFailure {
        crate::test_support::raised_failure(action).expect("structured assertion failure")
    }

    mod diagnostics {
        use super::*;

        #[test]
        fn missing_attribute_report() {
            let (element, transport) = element([ok(Value::Null)]);
            let failure = failure(|presentation| {
                block_on(
                    assert_that!(element)
                        .with_location(false)
                        .with_panic_presentation(presentation)
                        .has_attribute("aria-label"),
                );
            });
            assert_that!(failure.to_string()).is_equal_to(indoc::indoc! {r#"
                -------- assertr --------
                Expression: `element`

                does not have the attribute

                Expected: "aria-label"

                Messages:
                  - Observed attribute "aria-label" of element "node"
                -------- assertr --------
            "#});
            assert_that!(transport.paths()).has_length(1);
        }

        #[test]
        fn browser_error_report() {
            let (element, _) = element([stale()]);
            let failure = failure(|presentation| {
                block_on(
                    assert_that!(element)
                        .with_location(false)
                        .with_panic_presentation(presentation)
                        .text(),
                );
            });
            assert_that!(failure.to_string()).is_equal_to(indoc::indoc! {r#"
                -------- assertr --------
                Expression: `element`

                could not be observed

                Details:
                  - Operation: text
                  - Element: "node"
                  - WebDriver error: WebDriverError(
                        StaleElementReference(
                            WebDriverErrorInfo {
                                status: 404,
                                error: "",
                                value: WebDriverErrorValue {
                                    message: "detached node",
                                    error: Some(
                                        "stale element reference",
                                    ),
                                    stacktrace: Some(
                                        "",
                                    ),
                                    data: None,
                                },
                            },
                        ),
                    )
                -------- assertr --------
            "#});
        }

        #[test]
        fn value_failure_retains_parent_metadata_and_its_own_caller() {
            let (element, transport) = element([ok(json!("Close menu"))]);
            let value = block_on(
                assert_that!(element)
                    .with_subject_name("trigger")
                    .with_detail_message("fixture context")
                    .has_attribute("aria-label"),
            );
            assert_caller_location!(value, starts_with("Open"));
            assert_that!(transport.paths()).has_length(1);
        }

        #[test]
        fn value_failure_keeps_property_and_messages() {
            let (element, _) = element([ok(json!("Close"))]);
            let failure = failure(|presentation| {
                block_on(async {
                    assert_that!(element)
                        .with_subject_name("trigger")
                        .with_detail_message("fixture context")
                        .with_panic_presentation(presentation)
                        .has_attribute("aria-label")
                        .await
                        .starts_with("Open");
                });
            });
            assert_that!(failure.subject_name).is_equal_to(Some("trigger".to_owned()));
            assert_that!(failure.to_string())
                .contains("fixture context")
                .contains(r#"attribute "aria-label""#)
                .contains("node")
                .contains("Close");
        }

        #[test]
        fn protocol_errors_are_never_absence() {
            for code in [
                "stale element reference",
                "invalid session id",
                "unknown command",
            ] {
                let (element, transport) = element([browser_error(code, "read failed")]);
                let failure = failure(|presentation| {
                    block_on(async {
                        assert_that!(element)
                            .with_panic_presentation(presentation)
                            .attribute("missing")
                            .await
                            .is_none();
                    });
                });
                assert_that!(failure.to_string())
                    .contains("could not be observed")
                    .contains("WebDriver error")
                    .contains("read failed");
                assert_that!(transport.paths()).has_length(1);
            }
        }
    }

    mod repeated {
        use core::time::Duration;

        use super::*;
        use crate::matchers::{eq, satisfying};

        #[test]
        fn retries_values_and_allows_non_equality_matchers() {
            let (element, transport) = element([ok(Value::Null), ok(json!("Open menu"))]);
            block_on(send(async {
                assert_that!(|| element.attr("label"))
                    .eventually_ok()
                    .giving_up_on_any_error()
                    .within(Duration::from_secs(1))
                    .polling_every(Duration::from_millis(1))
                    .matches(satisfying(|it: AssertThat<Option<String>, Capture>| {
                        it.is_some_satisfying(|value| {
                            value.starts_with("Open");
                        });
                    }))
                    .await
                    .get_some()
                    .contains("menu");
            }));
            assert_that!(transport.paths()).has_length(2);
        }

        #[test]
        fn terminal_browser_error_is_not_retried() {
            let (element, transport) = element([stale()]);
            let result = block_on(
                assert_that!(|| element.attr("label"))
                    .eventually_ok()
                    .giving_up_on_any_error()
                    .try_matches(eq(Some("yes".to_owned()))),
            );
            assert_that!(result).is_err();
            assert_that!(transport.paths()).has_length(1);
        }

        #[test]
        fn pending_read_respects_deadline_and_can_be_cancelled() {
            let (element, transport) = element([Reply::Pending]);
            let result = block_on(
                assert_that!(|| element.attr("label"))
                    .eventually_ok()
                    .within(Duration::from_millis(5))
                    .try_matches(eq(None::<String>)),
            );
            assert_that!(result).is_err();
            assert_that!(transport.paths()).has_length(1);
            let future = assert_that!(element).attribute("label");
            drop(future);
            assert_that!(transport.paths()).has_length(1);
        }

        #[test]
        fn consistency_checks_immediately_and_rejects_changes() {
            let (element, transport) = element([ok(json!("same")), ok(json!("changed"))]);
            let result = block_on(
                assert_that!(|| element.attr("label"))
                    .consistently_ok()
                    .for_at_least(Duration::from_millis(100))
                    .polling_every(Duration::from_millis(1))
                    .try_matches(eq(Some("same".to_owned()))),
            );
            assert_that!(result).is_err();
            assert_that!(transport.paths()).has_length(2);
        }
    }

    mod property {
        use super::*;

        #[test]
        fn caller_location_is_as_expected() {
            let (element, _) = element([stale()]);
            assert_caller_location!(async assert_that!(element), property("value"));
        }

        #[test]
        fn projects_native_value_once() {
            let (element, transport) = element([ok(json!("42"))]);
            block_on(send(async {
                assert_that!(element)
                    .property("value")
                    .await
                    .get_some()
                    .is_equal_to("42");
            }));
            assert_that!(transport.paths())
                .is_equal_to(["/session/session/element/node/property/value"]);
        }
    }

    mod text {
        use super::*;

        #[test]
        fn caller_location_is_as_expected() {
            let (element, _) = element([stale()]);
            assert_caller_location!(async assert_that!(element), text());
        }

        #[test]
        fn projects_native_value_once() {
            let (element, transport) = element([ok(json!("WebDriver text"))]);
            block_on(send(async {
                assert_that!(element)
                    .text()
                    .await
                    .is_equal_to("WebDriver text");
            }));
            assert_that!(transport.paths()).is_equal_to(["/session/session/element/node/text"]);
        }
    }

    mod text_content {
        use super::*;

        #[test]
        fn caller_location_is_as_expected() {
            let (element, _) = element([stale()]);
            assert_caller_location!(async assert_that!(element), text_content());
        }

        #[test]
        fn projects_native_value_once() {
            let (element, transport) = element([ok(json!("DOM text"))]);
            block_on(send(async {
                assert_that!(element)
                    .text_content()
                    .await
                    .get_some()
                    .is_equal_to("DOM text");
            }));
            assert_that!(transport.paths())
                .is_equal_to(["/session/session/element/node/property/textContent"]);
        }
    }

    mod inner_text {
        use super::*;

        #[test]
        fn caller_location_is_as_expected() {
            let (element, _) = element([stale()]);
            assert_caller_location!(async assert_that!(element), inner_text());
        }

        #[test]
        fn projects_native_value_once() {
            let (element, transport) = element([ok(json!("  inner text  "))]);
            block_on(send(async {
                assert_that!(element)
                    .inner_text()
                    .await
                    .is_equal_to("inner text");
            }));
            assert_that!(transport.paths())
                .is_equal_to(["/session/session/element/node/property/innerText"]);
        }
    }

    mod displayed {
        use super::*;

        #[test]
        fn caller_location_is_as_expected() {
            let (element, _) = element([stale()]);
            assert_caller_location!(async assert_that!(element), displayed());
        }

        #[test]
        fn projects_native_value_once() {
            let (element, transport) = element([ok(json!(true))]);
            block_on(send(async {
                assert_that!(element).displayed().await.is_true();
            }));
            assert_that!(transport.paths())
                .is_equal_to(["/session/session/element/node/displayed"]);
        }
    }

    mod enabled {
        use super::*;

        #[test]
        fn caller_location_is_as_expected() {
            let (element, _) = element([stale()]);
            assert_caller_location!(async assert_that!(element), enabled());
        }

        #[test]
        fn projects_native_value_once() {
            let (element, transport) = element([ok(json!(false))]);
            block_on(send(async {
                assert_that!(element).enabled().await.is_false();
            }));
            assert_that!(transport.paths()).is_equal_to(["/session/session/element/node/enabled"]);
        }
    }

    mod selected {
        use super::*;

        #[test]
        fn caller_location_is_as_expected() {
            let (element, _) = element([stale()]);
            assert_caller_location!(async assert_that!(element), selected());
        }

        #[test]
        fn projects_native_value_once() {
            let (element, transport) = element([ok(json!(true))]);
            block_on(send(async {
                assert_that!(element).selected().await.is_true();
            }));
            assert_that!(transport.paths()).is_equal_to(["/session/session/element/node/selected"]);
        }
    }

    mod accessible_name {
        use super::*;

        #[test]
        fn caller_location_is_as_expected() {
            let (element, _) = element([stale()]);
            assert_caller_location!(async assert_that!(element), accessible_name());
        }

        #[test]
        fn projects_native_value_once() {
            let (element, transport) = element([ok(json!("Open menu"))]);
            block_on(send(async {
                assert_that!(element)
                    .accessible_name()
                    .await
                    .starts_with("Open");
            }));
            assert_that!(transport.paths())
                .is_equal_to(["/session/session/element/node/computedlabel"]);
        }
    }

    mod accessible_role {
        use super::*;

        #[test]
        fn caller_location_is_as_expected() {
            let (element, _) = element([stale()]);
            assert_caller_location!(async assert_that!(element), accessible_role());
        }

        #[test]
        fn projects_native_value_once() {
            let (element, transport) = element([ok(json!("button"))]);

            block_on(send(async {
                assert_that!(element)
                    .accessible_role()
                    .await
                    .is_equal_to("button");
            }));

            assert_that!(transport.paths())
                .is_equal_to(["/session/session/element/node/computedrole"]);
        }
    }

    mod focused {
        use super::*;

        #[test]
        fn caller_location_is_as_expected() {
            let (element, _) = element([stale()]);
            assert_caller_location!(async assert_that!(element), focused());
        }

        #[test]
        fn compares_remote_identity_in_the_same_session() {
            let (element, transport) = element([
                ok(json!("button")),
                ok(json!({"element-6066-11e4-a52e-4f735466cecf":"node"})),
                ok(json!("button")),
                ok(json!({"element-6066-11e4-a52e-4f735466cecf":"another"})),
            ]);

            block_on(async {
                assert_that!(element).focused().await.is_true();
                assert_that!(element).focused().await.is_false();
            });

            assert_that!(transport.paths()).is_equal_to([
                "/session/session/element/node/name",
                "/session/session/element/active",
                "/session/session/element/node/name",
                "/session/session/element/active",
            ]);
        }
    }

    #[cfg(feature = "thirtyfour-cdp")]
    mod accessible_description {
        use super::*;

        #[test]
        fn caller_location_is_as_expected() {
            let (element, _) = element([stale()]);
            assert_caller_location!(async assert_that!(element), accessible_description());
        }

        fn acquired() -> [Reply; 2] {
            [
                ok(Value::Null),
                ok(json!({"result":{"type":"object", "objectId":"object-1"}})),
            ]
        }

        #[test]
        fn releases_object_on_success_and_on_tree_failure() {
            for (tree, expected) in [
                (
                    ok(json!({"nodes":[{"description":{"value":"Details"}}]})),
                    Some("Details"),
                ),
                (stale(), None),
                (ok(json!({"nodes":[]})), None),
            ] {
                let (element, transport) =
                    element(acquired().into_iter().chain([tree, ok(json!({}))]));
                let result = block_on(read::accessible_description(&element));
                assert_that!(result.ok()).is_equal_to(expected.map(String::from));
                assert_that!(transport.commands()).is_equal_to([
                    "Runtime.evaluate",
                    "Accessibility.getPartialAXTree",
                    "Runtime.releaseObject",
                ]);
            }
        }

        #[test]
        fn missing_description_is_empty_and_release_errors_propagate() {
            let (element, _) = element(
                acquired()
                    .into_iter()
                    .chain([ok(json!({"nodes":[{}]})), ok(json!({}))]),
            );
            block_on(async {
                assert_that!(element)
                    .accessible_description()
                    .await
                    .is_empty();
            });
            let (element, _) = super::element(
                acquired()
                    .into_iter()
                    .chain([ok(json!({"nodes":[{}]})), stale()]),
            );
            assert_that!(block_on(read::accessible_description(&element))).is_err();
        }
    }

    mod custom_rendering {
        use core::fmt;

        use super::*;
        use crate::renderer::RenderingBudget;

        struct Renderer;

        impl ValueRenderer<str> for Renderer {
            fn fmt(&self, value: &str, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "custom:{value}")
            }
        }

        impl ValueRenderer<WebDriverError> for Renderer {
            fn fmt(&self, _: &WebDriverError, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str("custom:error")
            }
        }

        #[test]
        fn extraction_preserves_non_clone_renderer_and_budget() {
            let (element, _) = element([ok(json!("Open menu"))]);
            let value = block_on(
                assert_that!(element)
                    .with_renderer(Renderer)
                    .has_attribute("label"),
            );
            value.starts_with("Open").contains("menu");
            let (element, _) = super::element([stale()]);
            let failure = failure(|presentation| {
                block_on(
                    assert_that!(element)
                        .with_renderer(Renderer)
                        .with_rendering_budget(
                            RenderingBudget::DEFAULT.with_max_leaf_characters(12),
                        )
                        .with_panic_presentation(presentation)
                        .attribute("long-attribute-name"),
                );
            });
            assert_that!(failure.to_string())
                .contains("custom:error")
                .does_not_contain("long-attribute-name");
        }

        #[test]
        fn extraction_tracks_once_on_parent() {
            let (element, _) = element([ok(json!(""))]);
            let parent = assert_that!(element);
            block_on(parent.derive(|element| element).has_attribute("label"));
            assert_that!(parent.state.records.assertion_count()).is_equal_to(1);
        }
    }
}
