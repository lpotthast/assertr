#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

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

    assert_that_owned!(0..).contains(2);
    assert_that_owned!(1..).contains_satisfying(positive);
    assert_that_owned!(0..).contains_contiguous([2, 3]);
    assert_that_owned!([2, 1].into_iter()).contains_exactly_in_any_order([1, 2]);
    assert_that!([1, 2].into_iter()).has_remaining_count(2);

    assert_that!([1, 2])
        .into_iter_contains_all([2, 1])
        .into_iter_starts_with([1])
        .into_iter_ends_with([2])
        .into_iter_contains_contiguous_satisfying([positive, positive])
        .into_iter_contains_exactly_in_any_order([2, 1]);
}

/// The set and map families live outside the `std` module, so `BTreeSet` and `BTreeMap` carry
/// them into `no_std` builds.
#[allow(dead_code)]
fn set_and_map_assertions_compile_without_std() {
    use alloc::collections::{BTreeMap, BTreeSet};

    #[allow(clippy::trivially_copy_pass_by_ref)]
    fn is_one(value: &i32) -> bool {
        *value == 1
    }

    fn satisfies_one(it: AssertThat<i32, Capture>) {
        it.is_equal_to(1);
    }

    assert_that!(BTreeSet::from([1, 2, 3]))
        .contains(2)
        .does_not_contain(4)
        .contains_all([1, 3])
        .contains_matching(|it: &i32| *it > 2)
        .contains_exactly_in_any_order([3, 2, 1])
        .is_subset_of(BTreeSet::from([1, 2, 3, 4]))
        .is_superset_of(BTreeSet::from([1]))
        .is_disjoint_from(BTreeSet::from([9]))
        .has_length(3);

    assert_that!(BTreeMap::from([("a", 1)]))
        .contains_key("a")
        .does_not_contain_key("b")
        .contains_value(1)
        .contains_entry::<i32, _>("a", 1)
        .contains_entry_satisfying("a", satisfies_one)
        .contains_keys(["a"])
        .contains_exactly_entries([("a", 1)])
        .contains_exactly_entries_matching([("a", is_one)])
        .contains_exactly_entries_satisfying([("a", satisfies_one)])
        .has_length(1);
}

/// A `LinkedList` is an ordered collection, so it gets the order-sensitive assertions too.
#[allow(dead_code)]
fn linked_list_assertions_compile_without_std() {
    use alloc::collections::LinkedList;

    let list = [1, 2, 3].into_iter().collect::<LinkedList<_>>();
    assert_that!(list)
        .contains(2)
        .contains_exactly([1, 2, 3])
        .contains_exactly_in_any_order([3, 2, 1])
        .has_length(3);
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
        let failures = assertr::assert_that_owned!(0..20).capture(|it| it.does_not_contain(19));

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
    fn dropping_an_unused_assertion_during_unwinding_preserves_the_original_panic() {
        let panic = std::panic::catch_unwind(|| {
            let _assertion = assertr::assert_that!(42);
            panic!("original panic");
        })
        .expect_err("the closure should panic");

        assertr::assert_that!(panic.downcast_ref::<&str>()).is_equal_to(Some(&"original panic"));
    }

    #[test]
    // The `if` around the panic keeps the closure's return type inferable; an `assert!` would
    // change the panic payload.
    #[allow(clippy::manual_assert)]
    fn a_panic_inside_a_capture_closure_preserves_the_original_panic() {
        let panic = std::panic::catch_unwind(|| {
            let _failures = assertr::assert_that!(42).capture(|it| {
                let it = it.is_equal_to(43);
                if it.actual() == &42 {
                    panic!("original panic");
                }
                it
            });
        })
        .expect_err("the closure should panic");

        assertr::assert_that!(panic.downcast_ref::<&str>()).is_equal_to(Some(&"original panic"));
    }
}
