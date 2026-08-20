#![cfg_attr(not(feature = "std"), no_std)]

use assertr::prelude::*;

#[allow(dead_code)]
fn pattern_assertions_compile_without_std() {
    assert_that!(Some(42)).is_matching(pattern!(Some(42)));
}

#[allow(dead_code)]
fn iterator_assertions_compile_without_std() {
    fn positive(it: AssertThat<i32, Capture>) {
        it.is_greater_than(0);
    }

    assert_that!(0..).contains(2);
    assert_that!(1..).contains_satisfying(positive);
    assert_that!(0..).contains_contiguous([2, 3]);
    assert_that!([2, 1].into_iter()).contains_exactly_in_any_order([1, 2]);
    assert_that!([1, 2].into_iter()).has_remaining_count(2);

    assert_that!([1, 2])
        .into_iter_starts_with([1])
        .into_iter_ends_with([2])
        .into_iter_contains_contiguous_satisfying([positive, positive])
        .into_iter_contains_exactly_in_any_order([2, 1]);
}

#[cfg(all(test, not(feature = "std")))]
extern crate std;

#[cfg(all(test, not(feature = "std")))]
mod tests {
    use assertr::prelude::{
        BoolAssertions, IteratorAssertions, LengthAssertions, PartialEqAssertions,
    };

    #[test]
    fn streaming_iterator_assertions_run_in_the_hosted_no_std_fixture() {
        let failures = assertr::assert_that!(0..20)
            .with_capture()
            .does_not_contain(19)
            .capture_failures();

        assertr::assert_that!(failures).has_length(1);
    }

    #[test]
    fn dropping_an_unused_assertion_does_not_panic() {
        let result = std::panic::catch_unwind(|| {
            let _assertion = assertr::assert_that!(42);
        });

        assertr::assert_that!(result.is_ok()).is_true();
    }

    #[test]
    fn dropping_an_uncaptured_assertion_does_not_panic() {
        let result = std::panic::catch_unwind(|| {
            let _assertion = assertr::assert_that!(42).with_capture().is_equal_to(43);
        });

        assertr::assert_that!(result.is_ok()).is_true();
    }

    #[test]
    fn dropping_an_unused_assertion_during_unwinding_preserves_the_original_panic() {
        let panic = std::panic::catch_unwind(|| {
            let _assertion = assertr::assert_that!(42);
            panic!("original panic");
        })
        .expect_err("the closure should panic");

        assertr::assert_that!(panic.downcast_ref::<&str>()).is_equal_to(Some(&"original panic"));
    }

    #[test]
    fn dropping_an_uncaptured_assertion_during_unwinding_preserves_the_original_panic() {
        let panic = std::panic::catch_unwind(|| {
            let _assertion = assertr::assert_that!(42).with_capture().is_equal_to(43);
            panic!("original panic");
        })
        .expect_err("the closure should panic");

        assertr::assert_that!(panic.downcast_ref::<&str>()).is_equal_to(Some(&"original panic"));
    }
}
