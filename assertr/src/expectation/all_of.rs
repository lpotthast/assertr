use super::lists::MatcherList;
use crate::{
    expectation::{AssertionContext, Evidence, Expectation, composite_items},
    failure::{FailureBuilder, FailureKind},
};

/// A conjunction of constraints.
#[derive(Debug, Clone)]
pub struct AllOf<L>(L);

/// Evaluates all constraints. An empty conjunction succeeds.
#[must_use]
pub const fn all_of<L>(matchers: L) -> AllOf<L> {
    AllOf(matchers)
}

impl<A: ?Sized, R, L> Expectation<A, R> for AllOf<L>
where
    L: MatcherList<A, R>,
{
    composite_items!(A);

    fn evaluate(&self, actual: &A, settings: &AssertionContext<'_, R>) -> Result<(), Evidence> {
        let mut context = settings.isolated();
        let mut matched = true;
        for index in 0..self.0.len() {
            matched &= self.0.evaluate_at(index, actual, &mut context);
        }
        context.finish(matched, |context| context.describe::<A, _>(self))
    }

    const KIND: FailureKind = FailureKind::Matching;

    fn explain(
        &self,
        rejected: Option<(&A, Evidence)>,
        failure: FailureBuilder,
        context: &AssertionContext<'_, R>,
    ) -> FailureBuilder {
        match rejected {
            None => context
                .describe_list::<A, _>(&self.0, failure.relation("satisfies every constraint")),
            Some((_, evidence)) => failure.relation("does not match").evidence(evidence),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::all_of;
    use crate::{
        assertions::core::{partial_eq::eq, partial_ord::ge},
        prelude::*,
    };

    #[test]
    fn composes_constraints() {
        assert_that!(2).matches(all_of(matchers![ge(1), eq(2)]));
    }

    #[test]
    fn empty_conjunction_succeeds() {
        assert_that!(2).matches(all_of(crate::matchers![]));
    }
}
