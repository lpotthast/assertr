use crate::{
    AssertionContext, Expectation, ExpectationDiagnostics,
    failure::{FailureBuilder, FailureKind},
};

/// Explicitly dereferences an actual reference before matching. No actual values are moved.
pub struct Dereferenced<M>(M);

/// Adapts a matcher for owned reference subjects or reference-valued fields.
pub fn dereferenced<M>(matcher: M) -> Dereferenced<M> {
    Dereferenced(matcher)
}

impl<'t, A: ?Sized, R, M> Expectation<&'t A, R> for Dereferenced<M>
where
    M: Expectation<A, R>,
{
    type Success<'a>
        = <M as Expectation<A, R>>::Success<'a>
    where
        Self: 'a,
        &'t A: 'a;
    type Rejection<'a>
        = <M as Expectation<A, R>>::Rejection<'a>
    where
        Self: 'a,
        &'t A: 'a;
    fn evaluate<'a>(
        &'a self,
        actual: &'a &'t A,
        context: &AssertionContext<'_, R>,
    ) -> Result<Self::Success<'a>, Self::Rejection<'a>> {
        self.0.evaluate(*actual, context)
    }
}
impl<'t, A: ?Sized, R, M> ExpectationDiagnostics<&'t A, R> for Dereferenced<M>
where
    M: ExpectationDiagnostics<A, R>,
{
    const KIND: FailureKind = <M as ExpectationDiagnostics<A, R>>::KIND;
    const FLATTEN: bool = <M as ExpectationDiagnostics<A, R>>::FLATTEN;
    fn explain<'a, Target>(
        &'a self,
        rejected: Option<(&'a &'t A, Self::Rejection<'a>)>,
        failure: FailureBuilder<Target>,
        context: &AssertionContext<'_, R>,
    ) -> FailureBuilder<Target> {
        self.0.explain(
            rejected.map(|(actual, rejection)| (*actual, rejection)),
            failure,
            context,
        )
    }
}
#[cfg(test)]
mod tests {
    use super::dereferenced;
    use crate::{assertions::core::partial_eq::equal_to, prelude::*};

    #[test]
    fn accepts_owned_references() {
        assert_that_owned!(&42).matches(dereferenced(equal_to(42)));
    }
}
