//! Presentation used exclusively when raising an assertion panic.
//!
//! Capture mode stores structured failures without invoking this module.

use alloc::string::{String, ToString};
use core::panic::RefUnwindSafe;

use super::AssertionFailure;

/// The chain's panic text closure, used only by panic-mode failure handling.
///
/// The `'static` bound keeps the owned closure's destructor independent of subject borrows,
/// allowing those borrows to end at the assertion chain's last use.
/// Preserve unwind safety when erasing the closure type, including through the shared `Arc`. It is
/// `Send` and `Sync`, so that assertions awaiting an observation can move between threads.
pub(crate) type PanicPresentation =
    dyn Fn(&AssertionFailure) -> String + RefUnwindSafe + Send + Sync + 'static;

/// Produces panic text, preserving the assertion report if the presentation panics.
pub(crate) fn render(
    failure: &AssertionFailure,
    presentation: Option<&PanicPresentation>,
) -> String {
    let Some(presentation) = presentation else {
        return failure.to_string();
    };

    #[cfg(feature = "std")]
    {
        match std::panic::catch_unwind(|| presentation(failure)) {
            Ok(text) => text,
            Err(payload) => fallback(failure, payload.as_ref()),
        }
    }

    #[cfg(not(feature = "std"))]
    presentation(failure)
}

#[cfg(feature = "std")]
fn fallback(failure: &AssertionFailure, payload: &(dyn core::any::Any + Send)) -> String {
    let detail = crate::entry::panic_message(payload).unwrap_or("non-string panic payload");
    alloc::format!(
        "{failure}\n-------- assertr presentation diagnostic --------\n\
         The failure presentation panicked: {detail}\n\
         ------ end assertr presentation diagnostic ------\n"
    )
}
