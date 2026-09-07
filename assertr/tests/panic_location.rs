use std::panic::{AssertUnwindSafe, Location, catch_unwind, set_hook, take_hook};
use std::sync::mpsc;

use assertr::{FailureKind, prelude::*};

#[track_caller]
fn check_panic_location(assertion: impl FnOnce()) {
    let expected = Location::caller();
    let (sender, receiver) = mpsc::channel();
    let previous_hook = take_hook();
    set_hook(Box::new(move |info| {
        let location = info.location().map(|location| {
            (
                location.file().to_owned(),
                location.line(),
                location.column(),
            )
        });
        let _ = sender.send(location);
    }));

    let outcome = catch_unwind(AssertUnwindSafe(assertion));
    set_hook(previous_hook);

    assert!(outcome.is_err(), "expected an assertion failure");
    assert_eq!(
        receiver.try_recv().expect("the panic hook should run"),
        Some((
            expected.file().to_owned(),
            expected.line(),
            expected.column()
        )),
        "the native panic location should point to the assertion call",
    );
}

// Reconstruct the call so its punctuation and the check share the macro invocation's span.
// This checks the exact file, line, and column without fixed source coordinates.
macro_rules! assert_panic_location {
    ($context:expr, $method:ident($($arg:expr),* $(,)?)) => {
        check_panic_location(|| {
            ($context).$method($($arg),*);
        });
    };
}

#[test]
fn native_panic_location_points_to_the_assertion_call() {
    // Keep hook changes in a single test in a separate integration-test binary so they cannot
    // interfere with other tests. This also exercises a no_std library on the hosted harness.
    assert_panic_location!(assert_that!(1), is_equal_to(2));
    assert_panic_location!(assert_that!(1).with_location(false), is_equal_to(2));
    assert_panic_location!(assert_that!(1).failure(FailureKind::Other), raise());

    #[cfg(feature = "fluent")]
    assert_panic_location!(1.must(), be_equal_to(2));
}
