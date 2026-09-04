//! Typed adapters and pipelines for structured assertion failures.
//!
//! [`Adapter`] transforms a borrowed input into an owned output. [`AdapterExt::then`] builds a
//! linear chain, [`AdapterExt::tap`] observes an intermediate value without replacing it,
//! [`FanOut`] sends one input to two branches, and [`FailurePipeline`] retains one primary output
//! while running independent branches against the same [`AssertionFailure`](crate::AssertionFailure).
//!
//! [`ToHumanReadableText`] is the default adapter used for panic messages. With `std`,
//! [`set_failure_pipeline`] installs a process-wide pipeline whose output becomes the panic
//! payload.
//!
//! ```
//! use core::convert::Infallible;
//! use assertr::failure::adapter::{
//!     Adapter, AdapterExt, HumanReadableText, ToHumanReadableText,
//! };
//! use assertr::prelude::*;
//!
//! struct TextLength;
//!
//! impl Adapter<HumanReadableText> for TextLength {
//!     type Output = usize;
//!     type Error = Infallible;
//!
//!     fn adapt(&self, text: &HumanReadableText) -> Result<usize, Self::Error> {
//!         Ok(text.as_str().len())
//!     }
//! }
//!
//! let failures = assert_that!(1)
//!     .with_location(false)
//!     .capture(|it| it.is_equal_to(2));
//! let pipeline = ToHumanReadableText.then(TextLength);
//! let length = pipeline.adapt(&failures[0]).unwrap();
//! assert!(length > 0);
//! ```

mod boundary;
mod composition;
mod human_readable;

pub(crate) use boundary::message_for_panic;
#[cfg(feature = "std")]
pub use boundary::{IntoPanicMessage, SetFailurePipelineError, set_failure_pipeline};
pub use composition::{
    FailurePipeline, FailurePipelineError, FanOut, FanOutError, NoBranches, Tap, Then, ThenError,
};
pub use human_readable::{HumanReadableText, ToHumanReadableText};

/// Transforms a borrowed input into an owned output.
///
/// An adapter can change representation or perform a side effect. Side-effect-only adapters use
/// `()` as their output. The input is generic so the output of one adapter can be the input of the
/// next one.
pub trait Adapter<Input: ?Sized> {
    /// The owned value produced by this adapter.
    type Output;

    /// The error produced by this adapter.
    type Error;

    /// Adapts one borrowed input.
    ///
    /// # Errors
    ///
    /// Returns the adapter's typed error when it cannot produce its output.
    fn adapt(&self, input: &Input) -> Result<Self::Output, Self::Error>;
}

/// Fluent composition methods for adapters.
///
/// This is separate from [`Adapter`] because that trait's generic input cannot always be inferred
/// at the point where a chain is assembled. The resulting composition implements [`Adapter`]
/// only when its adjacent stages have compatible types.
pub trait AdapterExt: Sized {
    /// Passes this adapter's successful output to `next`.
    fn then<Next>(self, next: Next) -> Then<Self, Next> {
        Then::new(self, next)
    }

    /// Runs `sink` on this adapter's successful output, then preserves that output.
    fn tap<Sink>(self, sink: Sink) -> Tap<Self, Sink> {
        Tap::new(self, sink)
    }
}

impl<T> AdapterExt for T {}
