use super::{AssertrMatcher, ConstraintDescription, MatchContext, MatchResult};
use crate::assertions::collection::Collection;

/// Requires at least one matching element. Used by collection membership assertions.
pub struct ContainsMatching<M>(M);

/// Matches collections containing at least one matching element.
pub fn contains_matching<M>(matcher: M) -> ContainsMatching<M> {
    ContainsMatching(matcher)
}

impl<C, R, M> AssertrMatcher<C, R> for ContainsMatching<M>
where
    C: Collection + ?Sized,
    M: AssertrMatcher<C::Item, R>,
{
    fn describe(&self, context: &MatchContext<'_, R>) -> ConstraintDescription {
        ConstraintDescription::new("contains a matching element")
            .children([self.0.describe(context)])
    }

    fn evaluate(&self, actual: &C, context: &mut MatchContext<'_, R>) -> MatchResult {
        let mut alternatives = context.isolated_for_order(C::PRESENTATION.order());
        let mut successes = context.isolated_for_order(C::PRESENTATION.order());
        let mut matched = false;
        for item in actual.elements() {
            let mut branch = if context.is_positive() {
                alternatives.isolated()
            } else {
                successes.isolated()
            };
            let result = self.0.evaluate(item, &mut branch);
            if result.matched {
                matched = true;
                if context.is_positive() {
                    return result;
                }
                successes.append(branch);
            } else {
                alternatives.append(branch);
            }
        }
        if matched {
            context.append(successes);
        } else if context.is_positive() {
            if alternatives.evidence.is_empty() && alternatives.omitted == 0 {
                alternatives.outcome(false, |context| {
                    <Self as AssertrMatcher<C, R>>::describe(self, context)
                });
            }
            context.append(alternatives);
        }
        MatchResult::new(matched)
    }
}

#[cfg(test)]
mod tests {
    use super::contains_matching;
    use crate::{
        matchers::{
            MatchContext, all_of, equal_to, ge,
            test_support::{assert_bounded_order, bounded_failures},
        },
        prelude::*,
        renderer::IntoRendered,
        test_support::{UnorderedSet, rendered_text},
    };
    use core::{cell::Cell, fmt};

    struct ReverseRenderer<'a>(&'a Cell<usize>);

    impl ValueRenderer<i32> for ReverseRenderer<'_> {
        fn fmt(&self, value: &i32, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            self.0.set(self.0.get() + 1);
            write!(formatter, "{}", 10 - value)
        }
    }

    #[test]
    fn bounded_evidence_is_independent_of_iteration_order() {
        assert_bounded_order(&contains_matching(equal_to(9)), true);
        assert_bounded_order(&contains_matching(ge(0)), false);
    }

    #[test]
    fn sorts_nested_branches_before_limiting_them() {
        let matcher = contains_matching(all_of(matchers![equal_to(9), equal_to(0)]));
        let failures = bounded_failures(&[3, 2, 1], &matcher, true, 1);

        assert_that!(failures[0].children).has_length(1);
        assert_that!(failures[0].omitted_children).is_equal_to(5);
        assert_that!(failures[0].children[0].expected.as_ref()).is_equal_to(Some(
            &MatchContext::default().render().value(&0).into_rendered(),
        ));
        assert_bounded_order(&matcher, true);
    }

    #[test]
    fn selects_evidence_using_the_active_renderer_and_skips_zero_budget_rendering() {
        for limit in [0, 1] {
            let renders = Cell::new(0);
            let failures = assert_that!(UnorderedSet(vec![1, 2, 3]))
                .with_renderer(ReverseRenderer(&renders))
                .with_rendering_budget(RenderingBudget::builder().max_items(limit).build())
                .capture(|it| it.matches(contains_matching(equal_to(9))));

            assert_that!(failures[0].children).has_length(limit);
            assert_that!(failures[0].omitted_children).is_equal_to(3 - limit);
            assert_that!(renders.get()).is_equal_to(if limit == 0 { 0 } else { 6 });
            if limit > 0 {
                assert_that!(rendered_text(
                    failures[0].children[0].actual.as_ref().unwrap(),
                ))
                .is_equal_to("7");
            }
        }
    }
}
