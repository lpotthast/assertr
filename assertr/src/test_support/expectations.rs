//! Expectations shared by renderer-independence fixtures.

use crate::{
    expectation::{AssertionContext, Expectation, passed},
    failure::{FailureBuilder, FailureKind},
};

/// A boolean matcher whose diagnostics render nothing, for renderer-independence fixtures.
///
/// Unlike [`crate::matchers::predicate`], it never renders a rejected subject.
pub(crate) struct OpaquePredicate<F>(F);

pub(crate) fn opaque_predicate<A: ?Sized, F: Fn(&A) -> bool>(callback: F) -> OpaquePredicate<F> {
    OpaquePredicate(callback)
}

impl<A: ?Sized, R, F: Fn(&A) -> bool> Expectation<A, R> for OpaquePredicate<F> {
    type Success<'a>
        = ()
    where
        Self: 'a,
        A: 'a;
    type Rejection<'a>
        = ()
    where
        Self: 'a,
        A: 'a;
    fn evaluate(&self, actual: &A, _: &AssertionContext<'_, R>) -> Result<(), ()> {
        passed((self.0)(actual))
    }

    const KIND: FailureKind = FailureKind::Matching;
    fn explain(
        &self,
        rejected: Option<(&A, ())>,
        failure: FailureBuilder,
        _: &AssertionContext<'_, R>,
    ) -> FailureBuilder {
        match rejected {
            None => failure.relation("satisfies the opaque predicate"),
            Some(_) => failure.relation("does not satisfy the opaque predicate"),
        }
    }
}
