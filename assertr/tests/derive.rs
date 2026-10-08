//! Projections end their borrow of the subject at their last use, not when the chain is dropped.

use assertr::prelude::*;

#[test]
fn extracting_does_not_retain_the_subject_borrow_until_the_context_is_dropped() {
    let mut values = vec![1];
    let assertion = assert_that!(values);
    assertion.get_first().is_equal_to(1);

    // Keep `assertion` in scope. Its last use, rather than its drop, must end the borrow.
    values.push(2);
    assert_that!(values).contains_exactly([1, 2]);
}

#[test]
fn deriving_does_not_retain_the_subject_borrow_until_the_context_is_dropped() {
    let mut values = vec![1];
    let assertion = assert_that!(values);
    assertion.derive_owned(Vec::len).is_equal_to(1);

    values.push(2);
    assert_that!(values).contains_exactly([1, 2]);
}
