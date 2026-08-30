use alloc::boxed::Box;
use core::any::Any;
#[cfg(feature = "std")]
use core::future::Future;

#[cfg(any(feature = "std", all(test, not(feature = "std"))))]
use crate::{AssertThat, actual::Actual, mode::Panic};

/// A captured panic payload used as the subject of panic-value assertions.
///
/// With the `std` feature, `assert_that_panic_by` and `assert_that_panic_by_async` create this
/// subject. Its payload is type-erased. Use the
/// [`PanicValueAssertions`](crate::assertions::alloc::panic_value::PanicValueAssertions) methods to
/// inspect it.
pub struct PanicValue(pub(crate) Box<dyn Any>);

/// Invokes `fun`, asserts that the call or dropping its output panics, and returns an assertion
/// over the panic payload.
///
/// This is a synonym for `assert_that_owned!(fun).panics()`.
#[track_caller]
#[must_use]
#[cfg(feature = "std")]
pub fn assert_that_panic_by<'t, R>(
    fun: impl FnOnce() -> R + 't,
) -> AssertThat<'t, PanicValue, Panic> {
    use crate::prelude::FnOnceAssertions;

    AssertThat::new_panicking(Actual::Owned(fun)).panics()
}

/// Invokes `fun`, asserts that the call, polling its future, or dropping its output panics, and
/// returns an assertion over the panic payload.
#[track_caller]
#[must_use = "futures do nothing unless awaited or polled"]
#[cfg(feature = "std")]
pub fn assert_that_panic_by_async<'t, F, Fut, R>(
    fun: F,
) -> impl Future<Output = AssertThat<'t, PanicValue, Panic>>
where
    F: FnOnce() -> Fut + 't,
    Fut: Future<Output = R>,
{
    crate::assertions::core::r#fn::panics_async_at(
        AssertThat::new_panicking(Actual::Owned(fun)),
        core::panic::Location::caller(),
    )
}

#[cfg(all(test, not(feature = "std")))]
pub(crate) mod no_std_test_support {
    use super::{Actual, AssertThat, Panic, PanicValue};
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
}

#[cfg(all(test, feature = "std"))]
mod tests {
    use crate::prelude::*;

    #[cfg(feature = "std")]
    #[tokio::test]
    async fn assert_that_panic_by_async_failure_location_points_at_its_caller() {
        let expected_line = line!() + 2;
        let panic = assert_that_panic_by_async(async || {
            let _ = assert_that_panic_by_async(async || {}).await;
        })
        .await;

        panic
            .has_type::<String>()
            .contains(format!("Assertion failed at {}:{expected_line}:", file!()));
    }

    #[cfg(feature = "std")]
    #[tokio::test]
    async fn assert_that_panic_by_async_catches_a_panic_after_yielding() {
        assert_that_panic_by_async(async || {
            tokio::task::yield_now().await;
            panic!("boom");
        })
        .await
        .has_type::<&str>()
        .is_equal_to("boom");
    }
}
