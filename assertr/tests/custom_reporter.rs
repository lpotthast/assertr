#![cfg(feature = "std")]

use assertr::prelude::*;
use assertr::report::{FailureReporter, set_reporter};

struct KindReporter;

impl FailureReporter for KindReporter {
    type Output = String;

    fn report(&self, failure: &AssertionFailure) -> Self::Output {
        format!("custom reporter: {:?}", failure.kind)
    }
}

#[test]
fn the_process_reporter_output_becomes_the_panic_payload() {
    set_reporter(KindReporter).expect("this test binary installs exactly one reporter");
    assert!(set_reporter(KindReporter).is_err());

    let panic = std::panic::catch_unwind(|| {
        assert_that!(1).with_location(false).is_equal_to(2);
    })
    .expect_err("the assertion should fail");
    let message = panic
        .downcast::<String>()
        .expect("the panic payload should remain a String");

    assert_eq!(*message, "custom reporter: Equality");
}
