use super::MatcherList;
use crate::{
    AssertionContext, Expectation, ExpectationDiagnostics,
    expectation::Evidence,
    failure::{FailureBuilder, FailureKind},
};

/// A conjunction of constraints.
pub struct AllOf<L>(L);

/// Evaluates all constraints. An empty conjunction succeeds.
pub fn all_of<L>(matchers: L) -> AllOf<L> {
    AllOf(matchers)
}

impl<A: ?Sized, R, L> Expectation<A, R> for AllOf<L>
where
    L: MatcherList<A, R>,
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
        let mut matched = true;
        for index in 0..self.0.len() {
            matched &= self.0.evaluate_at(index, actual, &mut context);
        }
        if !matched && context.evidence.is_empty() {
            context.outcome(false, |context| context.describe::<A, _>(self));
        }
        let evidence = context.into_evidence();
        if matched { Ok(()) } else { Err(evidence) }
    }
}
impl<A: ?Sized, R, L> ExpectationDiagnostics<A, R> for AllOf<L>
where
    L: MatcherList<A, R>,
{
    const KIND: FailureKind = FailureKind::Matching;
    const FLATTEN: bool = true;
    fn explain<Target>(
        &self,
        rejected: Option<(&A, Evidence)>,
        failure: FailureBuilder<Target>,
        context: &AssertionContext<'_, R>,
    ) -> FailureBuilder<Target> {
        match rejected {
            None => context
                .describe_list::<A, _, _>(&self.0, failure.relation("satisfies every constraint")),
            Some((_, evidence)) => evidence.explain(failure.relation("does not match")),
        }
    }
}
#[cfg(test)]
mod tests {
    use super::all_of;
    use crate::{
        assertions::core::{partial_eq::equal_to, partial_ord::ge},
        prelude::*,
    };

    #[test]
    fn composes_constraints() {
        assert_that!(2).matches(all_of((ge(1), equal_to(2))));
    }

    #[test]
    fn empty_conjunction_succeeds() {
        assert_that!(2).matches(all_of(()));
    }
}
