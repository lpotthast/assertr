//! Checks that bounded evidence retention does not depend on iteration order.

use alloc::{string::String, vec::Vec};

use super::UnorderedSet;
use crate::{expectation::Expectation, prelude::*};

/// Captures the failures of `matcher` on an [`UnorderedSet`] of `values`, retaining at most
/// `limit` items per evidence group.
pub(crate) fn bounded_failures(
    values: &[i32],
    matcher: &impl Expectation<UnorderedSet>,
    limit: usize,
) -> AssertionFailures {
    let actual = UnorderedSet(values.to_vec());
    assert_that!(actual)
        .with_location(false)
        .with_rendering_budget(RenderingBudget::default().with_max_items(limit))
        .capture(|it| it.matches(matcher))
}

/// Asserts that `matcher` reports one failure whose text is identical for every iteration order
/// of the same elements and every item limit.
pub(crate) fn assert_bounded_order(matcher: &impl Expectation<UnorderedSet>) {
    let reports = |values: &[i32], limit| {
        bounded_failures(values, matcher, limit)
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<String>>()
    };
    for limit in [0, 1, 2, usize::MAX] {
        let expected = reports(&[1, 2, 3], limit);
        assert_that!(&expected).has_length(1);
        for values in [[3, 2, 1], [2, 1, 3], [1, 3, 2]] {
            assert_that!(reports(&values, limit)).is_equal_to(&expected);
        }
    }
}
