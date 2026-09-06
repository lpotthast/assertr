use super::{AssertrMatcher, ConstraintDescription, MatchContext, MatchResult, MatcherList};

/// A conjunction of constraints.
pub struct AllOf<L>(L);

/// Evaluates all constraints. An empty conjunction succeeds.
pub fn all_of<L>(matchers: L) -> AllOf<L> {
    AllOf(matchers)
}

impl<A: ?Sized, R, L> AssertrMatcher<A, R> for AllOf<L>
where
    L: MatcherList<A, R>,
{
    fn describe(&self, context: &MatchContext<'_, R>) -> ConstraintDescription {
        ConstraintDescription::new("satisfies all constraints")
            .omitted_children(self.0.len().saturating_sub(context.render().max_items()))
            .children(
                (0..self.0.len().min(context.render().max_items()))
                    .map(|index| self.0.describe_at(index, context)),
            )
    }

    fn evaluate(&self, actual: &A, context: &mut MatchContext<'_, R>) -> MatchResult {
        let mut group = context.isolated();
        let mut matched = true;
        for index in 0..self.0.len() {
            let result = self.0.evaluate_at(index, actual, &mut group);
            matched &= result.matched;
        }
        if matched != context.is_positive() {
            if group.evidence.is_empty() && group.omitted == 0 {
                group.outcome(matched, |context| {
                    <Self as AssertrMatcher<A, R>>::describe(self, context)
                });
            }
            context.append(group);
        }
        MatchResult::new(matched)
    }
}

#[cfg(test)]
mod tests {
    use super::all_of;
    use crate::{
        matchers::{equal_to, ge, not},
        prelude::*,
    };

    #[test]
    fn composes_constraints() {
        assert_that!(2).matches(all_of((ge(1), not(equal_to(3)))));
    }

    #[test]
    fn empty_conjunction_succeeds() {
        assert_that!(2).matches(all_of(()));
    }
}
