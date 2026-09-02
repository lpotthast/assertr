//! Maximum bipartite matching between actual values and expectations.
//!
//! Every unordered exact comparison (`contains_exactly_in_any_order` and its `_matching` /
//! `_satisfying` variants on collections and iterators, the order-free fallback of `contains_exactly`,
//! and [`crate::cmp::slice::compare`]) reduces to the same question: can each actual value be paired
//! with exactly one expectation, and which ones are left over when it cannot? This module answers it
//! once, for any relation given as a predicate over index pairs.

use alloc::vec;
use alloc::vec::Vec;

pub(crate) struct BipartiteMatchResult {
    pub(crate) unmatched_actual: Vec<usize>,
    pub(crate) unmatched_expected: Vec<usize>,
}

impl BipartiteMatchResult {
    pub(crate) fn is_exact(&self) -> bool {
        self.unmatched_actual.is_empty() && self.unmatched_expected.is_empty()
    }
}

/// One value on the alternating path an augmenting search is building.
struct PathStep {
    /// The actual value looking for an expected slot.
    actual: usize,
    /// The slot this value currently holds and gives up if the search succeeds. `None` for the
    /// value that started the search, which holds no slot yet.
    via: Option<usize>,
    /// Whether the free slots were already scanned for this value.
    scanned_free_slots: bool,
    /// The first slot not yet considered for reassignment.
    next_candidate: usize,
}

/// Finds a maximum one-to-one matching between actual and expected values.
///
/// A greedy matcher is insufficient when predicates overlap: an early actual value may match
/// several expected predicates while a later value only matches one of them. The augmenting-path
/// search below revisits earlier choices so an exact matching is found whenever one exists.
///
/// Each value first looks for a free slot and only falls back to displacing an earlier
/// assignment when there is none. Interchangeable expectations, such as the duplicates of a plain
/// equality comparison, are therefore assigned in one comparison per value, without ever walking
/// the chain of earlier assignments.
///
/// A value that finds no free slot even through reassignment proves that none of the slots it
/// visited can reach a free slot until the matching changes. Those marks are kept for the
/// following values and cleared only after a successful assignment, so surplus duplicates share
/// one exhausted search instead of repeating it.
///
/// The path is an explicit stack, so the input size cannot overflow the call stack.
pub(crate) fn match_bipartite(
    actual_len: usize,
    expected_len: usize,
    mut matches: impl FnMut(usize, usize) -> bool,
) -> BipartiteMatchResult {
    let mut expected_to_actual = vec![None; expected_len];
    let mut visited_expected = vec![false; expected_len];
    let mut path = Vec::new();

    for actual_index in 0..actual_len {
        let assigned = augment(
            actual_index,
            &mut expected_to_actual,
            &mut visited_expected,
            &mut path,
            &mut matches,
        );
        if assigned {
            visited_expected.fill(false);
        }
    }

    let mut matched_actual = vec![false; actual_len];
    for actual_index in expected_to_actual.iter().flatten() {
        matched_actual[*actual_index] = true;
    }

    BipartiteMatchResult {
        unmatched_actual: matched_actual
            .iter()
            .enumerate()
            .filter_map(|(index, matched)| (!matched).then_some(index))
            .collect(),
        unmatched_expected: expected_to_actual
            .iter()
            .enumerate()
            .filter_map(|(index, actual)| actual.is_none().then_some(index))
            .collect(),
    }
}

/// Assigns `root_actual` to a slot, shifting earlier assignments along an alternating path when
/// necessary. Returns whether a slot was found.
fn augment<F>(
    root_actual: usize,
    expected_to_actual: &mut [Option<usize>],
    visited_expected: &mut [bool],
    path: &mut Vec<PathStep>,
    matches: &mut F,
) -> bool
where
    F: FnMut(usize, usize) -> bool,
{
    let expected_len = expected_to_actual.len();
    path.clear();
    path.push(PathStep {
        actual: root_actual,
        via: None,
        scanned_free_slots: false,
        next_candidate: 0,
    });

    while let Some(step) = path.last_mut() {
        let actual_index = step.actual;
        let scan_free_slots = !step.scanned_free_slots;
        step.scanned_free_slots = true;
        let next_candidate = step.next_candidate;

        if scan_free_slots {
            let free_slot = (0..expected_len)
                .find(|slot| expected_to_actual[*slot].is_none() && matches(actual_index, *slot));
            if let Some(mut slot) = free_slot {
                // Every value on the path moves into the slot given up by the value after it.
                while let Some(step) = path.pop() {
                    expected_to_actual[slot] = Some(step.actual);
                    let Some(via) = step.via else { break };
                    slot = via;
                }
                return true;
            }
        }

        let candidate = (next_candidate..expected_len).find_map(|slot| {
            if visited_expected[slot] {
                return None;
            }
            let occupant = expected_to_actual[slot]?;
            matches(actual_index, slot).then_some((slot, occupant))
        });
        match candidate {
            Some((slot, occupant)) => {
                visited_expected[slot] = true;
                if let Some(step) = path.last_mut() {
                    step.next_candidate = slot + 1;
                }
                path.push(PathStep {
                    actual: occupant,
                    via: Some(slot),
                    scanned_free_slots: false,
                    next_candidate: 0,
                });
            }
            None => {
                path.pop();
            }
        }
    }

    false
}

#[cfg(test)]
mod tests {
    mod match_bipartite {
        use crate::prelude::*;
        use crate::util::matching::match_bipartite;
        use alloc::vec::Vec;

        #[test]
        fn returns_equal_on_matching_input() {
            let actual = [1, 2, 3];
            let predicates: [fn(&i32) -> bool; 3] = [|it| *it == 1, |it| *it == 2, |it| *it == 3];
            let result = match_bipartite(
                actual.len(),
                predicates.len(),
                |actual_index, predicate_index| predicates[predicate_index](&actual[actual_index]),
            );

            assert_that!(result.is_exact()).is_true();
        }

        #[test]
        fn finds_exact_matching_when_predicates_overlap() {
            let actual = [1, 2];
            let predicates: [fn(&i32) -> bool; 2] = [|it| *it <= 2, |it| *it == 1];
            let result = match_bipartite(
                actual.len(),
                predicates.len(),
                |actual_index, predicate_index| predicates[predicate_index](&actual[actual_index]),
            );

            assert_that!(result.is_exact()).is_true();
        }

        #[test]
        fn reports_unmatched_actual_values_and_expected_predicates() {
            let actual = [1, 5, 7];
            let predicates: [fn(&i32) -> bool; 4] =
                [|it| *it == 5, |it| *it == 3, |it| *it == 4, |it| *it == 42];
            let result = match_bipartite(
                actual.len(),
                predicates.len(),
                |actual_index, predicate_index| predicates[predicate_index](&actual[actual_index]),
            );

            assert_that!(result.unmatched_actual.as_slice()).is_equal_to([0, 2].as_slice());
            assert_that!(result.unmatched_expected.as_slice()).is_equal_to([1, 2, 3].as_slice());
        }

        #[test]
        fn assigns_interchangeable_expected_values_without_reassignment() {
            let len = 4000;
            let mut comparisons = 0;
            let result = match_bipartite(len, len, |_, _| {
                comparisons += 1;
                true
            });

            assert_that!(result.is_exact()).is_true();
            // Every value takes the first free slot. Occupied slots are skipped without a
            // comparison and no earlier assignment is revisited.
            assert_that!(comparisons).is_equal_to(len);
        }

        #[test]
        fn exhausts_the_search_once_for_surplus_interchangeable_values() {
            let actual_len = 2000;
            let expected_len = 1000;
            let mut comparisons = 0;
            let result = match_bipartite(actual_len, expected_len, |_, _| {
                comparisons += 1;
                true
            });

            assert_that!(result.unmatched_actual.as_slice())
                .is_equal_to((expected_len..actual_len).collect::<Vec<_>>().as_slice());
            assert_that!(result.unmatched_expected).is_empty();
            // The first value that finds no free slot compares against every slot once while
            // proving that no reassignment can free one. The remaining surplus values reuse
            // that proof instead of repeating the search.
            assert_that!(comparisons).is_equal_to(2 * expected_len);
        }

        #[test]
        fn does_not_recurse_into_reassignment_chains() {
            let len = 4000;
            let search = std::thread::Builder::new()
                .stack_size(64 * 1024)
                .spawn(move || {
                    assert_that!(match_bipartite(len, len, |_, _| true).is_exact()).is_true();
                    let surplus = match_bipartite(len + 1, len, |_, _| true);
                    assert_that!(surplus.unmatched_actual.as_slice()).is_equal_to([len].as_slice());
                })
                .expect("thread spawns");

            search.join().expect("matching finishes on a small stack");
        }

        #[test]
        fn finds_the_maximum_matching_after_an_exhausted_search() {
            let actual = [1, 1, 1, 3, 5];
            let predicates: [fn(&i32) -> bool; 3] =
                [|it| *it == 1, |it| it % 2 == 1, |it| *it == 5];
            let result = match_bipartite(
                actual.len(),
                predicates.len(),
                |actual_index, predicate_index| predicates[predicate_index](&actual[actual_index]),
            );

            // The third `1` exhausts every reassignment through the first two predicates. That
            // must neither block the `5` from taking its own predicate nor leave a predicate
            // unmatched.
            assert_that!(result.unmatched_actual.as_slice()).is_equal_to([2, 3].as_slice());
            assert_that!(result.unmatched_expected).is_empty();
        }
    }
}
