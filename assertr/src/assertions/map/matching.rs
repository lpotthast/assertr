//! Reusable map value matcher expectations.

use super::{EntryRejection, Map, MapLookup};
use crate::{
    AssertionContext, Expectation, ExpectationDiagnostics, ValueRenderer,
    expectation::Evidence,
    failure::{FailureBuilder, FailureKind, PathSegment},
    renderer::IntoRendered,
};

/// Checks map values with a reusable expectation and retains the original child evidence.
pub struct ContainsEntryMatching<'e, Q: ?Sized, E> {
    key: &'e Q,
    expected: E,
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
    E: ExpectationDiagnostics<Mp::Value, R>,
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
        super::entry::evaluate_entry(actual, self.key, &self.expected, context, |render| {
            PathSegment::Key(render.value(self.key).into_rendered())
        })
    }
}

impl<Mp: MapLookup<Q> + ?Sized, Q: ?Sized, E, R> ExpectationDiagnostics<Mp, R>
    for ContainsEntryMatching<'_, Q, E>
where
    E: ExpectationDiagnostics<Mp::Value, R>,
    R: ValueRenderer<Q>,
{
    const KIND: FailureKind = FailureKind::Matching;
    const FLATTEN: bool = true;

    fn explain<'a, Target>(
        &'a self,
        rejected: Option<(&'a Mp, Self::Rejection<'a>)>,
        failure: FailureBuilder<Target>,
        context: &AssertionContext<'_, R>,
    ) -> FailureBuilder<Target> {
        super::entry::explain_entry::<Mp::Value, _, _, _, _>(
            self.key,
            &self.expected,
            rejected.map(|(_, rejection)| rejection.evidence),
            failure,
            context,
        )
    }
}

/// Checks map values with a reusable expectation and retains the original child evidence.
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
    E: ExpectationDiagnostics<Mp::Value, R>,
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
        let mut context = settings.isolated_for_order(Mp::RENDERING_ORDER);
        let mut matched = false;
        for (_, value) in actual.entries() {
            let mut branch = context.isolated();
            if branch.evaluate(value, &self.0) {
                matched = true;
                break;
            }
            context.append(branch.into_evidence());
        }
        if matched {
            return Ok(());
        }
        if context.evidence.is_empty() {
            context.outcome(false, |context| context.describe(&self.0));
        }
        Err(context.into_evidence())
    }
}

impl<Mp: Map + ?Sized, E, R> ExpectationDiagnostics<Mp, R> for ContainsValueMatching<E>
where
    E: ExpectationDiagnostics<Mp::Value, R>,
{
    const KIND: FailureKind = FailureKind::Matching;
    const FLATTEN: bool = true;

    fn explain<'a, Target>(
        &'a self,
        rejected: Option<(&'a Mp, Self::Rejection<'a>)>,
        failure: FailureBuilder<Target>,
        context: &AssertionContext<'_, R>,
    ) -> FailureBuilder<Target> {
        match rejected {
            None => failure
                .relation("contains a matching value")
                .child(context.describe::<Mp::Value, _>(&self.0)),
            Some((_, rejection)) => {
                rejection.explain(failure.relation("does not contain a matching value"))
            }
        }
    }
}
