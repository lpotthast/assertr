use super::{AssertrMatcher, Description, MatchContext, MatchResult, MatcherList};
use crate::failure::Fact;

/// A disjunction of constraints.
pub struct AnyOf<L>(L);

/// Stops at the first matching branch. An empty disjunction fails.
pub fn any_of<L>(matchers: L) -> AnyOf<L> {
    AnyOf(matchers)
}

impl<A: ?Sized, R, L> AssertrMatcher<A, R> for AnyOf<L>
where
    L: MatcherList<A, R>,
{
    fn describe(&self, context: &MatchContext<'_, R>) -> Description {
        Description::new("satisfies any constraint")
            .omitted_children(self.0.len().saturating_sub(context.render().max_items()))
            .children(
                (0..self.0.len().min(context.render().max_items()))
                    .map(|index| self.0.describe_at(index, context)),
            )
    }

    fn evaluate(&self, actual: &A, context: &mut MatchContext<'_, R>) -> MatchResult {
        let mut alternatives = context.isolated();
        for index in 0..self.0.len() {
            let mut branch = alternatives.isolated();
            let result = self.0.evaluate_at(index, actual, &mut branch);
            for failure in &mut branch.evidence {
                failure.facts.push(Fact::new("branch", index));
            }
            if result.matched {
                if !context.is_positive() {
                    context.append(branch);
                }
                return result;
            }
            alternatives.append(branch);
        }
        if context.is_positive() {
            if alternatives.evidence.is_empty() && alternatives.omitted == 0 {
                alternatives.outcome(false, |context| {
                    <Self as AssertrMatcher<A, R>>::describe(self, context)
                });
            }
            context.append(alternatives);
        }
        MatchResult::new(false)
    }
}

#[cfg(test)]
mod tests {
    use super::any_of;
    use crate::{
        matchers::{equal_to, predicate},
        prelude::*,
    };
    use core::cell::Cell;

    #[test]
    fn stops_at_the_first_matching_branch() {
        let calls = Cell::new(0);
        let matcher = predicate(|actual: &i32| {
            calls.set(calls.get() + 1);
            *actual == 2
        });
        let failures = assert_that!(2).capture(|it| it.matches(any_of((matcher, equal_to(3)))));

        assert_that!(failures).is_empty();
        assert_that!(calls.get()).is_equal_to(1);
    }

    #[test]
    fn empty_disjunction_fails() {
        assert_that!(2).does_not_match(any_of(()));
    }
}
