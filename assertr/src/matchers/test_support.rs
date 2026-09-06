use super::AssertrMatcher;
use crate::{prelude::*, test_support::UnorderedSet};

pub(super) fn bounded_failures(
    values: &[i32],
    matcher: &impl AssertrMatcher<UnorderedSet>,
    positive: bool,
    limit: usize,
) -> crate::AssertionFailures {
    let actual = UnorderedSet(values.to_vec());
    assert_that!(actual)
        .with_location(false)
        .with_rendering_budget(RenderingBudget::builder().max_items(limit).build())
        .capture(|it| {
            if positive {
                it.matches(matcher)
            } else {
                it.does_not_match(matcher)
            }
        })
}

pub(super) fn assert_bounded_order(matcher: &impl AssertrMatcher<UnorderedSet>, positive: bool) {
    for limit in [0, 1, 2, usize::MAX] {
        let expected = bounded_failures(&[1, 2, 3], matcher, positive, limit);
        assert_that!(expected).has_length(1);
        for values in [[3, 2, 1], [2, 1, 3], [1, 3, 2]] {
            let actual = bounded_failures(&values, matcher, positive, limit);
            assert_that!(actual).has_length(1);
            assert_that!(ToHumanReadableText.render(&actual[0]))
                .is_equal_to(ToHumanReadableText.render(&expected[0]));
        }
    }
}
