use crate::{
    assertions::collection::Collection,
    expectation::{AssertionContext, Evidence, Expectation, composite_items},
    failure::{FailureBuilder, FailureKind},
};

/// Applies one constraint to every element of an order-free collection.
#[derive(Debug, Clone)]
pub struct Each<M>(M);

/// Every element must match. Empty collections succeed. No positional capability is implied.
#[must_use]
pub const fn each<M>(matcher: M) -> Each<M> {
    Each(matcher)
}

impl<C: Collection + ?Sized, R, M> Expectation<C, R> for Each<M>
where
    M: Expectation<C::Item, R>,
{
    composite_items!(C);
    fn evaluate(&self, actual: &C, settings: &AssertionContext<'_, R>) -> Result<(), Evidence> {
        let mut context = settings.isolated_for_order(C::PRESENTATION.order());
        let mut matched = true;
        for item in actual.elements() {
            matched &= context.evaluate(item, &self.0);
        }
        context.finish(matched, |context| context.describe::<C, _>(self))
    }

    const KIND: FailureKind = FailureKind::Matching;
    fn explain(
        &self,
        rejected: Option<(&C, Evidence)>,
        failure: FailureBuilder,
        context: &AssertionContext<'_, R>,
    ) -> FailureBuilder {
        match rejected {
            None => failure
                .relation("has every element matching")
                .children([context.describe(&self.0)]),
            Some((_, evidence)) => failure.relation("does not match").evidence(evidence),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::each;
    use crate::{assertions::core::partial_eq::eq, test_support::assert_bounded_order};

    #[test]
    fn bounded_evidence_is_independent_of_iteration_order() {
        assert_bounded_order(&each(eq(9)));
    }
}
