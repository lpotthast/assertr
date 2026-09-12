//! Resource lifetime and consumption contracts shared by every streaming family.

use crate::{AssertionFailures, matchers::eq, prelude::*};
use core::{
    cell::{Cell, RefCell, RefMut},
    fmt,
};

#[derive(Default)]
struct State {
    resource: RefCell<()>,
    next: Cell<usize>,
    hints: Cell<usize>,
    drops: Cell<usize>,
    renders: Cell<usize>,
    iterations: Cell<usize>,
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
        assert_that!(self.resource.try_borrow_mut().is_ok()).is_true();
    }
}

struct ResourceRenderer<'a>(&'a State);

impl ResourceRenderer<'_> {
    fn render(&self, value: impl fmt::Display, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // The items are plain numbers. Their interpretation still needs the iterator's resource.
        assert_that!(self.0.drops.get()).is_equal_to(0);
        assert_that!(self.0.resource.try_borrow_mut().is_err()).is_true();
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

struct Source<'a> {
    values: &'a [i32],
    state: State,
    exact_hint: bool,
}

impl<'a> IntoIterator for &'a Source<'_> {
    type Item = &'a i32;
    type IntoIter = Observed<'a, core::slice::Iter<'a, i32>>;

    fn into_iter(self) -> Self::IntoIter {
        self.state.observe(self.values.iter(), self.exact_hint)
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

fn verify_failure(failures: &AssertionFailures, state: &State, failed: bool) {
    assert_that!(failures.len()).is_equal_to(usize::from(failed));
    if failed {
        assert_that!(state.renders.get()).is_greater_than(0);
        let renders = state.renders.get();
        // A completed report no longer needs either the items or the iterator's resource.
        assert_that!(ToHumanReadableText.render(&failures[0]).as_str()).contains("resource(");
        assert_that!(state.renders.get()).is_equal_to(renders);
    }
}

mod direct {
    use super::*;
    use Operation::{Contains, Contiguous, Exact, Prefix, Reject, Suffix, Unordered};

    type Case<'a> = (Operation, &'a [i32], &'a [i32], usize, bool);

    #[test]
    fn scans_keep_resources_until_rendering_and_preserve_stopping_points() {
        // Each row pins next calls for unknown hints, the outcome, and every stopping boundary.
        let cases: &[Case<'_>] = &[
            (Contains, &[1, 2, 3], &[2], 2, false),
            (Contains, &[1, 2, 3], &[9], 4, true),
            (Reject, &[1, 2, 3], &[2], 2, true),
            (Reject, &[1, 2, 3], &[9], 4, false),
            (Prefix, &[1, 2, 3], &[1, 2], 2, false),
            (Prefix, &[1, 2, 3], &[9], 1, true),
            (Prefix, &[1], &[1, 2], 2, true),
            (Prefix, &[1, 2, 3], &[], 0, false),
            (Suffix, &[1, 2, 3], &[2, 3], 4, false),
            (Suffix, &[1, 2, 3], &[9], 4, true),
            (Suffix, &[1], &[1, 2], 2, true),
            (Suffix, &[1, 2, 3], &[], 0, false),
            (Contiguous, &[1, 2, 3], &[1, 2], 2, false),
            (Contiguous, &[1, 2, 3], &[9], 4, true),
            (Contiguous, &[1], &[1, 2], 2, true),
            (Contiguous, &[1, 2, 3], &[], 0, false),
            (Exact, &[1, 2, 3], &[1, 2, 3], 4, false),
            (Exact, &[1, 2, 3], &[9, 2, 3], 1, true),
            (Exact, &[1, 2, 3], &[1], 2, true),
            (Exact, &[1], &[1, 2], 2, true),
            (Unordered, &[1, 2, 3], &[3, 2, 1], 4, false),
            (Unordered, &[1, 2, 3], &[9, 2, 1], 4, true),
            (Unordered, &[1, 2, 3], &[1], 2, true),
            (Unordered, &[1], &[1, 2], 2, true),
        ];
        for &(operation, values, expected, unknown_next, failed) in cases {
            for matching in [false, true] {
                for exact_hint in [false, true] {
                    let state = State::default();
                    let iterator = state.observe(values.iter().copied(), exact_hint);
                    let matchers: Vec<_> = expected.iter().copied().map(eq).collect();
                    let failures = assert_that_owned!(iterator)
                        .with_renderer(ResourceRenderer(&state))
                        .capture(|it| match (operation, matching) {
                            (Contains, false) => it.contains(expected[0]),
                            (Contains, true) => it.contains_matching(eq(expected[0])),
                            (Reject, false) => it.does_not_contain(expected[0]),
                            (Reject, true) => it.does_not_contain_matching(eq(expected[0])),
                            (Prefix, false) => it.starts_with(expected),
                            (Prefix, true) => it.starts_with_matching(matchers),
                            (Suffix, false) => it.ends_with(expected),
                            (Suffix, true) => it.ends_with_matching(matchers),
                            (Contiguous, false) => it.contains_contiguous(expected),
                            (Contiguous, true) => it.contains_contiguous_matching(matchers),
                            (Exact, false) => it.contains_exactly(expected),
                            (Exact, true) => it.contains_exactly_matching(matchers),
                            (Unordered, false) => it.contains_exactly_in_any_order(expected),
                            (Unordered, true) => {
                                it.contains_exactly_in_any_order_matching(matchers)
                            }
                        });
                    let short_circuit = exact_hint
                        && match operation {
                            Prefix => values.len() < expected.len(),
                            Exact | Unordered => values.len() != expected.len(),
                            _ => false,
                        };
                    let hints = match operation {
                        Prefix | Exact => 1,
                        Unordered => 1 + usize::from(matching && !short_circuit),
                        _ => 0,
                    };
                    state.verify(if short_circuit { 0 } else { unknown_next }, hints);
                    verify_failure(&failures, &state, failed);
                }
            }
        }
    }
}

mod borrowed {
    use super::*;
    use Operation::{Contains, Reject, Unordered};

    #[test]
    fn membership_and_unordered_adapters_retain_the_owning_iterator() {
        for (operation, expected, next, failed) in [
            (Contains, 2, 2, false),
            (Contains, 9, 4, true),
            (Reject, 2, 2, true),
            (Reject, 9, 4, false),
            (Unordered, 2, 2, true),
        ] {
            for matching in [false, true] {
                let source = Source {
                    values: &[1, 2, 3],
                    state: State::default(),
                    exact_hint: false,
                };
                let failures = assert_that!(source)
                    .with_renderer(ResourceRenderer(&source.state))
                    .capture(|it| match (operation, matching) {
                        (Contains, false) => it.into_iter_contains(expected),
                        (Contains, true) => it.into_iter_contains_matching(eq(expected)),
                        (Reject, false) => it.into_iter_does_not_contain(expected),
                        (Reject, true) => it.into_iter_does_not_contain_matching(eq(expected)),
                        (Unordered, false) => {
                            it.into_iter_contains_exactly_in_any_order([expected])
                        }
                        (Unordered, true) => {
                            it.into_iter_contains_exactly_in_any_order_matching([eq(expected)])
                        }
                        _ => unreachable!(),
                    });
                let hints = if matches!(operation, Unordered) {
                    1 + usize::from(matching)
                } else {
                    0
                };
                source.state.verify(next, hints);
                verify_failure(&failures, &source.state, failed);
            }
        }
    }

    #[test]
    fn contains_all_stops_on_success_or_exhaustion() {
        for (expected, next, failed) in [
            (&[2][..], 2, false),
            (&[9][..], 4, true),
            (&[][..], 0, false),
        ] {
            let source = Source {
                values: &[1, 2, 3],
                state: State::default(),
                exact_hint: false,
            };
            let failures = assert_that!(source)
                .with_renderer(ResourceRenderer(&source.state))
                .capture(|it| it.into_iter_contains_all(expected.iter().copied()));
            source.state.verify(next, 0);
            verify_failure(&failures, &source.state, failed);
        }
    }

    #[test]
    fn cardinality_keeps_resources_through_explanation_without_repeating_observations() {
        for values in [&[][..], &[1, 2, 3][..]] {
            for exact_hint in [false, true] {
                for operation in 0..3 {
                    let source = Source {
                        values,
                        state: State::default(),
                        exact_hint,
                    };
                    let failures = assert_that!(source)
                        .with_renderer(ResourceRenderer(&source.state))
                        .capture(|it| match operation {
                            0 => it.into_iter_is_empty(),
                            1 => it.into_iter_is_not_empty(),
                            _ => it.into_iter_has_length(1),
                        });
                    let next = if operation < 2 {
                        1
                    } else if exact_hint {
                        0
                    } else {
                        values.len().min(1) + 1
                    };
                    source.state.verify(next, usize::from(operation == 2));
                    let failed = match operation {
                        0 => !values.is_empty(),
                        1 => values.is_empty(),
                        _ => true,
                    };
                    assert_that!(failures.len()).is_equal_to(usize::from(failed));
                    if failed && operation != 1 {
                        verify_failure(&failures, &source.state, true);
                    }
                }
            }
        }
    }
}

mod release {
    use super::*;
    use crate::failure::adapter::{Adapter, HumanReadableText};
    use core::convert::Infallible;
    use std::sync::{Arc, Mutex, MutexGuard};

    struct Guarded<'a> {
        _guard: MutexGuard<'a, ()>,
    }

    impl Iterator for Guarded<'_> {
        type Item = i32;
        fn next(&mut self) -> Option<i32> {
            Some(1)
        }
    }

    struct ReleasedPresentation(Arc<Mutex<()>>);

    impl Adapter<AssertionFailure> for ReleasedPresentation {
        type Output = HumanReadableText;
        type Error = Infallible;

        fn adapt(&self, _: &AssertionFailure) -> Result<HumanReadableText, Infallible> {
            Ok(HumanReadableText::new(if self.0.try_lock().is_ok() {
                "iterator released before presentation"
            } else {
                "iterator still holds its guard"
            }))
        }
    }

    #[test]
    fn panic_routing_releases_the_iterator_before_presentation_without_poisoning() {
        let resource = Arc::new(Mutex::new(()));
        let text = std::panic::catch_unwind(|| {
            assert_that_owned!(Guarded {
                _guard: resource.lock().unwrap()
            })
            .with_panic_presentation(ReleasedPresentation(Arc::clone(&resource)))
            .does_not_contain(1);
        })
        .unwrap_err()
        .downcast::<String>()
        .unwrap();
        assert_that!(*text).is_equal_to("iterator released before presentation");
        assert_that!(resource.is_poisoned()).is_false();
        assert_that!(resource.try_lock().is_ok()).is_true();
    }

    #[test]
    fn candidate_observations_are_not_repeated_and_guards_are_released_between_candidates() {
        for borrowed in [false, true] {
            for expected in [2, 9] {
                let source = Source {
                    values: &[1, 2, 3],
                    state: State::default(),
                    exact_hint: false,
                };
                let calls = Cell::new(0);
                let candidate_guard = RefCell::new(());
                let matcher = crate::expectation::predicate(|value: &i32| {
                    let _guard = candidate_guard.borrow_mut();
                    calls.set(calls.get() + 1);
                    *value == expected
                });
                let failures = if borrowed {
                    assert_that!(source)
                        .with_renderer(ResourceRenderer(&source.state))
                        .capture(|it| it.into_iter_contains_matching(matcher))
                } else {
                    let iterator = source.state.observe(source.values.iter().copied(), false);
                    assert_that_owned!(iterator)
                        .with_renderer(ResourceRenderer(&source.state))
                        .capture(|it| it.contains_matching(matcher))
                };
                assert_that!(calls.get()).is_equal_to(if expected == 2 { 2 } else { 3 });
                assert_that!(candidate_guard.try_borrow_mut().is_ok()).is_true();
                source.state.verify(if expected == 2 { 2 } else { 4 }, 0);
                verify_failure(&failures, &source.state, expected == 9);
            }
        }
    }
}
