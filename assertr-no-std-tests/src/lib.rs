#![cfg_attr(not(feature = "std"), no_std)]

use assertr::prelude::*;

#[allow(dead_code)]
fn pattern_assertions_compile_without_std() {
    assert_that!(Some(42)).is_matching(pattern!(Some(42)));
}

#[cfg(all(test, not(feature = "std")))]
extern crate std;

#[cfg(all(test, not(feature = "std")))]
mod tests {
    use assertr::prelude::PartialEqAssertions;

    #[test]
    fn dropping_an_unused_assertion_does_not_panic() {
        let result = std::panic::catch_unwind(|| {
            let _assertion = assertr::assert_that!(42);
        });

        assert!(result.is_ok());
    }

    #[test]
    fn dropping_an_uncaptured_assertion_does_not_panic() {
        let result = std::panic::catch_unwind(|| {
            let _assertion = assertr::assert_that!(42).with_capture().is_equal_to(43);
        });

        assert!(result.is_ok());
    }

    #[test]
    fn dropping_an_unused_assertion_during_unwinding_preserves_the_original_panic() {
        let panic = std::panic::catch_unwind(|| {
            let _assertion = assertr::assert_that!(42);
            panic!("original panic");
        })
        .expect_err("the closure should panic");

        assert_eq!(panic.downcast_ref::<&str>(), Some(&"original panic"));
    }

    #[test]
    fn dropping_an_uncaptured_assertion_during_unwinding_preserves_the_original_panic() {
        let panic = std::panic::catch_unwind(|| {
            let _assertion = assertr::assert_that!(42).with_capture().is_equal_to(43);
            panic!("original panic");
        })
        .expect_err("the closure should panic");

        assert_eq!(panic.downcast_ref::<&str>(), Some(&"original panic"));
    }
}
