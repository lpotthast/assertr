#![cfg(feature = "std")]

use core::convert::Infallible;

use assertr::failure::adapter::{
    Adapter, AdapterExt, FailurePipeline, FanOut, HumanReadableText, ToHumanReadableText,
    set_failure_pipeline,
};
use assertr::prelude::*;

struct KindAdapter;

impl Adapter<AssertionFailure> for KindAdapter {
    type Output = String;
    type Error = Infallible;

    fn adapt(&self, failure: &AssertionFailure) -> Result<Self::Output, Self::Error> {
        Ok(format!("custom adapter: {:?}", failure.kind))
    }
}

#[test]
fn the_process_pipeline_output_becomes_the_panic_payload_and_is_set_once() {
    set_failure_pipeline(FailurePipeline::new(KindAdapter))
        .expect("this test binary installs exactly one failure pipeline");
    assert!(set_failure_pipeline(KindAdapter).is_err());

    let panic = std::panic::catch_unwind(|| {
        assert_that!(1).with_location(false).is_equal_to(2);
    })
    .expect_err("the assertion should fail");
    let message = panic
        .downcast::<String>()
        .expect("the panic payload should remain a String");

    assert_eq!(*message, "custom adapter: Equality");
}

struct JsonFailure(String);
struct AiTriage(String);

struct ToJson;

impl Adapter<AssertionFailure> for ToJson {
    type Output = JsonFailure;
    type Error = Infallible;

    fn adapt(&self, failure: &AssertionFailure) -> Result<Self::Output, Self::Error> {
        Ok(JsonFailure(format!(r#"{{"kind":"{:?}"}}"#, failure.kind)))
    }
}

struct TriageThroughAi;

impl Adapter<JsonFailure> for TriageThroughAi {
    type Output = AiTriage;
    type Error = Infallible;

    fn adapt(&self, json: &JsonFailure) -> Result<Self::Output, Self::Error> {
        Ok(AiTriage(format!("triaged {}", json.0)))
    }
}

struct StdoutLogger;

impl Adapter<HumanReadableText> for StdoutLogger {
    type Output = ();
    type Error = Infallible;

    fn adapt(&self, text: &HumanReadableText) -> Result<Self::Output, Self::Error> {
        let _ = text.as_str();
        Ok(())
    }
}

impl Adapter<AiTriage> for StdoutLogger {
    type Output = ();
    type Error = Infallible;

    fn adapt(&self, triage: &AiTriage) -> Result<Self::Output, Self::Error> {
        let _ = triage.0.as_str();
        Ok(())
    }
}

struct NotifyRemote;

impl Adapter<JsonFailure> for NotifyRemote {
    type Output = ();
    type Error = Infallible;

    fn adapt(&self, json: &JsonFailure) -> Result<Self::Output, Self::Error> {
        let _ = json.0.as_str();
        Ok(())
    }
}

fn accepts_failure_adapter<A: Adapter<AssertionFailure>>(_adapter: A) {}

#[test]
fn the_proposed_adapter_chains_type_check() {
    accepts_failure_adapter(ToHumanReadableText.then(StdoutLogger));
    accepts_failure_adapter(ToJson.then(TriageThroughAi).then(StdoutLogger));
    accepts_failure_adapter(ToJson.then(NotifyRemote));

    let json_actions = ToJson.then(FanOut::new(
        TriageThroughAi.then(StdoutLogger),
        NotifyRemote,
    ));
    accepts_failure_adapter(
        FailurePipeline::new(ToHumanReadableText.tap(StdoutLogger)).branch(json_actions),
    );
}
