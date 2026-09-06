use super::{AssertrMatcher, Description, MatchContext, MatchResult};
use crate::assertions::collection::Collection;

/// Applies one constraint to every element of an order-free collection.
pub struct Each<M>(M);

/// Every element must match. Empty collections succeed. No positional capability is implied.
pub fn each<M>(matcher: M) -> Each<M> {
    Each(matcher)
}

impl<C, R, M> AssertrMatcher<C, R> for Each<M>
where
    C: Collection + ?Sized,
    M: AssertrMatcher<C::Item, R>,
{
    fn describe(&self, context: &MatchContext<'_, R>) -> Description {
        Description::new("has every element matching").children([self.0.describe(context)])
    }

    fn evaluate(&self, actual: &C, context: &mut MatchContext<'_, R>) -> MatchResult {
        let mut group = context.isolated_for_order(C::PRESENTATION.order());
        let mut matched = true;
        for item in actual.elements() {
            matched &= self.0.evaluate(item, &mut group).matched;
        }
        if matched != context.is_positive() {
            if group.evidence.is_empty() && group.omitted == 0 {
                group.outcome(matched, |context| {
                    <Self as AssertrMatcher<C, R>>::describe(self, context)
                });
            }
            context.append(group);
        }
        MatchResult::new(matched)
    }
}

#[cfg(test)]
mod tests {
    use super::each;
    use crate::matchers::{equal_to, ge, test_support::assert_bounded_order};

    #[test]
    fn bounded_evidence_is_independent_of_iteration_order() {
        assert_bounded_order(&each(equal_to(9)), true);
        assert_bounded_order(&each(ge(0)), false);
    }
}
