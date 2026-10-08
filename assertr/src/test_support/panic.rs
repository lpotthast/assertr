use crate::{AssertThat, PanicValue, actual::Actual, mode::Panic};
use core::panic::AssertUnwindSafe;

/// Captures a panic for unit tests, including builds without the library's `std` feature.
///
/// The public way to assert a panic is `assert_that_owned!(f).panics()`, which needs `std`. The
/// test harness is hosted and can use `std` anyway. This helper is crate-private and never present
/// in a production build.
#[track_caller]
pub(crate) fn assert_that_panic_by<'t, R>(
    fun: impl FnOnce() -> R + 't,
) -> AssertThat<'t, PanicValue, Panic> {
    let result = std::panic::catch_unwind(AssertUnwindSafe(fun));
    let result = std::panic::catch_unwind(AssertUnwindSafe(move || result.map(drop)));
    let panic = result
        .flatten()
        .expect_err("expected the tested function to panic");

    AssertThat::new(Actual::Owned(PanicValue(panic)))
}
