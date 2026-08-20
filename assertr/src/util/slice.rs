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

/// Matches values one-to-one when `matches` represents an equivalence relation.
///
/// Unlike general predicate matching, equivalent values can be assigned greedily: any unmatched
/// equivalent expected value is interchangeable with every other one. Skipping already matched
/// expected values before comparing keeps the algorithm quadratic and avoids recursion.
pub(crate) fn match_multiset(
    actual_len: usize,
    expected_len: usize,
    matches: impl Fn(usize, usize) -> bool,
) -> BipartiteMatchResult {
    let mut matched_expected = alloc::vec![false; expected_len];
    let mut unmatched_actual = Vec::new();

    for actual_index in 0..actual_len {
        let matched_index = (0..expected_len).find(|expected_index| {
            !matched_expected[*expected_index] && matches(actual_index, *expected_index)
        });

        if let Some(expected_index) = matched_index {
            matched_expected[expected_index] = true;
        } else {
            unmatched_actual.push(actual_index);
        }
    }

    BipartiteMatchResult {
        unmatched_actual,
        unmatched_expected: matched_expected
            .iter()
            .enumerate()
            .filter_map(|(index, matched)| (!matched).then_some(index))
            .collect(),
    }
}

/// Finds a maximum one-to-one matching between actual and expected values.
///
/// A greedy matcher is insufficient when predicates overlap: an early actual value may match
/// several expected predicates while a later value only matches one of them. The augmenting-path
/// search below revisits earlier choices so an exact matching is found whenever one exists.
///
/// Recursion depth is bounded by `expected_len`; assertion inputs are small enough in practice
/// that this cannot overflow the stack.
pub(crate) fn match_bipartite(
    actual_len: usize,
    expected_len: usize,
    matches: impl Fn(usize, usize) -> bool,
) -> BipartiteMatchResult {
    fn augment(
        actual_index: usize,
        expected_len: usize,
        visited_expected: &mut [bool],
        expected_to_actual: &mut [Option<usize>],
        matches: &impl Fn(usize, usize) -> bool,
    ) -> bool {
        for expected_index in 0..expected_len {
            if visited_expected[expected_index] || !matches(actual_index, expected_index) {
                continue;
            }

            visited_expected[expected_index] = true;
            let can_reassign = match expected_to_actual[expected_index] {
                None => true,
                Some(previous_actual) => augment(
                    previous_actual,
                    expected_len,
                    visited_expected,
                    expected_to_actual,
                    matches,
                ),
            };

            if can_reassign {
                expected_to_actual[expected_index] = Some(actual_index);
                return true;
            }
        }

        false
    }

    let mut expected_to_actual = alloc::vec![None; expected_len];

    for actual_index in 0..actual_len {
        let mut visited_expected = alloc::vec![false; expected_len];
        let _ = augment(
            actual_index,
            expected_len,
            &mut visited_expected,
            &mut expected_to_actual,
            &matches,
        );
    }

    let mut matched_actual = alloc::vec![false; actual_len];
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

#[cfg(test)]
mod tests {
    mod match_bipartite {
        use crate::util::slice::match_bipartite;

        #[test]
        fn returns_equal_on_matching_input() {
            let actual = [1, 2, 3];
            let predicates: [fn(&i32) -> bool; 3] = [|it| *it == 1, |it| *it == 2, |it| *it == 3];
            let result = match_bipartite(
                actual.len(),
                predicates.len(),
                |actual_index, predicate_index| predicates[predicate_index](&actual[actual_index]),
            );

            assert!(result.is_exact());
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

            assert!(result.is_exact());
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

            assert_eq!(result.unmatched_actual, [0, 2]);
            assert_eq!(result.unmatched_expected, [1, 2, 3]);
        }
    }
}
