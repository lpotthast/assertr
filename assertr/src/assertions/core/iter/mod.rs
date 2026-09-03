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
    use std::sync::{Arc, Mutex};

    use crate::assertions::collection::Collection;
    use crate::prelude::*;
    use crate::renderer::CollectionPresentation;

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

    impl HasLength for Counted<'_> {
        fn length(&self) -> usize {
            self.values.len()
        }
    }

    impl Collection for Counted<'_> {
        type Item = i32;
        const PRESENTATION: CollectionPresentation = CollectionPresentation::list();

        fn elements(&self) -> impl Iterator<Item = &i32> {
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
            .into_iter_contains_all([1, 3])
            .into_iter_does_not_contain(4)
            .into_iter_has_length(3);
        assert_that!(values.calls.get()).is_equal_to(4);
    }

    #[test]
    fn decisive_streaming_assertions_stop_immediately() {
        let calls = Cell::new(0);
        assert_that_owned!(core::iter::from_fn(|| {
            let value = calls.get();
            calls.set(value + 1);
            Some(value)
        }))
        .contains(2);
        assert_that!(calls.get()).is_equal_to(3);

        let calls = Cell::new(0);
        let failures = assert_that_owned!(core::iter::from_fn(|| {
            let value = calls.get();
            calls.set(value + 1);
            Some(value)
        }))
        .capture(|it| it.does_not_contain(2));
        assert_that!(calls.get()).is_equal_to(3);
        assert_that!(failures).has_length(1);

        let calls = Cell::new(0);
        assert_that_owned!(core::iter::from_fn(|| {
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
        assert_that_owned!(iterator).starts_with::<i32>([]);
        assert_that!(calls.get()).is_equal_to(0);

        let calls = Cell::new(0);
        let iterator = core::iter::from_fn(|| {
            calls.set(calls.get() + 1);
            Some(1)
        });
        assert_that_owned!(iterator).ends_with::<i32>([]);
        assert_that!(calls.get()).is_equal_to(0);

        let calls = Cell::new(0);
        let iterator = core::iter::from_fn(|| {
            calls.set(calls.get() + 1);
            Some(1)
        });
        assert_that_owned!(iterator).contains_contiguous::<i32>([]);
        assert_that!(calls.get()).is_equal_to(0);
    }

    #[test]
    fn exact_assertions_consume_at_most_expected_length_plus_one() {
        let calls = Cell::new(0);
        let failures = assert_that_owned!(core::iter::from_fn(|| {
            let value = calls.get();
            calls.set(value + 1);
            Some(value)
        }))
        .capture(|it| it.contains_exactly([0, 9, 2]));
        assert_that!(calls.get()).is_equal_to(2);
        assert_that!(failures).has_length(1);

        let calls = Cell::new(0);
        let failures = assert_that_owned!(core::iter::from_fn(|| {
            let value = calls.get();
            calls.set(value + 1);
            Some(value)
        }))
        .capture(|it| it.contains_exactly_in_any_order([0, 1, 2]));
        assert_that!(calls.get()).is_equal_to(4);
        assert_that!(failures).has_length(1);
    }

    #[test]
    fn failure_preview_is_capped_and_retains_the_decisive_item() {
        let failures = assert_that_owned!(0..100)
            .with_location(false)
            .capture(|it| it.does_not_contain(99));
        let failure = failures[0].to_string();
        assert_that!(failure.as_str())
            .contains("last 16 consumed elements")
            .contains("84,")
            .contains("99,")
            .does_not_contain("83,")
            .contains("zero-based index 99");
    }

    #[test]
    fn failure_locations_point_at_the_callers_assertion() {
        let failures = assert_that_owned!([1, 2, 3].into_iter()).capture(|it| it.contains(9));
        assert_that!(failures[0].location.expect("present").file()).contains("core/iter/mod.rs");

        let failures = assert_that!(vec![1, 2, 3]).capture(|it| it.into_iter_contains(9));
        assert_that!(failures[0].location.expect("present").file()).contains("core/iter/mod.rs");
    }

    #[test]
    fn borrowed_iterator_ownership_panic_points_at_the_callers_assertion() {
        let panic_location = Arc::new(Mutex::new(None));
        let panic_location_for_hook = Arc::clone(&panic_location);
        let test_thread = std::thread::current().id();
        let previous_hook: Arc<dyn Fn(&std::panic::PanicHookInfo<'_>) + Send + Sync> =
            Arc::from(std::panic::take_hook());
        let previous_hook_for_hook = Arc::clone(&previous_hook);

        std::panic::set_hook(Box::new(move |panic| {
            if std::thread::current().id() == test_thread {
                *panic_location_for_hook.lock().expect("not poisoned") = panic
                    .location()
                    .map(|location| (location.file().to_owned(), location.line()));
            } else {
                previous_hook_for_hook(panic);
            }
        }));

        let iterator = [1, 2, 3].into_iter();
        let expected_line = line!() + 1;
        let result = std::panic::catch_unwind(|| assert_that!(iterator).contains(2));

        std::panic::set_hook(Box::new(move |panic| previous_hook(panic)));

        assert_that!(result.is_err()).is_true();
        let location = panic_location.lock().expect("not poisoned").clone();
        let (file, line) = location.expect("panic location");
        assert_that!(file.as_str()).contains("core/iter/mod.rs");
        assert_that!(line).is_equal_to(expected_line);
    }

    #[test]
    fn capture_mode_scopes_assertion_details_to_their_failure() {
        let failures = assert_that!(vec![1, 2, 3])
            .with_location(false)
            .with_detail_message("user context")
            .capture(|it| it.into_iter_does_not_contain(2).into_iter_contains(9));

        assert_that!(&failures).has_length(2);
        assert_that!(failures[0].messages.as_slice()).contains_exactly(["user context"]);
        assert_that!(failures[0].details.as_slice())
            .does_not_contain_matching(|it: &String| it.contains("Decisive element"));
        assert_that!(failures[1].messages.as_slice()).contains_exactly(["user context"]);
        assert_that!(failures[1].details.as_slice())
            .contains_matching(|it: &String| it.contains("Consumed 3 element(s)."))
            .does_not_contain_matching(|it: &String| it.contains("Decisive element"));
    }
}
