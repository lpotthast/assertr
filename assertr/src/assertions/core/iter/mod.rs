//! Assertions for consuming iterators, borrowed iteration, and exact remaining counts.

mod exact_size;
mod into_iterator;
mod iterator;

pub use exact_size::ExactSizeIteratorAssertions;
pub use into_iterator::IntoIteratorAssertions;
pub use iterator::IteratorAssertions;

/// Cross-cutting streaming and diagnostics behavior shared by the iterator assertion traits.
/// Per-method success and failure-message tests live next to each trait implementation.
#[cfg(test)]
mod tests {
    use core::cell::Cell;

    use crate::prelude::*;

    struct Counted<'a> {
        values: &'a [i32],
        calls: Cell<usize>,
    }

    impl<'b> IntoIterator for &'b Counted<'_> {
        type Item = &'b i32;
        type IntoIter = core::slice::Iter<'b, i32>;

        fn into_iter(self) -> Self::IntoIter {
            self.calls.set(self.calls.get() + 1);
            self.values.iter()
        }
    }

    #[test]
    fn each_borrowed_assertion_creates_exactly_one_fresh_iterator() {
        let values = Counted {
            values: &[1, 2, 3],
            calls: Cell::new(0),
        };
        assert_that!(&values)
            .into_iter_contains(2)
            .into_iter_does_not_contain(4)
            .into_iter_has_length(3)
            .into_iter_starts_with([1])
            .into_iter_ends_with([3])
            .into_iter_contains_contiguous([2, 3]);
        assert_that!(values.calls.get()).is_equal_to(6);
    }

    #[test]
    fn decisive_streaming_assertions_stop_immediately() {
        let calls = Cell::new(0);
        assert_that!(core::iter::from_fn(|| {
            let value = calls.get();
            calls.set(value + 1);
            Some(value)
        }))
        .contains(2);
        assert_that!(calls.get()).is_equal_to(3);

        let calls = Cell::new(0);
        let failures = assert_that!(core::iter::from_fn(|| {
            let value = calls.get();
            calls.set(value + 1);
            Some(value)
        }))
        .with_capture()
        .does_not_contain(2)
        .capture_failures();
        assert_that!(calls.get()).is_equal_to(3);
        assert_that!(failures).has_length(1);

        let calls = Cell::new(0);
        assert_that!(core::iter::from_fn(|| {
            let value = calls.get();
            calls.set(value + 1);
            Some(value)
        }))
        .contains_contiguous([2, 3]);
        assert_that!(calls.get()).is_equal_to(4);
    }

    #[test]
    fn empty_sequence_criteria_are_decisive_without_consumption() {
        let calls = Cell::new(0);
        let iterator = core::iter::from_fn(|| {
            calls.set(calls.get() + 1);
            Some(1)
        });
        assert_that!(iterator).starts_with::<i32>([]);
        assert_that!(calls.get()).is_equal_to(0);

        let calls = Cell::new(0);
        let iterator = core::iter::from_fn(|| {
            calls.set(calls.get() + 1);
            Some(1)
        });
        assert_that!(iterator).ends_with::<i32>([]);
        assert_that!(calls.get()).is_equal_to(0);

        let calls = Cell::new(0);
        let iterator = core::iter::from_fn(|| {
            calls.set(calls.get() + 1);
            Some(1)
        });
        assert_that!(iterator).contains_contiguous::<i32>([]);
        assert_that!(calls.get()).is_equal_to(0);
    }

    #[test]
    fn exact_assertions_consume_at_most_expected_length_plus_one() {
        let calls = Cell::new(0);
        let failures = assert_that!(core::iter::from_fn(|| {
            let value = calls.get();
            calls.set(value + 1);
            Some(value)
        }))
        .with_capture()
        .contains_exactly([0, 9, 2])
        .capture_failures();
        assert_that!(calls.get()).is_equal_to(2);
        assert_that!(failures).has_length(1);

        let calls = Cell::new(0);
        let failures = assert_that!(core::iter::from_fn(|| {
            let value = calls.get();
            calls.set(value + 1);
            Some(value)
        }))
        .with_capture()
        .contains_exactly_in_any_order([0, 1, 2])
        .capture_failures();
        assert_that!(calls.get()).is_equal_to(4);
        assert_that!(failures).has_length(1);
    }

    #[test]
    fn failure_preview_is_capped_and_retains_the_decisive_item() {
        let failures = assert_that!(0..100)
            .with_location(false)
            .with_capture()
            .does_not_contain(99)
            .capture_failures();
        let failure = &failures[0];
        assert_that!(failure.as_str())
            .contains("last 16 consumed elements")
            .contains("84,")
            .contains("99,")
            .does_not_contain("83,")
            .contains("zero-based index 99");
    }

    #[test]
    fn failure_locations_point_at_the_callers_assertion() {
        let failures = assert_that!([1, 2, 3].into_iter())
            .with_capture()
            .contains(9)
            .capture_failures();
        assert_that!(failures[0].as_str()).contains("core/iter/mod.rs");

        let failures = assert_that!(vec![1, 2, 3])
            .with_capture()
            .into_iter_contains(9)
            .capture_failures();
        assert_that!(failures[0].as_str()).contains("core/iter/mod.rs");
    }

    #[test]
    fn capture_mode_scopes_assertion_details_to_their_failure() {
        let failures = assert_that!(vec![1, 2, 3])
            .with_location(false)
            .with_detail_message("user context")
            .with_capture()
            .into_iter_does_not_contain(2)
            .into_iter_contains(9)
            .capture_failures();

        assert_that!(&failures).has_length(2);
        assert_that!(failures[0].as_str())
            .contains("user context")
            .contains("Decisive element is at zero-based index 1.");
        assert_that!(failures[1].as_str())
            .contains("user context")
            .contains("Consumed 3 element(s).")
            .does_not_contain("Decisive element");
    }
}
