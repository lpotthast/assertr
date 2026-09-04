#[cfg(feature = "std")]
use alloc::boxed::Box;
use alloc::string::String;

use crate::AssertionFailure;

#[cfg(feature = "std")]
use super::FailureReporter;
use super::text::TextReporter;

#[cfg(feature = "std")]
type PanicReporter = dyn FailureReporter<Output = String> + Send + Sync;

#[cfg(feature = "std")]
static REPORTER: std::sync::OnceLock<Box<PanicReporter>> = std::sync::OnceLock::new();

/// Installs the process-wide reporter used to produce panic payloads.
///
/// The reporter can be installed once. Captured failures are unaffected and can be passed to any
/// reporter directly.
///
/// # Errors
///
/// Returns [`SetReporterError`] if a reporter was installed earlier in the process.
#[cfg(feature = "std")]
pub fn set_reporter<R>(reporter: R) -> Result<(), SetReporterError>
where
    R: FailureReporter<Output = String> + Send + Sync + 'static,
{
    REPORTER
        .set(Box::new(reporter))
        .map_err(|_reporter| SetReporterError)
}

/// Returned when a process-wide panic reporter has already been installed.
#[cfg(feature = "std")]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SetReporterError;

pub(crate) fn report_for_panic(failure: &AssertionFailure) -> String {
    #[cfg(feature = "std")]
    if let Some(reporter) = REPORTER.get() {
        return FailureReporter::report(&**reporter, failure);
    }

    TextReporter.report(failure)
}
