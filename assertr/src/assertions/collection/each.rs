use crate::{
    AssertionContext, Expectation, ExpectationDiagnostics,
    assertions::collection::Collection,
    expectation::Evidence,
    failure::{FailureBuilder, FailureKind},
};

/// Applies one constraint to every element of an order-free collection.
pub struct Each<M>(M);

/// Every element must match. Empty collections succeed. No positional capability is implied.
pub fn each<M>(matcher: M) -> Each<M> {
    Each(matcher)
}

impl<C: Collection + ?Sized, R, M> Expectation<C, R> for Each<M>
where
    M: ExpectationDiagnostics<C::Item, R>,
{
    type Success<'a>
        = ()
    where
        Self: 'a,
        C: 'a;
    type Rejection<'a>
        = Evidence
    where
        Self: 'a,
        C: 'a;
    fn evaluate(&self, actual: &C, settings: &AssertionContext<'_, R>) -> Result<(), Evidence> {
        let mut context = settings.isolated_for_order(C::PRESENTATION.order());
        let mut matched = true;
        for item in actual.elements() {
            matched &= context.evaluate(item, &self.0);
        }
        if !matched && context.evidence.is_empty() {
            context.outcome(false, |context| context.describe::<C, _>(self));
        }
        let evidence = context.into_evidence();
        if matched { Ok(()) } else { Err(evidence) }
    }
}
impl<C: Collection + ?Sized, R, M> ExpectationDiagnostics<C, R> for Each<M>
where
    M: ExpectationDiagnostics<C::Item, R>,
{
    const KIND: FailureKind = FailureKind::Matching;
    const FLATTEN: bool = true;
    fn explain<Target>(
        &self,
        rejected: Option<(&C, Evidence)>,
        failure: FailureBuilder<Target>,
        context: &AssertionContext<'_, R>,
    ) -> FailureBuilder<Target> {
        match rejected {
            None => failure
                .relation("has every element matching")
                .children([context.describe(&self.0)]),
            Some((_, evidence)) => evidence.explain(failure.relation("does not match")),
        }
    }
}
#[cfg(test)]
mod tests {
    use super::each;
    use crate::{
        assertions::core::partial_eq::equal_to, expectation::test_support::assert_bounded_order,
    };

    #[test]
    fn bounded_evidence_is_independent_of_iteration_order() {
        assert_bounded_order(&each(equal_to(9)));
    }
}
