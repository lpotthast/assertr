//! Reusable map value matcher expectations.

use super::Map;
use crate::{
    assertions::collection::matching::MatchingItem,
    expectation::{AssertionContext, Expectation, evidence_items},
    failure::{FailureBuilder, FailureKind},
};

const MATCHING_VALUE: MatchingItem = MatchingItem {
    contains: "contains a matching value",
    does_not_contain: "does not contain a matching value",
};

/// Requires at least one map value matching a reusable expectation, without key lookup.
///
/// Evaluation stops at the first match. On rejection, it retains every value's child evidence,
/// ordered and limited like the map's rendered entries. An empty map rejects the expectation.
#[derive(Debug, Clone)]
pub struct ContainsValueMatching<E>(E);

impl<E> ContainsValueMatching<E> {
    /// Owns the expected operand.
    #[must_use]
    pub const fn new(expected: E) -> Self {
        Self(expected)
    }
}

impl<Mp: Map + ?Sized, E, R> Expectation<Mp, R> for ContainsValueMatching<E>
where
    E: Expectation<Mp::Value, R>,
{
    evidence_items!(Mp);

    fn evaluate<'a>(
        &'a self,
        actual: &'a Mp,
        settings: &AssertionContext<'_, R>,
    ) -> Result<Self::Success<'a>, Self::Rejection<'a>> {
        MATCHING_VALUE.find(
            || actual.entries().map(|(_, value)| value),
            &self.0,
            Mp::RENDERING_ORDER,
            settings,
        )
    }

    const KIND: FailureKind = FailureKind::Matching;

    fn explain<'a>(
        &'a self,
        rejected: Option<(&'a Mp, Self::Rejection<'a>)>,
        failure: FailureBuilder,
        context: &AssertionContext<'_, R>,
    ) -> FailureBuilder {
        MATCHING_VALUE.explain::<Mp::Value, _, _>(
            &self.0,
            rejected.map(|(_, evidence)| evidence),
            failure,
            context,
        )
    }
}
