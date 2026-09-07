use crate::{AssertThat, PanicValue, actual::Actual, mode::Panic};
use core::panic::AssertUnwindSafe;

/// Captures a panic for unit tests while the library itself is built without its `std` feature.
///
/// The test harness is hosted and can therefore use `std`. This helper is crate-private and is
/// never present in a production build.
#[track_caller]
pub(crate) fn assert_that_panic_by<'t, R>(
    fun: impl FnOnce() -> R + 't,
) -> AssertThat<'t, PanicValue, Panic> {
    let result = std::panic::catch_unwind(AssertUnwindSafe(fun));
    let result = std::panic::catch_unwind(AssertUnwindSafe(move || result.map(drop)));
    let panic = result
        .flatten()
        .expect_err("expected the tested function to panic");

    AssertThat::new_panicking(Actual::Owned(PanicValue(panic)))
}
