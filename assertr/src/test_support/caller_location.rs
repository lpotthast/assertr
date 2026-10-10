//! Exact assertion call-site checks without fixed source coordinates.

use crate::{
    failure::AssertionFailure,
    prelude::{PartialEqAssertions, ResultAssertions, assert_that},
};
use alloc::{
    boxed::Box,
    string::{String, ToString},
    sync::Arc,
    vec::Vec,
};
use core::panic::{Location, RefUnwindSafe};
use std::sync::Mutex;

/// A panic presentation recording the failure's location before producing the default report.
pub(crate) type LocationRecorder =
    Box<dyn Fn(&AssertionFailure) -> String + RefUnwindSafe + Send + Sync>;

/// Runs an assertion with a recording presentation, then checks its captured caller location.
#[track_caller]
pub(crate) fn check_caller_location(assertions: impl FnOnce(LocationRecorder)) {
    let expected = Location::caller();
    let recorded_locations = Arc::new(Mutex::new(Vec::new()));

    // The assertion context owns its presentation. Retain a handle to the recorded locations so
    // they survive unwinding and can be checked after the assertion panics.
    let recorder = Arc::clone(&recorded_locations);
    let outcome = std::panic::catch_unwind(core::panic::AssertUnwindSafe(|| {
        assertions(Box::new(move |failure: &AssertionFailure| {
            recorder.lock().unwrap().push(failure.location);
            failure.to_string()
        }));
    }));
    assert_that!(outcome)
        .with_detail_message("expected an assertion failure")
        .is_err();
    assert_that!(*recorded_locations.lock().unwrap())
        .with_detail_message(
            "incorrect assertion caller location (or no structured assertion failure)",
        )
        .is_equal_to([Some(expected)]);
}

/// Checks one failing assertion method's exact caller location.
///
/// Choose the macro form according to the assertion method's return type:
/// - Synchronous: `assert_caller_location!(assert_that!(false), is_true())`.
/// - Future: `assert_caller_location!(async assert_that!(|| async {}), panics_async())`.
/// - Future after a builder step: `assert_caller_location!(async assert_that!(observe),
///   eventually().matches(eq(1)))`.
///
/// Both forms can be called from a synchronous `#[test]` function. In the `async` form, the macro
/// creates a Tokio runtime and awaits the method internally using `block_on`. The test itself
/// stays synchronous. This form requires the `std` feature.
///
/// Keep the method separate from its receiver: reconstructing the call's punctuation gives it
/// the macro invocation's span, even when arguments span multiple lines. Forwarding an opaque
/// call expression would retain that expression's span.
macro_rules! assert_caller_location {
    (async $context:expr, $builder:ident ($($builder_arg:expr),* $(,)?) . $method:ident $(::<$($ty:ty),+>)? ($($arg:expr),* $(,)?) $(,)?) => {{
        $crate::test_support::check_caller_location(|presentation| {
            $crate::test_support::block_on(async {
                ($context)
                    .with_panic_presentation(presentation)
                    .$builder($($builder_arg),*)
                    .$method $(::<$($ty),+>)? ($($arg),*).await;
            });
        });
    }};
    (async $context:expr, $method:ident $(::<$($ty:ty),+>)? ($($arg:expr),* $(,)?) $(,)?) => {{
        $crate::test_support::check_caller_location(|presentation| {
            $crate::test_support::block_on(async {
                ($context)
                    .with_panic_presentation(presentation)
                    .$method $(::<$($ty),+>)? ($($arg),*).await;
            });
        });
    }};
    ($context:expr, $method:ident $(::<$($ty:ty),+>)? ($($arg:expr),* $(,)?) $(,)?) => {{
        $crate::test_support::check_caller_location(|presentation| {
            ($context)
                .with_panic_presentation(presentation)
                .$method $(::<$($ty),+>)? ($($arg),*);
        });
    }};
}

pub(crate) use assert_caller_location;

pub(crate) fn block_on<F: Future>(future: F) -> F::Output {
    tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap()
        .block_on(future)
}

mod tests {
    use crate::prelude::*;

    trait ProbeAssertions: Sized {
        fn untracked(self) -> Self;

        #[track_caller]
        fn tracked(self) -> Self;

        fn untracked_default(self) -> Self {
            self.tracked()
        }

        #[track_caller]
        fn tracked_default(self) -> Self {
            self.tracked()
        }
    }

    impl ProbeAssertions for AssertThat<'_, bool, Panic> {
        fn untracked(self) -> Self {
            self.is_true()
        }

        #[track_caller]
        fn tracked(self) -> Self {
            self.is_true()
        }
    }

    #[test]
    fn accepts_tracked_methods_and_defaults() {
        assert_caller_location!(assert_that!(false), tracked());
        assert_caller_location!(assert_that!(false), tracked_default());
    }

    #[test]
    fn accepts_multiline_generic_extractions() {
        assert_caller_location!(
            assert_that_owned!(Box::new(1_i32) as Box<dyn core::any::Any>),
            has_type::<u8>()
        );
    }

    #[test]
    #[should_panic(expected = "incorrect assertion caller location")]
    fn rejects_untracked_methods_in_the_same_file() {
        assert_caller_location!(assert_that!(false), untracked());
    }

    #[test]
    #[should_panic(expected = "incorrect assertion caller location")]
    fn rejects_untracked_defaults_in_the_same_file() {
        assert_caller_location!(assert_that!(false), untracked_default());
    }

    #[test]
    #[should_panic(expected = "expected an assertion failure")]
    fn rejects_passing_assertions() {
        assert_caller_location!(assert_that!(true), is_true());
    }

    #[test]
    #[should_panic(expected = "incorrect assertion caller location")]
    fn rejects_unrelated_panics() {
        fn unrelated() -> bool {
            panic!("unrelated");
        }
        assert_caller_location!(assert_that!(false), is_equal_to(unrelated()));
    }

    #[test]
    #[cfg(feature = "std")]
    fn accepts_awaited_failures() {
        assert_caller_location!(
            async assert_that!(|| async { tokio::task::yield_now().await; }),
            panics_async()
        );
    }

    #[test]
    #[cfg(feature = "fluent")]
    fn generated_fluent_alias_preserves_the_caller() {
        assert_caller_location!(false.must(), be_true());
    }
}
