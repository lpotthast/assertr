use crate::{
    assertions::map::{Map, MapLookup},
    borrow_for::{BorrowFor, borrow_for},
    expectation::{AssertionContext, Evidence, Expectation, context::unsatisfied},
    failure::{FailureBuilder, FailureKind, PathSegment},
    renderer::{RenderingContext, ValueRenderer},
};

/// A value matcher under one native map key query.
///
/// `K` stores the supplied operand. Its [`BorrowFor`] view is selected using the map
/// key type as context. Evaluation borrows that view once for native lookup and the matcher path.
/// Only the view and nested matcher need rendering capabilities. See [`super::MapAssertions`]
/// for custom operand registration and lookup requirements.
#[derive(Debug, Clone)]
pub struct Entry<K, M> {
    pub(super) key: K,
    pub(super) matcher: M,
}

/// The rejection of [`entry`](crate::matchers::entry) and
/// [`map::ContainsEntryMatching`](crate::matchers::map::ContainsEntryMatching): the stored key
/// that was found, if any, and the evidence explaining the rejection.
///
/// `K` is the map's key type. When the key is present but its value was rejected, the rejection
/// borrows the stored key and holds the value matcher's nested failures. When the key is missing,
/// it holds no stored key and a "contains the required key" failure instead. Diagnostic
/// evaluation scopes this evidence under the key's path. Explanation reports it without looking
/// the key up or evaluating the value again. The contents are private. Pass the
/// rejection back to the `explain` method of the expectation that produced it.
#[derive(Debug)]
pub struct EntryRejection<'a, K: ?Sized> {
    key: Option<&'a K>,
    pub(super) evidence: Evidence,
}

/// Requires a map to contain `key`, with a value matching `matcher`.
///
/// The key is a lookup operand, not a matcher. It is looked up with the map's native
/// [`MapLookup`], and may be a borrowed form of the stored key. Extra entries are allowed. Collect
/// entries in a list for [`entries_are`](super::entries_are) to require exact keys.
///
/// ```
/// use assertr::{matchers::{entry, gt}, prelude::*};
/// use std::collections::BTreeMap;
///
/// let map = BTreeMap::from([(String::from("retries"), 3), (String::from("timeout"), 30)]);
/// assert_that!(map).matches(entry("retries", gt(0)));
/// ```
pub fn entry<K, M>(key: K, matcher: M) -> Entry<K, M> {
    Entry { key, matcher }
}

/// The path segment locating evidence at a key, rendered compactly.
pub(super) fn key_segment<Q: ?Sized, R: ValueRenderer<Q>>(
    render: RenderingContext<'_, R>,
    key: &Q,
) -> PathSegment {
    PathSegment::Key(render.compact().value(key))
}

// Both borrowed native queries and owned/adapted keys execute this operation.
// The stored key is retained even if its value rejects. Exact entry checks never look it up again.
pub(super) fn evaluate_entry<'a, Mp, Q, E, R>(
    actual: &'a Mp,
    query: &Q,
    expected: &E,
    settings: &AssertionContext<'_, R>,
) -> Result<&'a Mp::Key, EntryRejection<'a, Mp::Key>>
where
    Mp: MapLookup<Q> + ?Sized,
    Q: ?Sized,
    E: Expectation<Mp::Value, R>,
    R: ValueRenderer<Q>,
{
    let mut context = settings.isolated();
    let found = actual.get_key_value(query);
    let key = found.map(|(key, _)| key);
    let evaluate = |context: &mut AssertionContext<'_, R>| {
        if let Some((_, value)) = found {
            let matched = context.evaluate(value, expected);
            context.complete(matched, |context| context.describe(expected))
        } else {
            context.record(|_| {
                unsatisfied(
                    FailureBuilder::new::<()>(FailureKind::Matching)
                        .relation("contains the required key")
                        .build(),
                )
            });
            false
        }
    };
    let matched = if context.is_diagnostic() {
        context.scoped(key_segment(context.render(), query), evaluate)
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

// Query storage differs, but both entry forms use the same diagnostic grammar. Only a requirement
// description borrows the key. A rejection explains its retained evidence.
pub(super) fn explain_entry<'k, V: ?Sized, K: ?Sized + 'k, E, R>(
    key: impl FnOnce() -> &'k K,
    expected: &E,
    rejection: Option<Evidence>,
    failure: FailureBuilder,
    context: &AssertionContext<'_, R>,
) -> FailureBuilder
where
    E: Expectation<V, R>,
    R: ValueRenderer<K>,
{
    match rejection {
        None => failure
            .relation("contains a matching entry")
            .expected(context.render().value(key()))
            .child(context.describe(expected)),
        Some(evidence) => failure
            .relation("does not contain a matching entry")
            .evidence(evidence),
    }
}

impl<MapType, R, K, M> Expectation<MapType, R> for Entry<K, M>
where
    MapType: Map + MapLookup<<K as BorrowFor<<MapType as Map>::Key>>::View> + ?Sized,
    K: BorrowFor<MapType::Key>,
    M: Expectation<MapType::Value, R>,
    R: ValueRenderer<K::View>,
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
        let query = borrow_for::<MapType::Key, _>(&self.key);
        evaluate_entry(actual, query, &self.matcher, context)
    }

    const KIND: FailureKind = FailureKind::Matching;
    const FLATTEN: bool = true;

    fn explain<'a>(
        &'a self,
        rejected: Option<(&'a MapType, Self::Rejection<'a>)>,
        failure: FailureBuilder,
        context: &AssertionContext<'_, R>,
    ) -> FailureBuilder {
        explain_entry::<MapType::Value, _, _, _>(
            || borrow_for::<MapType::Key, _>(&self.key),
            &self.matcher,
            rejected.map(|(_, rejection)| rejection.evidence),
            failure,
            context,
        )
    }
}

/// Evaluates one keyed operand and commits its scoped children, returning truth and the stored key.
/// A present key is returned even when its value rejects, so exact checks never look it up again.
pub(super) fn record_entry<'a, Mp, StoredKey, K, M, R>(
    key: &'a K,
    matcher: &M,
    actual: &'a Mp,
    context: &mut AssertionContext<'_, R>,
) -> (bool, Option<&'a Mp::Key>)
where
    Mp: Map<Key = StoredKey> + MapLookup<K::View> + ?Sized,
    K: BorrowFor<StoredKey>,
    M: Expectation<Mp::Value, R>,
    R: ValueRenderer<K::View>,
{
    match evaluate_entry(actual, borrow_for::<StoredKey, _>(key), matcher, context) {
        Ok(key) => (true, Some(key)),
        Err(EntryRejection { key, evidence }) => {
            context.append(evidence);
            (false, key)
        }
    }
}

#[cfg(test)]
mod tests {
    use alloc::{collections::BTreeMap, string::String};

    use super::entry;
    use crate::{
        assertions::core::partial_eq::eq,
        failure::{FailureKind, PathSegment},
        matchers::anything,
        prelude::*,
    };

    #[test]
    fn accepts_borrowed_key_queries() {
        let actual = BTreeMap::from([(String::from("a"), 1), (String::from("b"), 2)]);

        assert_that!(actual).matches(entry("a", eq(1)));
        let failures = assert_that!(actual).capture(|it| it.matches(entry("a", eq(2))));
        assert_that!(failures).has_length(1);
        let failures = assert_that!(actual).capture(|it| it.matches(entry("missing", anything())));
        assert_that!(failures).has_length(1);
    }

    #[test]
    fn scopes_value_failures_to_the_queried_key() {
        let failures =
            assert_that!(BTreeMap::from([("a", 1)])).capture(|it| it.matches(entry("a", eq(2))));

        assert_that!(&failures).has_length(1);
        let child = &failures[0].children[0];
        assert_that!(&child.path).contains_exactly_matching([pattern!(PathSegment::Key(_))]);
        assert_that!(child.kind).is_equal_to(FailureKind::Equality);
    }
}
