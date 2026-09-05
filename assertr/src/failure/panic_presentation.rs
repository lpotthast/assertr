//! Presentation used exclusively when raising an assertion panic.
//!
//! Capture mode stores structured failures without invoking this module. General-purpose
//! adapters remain independent of how assertion failures are handled.

use alloc::string::String;
use core::fmt::Write;

use super::{
    AssertionFailure,
    adapter::{Adapter, HumanReadableText, ToHumanReadableText},
};

/// The context's text-producing adapter, used only by panic-mode failure handling.
///
/// The `'static` bound keeps the owned adapter's destructor independent of subject borrows,
/// allowing those borrows to end at the assertion context's last use.
pub(crate) type PanicPresentation =
    dyn Adapter<AssertionFailure, Output = HumanReadableText, Error = String> + 'static;

/// Produces panic text, preserving the assertion report if the adapter fails.
pub(crate) fn render(
    failure: &AssertionFailure,
    presentation: Option<&PanicPresentation>,
) -> HumanReadableText {
    let Some(adapter) = presentation else {
        return ToHumanReadableText.render(failure);
    };

    // Catch both adapter panics and panics from its error's Display implementation.
    // AssertUnwindSafe is limited to this call: fallback uses independent failure data
    // and never retries the adapter or relies on its state after unwinding.
    #[cfg(feature = "std")]
    let result =
        match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| adapter.adapt(failure))) {
            Ok(result) => result,
            Err(payload) => {
                return fallback(failure, "panicked", panic_payload(payload.as_ref()));
            }
        };

    #[cfg(not(feature = "std"))]
    let result = adapter.adapt(failure);

    match result {
        Ok(message) => message,
        Err(error) => fallback(failure, "returned an error", &error),
    }
}

fn fallback(failure: &AssertionFailure, reason: &str, detail: &str) -> HumanReadableText {
    let mut message = ToHumanReadableText.render(failure).into_string();
    message.push_str("\n-------- assertr presentation diagnostic --------\n");
    writeln!(message, "The failure presentation {reason}: {detail}")
        .expect("writing a presentation diagnostic to a String cannot fail");
    message.push_str("------ end assertr presentation diagnostic ------\n");
    HumanReadableText::new(message)
}

#[cfg(feature = "std")]
fn panic_payload(payload: &(dyn core::any::Any + Send)) -> &str {
    if let Some(message) = payload.downcast_ref::<&str>() {
        message
    } else if let Some(message) = payload.downcast_ref::<String>() {
        message
    } else {
        "non-string panic payload"
    }
}
