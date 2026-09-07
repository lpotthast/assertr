use super::{AssertrMatcher, ConstraintDescription, MatchContext, MatchResult, MatcherList};
use crate::{
    Fact,
    assertions::collection::Collection,
    failure::{FailureBuilder, FailureKind},
    util::matching::match_bipartite,
};
use alloc::{collections::BTreeMap, vec::Vec};

/// Exact one-to-one order-free matching, preserving multiplicity.
pub struct ElementsAreInAnyOrder<L>(L);

/// Matches every actual element to one distinct expectation using maximum bipartite matching.
///
/// Each visited pair is evaluated once. Cached candidate evidence can require quadratic space.
pub fn elements_are_in_any_order<L>(list: L) -> ElementsAreInAnyOrder<L> {
    ElementsAreInAnyOrder(list)
}

impl<C, R, L> AssertrMatcher<C, R> for ElementsAreInAnyOrder<L>
where
    R: crate::ValueRenderer<usize>,
    C: Collection + ?Sized,
    L: MatcherList<C::Item, R>,
{
    fn describe(&self, context: &MatchContext<'_, R>) -> ConstraintDescription {
        ConstraintDescription::new("has exactly these elements in any order")
            .omitted_children(self.0.len().saturating_sub(context.render().max_items()))
            .children(
                (0..self.0.len().min(context.render().max_items()))
                    .map(|index| self.0.describe_at(index, context)),
            )
    }

    fn evaluate(&self, actual: &C, context: &mut MatchContext<'_, R>) -> MatchResult {
        let actual = actual.elements().collect::<Vec<_>>();
        let expected_length = self.0.len();
        // A sparse cache avoids allocating the full Cartesian product for easy exact matches.
        let mut cache = BTreeMap::new();
        let result = match_bipartite(actual.len(), expected_length, |index, slot| {
            cache
                .entry((index, slot))
                .or_insert_with(|| {
                    let mut branch = context.isolated_for_order(C::PRESENTATION.order());
                    let matched = self.0.evaluate_at(slot, actual[index], &mut branch).matched;
                    (matched, branch)
                })
                .0
        });
        let matched = result.is_exact();
        if matched != context.is_positive() {
            if matched {
                // Cached successful constraints explain negative matching without replaying code.
                let mut successes = context.isolated_for_order(C::PRESENTATION.order());
                for pair in result.matched_pairs {
                    if let Some((_, branch)) = cache.remove(&pair) {
                        successes.append(branch);
                    }
                }
                context.append(successes);
                if context.evidence.is_empty() && context.omitted == 0 {
                    context.outcome(true, |context| {
                        <Self as AssertrMatcher<C, R>>::describe(self, context)
                    });
                }
            } else {
                for slot in result.unmatched_expected {
                    let mut alternatives = context.isolated_for_order(C::PRESENTATION.order());
                    for index in 0..actual.len() {
                        if let Some((false, branch)) = cache.remove(&(index, slot)) {
                            alternatives.append(branch);
                        }
                    }
                    if context.is_diagnostic() {
                        context.record_group(
                            FailureBuilder::detached::<C>(FailureKind::Matching)
                                .relation("has no distinct matching element")
                                .fact(Fact::labelled("expected slot", slot))
                                .children(alternatives.evidence)
                                .omitted_children(alternatives.omitted)
                                .constraint(self.0.describe_at(slot, context))
                                .build(),
                        );
                    } else {
                        context.outcome(false, |context| {
                            <Self as AssertrMatcher<C, R>>::describe(self, context)
                        });
                    }
                }
                if !result.unmatched_actual.is_empty() {
                    let mut unexpected = context.isolated_for_order(C::PRESENTATION.order());
                    for index in &result.unmatched_actual {
                        for slot in 0..expected_length {
                            if let Some((false, branch)) = cache.remove(&(*index, slot)) {
                                unexpected.append(branch);
                            }
                        }
                    }
                    if context.is_diagnostic() {
                        context.record_group(
                            FailureBuilder::detached::<C>(FailureKind::Matching)
                                .relation("has unexpected elements")
                                .fact(Fact::labelled(
                                    "unexpected count",
                                    context.render().value(&result.unmatched_actual.len()),
                                ))
                                .children(unexpected.evidence)
                                .omitted_children(unexpected.omitted)
                                .build(),
                        );
                    } else {
                        context.outcome(false, |context| {
                            <Self as AssertrMatcher<C, R>>::describe(self, context)
                        });
                    }
                }
            }
        }
        MatchResult::new(matched)
    }
}

/// Exact unordered matcher list with equality shorthand and duplicate preservation.
#[macro_export]
macro_rules! elements_are_in_any_order {
    ($($value:expr),* $(,)?) => {
        $crate::matchers::elements_are_in_any_order($crate::matchers![$($value),*])
    };
}

#[cfg(test)]
mod tests {
    use crate::{
        matchers::{
            ge,
            test_support::{assert_bounded_order, bounded_failures},
        },
        prelude::*,
    };

    #[test]
    fn bounded_candidate_evidence_is_independent_of_iteration_order() {
        assert_bounded_order(&elements_are_in_any_order![9], true);
        assert_bounded_order(&elements_are_in_any_order![ge(0), ge(0), ge(0)], false);
    }

    #[test]
    fn sorts_unexpected_evidence_before_limiting_it() {
        let matcher = elements_are_in_any_order![9];
        let expected = bounded_failures(&[9, 1, 2, 3], &matcher, true, 1);
        let actual = bounded_failures(&[9, 3, 2, 1], &matcher, true, 1);

        assert_that!(actual[0].children).contains_exactly_satisfying([
            |element: AssertThat<AssertionFailure, Capture>| {
                element.derive(|value| &value.children).has_length(1);
                element
                    .derive(|value| &value.omitted_children)
                    .is_equal_to(2);
            },
        ]);
        assert_that!(ToHumanReadableText.render(&actual[0]))
            .is_equal_to(ToHumanReadableText.render(&expected[0]));
    }

    #[test]
    fn negative_unordered_evidence_uses_each_assigned_element_once() {
        let failures = assert_that!([1, 2])
            .capture(|it| it.does_not_match(elements_are_in_any_order![ge(0), ge(0)]));

        assert_that!(failures).contains_exactly_satisfying([
            |element: AssertThat<AssertionFailure, Capture>| {
                element
                    .derive_owned(|value| {
                        value
                            .children
                            .iter()
                            .map(|child| rendered_text(child.actual.as_ref().unwrap()))
                            .collect::<Vec<_>>()
                    })
                    .contains_exactly_in_any_order(["1", "2"]);
            },
        ]);
    }

    #[test]
    fn supports_overlapping_constraints() {
        assert_that!([1, 2]).matches(elements_are_in_any_order![ge(1), 1]);
    }

    mod evaluate {
        use super::*;

        #[test]
        fn a_zero_item_budget_preserves_length_failure_without_rendering_evidence() {
            use indoc::formatdoc;
            struct NeverRender;
            impl<T: ?Sized> ValueRenderer<T> for NeverRender {
                fn fmt(&self, _: &T, _: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                    panic!("rendered omitted evidence")
                }
            }
            let failures = assert_that!([1, 2])
                .with_renderer(NeverRender)
                .with_location(false)
                .with_rendering_budget(RenderingBudget::builder().max_items(0).build())
                .capture(|it| it.matches(elements_are_in_any_order![]));
            assert_that!(failures).contains_exactly_satisfying([
                |element: AssertThat<AssertionFailure, Capture>| {
                    element.derive(|value| value).has_text_report(formatdoc! {r"
        -------- assertr --------
        Expression: `[1, 2]`

        does not match

        Details:
          - ... 1 more nested failure ...
        -------- assertr --------
    "});
                    element
                        .derive(|value| &value.omitted_children)
                        .is_equal_to(1);
                },
            ]);
        }
    }

    mod probe {
        use super::*;
        use crate::matchers::MatchContext;
        #[test]
        fn length_mismatches_do_not_render_numeric_evidence() {
            struct NeverRender;
            impl<T: ?Sized> ValueRenderer<T> for NeverRender {
                fn fmt(&self, _: &T, _: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                    panic!("probe rendered evidence")
                }
            }
            let context = MatchContext::new(&NeverRender, RenderingBudget::default());
            assert_that!(context.probe(&[1, 2], &elements_are_in_any_order![])).is_false();
            assert_that!(context.into_failures()).is_empty();
        }
    }
}
