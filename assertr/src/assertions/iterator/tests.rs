//! Resource lifetime and consumption contracts shared by every streaming family.

use core::{
    cell::{Cell, RefCell, RefMut},
    fmt,
};

use crate::{failure::AssertionFailures, matchers::eq, prelude::*};

#[derive(Default)]
struct State {
    resource: RefCell<()>,
    next: Cell<usize>,
    hints: Cell<usize>,
    drops: Cell<usize>,
    renders: Cell<usize>,
    iterations: Cell<usize>,
    callbacks: Cell<usize>,
    clones: Cell<usize>,
}

struct Observed<'a, I> {
    inner: I,
    state: &'a State,
    exact_hint: bool,
    _resource: RefMut<'a, ()>,
}

impl<I: Iterator> Iterator for Observed<'_, I> {
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        self.state.next.set(self.state.next.get() + 1);
        self.inner.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.state.hints.set(self.state.hints.get() + 1);
        if self.exact_hint {
            self.inner.size_hint()
        } else {
            (0, None)
        }
    }
}

impl<I> Drop for Observed<'_, I> {
    fn drop(&mut self) {
        self.state.drops.set(self.state.drops.get() + 1);
    }
}

impl State {
    fn observe<I>(&self, inner: I, exact_hint: bool) -> Observed<'_, I> {
        self.iterations.set(self.iterations.get() + 1);
        Observed {
            inner,
            state: self,
            exact_hint,
            _resource: self.resource.borrow_mut(),
        }
    }

    fn verify(&self, next: usize, hints: usize) {
        assert_that!(self.next.get()).is_equal_to(next);
        assert_that!(self.hints.get()).is_equal_to(hints);
        assert_that!(self.iterations.get()).is_equal_to(1);
        assert_that!(self.drops.get()).is_equal_to(1);
        assert_that!(self.resource.try_borrow_mut()).is_ok();
    }
}

struct ResourceRenderer<'a>(&'a State);

impl Clone for ResourceRenderer<'_> {
    fn clone(&self) -> Self {
        self.0.clones.set(self.0.clones.get() + 1);
        Self(self.0)
    }
}

fn callback<'s>(
    state: &'s State,
    expected: i32,
) -> impl for<'a> Fn(AssertThat<'a, i32, Capture, ResourceRenderer<'s>>) {
    move |it| {
        state.callbacks.set(state.callbacks.get() + 1);
        it.is_equal_to(expected);
    }
}

impl ResourceRenderer<'_> {
    fn render(&self, value: impl fmt::Display, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // The items are plain numbers. Their interpretation still needs the iterator's resource.
        assert_that!(self.0.drops.get() + 1).is_equal_to(self.0.iterations.get());
        assert_that!(self.0.resource.try_borrow_mut()).is_err();
        self.0.renders.set(self.0.renders.get() + 1);
        write!(f, "resource({value})")
    }
}

impl ValueRenderer<i32> for ResourceRenderer<'_> {
    fn fmt(&self, value: &i32, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.render(value, f)
    }
}

impl ValueRenderer<usize> for ResourceRenderer<'_> {
    fn fmt(&self, value: &usize, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.render(value, f)
    }
}

#[derive(Clone, Copy, Debug)]
enum Operation {
    Contains,
    Reject,
    Prefix,
    Suffix,
    Contiguous,
    Exact,
    Unordered,
}

#[derive(Clone, Copy, Debug)]
enum Entry {
    Equality,
    Matcher,
    Callback,
}
use Entry::{Callback, Equality, Matcher};

fn verify_failure(failures: &AssertionFailures, state: &State, failed: bool) {
    assert_that!(failures.len()).is_equal_to(usize::from(failed));
    if failed {
        assert_that!(state.renders.get()).is_greater_than(0);
        let renders = state.renders.get();
        // A completed report no longer needs either the items or the iterator's resource.
        assert_that!(failures[0].to_string()).contains("resource(");
        assert_that!(state.renders.get()).is_equal_to(renders);
    }
}

mod direct {
    use Operation::{Contains, Contiguous, Exact, Prefix, Reject, Suffix, Unordered};

    use super::*;

    type Case<'a> = (Operation, &'a [i32], &'a [i32], usize, bool, Option<usize>);

    // Each row pins next calls for unknown hints, the outcome, and every stopping boundary.
    const CASES: &[Case<'_>] = &[
        (Contains, &[], &[2], 1, true, Some(0)),
        (Reject, &[], &[2], 1, false, Some(0)),
        (Contains, &[1, 2, 3], &[2], 2, false, Some(2)),
        (Contains, &[1, 2, 3], &[9], 4, true, Some(3)),
        (Reject, &[1, 2, 3], &[2], 2, true, Some(2)),
        (Reject, &[1, 2, 3], &[9], 4, false, Some(3)),
        (Prefix, &[1, 2, 3], &[1, 2], 2, false, Some(2)),
        (Prefix, &[1, 2, 3], &[9], 1, true, Some(1)),
        (Prefix, &[1], &[1, 2], 2, true, Some(1)),
        (Prefix, &[1, 2, 3], &[], 0, false, Some(0)),
        (Suffix, &[1, 2, 3], &[2, 3], 4, false, Some(2)),
        (Suffix, &[1, 2, 3], &[9], 4, true, Some(1)),
        (Suffix, &[1], &[1, 2], 2, true, Some(0)),
        (Suffix, &[1, 2, 3], &[], 0, false, Some(0)),
        (Contiguous, &[1, 2, 3], &[1, 2], 2, false, Some(2)),
        (Contiguous, &[1, 1, 2], &[1, 2], 3, false, Some(4)),
        (Contiguous, &[0, 1, 2, 3], &[2, 3], 4, false, Some(6)),
        (Contiguous, &[1, 2, 3], &[9], 4, true, Some(3)),
        (Contiguous, &[1], &[1, 2], 2, true, Some(0)),
        (Contiguous, &[1, 2, 3], &[], 0, false, Some(0)),
        (Exact, &[1, 2, 3], &[1, 2, 3], 4, false, Some(3)),
        (Exact, &[1, 2, 3], &[9, 2, 3], 1, true, Some(1)),
        (Exact, &[1, 2, 3], &[1], 2, true, Some(1)),
        (Exact, &[1], &[1, 2], 2, true, Some(1)),
        (Exact, &[], &[], 1, false, Some(0)),
        (Exact, &[1], &[], 1, true, Some(0)),
        (Prefix, &[], &[], 0, false, Some(0)),
        (Suffix, &[], &[], 0, false, Some(0)),
        (Contiguous, &[], &[], 0, false, Some(0)),
        (Unordered, &[], &[], 1, false, Some(0)),
        (Unordered, &[1, 1, 2], &[1, 2, 2], 4, true, None),
        (Unordered, &[1, 2, 3], &[3, 2, 1], 4, false, None),
        (Unordered, &[1, 2, 3], &[9, 2, 1], 4, true, None),
        (Unordered, &[1, 2, 3], &[1], 2, true, None),
        (Unordered, &[1], &[1, 2], 2, true, None),
    ];

    #[test]
    fn scans_keep_resources_until_rendering_and_preserve_stopping_points() {
        for &(operation, values, expected, unknown_next, failed, callback_calls) in CASES {
            for entry in [Equality, Matcher, Callback] {
                for exact_hint in [false, true] {
                    let state = State::default();
                    let iterator = state.observe(values.iter().copied(), exact_hint);
                    let matchers: Vec<_> = expected.iter().copied().map(eq).collect();
                    let callbacks: Vec<_> = expected
                        .iter()
                        .map(|&value| callback(&state, value))
                        .collect();
                    let failures = assert_that_owned!(iterator)
                        .with_renderer(ResourceRenderer(&state))
                        .capture(|it| match (operation, entry) {
                            (Contains, Equality) => it.contains(expected[0]),
                            (Contains, Matcher) => it.contains_matching(eq(expected[0])),
                            (Reject, Equality) => it.does_not_contain(expected[0]),
                            (Reject, Matcher) => it.does_not_contain_matching(eq(expected[0])),
                            (Prefix, Equality) => it.starts_with(expected),
                            (Prefix, Matcher) => it.starts_with_matching(matchers),
                            (Suffix, Equality) => it.ends_with(expected),
                            (Suffix, Matcher) => it.ends_with_matching(matchers),
                            (Contiguous, Equality) => it.contains_contiguous(expected),
                            (Contiguous, Matcher) => it.contains_contiguous_matching(matchers),
                            (Exact, Equality) => it.contains_exactly(expected),
                            (Exact, Matcher) => it.contains_exactly_matching(matchers),
                            (Unordered, Equality) => it.contains_exactly_in_any_order(expected),
                            (Unordered, Matcher) => {
                                it.contains_exactly_in_any_order_matching(matchers)
                            }
                            (Contains, Callback) => it.contains_satisfying(&callbacks[0]),
                            (Reject, Callback) => it.does_not_contain_satisfying(&callbacks[0]),
                            (Prefix, Callback) => it.starts_with_satisfying(&callbacks),
                            (Suffix, Callback) => it.ends_with_satisfying(&callbacks),
                            (Contiguous, Callback) => it.contains_contiguous_satisfying(&callbacks),
                            (Exact, Callback) => it.contains_exactly_satisfying(&callbacks),
                            (Unordered, Callback) => {
                                it.contains_exactly_in_any_order_satisfying(&callbacks)
                            }
                        });
                    let short_circuit = exact_hint
                        && match operation {
                            Prefix => values.len() < expected.len(),
                            Exact | Unordered => values.len() != expected.len(),
                            _ => false,
                        };
                    let hints = usize::from(matches!(operation, Prefix | Exact | Unordered));
                    state.verify(if short_circuit { 0 } else { unknown_next }, hints);
                    verify_failure(&failures, &state, failed);
                    assert_that!(state.clones.get()).is_equal_to(2 * state.callbacks.get());
                    let callback_calls = if short_circuit || !matches!(entry, Callback) {
                        Some(0)
                    } else {
                        callback_calls
                    };
                    let calls = assert_that_owned!(state.callbacks.get())
                        .with_detail_message(format!("{operation:?}, {entry:?}, exact hint: {exact_hint}, values: {values:?}, expected: {expected:?}"));
                    if let Some(expected) = callback_calls {
                        calls.is_equal_to(expected);
                    } else {
                        // Unordered assignment owns pair scheduling and its once-per-pair tests.
                        calls.is_greater_than(0);
                    }
                }
            }
        }
    }
}

mod direct_counting {
    use super::*;

    #[test]
    fn contains_all_stops_on_success_or_exhaustion() {
        for (expected, next, failed) in [
            (&[2][..], 2, false),
            (&[1, 3][..], 3, false),
            (&[1, 1, 1][..], 1, false),
            (&[9][..], 4, true),
            (&[][..], 0, false),
        ] {
            let state = State::default();
            let iterator = state.observe([1, 2, 3].into_iter(), false);
            let failures = assert_that_owned!(iterator)
                .with_renderer(ResourceRenderer(&state))
                .capture(|it| it.contains_all(expected));
            state.verify(next, 0);
            verify_failure(&failures, &state, failed);
        }
    }

    #[test]
    fn cardinality_keeps_resources_through_explanation_without_repeating_observations() {
        for values in [&[][..], &[1, 2, 3][..]] {
            for exact_hint in [false, true] {
                for operation in 0..3 {
                    let state = State::default();
                    let iterator = state.observe(values.iter().copied(), exact_hint);
                    let failures = assert_that_owned!(iterator)
                        .with_renderer(ResourceRenderer(&state))
                        .capture(|it| match operation {
                            0 => it.is_exhausted(),
                            1 => it.is_not_exhausted(),
                            _ => it.has_count(1),
                        });
                    let next = if operation < 2 {
                        1
                    } else if exact_hint {
                        0
                    } else {
                        values.len().min(1) + 1
                    };
                    state.verify(next, usize::from(operation == 2));
                    let failed = match operation {
                        0 => !values.is_empty(),
                        1 => values.is_empty(),
                        _ => true,
                    };
                    assert_that!(failures.len()).is_equal_to(usize::from(failed));
                    if failed && operation != 1 {
                        verify_failure(&failures, &state, true);
                    }
                }
            }
        }
    }
}

mod release {
    use std::sync::{Arc, Mutex, MutexGuard};

    use super::*;

    struct Guarded<'a> {
        _guard: MutexGuard<'a, ()>,
    }

    impl Iterator for Guarded<'_> {
        type Item = i32;
        fn next(&mut self) -> Option<i32> {
            Some(1)
        }
    }

    #[test]
    fn panic_routing_releases_the_iterator_before_presentation_without_poisoning() {
        let resource = Arc::new(Mutex::new(()));
        let text = std::panic::catch_unwind(|| {
            assert_that_owned!(Guarded {
                _guard: resource.lock().unwrap()
            })
            .with_panic_presentation({
                let resource = Arc::clone(&resource);
                move |_| {
                    String::from(if resource.try_lock().is_ok() {
                        "iterator released before presentation"
                    } else {
                        "iterator still holds its guard"
                    })
                }
            })
            .does_not_contain(1);
        })
        .unwrap_err()
        .downcast::<String>()
        .unwrap();
        assert_that!(*text).is_equal_to("iterator released before presentation");
        assert_that!(resource.is_poisoned()).is_false();
        assert_that!(resource.try_lock()).is_ok();
    }

    #[test]
    fn candidate_observations_are_not_repeated_and_guards_are_released_between_candidates() {
        for expected in [2, 9] {
            let state = State::default();
            let calls = Cell::new(0);
            let candidate_guard = RefCell::new(());
            let matcher = matchers::predicate(|value: &i32| {
                let _guard = candidate_guard.borrow_mut();
                calls.set(calls.get() + 1);
                *value == expected
            });
            let iterator = state.observe([1, 2, 3].into_iter(), false);
            let failures = assert_that_owned!(iterator)
                .with_renderer(ResourceRenderer(&state))
                .capture(|it| it.contains_matching(matcher));
            assert_that!(calls.get()).is_equal_to(if expected == 2 { 2 } else { 3 });
            assert_that!(candidate_guard.try_borrow_mut()).is_ok();
            state.verify(if expected == 2 { 2 } else { 4 }, 0);
            verify_failure(&failures, &state, expected == 9);
        }
    }
}

mod string_views {
    use crate::prelude::*;

    #[test]
    fn all_streaming_comparisons_accept_literal_operands() {
        fn values() -> core::array::IntoIter<String, 3> {
            [String::from("a"), String::from("b"), String::from("a")].into_iter()
        }
        assert_that_owned!(values()).contains("b");
        assert_that_owned!(values()).does_not_contain("c");
        assert_that_owned!(values()).starts_with(["a", "b"]);
        assert_that_owned!(values()).ends_with(["b", "a"]);
        assert_that_owned!(values()).contains_contiguous(["b", "a"]);
        assert_that_owned!(values()).contains_exactly(["a", "b", "a"]);
        assert_that_owned!(values()).contains_exactly_in_any_order(["a", "a", "b"]);
        assert_that_owned!(values()).contains_all(["a", "b"]);
        let borrowed = [String::from("a"), String::from("b")];
        assert_that_owned!(borrowed.iter()).contains("a");
    }

    #[test]
    fn non_copy_expected_values_can_be_reused_by_streaming_scans() {
        let a = String::from("a");
        let c = String::from("c");
        let values = || [String::from("a")].into_iter();
        assert_that_owned!(values()).contains(&a);
        assert_that_owned!(values()).does_not_contain(&c);
        assert_that_owned!(values()).starts_with([&a]);
        assert_that_owned!(values()).ends_with([&a]);
        assert_that_owned!(values()).contains_contiguous([&a]);
        assert_that_owned!(values()).contains_exactly([&a]);
        assert_that_owned!(values()).contains_exactly_in_any_order([&a]);
        assert_that_owned!(values()).contains_all([&a]);
        assert_that_owned!([&a].into_iter()).contains(&a);
    }
}

mod callbacks {
    use super::*;
    use crate::{
        failure::{FailureKind, PathSegment},
        test_support::assert_custom_value,
    };

    struct Opaque(i32);

    struct Renderer<'a>(&'a Cell<usize>);

    impl Clone for Renderer<'_> {
        fn clone(&self) -> Self {
            self.0.set(self.0.get() + 1);
            Self(self.0)
        }
    }

    impl<T: fmt::Debug + ?Sized> ValueRenderer<T> for Renderer<'_> {
        fn fmt(&self, value: &T, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "custom({value:?})")
        }
    }

    #[derive(Clone, Copy, Debug)]
    enum Adapter {
        Iterator,
        StableOrder,
    }

    fn equality_failures<'a>(
        failure: &'a AssertionFailure,
        leaves: &mut Vec<&'a AssertionFailure>,
    ) {
        if failure.kind == FailureKind::Equality {
            leaves.push(failure);
        }
        for child in &failure.children {
            equality_failures(child, leaves);
        }
    }

    #[test]
    fn adapters_propagate_callback_evidence_and_clone_the_renderer_for_opaque_items() {
        use Adapter::{Iterator, StableOrder};
        use Operation::{Contains, Contiguous, Exact, Prefix, Suffix, Unordered};

        // Each adapter runs one candidate. The callback has two failing assertions on a
        // projection. Capture clones for its root and child, then the projection clones again.
        for (adapter, operation) in [
            (Iterator, Contains),
            (Iterator, Prefix),
            (Iterator, Suffix),
            (Iterator, Contiguous),
            (Iterator, Exact),
            (Iterator, Unordered),
            (StableOrder, Prefix),
            (StableOrder, Suffix),
            (StableOrder, Contiguous),
            (StableOrder, Exact),
        ] {
            for maximum in [1, 2] {
                let clones = Cell::new(0);
                let calls = Cell::new(0);
                let values = [Opaque(1)];
                let check = |it: AssertThat<'_, Opaque, Capture, Renderer<'_>>| {
                    calls.set(calls.get() + 1);
                    it.derive(|item| &item.0).is_equal_to(9).is_equal_to(10);
                };
                let budget = RenderingBudget::default().with_max_items(maximum);
                let failures = if matches!(adapter, Iterator) {
                    assert_that_owned!(values.into_iter())
                        .with_renderer(Renderer(&clones))
                        .with_rendering_budget(budget)
                        .with_location(false)
                        .capture(|it| match operation {
                            Contains => it.contains_satisfying(check),
                            Prefix => it.starts_with_satisfying([check]),
                            Suffix => it.ends_with_satisfying([check]),
                            Contiguous => it.contains_contiguous_satisfying([check]),
                            Exact => it.contains_exactly_satisfying([check]),
                            Unordered => it.contains_exactly_in_any_order_satisfying([check]),
                            Operation::Reject => unreachable!(),
                        })
                } else {
                    assert_that!(values)
                        .with_renderer(Renderer(&clones))
                        .with_rendering_budget(budget)
                        .with_location(false)
                        .capture(|it| match (adapter, operation) {
                            (StableOrder, Prefix) => it.starts_with_satisfying([check]),
                            (StableOrder, Suffix) => it.ends_with_satisfying([check]),
                            (StableOrder, Contiguous) => it.contains_contiguous_satisfying([check]),
                            (StableOrder, Exact) => it.contains_exactly_satisfying([check]),
                            _ => unreachable!(),
                        })
                };
                // Unordered assignment and the collection's contiguous search probe first, then
                // evaluate again to explain the rejection. A streaming iterator cannot rescan.
                let probes_first = matches!(operation, Unordered)
                    || matches!((adapter, operation), (StableOrder, Contiguous));
                let evaluations = if probes_first { 2 } else { 1 };
                assert_that!(calls.get()).is_equal_to(evaluations);
                assert_that!(clones.get()).is_equal_to(3 * evaluations);
                assert_that!(failures).has_length(1);
                let mut leaves = Vec::new();
                equality_failures(&failures[0], &mut leaves);
                assert_that!(leaves.len()).is_equal_to(maximum);
                for (index, leaf) in leaves.into_iter().enumerate() {
                    assert_custom_value(leaf.actual.as_ref().unwrap(), &1_i32);
                    assert_custom_value(
                        leaf.expected.as_ref().unwrap(),
                        &(9 + i32::try_from(index).unwrap()),
                    );
                    assert_that!(leaf.location).is_none();
                    let path = if matches!(operation, Unordered) {
                        vec![]
                    } else {
                        vec![PathSegment::Index(0)]
                    };
                    assert_that!(leaf.path).is_equal_to(path);
                }
                if maximum == 1 {
                    assert_that!(failures[0].to_string()).contains("1 more");
                }
            }
        }
    }
}

mod tracking {
    use core::cell::Cell;

    use crate::prelude::*;

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
        for method in 0..4 {
            let conversions = Cell::new(0);
            let failures = assert_that!(()).capture(|root| {
                let observe = || {
                    assert_that!(root.state.records.assertion_count()).is_equal_to(1);
                    conversions.set(conversions.get() + 1);
                };
                let expected = ObservedView {
                    values: [9],
                    observe,
                };
                let callbacks = ObservedView {
                    values: [|it: AssertThat<i32, Capture>| {
                        it.is_equal_to(9);
                    }],
                    observe,
                };
                let direct = || root.derive_owned(|()| [1].into_iter());
                match method {
                    0 => drop(direct().ends_with(expected)),
                    1 => drop(direct().contains_exactly_satisfying(callbacks)),
                    2 => drop(direct().contains_exactly_in_any_order(expected)),
                    _ => drop(direct().contains_all(expected)),
                }
                assert_that!(root.state.records.assertion_count()).is_equal_to(1);
                root
            });
            assert_that!(conversions.get()).is_greater_than(0);
            assert_that!(failures).has_length(1);
        }
    }
}

mod reporting {
    use std::sync::{Arc, Mutex};

    use crate::{failure::Fact, prelude::*};

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
            .contains("Decisive index: 99");
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
        assert_that!(file.as_str()).contains("iterator/tests.rs");
        assert_that!(line).is_equal_to(expected_line);
    }

    #[test]
    fn capture_mode_scopes_assertion_details_to_their_failure() {
        let failures = assert_that!(vec![1, 2, 3])
            .with_location(false)
            .with_detail_message("user context")
            .capture(|it| {
                it.derive_owned(|values| values.clone().into_iter())
                    .does_not_contain(2);
                it.derive_owned(|values| values.clone().into_iter())
                    .contains(9);
                it
            });

        assert_that!(&failures).contains_exactly_satisfying([
            |element: AssertThat<AssertionFailure, Capture>| {
                element
                    .derive_owned(|value| value.messages.as_slice())
                    .contains_exactly(["user context"]);
                element
                    .derive_owned(|value| value.facts.as_slice())
                    .contains_matching(matchers::predicate(|it: &Fact| {
                        it.label.as_deref() == Some("Decisive index")
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
                    .does_not_contain_matching(matchers::predicate(|it: &Fact| {
                        it.label.as_deref() == Some("Decisive index")
                    }));
            },
        ]);
    }
}
