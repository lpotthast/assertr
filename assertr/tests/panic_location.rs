use std::{
    panic::{catch_unwind, set_hook, take_hook},
    sync::mpsc,
};

use assertr::prelude::*;

#[test]
fn panic_and_failure_locations_point_to_the_assertion_call() {
    // This binary has one test because panic hooks are process-global.
    let (sender, receiver) = mpsc::channel();
    let (failure_sender, failure_receiver) = mpsc::channel();
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

    let assertion = assert_that!(1).with_panic_presentation(move |failure| {
        let _ = failure_sender.send(failure.clone());
        failure.to_string()
    });
    // `line!()` shares the line of the failing call, so the expectation follows edits above.
    let (expected_line, outcome) = (line!(), catch_unwind(|| drop(assertion.is_equal_to(2))));
    set_hook(previous_hook);

    assert_that!(outcome)
        .with_detail_message("expected an assertion failure")
        .is_err();
    let (file, line, column) = receiver
        .try_recv()
        .expect("the panic hook should run")
        .expect("the panic should have a location");

    // Check Rust's native `PanicHookInfo::location()`, which panic hooks and tooling receive.
    // Assertr separately captures `AssertionFailure::location` when building the failure. That
    // field was already correct when `FailureBuilder::raise` lacked `#[track_caller]`, but Rust's
    // native location pointed to the `panic!` inside `raise`. Checking only the structured failure
    // would miss this regression. Both locations must point to the assertion above.
    assert_that!(file).is_equal_to(file!());
    assert_that!(line).is_equal_to(expected_line);
    assert_that!(column).is_equal_to(77); // The start of `is_equal_to` above.

    let failure = failure_receiver
        .try_recv()
        .expect("the presentation should receive the structured failure");
    let location = failure
        .location
        .expect("the failure should have a location");
    assert_that!(location.file()).is_equal_to(file.as_str());
    assert_that!(location.line()).is_equal_to(line);
    assert_that!(location.column()).is_equal_to(column);
}
