use alloc::string::String;

#[cfg(feature = "std")]
use alloc::{boxed::Box, string::ToString};
#[cfg(feature = "std")]
use core::fmt::{self, Display, Write};

use crate::AssertionFailure;

#[cfg(feature = "std")]
use super::Adapter;
#[cfg(feature = "std")]
use super::HumanReadableText;
use super::ToHumanReadableText;

#[cfg(feature = "std")]
mod sealed {
    pub trait Sealed {}

    impl Sealed for alloc::string::String {}
    impl Sealed for super::HumanReadableText {}
}

/// Converts a process-wide failure pipeline's output into a string panic payload.
///
/// This trait is sealed. [`String`] and [`HumanReadableText`] are the supported terminal output
/// types.
#[cfg(feature = "std")]
pub trait IntoPanicMessage: sealed::Sealed {
    /// Consumes the terminal output.
    #[doc(hidden)]
    fn into_panic_message(self) -> String;
}

#[cfg(feature = "std")]
impl IntoPanicMessage for String {
    fn into_panic_message(self) -> String {
        self
    }
}

#[cfg(feature = "std")]
impl IntoPanicMessage for HumanReadableText {
    fn into_panic_message(self) -> String {
        self.into_string()
    }
}

#[cfg(feature = "std")]
trait ErasedFailurePipeline: Send + Sync {
    fn adapt_for_panic(&self, failure: &AssertionFailure) -> Result<String, String>;
}

#[cfg(feature = "std")]
impl<P> ErasedFailurePipeline for P
where
    P: Adapter<AssertionFailure> + Send + Sync,
    P::Output: IntoPanicMessage,
    P::Error: Display,
{
    fn adapt_for_panic(&self, failure: &AssertionFailure) -> Result<String, String> {
        self.adapt(failure)
            .map(IntoPanicMessage::into_panic_message)
            .map_err(|error| error.to_string())
    }
}

#[cfg(feature = "std")]
type PanicPipeline = dyn ErasedFailurePipeline;

#[cfg(feature = "std")]
static FAILURE_PIPELINE: std::sync::OnceLock<Box<PanicPipeline>> = std::sync::OnceLock::new();

/// Installs the process-wide adapter pipeline used to produce panic payloads.
///
/// The pipeline can be installed once. Its terminal output must be [`String`] or
/// [`HumanReadableText`]. Captured failures are unaffected and can be passed to any adapter or
/// pipeline directly.
///
/// # Errors
///
/// Returns [`SetFailurePipelineError`] if a pipeline was installed earlier in the process.
#[cfg(feature = "std")]
pub fn set_failure_pipeline<P>(pipeline: P) -> Result<(), SetFailurePipelineError>
where
    P: Adapter<AssertionFailure> + Send + Sync + 'static,
    P::Output: IntoPanicMessage,
    P::Error: Display,
{
    FAILURE_PIPELINE
        .set(Box::new(pipeline))
        .map_err(|_pipeline| SetFailurePipelineError)
}

/// Returned when a process-wide failure pipeline has already been installed.
#[cfg(feature = "std")]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SetFailurePipelineError;

#[cfg(feature = "std")]
impl Display for SetFailurePipelineError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("a process-wide failure pipeline is already installed")
    }
}

#[cfg(feature = "std")]
impl std::error::Error for SetFailurePipelineError {}

pub(crate) fn message_for_panic(failure: &AssertionFailure) -> String {
    #[cfg(feature = "std")]
    if let Some(pipeline) = FAILURE_PIPELINE.get() {
        return message_from(&**pipeline, failure);
    }

    ToHumanReadableText.render(failure).into_string()
}

#[cfg(feature = "std")]
fn message_from(pipeline: &PanicPipeline, failure: &AssertionFailure) -> String {
    match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        pipeline.adapt_for_panic(failure)
    })) {
        Ok(Ok(message)) => message,
        Ok(Err(error)) => fallback_with_diagnostic(
            failure,
            "The configured failure pipeline returned an error",
            &error,
        ),
        Err(payload) => fallback_with_diagnostic(
            failure,
            "The configured failure pipeline panicked",
            panic_payload(&payload),
        ),
    }
}

#[cfg(feature = "std")]
fn fallback_with_diagnostic(failure: &AssertionFailure, summary: &str, detail: &str) -> String {
    let mut message = ToHumanReadableText.render(failure).into_string();
    message.push_str("\n-------- assertr adapter diagnostic --------\n");
    writeln!(message, "{summary}: {detail}")
        .expect("writing an adapter diagnostic to a String cannot fail");
    message.push_str("------ end assertr adapter diagnostic ------\n");
    message
}

#[cfg(feature = "std")]
fn panic_payload(payload: &Box<dyn core::any::Any + Send>) -> &str {
    if let Some(message) = payload.downcast_ref::<&str>() {
        message
    } else if let Some(message) = payload.downcast_ref::<String>() {
        message
    } else {
        "non-string panic payload"
    }
}

#[cfg(all(test, feature = "std"))]
mod tests {
    use core::convert::Infallible;

    use super::*;
    use crate::failure::{FailureBuilder, FailureKind};

    struct ReturnsError;

    impl Adapter<AssertionFailure> for ReturnsError {
        type Output = String;
        type Error = &'static str;

        fn adapt(&self, _failure: &AssertionFailure) -> Result<Self::Output, Self::Error> {
            Err("remote service unavailable")
        }
    }

    struct Panics;

    impl Adapter<AssertionFailure> for Panics {
        type Output = String;
        type Error = Infallible;

        fn adapt(&self, _failure: &AssertionFailure) -> Result<Self::Output, Self::Error> {
            panic!("adapter exploded")
        }
    }

    fn failure() -> AssertionFailure {
        FailureBuilder::detached::<i32>(FailureKind::Equality)
            .actual(1)
            .expected(2)
            .build()
    }

    #[test]
    fn an_adapter_error_preserves_the_failure_and_adds_a_diagnostic() {
        let message = message_from(&ReturnsError, &failure());

        assert!(message.contains("Expected: 2\n\n  Actual: 1"));
        assert!(message.contains("assertr adapter diagnostic"));
        assert!(message.contains("returned an error: remote service unavailable"));
    }

    #[test]
    fn an_adapter_panic_preserves_the_failure_and_adds_a_diagnostic() {
        let message = message_from(&Panics, &failure());

        assert!(message.contains("Expected: 2\n\n  Actual: 1"));
        assert!(message.contains("assertr adapter diagnostic"));
        assert!(message.contains("panicked: adapter exploded"));
    }
}
