//! Reporters that put structured assertion failures to use.
//!
//! [`FailureReporter`] is the adapter contract. [`TextReporter`] provides assertr's stable
//! human-readable report, while [`set_reporter`] selects the reporter used for panic payloads.

mod boundary;
mod text;

use crate::AssertionFailure;

pub(crate) use boundary::report_for_panic;
#[cfg(feature = "std")]
pub use boundary::{SetReporterError, set_reporter};
pub use text::TextReporter;

/// Puts an assertion failure to one use.
///
/// A reporter may produce text or another value, store the failure, send it elsewhere, or perform
/// only a side effect. The associated output type keeps all of those uses expressible without
/// making human-readable text the canonical form of a failure.
pub trait FailureReporter {
    /// What reporting produces. Side-effect-only reporters use `()`.
    type Output;

    /// Reports one structured assertion failure.
    fn report(&self, failure: &AssertionFailure) -> Self::Output;
}
