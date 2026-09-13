//! Assertions for consuming iterators, borrowed iteration, and exact remaining counts.

mod exact_size;
mod into_iterator;
mod iterator;

pub use exact_size::{
    ExactSizeIteratorAssertions, HasNoRemainingElements, HasRemainingCount, HasRemainingElements,
};
pub use into_iterator::IntoIteratorAssertions;
pub use iterator::IteratorAssertions;

/// Cross-cutting streaming and diagnostics behavior shared by the iterator assertion traits.
/// Per-method success and failure-message tests live next to each trait implementation.
#[cfg(test)]
mod tests {
    use core::cell::Cell;
    use std::sync::{Arc, Mutex};

    use crate::{
        Fact, assertions::collection::Collection, prelude::*, renderer::CollectionPresentation,
    };

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

    struct ObservedView<T, F> {
        values: [T; 1],
        observe: F,
    }

    impl<T, F: Fn()> AsRef<[T]> for ObservedView<T, F> {
        fn as_ref(&self) -> &[T] {
            (self.observe)();
            &self.values
        }
    }

    #[test]
    fn sequence_views_are_accessed_after_tracking() {
        for method in 0..5 {
            let conversions = Cell::new(0);
            let failures = assert_that!(()).capture(|root| {
                let expected = ObservedView {
                    values: [9],
                    observe: || {
                        assert_that!(root.state.records.assertion_count()).is_equal_to(1);
                        conversions.set(conversions.get() + 1);
                    },
                };
                let child = root.derive_owned(|()| [1].into_iter());
                match method {
                    0 => child.starts_with(expected),
                    1 => child.ends_with(expected),
                    2 => child.contains_contiguous(expected),
                    3 => child.contains_exactly(expected),
                    _ => child.contains_exactly_in_any_order(expected),
                };
                assert_that!(root.state.records.assertion_count()).is_equal_to(1);
                root
            });
            assert_that!(conversions.get()).is_greater_than(0);
            assert_that!(failures).has_length(1);
        }
    }

    #[test]
    fn borrowed_bulk_scan_tracks_before_accessing_either_input() {
        struct Input<F> {
            values: [i32; 1],
            observe: F,
        }
        impl<'a, F: Fn()> IntoIterator for &'a Input<F> {
            type Item = &'a i32;
            type IntoIter = core::slice::Iter<'a, i32>;
            fn into_iter(self) -> Self::IntoIter {
                (self.observe)();
                self.values.iter()
            }
        }
        let failures = assert_that!(()).capture(|root| {
            let observe = || {
                assert_that!(root.state.records.assertion_count()).is_equal_to(1);
            };
            let expected = ObservedView {
                values: [9],
                observe,
            };
            root.derive_owned(|()| Input {
                values: [1],
                observe,
            })
            .into_iter_contains_all(expected);
            root
        });
        assert_that!(failures).has_length(1);
    }

    #[test]
    fn callback_views_are_converted_once_after_tracking() {
        for method in 0..6 {
            let conversions = Cell::new(0);
            let root = assert_that!(());
            let assertions = ObservedView {
                values: [|it: AssertThat<i32, Capture>| {
                    it.is_equal_to(1);
                }],
                observe: || {
                    assert_that!(root.state.records.assertion_count()).is_equal_to(1);
                    conversions.set(conversions.get() + 1);
                },
            };
            if method == 5 {
                root.derive_owned(|()| [1])
                    .into_iter_contains_exactly_in_any_order_satisfying(assertions);
            } else {
                let child = root.derive_owned(|()| [1].into_iter());
                match method {
                    0 => child.starts_with_satisfying(assertions),
                    1 => child.ends_with_satisfying(assertions),
                    2 => child.contains_contiguous_satisfying(assertions),
                    3 => child.contains_exactly_satisfying(assertions),
                    _ => child.contains_exactly_in_any_order_satisfying(assertions),
                };
            }
            assert_that!(conversions.get()).is_equal_to(1);
            assert_that!(root.state.records.assertion_count()).is_equal_to(1);
        }
    }

    #[test]
    fn equality_rejections_do_not_repeat_comparisons() {
        #[derive(Debug)]
        struct Actual<'a>(&'a Cell<usize>);

        impl PartialEq for Actual<'_> {
            fn eq(&self, _: &Self) -> bool {
                self.0.set(self.0.get() + 1);
                false
            }
        }

        for method in 0..3 {
            let calls = Cell::new(0);
            let failures =
                assert_that_owned!([Actual(&calls)].into_iter()).capture(|it| match method {
                    0 => it.starts_with([Actual(&calls)]),
                    1 => it.ends_with([Actual(&calls)]),
                    _ => it.contains_exactly([Actual(&calls)]),
                });
            assert_that!(calls.get()).is_equal_to(1);
            assert_that!(failures[0].children).has_length(1);
            assert_that!(failures[0].children[0].kind)
                .is_equal_to(crate::failure::FailureKind::Equality);
        }
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
        let failure = ToHumanReadableText.render(&failures[0]);
        assert_that!(failure.as_str())
            .contains("last 16 consumed elements")
            .contains("84,")
            .contains("99,")
            .does_not_contain("83,")
            .contains("Decisive index: 99");
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

        assert_that!(&failures).contains_exactly_satisfying([
            |element: AssertThat<AssertionFailure, Capture>| {
                element
                    .derive_owned(|value| value.messages.as_slice())
                    .contains_exactly(["user context"]);
                element
                    .derive_owned(|value| value.facts.as_slice())
                    .does_not_contain_matching(crate::expectation::predicate(|it: &Fact| {
                        it.label == "Decisive index"
                    }));
            },
            |element: AssertThat<AssertionFailure, Capture>| {
                element
                    .derive_owned(|item| item.messages.as_slice())
                    .contains_exactly(["user context"]);
                element
                    .derive_owned(|item| item.facts.as_slice())
                    .contains(Fact::labelled(
                        "Consumed elements",
                        crate::renderer::RenderingContext::new(
                            &DebugRenderer,
                            RenderingBudget::default(),
                        )
                        .value(&3_usize),
                    ))
                    .does_not_contain_matching(crate::expectation::predicate(|it: &Fact| {
                        it.label == "Decisive index"
                    }));
            },
        ]);
    }

    #[test]
    fn value_operands_are_accessed_after_tracking_for_every_streaming_scan() {
        use crate::test_support::BorrowSpy;
        for method in 0..11 {
            let calls = Cell::new(0);
            let failures = assert_that!(()).capture(|root| {
                let expected = BorrowSpy {
                    value: if method == 1 || method == 8 { 2 } else { 9 },
                    observe: || {
                        assert_that!(root.state.records.assertion_count()).is_equal_to(1);
                        calls.set(calls.get() + 1);
                    },
                };
                if method < 7 {
                    let it = root.derive_owned(|()| [1, 2, 3].into_iter());
                    match method {
                        0 => it.contains(expected),
                        1 => it.does_not_contain(expected),
                        2 => it.starts_with([expected]),
                        3 => it.ends_with([expected]),
                        4 => it.contains_contiguous([expected]),
                        5 => it.contains_exactly([expected]),
                        _ => it.contains_exactly_in_any_order([expected]),
                    };
                } else {
                    let it = root.derive_owned(|()| [1, 2, 3]);
                    match method {
                        7 => it.into_iter_contains(expected),
                        8 => it.into_iter_does_not_contain(expected),
                        9 => it.into_iter_contains_all([expected]),
                        _ => it.into_iter_contains_exactly_in_any_order([expected]),
                    };
                }
                root
            });
            if matches!(method, 0 | 1 | 7 | 8) {
                assert_that!(calls.get()).is_equal_to(1);
            } else {
                assert_that!(calls.get()).is_greater_than(0);
            }
            assert_that!(failures).has_length(1);
        }
    }

    #[test]
    fn non_copy_expected_values_can_be_reused_by_streaming_scans() {
        let a = String::from("a");
        let b = String::from("b");
        assert_that_owned!([String::from("a")].into_iter()).contains(&a);
        assert_that_owned!([String::from("a")].into_iter()).does_not_contain(&b);
        assert_that_owned!([String::from("a")].into_iter()).starts_with([&a]);
        assert_that_owned!([String::from("a")].into_iter()).ends_with([&a]);
        assert_that_owned!([String::from("a")].into_iter()).contains_contiguous([&a]);
        assert_that_owned!([String::from("a")].into_iter()).contains_exactly([&a]);
        assert_that_owned!([String::from("a")].into_iter()).contains_exactly_in_any_order([&a]);
        assert_that!([String::from("a")])
            .into_iter_contains(&a)
            .into_iter_does_not_contain(&b)
            .into_iter_contains_all([&a])
            .into_iter_contains_exactly_in_any_order([&a]);
        assert_that_owned!([&a].into_iter()).contains(&a);
    }
}
