use super::*;
use crate::{prelude::*, test_support::block_on};
use alloc::{collections::VecDeque, sync::Arc};
use std::sync::Mutex;
use thirtyfour::{DesiredCapabilities, WebDriver, session::http::{Body, HttpClient}};
use serde_json::{Value, json};

#[derive(Clone, Default)]
struct Transport {
    replies: Arc<Mutex<VecDeque<(u16, Value)>>>,
    paths: Arc<Mutex<Vec<String>>>,
}
#[async_trait::async_trait]
impl HttpClient for Transport {
    async fn send(&self, request: http::Request<Body<'_>>) -> WebDriverResult<http::Response<bytes::Bytes>> {
        let path = request.uri().path().to_owned();
        let (status, value) = if path == "/session" {
            (200, json!({"sessionId": "session", "capabilities": {}}))
        } else {
            self.paths.lock().unwrap().push(path);
            self.replies.lock().unwrap().pop_front().expect("unexpected browser read")
        };
        Ok(http::Response::builder().status(status).body(serde_json::to_vec(&json!({"value":value})).unwrap().into()).unwrap())
    }
    async fn new(&self) -> Arc<dyn HttpClient> { Arc::new(self.clone()) }
}
fn element(replies: impl IntoIterator<Item = (u16, Value)>) -> (WebElement, Transport) {
    let transport = Transport::default();
    transport.replies.lock().unwrap().extend(replies);
    let driver = block_on(WebDriver::builder("http://localhost:4444", DesiredCapabilities::chrome()).client(transport.clone()).connect()).unwrap();
    let element = WebElement::from_json(json!({"element-6066-11e4-a52e-4f735466cecf":"node"}), driver.handle().clone()).unwrap();
    driver.leak().unwrap(); // No real session exists, and teardown must not pollute request counts.
    (element, transport)
}
fn ok(value: Value) -> (u16, Value) { (200, value) }
fn stale() -> (u16, Value) { (404, json!({"error":"stale element reference", "message":"detached node", "stacktrace":""})) }
fn send<F: Future + Send>(future: F) -> F { future }

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
            assert_that!(&element).has_attribute("aria-label").await.starts_with("Open").contains("menu");
            assert_that_owned!(element.clone()).has_attribute("disabled").await.is_empty();
        }));
        assert_that!(*transport.paths.lock().unwrap()).is_equal_to(["/session/session/element/node/attribute/aria-label", "/session/session/element/node/attribute/disabled"]);
    }
    #[test]
    fn unpolled_read_is_lazy() {
        let (element, transport) = element([]);
        drop(assert_that!(element).has_attribute("label"));
        assert_that!(*transport.paths.lock().unwrap()).is_empty();
    }
    #[test]
    fn protocol_error_keeps_the_call_site() {
        let (element, transport) = element([stale()]);
        assert_caller_location!(async assert_that!(element), has_attribute("aria-label"));
        assert_that!(*transport.paths.lock().unwrap()).has_length(1);
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
            assert_that!(element).attribute("empty").await.get_some().is_empty();
        }));
    }
}
#[test]
fn trait_does_not_require_a_renderer() {
    fn available<'a, T: ThirtyfourWebElementAssertions<'a, crate::test_support::NoRenderer>>() {}
    available::<AssertThat<'_, WebElement, Panic, crate::test_support::NoRenderer>>();
}
#[cfg(feature = "fluent")]
mod fluent_aliases {
    use super::*;
    #[test]
    fn are_as_expected() {
        let (element, _) = element([ok(json!("yes"))]);
        block_on(async { element.must().have_attribute("label").await.is_equal_to("yes"); });
    }
}
