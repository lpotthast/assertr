use crate::{
    AssertionContext, Expectation, ExpectationDiagnostics, ValueRenderer,
    assertions::map::{Map, MapKeyQuery, MapLookup},
    expectation::Evidence,
    failure::{FailureBuilder, FailureKind, PathSegment},
    renderer::IntoRendered,
};

/// A value matcher under one native map key query.
pub struct Entry<K, M> {
    key: K,
    matcher: M,
}

/// The original stored key and scoped value evidence from a rejected entry expectation.
pub struct EntryRejection<'a, K: ?Sized> {
    key: Option<&'a K>,
    pub(super) evidence: Evidence,
}

/// Matches a value at a key using the map's native lookup relation.
pub fn entry<K, M>(key: K, matcher: M) -> Entry<K, M> {
    Entry { key, matcher }
}

// Both borrowed native queries and owned/adapted keys execute this operation.
// The stored key is retained even if its value rejects. Exact entry checks never look it up again.
pub(super) fn evaluate_entry<'a, Mp, Q, E, R>(
    actual: &'a Mp,
    query: &Q,
    expected: &E,
    settings: &AssertionContext<'_, R>,
    path: impl FnOnce(crate::renderer::RenderingContext<'_, R>) -> PathSegment,
) -> Result<&'a Mp::Key, EntryRejection<'a, Mp::Key>>
where
    Mp: MapLookup<Q> + ?Sized,
    Q: ?Sized,
    E: ExpectationDiagnostics<Mp::Value, R>,
{
    let mut context = settings.isolated();
    let found = actual.get_key_value(query);
    let key = found.map(|(key, _)| key);
    let evaluate = |context: &mut AssertionContext<'_, R>| {
        if let Some((_, value)) = found {
            let matched = context.evaluate(value, expected);
            if !matched && context.evidence.is_empty() {
                context.outcome(false, |context| context.describe(expected));
            }
            matched
        } else {
            context.outcome(false, |_| {
                FailureBuilder::detached::<()>(FailureKind::Matching)
                    .relation("contains the required key")
                    .build()
            })
        }
    };
    let matched = if context.is_diagnostic() {
        context.scoped(path(context.render()), evaluate)
    } else {
        evaluate(&mut context)
    };
    if matched {
        Ok(key.expect("a matching entry has a stored key"))
    } else {
        Err(EntryRejection {
            key,
            evidence: context.into_evidence(),
        })
    }
}

// Query storage differs, but both entry forms use the same diagnostic grammar.
pub(super) fn explain_entry<V: ?Sized, K: ?Sized, E, R, Target>(
    key: &K,
    expected: &E,
    rejection: Option<Evidence>,
    failure: FailureBuilder<Target>,
    context: &AssertionContext<'_, R>,
) -> FailureBuilder<Target>
where
    E: ExpectationDiagnostics<V, R>,
    R: ValueRenderer<K>,
{
    match rejection {
        None => failure
            .relation("contains a matching entry")
            .expected(context.render().value(key))
            .child(context.describe(expected)),
        Some(evidence) => evidence.explain(failure.relation("does not contain a matching entry")),
    }
}

impl<MapType, R, K, M> Expectation<MapType, R> for Entry<K, M>
where
    MapType: Map + MapLookup<<K as MapKeyQuery<<MapType as Map>::Key>>::Query> + ?Sized,
    K: MapKeyQuery<MapType::Key>,
    M: ExpectationDiagnostics<MapType::Value, R>,
    R: ValueRenderer<K>,
{
    type Success<'a>
        = &'a MapType::Key
    where
        Self: 'a,
        MapType: 'a;
    type Rejection<'a>
        = EntryRejection<'a, MapType::Key>
    where
        Self: 'a,
        MapType: 'a;

    fn evaluate<'a>(
        &'a self,
        actual: &'a MapType,
        context: &AssertionContext<'_, R>,
    ) -> Result<Self::Success<'a>, Self::Rejection<'a>> {
        evaluate_entry(
            actual,
            self.key.as_query(),
            &self.matcher,
            context,
            |render| PathSegment::Key(render.value(&self.key).into_rendered()),
        )
    }
}

impl<MapType, R, K, M> ExpectationDiagnostics<MapType, R> for Entry<K, M>
where
    MapType: Map + MapLookup<<K as MapKeyQuery<<MapType as Map>::Key>>::Query> + ?Sized,
    K: MapKeyQuery<MapType::Key>,
    M: ExpectationDiagnostics<MapType::Value, R>,
    R: ValueRenderer<K>,
{
    const KIND: FailureKind = FailureKind::Matching;
    const FLATTEN: bool = true;

    fn explain<'a, Target>(
        &'a self,
        rejected: Option<(&'a MapType, Self::Rejection<'a>)>,
        failure: FailureBuilder<Target>,
        context: &AssertionContext<'_, R>,
    ) -> FailureBuilder<Target> {
        explain_entry::<MapType::Value, _, _, _, _>(
            &self.key,
            &self.matcher,
            rejected.map(|(_, rejection)| rejection.evidence),
            failure,
            context,
        )
    }
}

impl<K, M> Entry<K, M> {
    // Keyed lists retain the original lookup identity while committing already scoped children.
    pub(super) fn evaluate_and_record<'a, Mp, R>(
        &'a self,
        actual: &'a Mp,
        context: &mut AssertionContext<'_, R>,
    ) -> (bool, Option<&'a Mp::Key>)
    where
        Mp: Map + MapLookup<<K as MapKeyQuery<<Mp as Map>::Key>>::Query> + ?Sized,
        K: MapKeyQuery<Mp::Key>,
        M: ExpectationDiagnostics<Mp::Value, R>,
        R: ValueRenderer<K>,
    {
        match self.evaluate(actual, context) {
            Ok(key) => (true, Some(key)),
            Err(EntryRejection { key, evidence }) => {
                context.append(evidence);
                (false, key)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::entry;
    use crate::{
        assertions::core::partial_eq::equal_to,
        expectation::anything,
        failure::{FailureKind, PathSegment},
        prelude::*,
    };
    use alloc::{collections::BTreeMap, string::String};

    #[test]
    fn accepts_borrowed_key_queries() {
        let actual = BTreeMap::from([(String::from("a"), 1), (String::from("b"), 2)]);

        assert_that!(actual).matches(entry("a", equal_to(1)));
        let failures = assert_that!(actual).capture(|it| it.matches(entry("a", equal_to(2))));
        assert_that!(failures).has_length(1);
        let failures = assert_that!(actual).capture(|it| it.matches(entry("missing", anything())));
        assert_that!(failures).has_length(1);
    }

    #[test]
    fn scopes_value_failures_to_the_queried_key() {
        let failures = assert_that!(BTreeMap::from([("a", 1)]))
            .capture(|it| it.matches(entry("a", equal_to(2))));

        assert_that!(failures).contains_exactly_satisfying([
            |item: AssertThat<AssertionFailure, Capture>| {
                item.derive(|subject| &subject.children[0].path)
                    .contains_exactly_satisfying([|element: AssertThat<PathSegment, Capture>| {
                        element
                            .derive(|value| value)
                            .is_matching(pattern!(PathSegment::Key(_)));
                    }]);
                item.derive(|subject| &subject.children[0].kind)
                    .is_equal_to(FailureKind::Equality);
            },
        ]);
    }
}
