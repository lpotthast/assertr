use super::lists::MatcherList;
use crate::{
    expectation::{AssertionContext, Evidence, Expectation},
    failure::{Fact, FailureBuilder, FailureKind},
};

/// A disjunction of constraints.
///
/// A rejection stays one nested failure, so it remains distinguishable from the flattened
/// children of an enclosing conjunction. Each retained child carries a zero-based `Branch` fact,
/// because one alternative can contribute several failures and sorted scopes can reorder them.
#[derive(Debug, Clone)]
pub struct AnyOf<L>(L);

/// Stops at the first matching branch. An empty disjunction fails.
pub fn any_of<L>(matchers: L) -> AnyOf<L> {
    AnyOf(matchers)
}

impl<A: ?Sized, R, L> Expectation<A, R> for AnyOf<L>
where
    L: MatcherList<A, R>,
    R: crate::renderer::ValueRenderer<usize>,
{
    type Success<'a>
        = ()
    where
        Self: 'a,
        A: 'a;
    type Rejection<'a>
        = Evidence
    where
        Self: 'a,
        A: 'a;
    fn evaluate(&self, actual: &A, settings: &AssertionContext<'_, R>) -> Result<(), Evidence> {
        let mut context = settings.isolated();
        for index in 0..self.0.len() {
            let mut branch = context.isolated();
            if self.0.evaluate_at(index, actual, &mut branch) {
                return Ok(());
            }
            let render = branch.render();
            let mut evidence = branch.into_evidence();
            for failure in &mut evidence.children {
                failure
                    .facts
                    .push(Fact::labelled("Branch", render.value(&index)));
            }
            context.append(evidence);
        }
        context.finish(false, |context| context.describe::<A, _>(self))
    }

    const KIND: FailureKind = FailureKind::Matching;
    fn explain(
        &self,
        rejected: Option<(&A, Evidence)>,
        failure: FailureBuilder,
        context: &AssertionContext<'_, R>,
    ) -> FailureBuilder {
        match rejected {
            None => {
                context.describe_list::<A, _>(&self.0, failure.relation("satisfies any constraint"))
            }
            Some((_, evidence)) => {
                evidence.explain(failure.relation("does not match any alternative"))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use core::cell::Cell;

    use super::any_of;
    use crate::{assertions::core::partial_eq::eq, matchers::predicate, prelude::*};

    #[test]
    fn stops_at_the_first_matching_branch() {
        let calls = Cell::new(0);
        let matcher = predicate(|actual: &i32| {
            calls.set(calls.get() + 1);
            *actual == 2
        });
        let failures = assert_that!(2).capture(|it| it.matches(any_of(matchers![matcher, eq(3)])));

        assert_that!(failures).is_empty();
        assert_that!(calls.get()).is_equal_to(1);
    }

    #[test]
    fn branch_numbers_use_the_active_renderer_and_budget() {
        use crate::{renderer::RenderingBudget, test_support::SentinelRenderer};
        let failures = assert_that!(0)
            .with_renderer(SentinelRenderer)
            .with_rendering_budget(RenderingBudget::default().with_max_leaf_characters(2))
            .capture(|it| it.matches(any_of(matchers![predicate(|_: &i32| false)])));
        let branch = &failures[0].children[0].facts[0].value;
        assert_that!(format!("{branch:#}")).is_equal_to("<r... 8 more characters ...");
    }

    #[test]
    fn stays_nested_within_a_conjunction() {
        use crate::matchers::all_of;
        let failures = assert_that!(3)
            .with_location(false)
            .capture(|it| it.matches(all_of(matchers![any_of(matchers![eq(1), eq(2)]), eq(5)])));
        assert_that!(failures).has_length(1);
        assert_that!(failures[0]).has_text_report(indoc::indoc! {r"
            -------- assertr --------
            Expression: `3`

            does not match

            Nested failures:
              - does not match any alternative

                Nested failures:
                  - Expected: 1

                      Actual: 3

                    Details:
                      - Branch: 0

                  - Expected: 2

                      Actual: 3

                    Details:
                      - Branch: 1

              - Expected: 5

                  Actual: 3
            -------- assertr --------
        "});
    }

    #[test]
    fn nested_disjunctions_keep_their_own_branch_numbers() {
        let failures = assert_that!(3)
            .with_location(false)
            .capture(|it| it.matches(any_of(matchers![any_of(matchers![eq(1), eq(2)]), eq(5)])));
        assert_that!(failures).has_length(1);
        assert_that!(failures[0]).has_text_report(indoc::indoc! {r"
            -------- assertr --------
            Expression: `3`

            does not match any alternative

            Nested failures:
              - does not match any alternative

                Details:
                  - Branch: 0
                Nested failures:
                  - Expected: 1

                      Actual: 3

                    Details:
                      - Branch: 0

                  - Expected: 2

                      Actual: 3

                    Details:
                      - Branch: 1

              - Expected: 5

                  Actual: 3

                Details:
                  - Branch: 1
            -------- assertr --------
        "});
    }

    #[test]
    fn empty_disjunction_fails() {
        for limit in [0, 1, usize::MAX] {
            let failures = assert_that!(2)
                .with_rendering_budget(RenderingBudget::default().with_max_items(limit))
                .capture(|it| it.matches(any_of(crate::matchers![])));
            assert_that!(failures).has_length(1);
            assert_that!(failures[0].children).has_length(limit.min(1));
            assert_that!(failures[0].omitted_children).is_equal_to(usize::from(limit == 0));
            if limit > 0 {
                let constraint = failures[0].children[0].constraint.as_ref().unwrap();
                assert_that!(constraint.relation.as_deref())
                    .is_equal_to(Some("satisfies any constraint"));
            }
        }
    }
}
