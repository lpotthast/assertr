//! Reusable map value matcher expectations.

use super::{EntryRejection, Map, MapLookup};
use crate::{
    AssertionContext, Expectation, ValueRenderer,
    assertions::collection::matching::MatchingItem,
    expectation::Evidence,
    failure::{FailureBuilder, FailureKind},
};

const MATCHING_VALUE: MatchingItem = MatchingItem {
    contains: "contains a matching value",
    does_not_contain: "does not contain a matching value",
};

/// Checks the value at one native key query with a reusable expectation and retains the original
/// child evidence, located at the key.
#[derive(Debug)]
pub struct ContainsEntryMatching<'e, Q: ?Sized, E> {
    key: &'e Q,
    expected: E,
}

impl<Q: ?Sized, E: Clone> Clone for ContainsEntryMatching<'_, Q, E> {
    fn clone(&self) -> Self {
        Self {
            key: self.key,
            expected: self.expected.clone(),
        }
    }
}
impl<'e, Q: ?Sized, E> ContainsEntryMatching<'e, Q, E> {
    /// Borrows a native key query and stores its value expectation.
    #[must_use]
    pub const fn new(key: &'e Q, expected: E) -> Self {
        Self { key, expected }
    }
}

impl<Mp: MapLookup<Q> + ?Sized, Q: ?Sized, E, R> Expectation<Mp, R>
    for ContainsEntryMatching<'_, Q, E>
where
    E: Expectation<Mp::Value, R>,
    R: ValueRenderer<Q>,
{
    type Success<'a>
        = &'a Mp::Key
    where
        Self: 'a,
        Mp: 'a;
    type Rejection<'a>
        = EntryRejection<'a, Mp::Key>
    where
        Self: 'a,
        Mp: 'a;

    fn evaluate<'a>(
        &'a self,
        actual: &'a Mp,
        context: &AssertionContext<'_, R>,
    ) -> Result<Self::Success<'a>, Self::Rejection<'a>> {
        super::entry::evaluate_entry(actual, self.key, &self.expected, context)
    }

    const KIND: FailureKind = FailureKind::Matching;
    const FLATTEN: bool = true;

    fn explain<'a>(
        &'a self,
        rejected: Option<(&'a Mp, Self::Rejection<'a>)>,
        failure: FailureBuilder,
        context: &AssertionContext<'_, R>,
    ) -> FailureBuilder {
        super::entry::explain_entry::<Mp::Value, _, _, _>(
            || self.key,
            &self.expected,
            rejected.map(|(_, rejection)| rejection.evidence),
            failure,
            context,
        )
    }
}

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
    type Success<'a>
        = ()
    where
        Self: 'a,
        Mp: 'a;
    type Rejection<'a>
        = Evidence
    where
        Self: 'a,
        Mp: 'a;

    fn evaluate<'a>(
        &'a self,
        actual: &'a Mp,
        settings: &AssertionContext<'_, R>,
    ) -> Result<Self::Success<'a>, Self::Rejection<'a>> {
        MATCHING_VALUE.find(
            actual.entries().map(|(_, value)| value),
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
