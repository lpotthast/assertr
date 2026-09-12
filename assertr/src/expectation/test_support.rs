use super::ExpectationDiagnostics;
use crate::{prelude::*, test_support::UnorderedSet};

pub(crate) fn bounded_failures(
    values: &[i32],
    matcher: &impl ExpectationDiagnostics<UnorderedSet>,
    limit: usize,
) -> crate::AssertionFailures {
    let actual = UnorderedSet(values.to_vec());
    assert_that!(actual)
        .with_location(false)
        .with_rendering_budget(RenderingBudget::default().with_max_items(limit))
        .capture(|it| it.matches(matcher))
}

pub(crate) fn assert_bounded_order(matcher: &impl ExpectationDiagnostics<UnorderedSet>) {
    for limit in [0, 1, 2, usize::MAX] {
        let expected = bounded_failures(&[1, 2, 3], matcher, limit);
        assert_that!(expected).contains_exactly_satisfying([
            |expected: AssertThat<AssertionFailure, Capture>| {
                for values in [[3, 2, 1], [2, 1, 3], [1, 3, 2]] {
                    expected
                        .derive_owned(|_| bounded_failures(&values, matcher, limit))
                        .contains_exactly_satisfying([
                            |actual: AssertThat<AssertionFailure, Capture>| {
                                actual
                                    .derive_owned(|actual| ToHumanReadableText.render(actual))
                                    .is_equal_to(ToHumanReadableText.render(expected.actual()));
                            },
                        ]);
                }
            },
        ]);
    }
}
