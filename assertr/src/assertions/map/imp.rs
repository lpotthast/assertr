//! Reusable native map expectations and their rejection evidence.

use super::{Map, MapLookup};
use crate::borrow_for::{BorrowFor, borrow_for};
use crate::{
    AssertionContext, Expectation, ExpectationDiagnostics, ValueRenderer,
    failure::{Fact, FailureBuilder, FailureKind, PathSegment},
    renderer::{GroupStyle, IntoRendered, RenderingOrder},
};
use alloc::{collections::BTreeSet, vec::Vec};
use core::{marker::PhantomData, ptr};

/// The stored entries the expected keys resolved to, identified by the address of their stored key.
///
/// The exact-entry assertions have to report the actual entries that no expectation named. The only
/// lookup direction [`MapLookup`] offers is expected key to stored entry, so instead of indexing
/// the expected keys (which would require `Hash` or `Ord` on the key type) the entries hit by the
/// lookups are remembered by identity, and every entry [`Map::entries`] yields that was not hit is
/// unexpected. This is what the "same stored key" contract of [`MapLookup::get_key_value`] exists
/// for. Duplicate expected keys resolve to the same entry and are therefore harmless. A zero-sized
/// key type gives every key the same address, but a map with such a key holds at most one entry, so
/// identity stays unambiguous.
pub(crate) struct FoundEntries<K>(BTreeSet<*const K>);

impl<K> FoundEntries<K> {
    pub(crate) fn new() -> Self {
        Self(BTreeSet::new())
    }

    pub(crate) fn record(&mut self, stored_key: &K) {
        self.0.insert(ptr::from_ref(stored_key));
    }

    pub(crate) fn contains(&self, stored_key: &K) -> bool {
        self.0.contains(&ptr::from_ref(stored_key))
    }

    fn unexpected_entries<'m, Mp>(&self, actual: &'m Mp) -> Vec<(&'m K, &'m Mp::Value)>
    where
        Mp: Map<Key = K> + ?Sized,
    {
        actual
            .entries()
            .filter(|(actual_key, _)| !self.contains(actual_key))
            .collect()
    }
}

/// Checks key presence through native borrowed lookup, without imposing key equality bounds.
pub struct ContainsKey<'e, Q: ?Sized>(&'e Q);
impl<'e, Q: ?Sized> ContainsKey<'e, Q> {
    /// Borrows a query for the map's native lookup.
    #[must_use]
    pub const fn new(expected: &'e Q) -> Self {
        Self(expected)
    }
}

impl<Mp: MapLookup<Q> + ?Sized, Q: ?Sized, R> Expectation<Mp, R> for ContainsKey<'_, Q> {
    type Success<'a>
        = &'a Mp::Value
    where
        Self: 'a,
        Mp: 'a;
    type Rejection<'a>
        = ()
    where
        Self: 'a,
        Mp: 'a;

    fn evaluate<'a>(
        &'a self,
        actual: &'a Mp,
        _: &AssertionContext<'_, R>,
    ) -> Result<Self::Success<'a>, Self::Rejection<'a>> {
        match actual.get_key_value(self.0) {
            Some((_, value)) => Ok(value),
            None => Err(()),
        }
    }
}

impl<Mp: MapLookup<Q> + ?Sized, Q: ?Sized, R> ExpectationDiagnostics<Mp, R> for ContainsKey<'_, Q>
where
    R: ValueRenderer<Mp::Key> + ValueRenderer<Mp::Value> + ValueRenderer<Q>,
{
    const KIND: FailureKind = FailureKind::Membership;

    fn explain<'a, Target>(
        &'a self,
        rejected: Option<(&'a Mp, Self::Rejection<'a>)>,
        failure: FailureBuilder<Target>,
        context: &AssertionContext<'_, R>,
    ) -> FailureBuilder<Target> {
        let render = context.render();
        let failure = match rejected {
            None => failure.relation("contains key"),
            Some((actual, ())) => failure
                .actual(render.map(actual))
                .relation("does not contain key"),
        };
        failure.expected(render.value(self.0))
    }
}

/// Checks key absence through native borrowed lookup, without imposing key equality bounds.
pub struct DoesNotContainKey<'e, Q: ?Sized>(&'e Q);
impl<'e, Q: ?Sized> DoesNotContainKey<'e, Q> {
    /// Borrows a query for the map's native lookup.
    #[must_use]
    pub const fn new(expected: &'e Q) -> Self {
        Self(expected)
    }
}

impl<Mp: MapLookup<Q> + ?Sized, Q: ?Sized, R> Expectation<Mp, R> for DoesNotContainKey<'_, Q> {
    type Success<'a>
        = ()
    where
        Self: 'a,
        Mp: 'a;
    type Rejection<'a>
        = ()
    where
        Self: 'a,
        Mp: 'a;

    fn evaluate<'a>(
        &'a self,
        actual: &'a Mp,
        _: &AssertionContext<'_, R>,
    ) -> Result<Self::Success<'a>, Self::Rejection<'a>> {
        if actual.get_key_value(self.0).is_some() {
            Err(())
        } else {
            Ok(())
        }
    }
}

impl<Mp: MapLookup<Q> + ?Sized, Q: ?Sized, R> ExpectationDiagnostics<Mp, R>
    for DoesNotContainKey<'_, Q>
where
    R: ValueRenderer<Mp::Key> + ValueRenderer<Mp::Value> + ValueRenderer<Q>,
{
    const KIND: FailureKind = FailureKind::Membership;

    fn explain<'a, Target>(
        &'a self,
        rejected: Option<(&'a Mp, Self::Rejection<'a>)>,
        failure: FailureBuilder<Target>,
        context: &AssertionContext<'_, R>,
    ) -> FailureBuilder<Target> {
        let render = context.render();
        let failure = match rejected {
            None => failure.relation("does not contain key"),
            Some((actual, ())) => failure.actual(render.map(actual)).relation("contains key"),
        };
        failure.unexpected(render.value(self.0))
    }
}

/// Checks map value membership using a borrowed view for the declared value type, without key
/// lookup.
pub struct ContainsValue<E>(E);

impl<E> ContainsValue<E> {
    /// Owns the expected operand.
    #[must_use]
    pub const fn new(expected: E) -> Self {
        Self(expected)
    }
}

impl<Mp: Map + ?Sized, E, R> Expectation<Mp, R> for ContainsValue<E>
where
    Mp::Value: PartialEq<E::View>,
    E: BorrowFor<Mp::Value>,
{
    type Success<'a>
        = ()
    where
        Self: 'a,
        Mp: 'a;
    type Rejection<'a>
        = &'a E::View
    where
        Self: 'a,
        Mp: 'a;

    fn evaluate<'a>(
        &'a self,
        actual: &'a Mp,
        _: &AssertionContext<'_, R>,
    ) -> Result<Self::Success<'a>, Self::Rejection<'a>> {
        let expected = borrow_for::<Mp::Value, _>(&self.0);
        if actual.entries().any(|(_, value)| value.eq(expected)) {
            Ok(())
        } else {
            Err(expected)
        }
    }
}

impl<Mp: Map + ?Sized, E, R> ExpectationDiagnostics<Mp, R> for ContainsValue<E>
where
    Mp::Value: PartialEq<E::View>,
    E: BorrowFor<Mp::Value>,
    R: ValueRenderer<Mp::Key> + ValueRenderer<Mp::Value> + ValueRenderer<E::View>,
{
    const KIND: FailureKind = FailureKind::Membership;

    fn explain<'a, Target>(
        &'a self,
        rejected: Option<(&'a Mp, Self::Rejection<'a>)>,
        failure: FailureBuilder<Target>,
        context: &AssertionContext<'_, R>,
    ) -> FailureBuilder<Target> {
        let render = context.render();
        let (failure, expected) = match rejected {
            None => (
                failure.relation("contains value"),
                borrow_for::<Mp::Value, _>(&self.0),
            ),
            Some((actual, expected)) => (
                failure
                    .actual(render.map(actual))
                    .relation("does not contain value"),
                expected,
            ),
        };
        failure.expected(render.value(expected))
    }
}

/// Checks that no map value equals a borrowed view for the declared value type, without key
/// lookup.
pub struct DoesNotContainValue<E>(E);

impl<E> DoesNotContainValue<E> {
    /// Owns the expected operand.
    #[must_use]
    pub const fn new(expected: E) -> Self {
        Self(expected)
    }
}

impl<Mp: Map + ?Sized, E, R> Expectation<Mp, R> for DoesNotContainValue<E>
where
    Mp::Value: PartialEq<E::View>,
    E: BorrowFor<Mp::Value>,
{
    type Success<'a>
        = ()
    where
        Self: 'a,
        Mp: 'a;
    type Rejection<'a>
        = &'a E::View
    where
        Self: 'a,
        Mp: 'a;

    fn evaluate<'a>(
        &'a self,
        actual: &'a Mp,
        _: &AssertionContext<'_, R>,
    ) -> Result<Self::Success<'a>, Self::Rejection<'a>> {
        let expected = borrow_for::<Mp::Value, _>(&self.0);
        if actual.entries().any(|(_, value)| value.eq(expected)) {
            Err(expected)
        } else {
            Ok(())
        }
    }
}

impl<Mp: Map + ?Sized, E, R> ExpectationDiagnostics<Mp, R> for DoesNotContainValue<E>
where
    Mp::Value: PartialEq<E::View>,
    E: BorrowFor<Mp::Value>,
    R: ValueRenderer<Mp::Key> + ValueRenderer<Mp::Value> + ValueRenderer<E::View>,
{
    const KIND: FailureKind = FailureKind::Membership;

    fn explain<'a, Target>(
        &'a self,
        rejected: Option<(&'a Mp, Self::Rejection<'a>)>,
        failure: FailureBuilder<Target>,
        context: &AssertionContext<'_, R>,
    ) -> FailureBuilder<Target> {
        let render = context.render();
        let (failure, expected) = match rejected {
            None => (
                failure.relation("does not contain value"),
                borrow_for::<Mp::Value, _>(&self.0),
            ),
            Some((actual, expected)) => (
                failure
                    .actual(render.map(actual))
                    .relation("contains value"),
                expected,
            ),
        };
        failure.unexpected(render.value(expected))
    }
}

/// Checks a map entry with one native lookup and one expected-value borrow.
pub struct ContainsEntry<'e, Q: ?Sized, E> {
    key: &'e Q,
    value: E,
}
impl<'e, Q: ?Sized, E> ContainsEntry<'e, Q, E> {
    /// Borrows the query and owns the expected value.
    #[must_use]
    pub const fn new(key: &'e Q, value: E) -> Self {
        Self { key, value }
    }
}

impl<Mp: MapLookup<Q> + ?Sized, Q: ?Sized, E, R> Expectation<Mp, R> for ContainsEntry<'_, Q, E>
where
    Mp::Value: PartialEq<E::View>,
    E: BorrowFor<Mp::Value>,
{
    type Success<'a>
        = ()
    where
        Self: 'a,
        Mp: 'a;
    type Rejection<'a>
        = Option<(&'a Mp::Value, &'a E::View)>
    where
        Self: 'a,
        Mp: 'a;

    fn evaluate<'a>(
        &'a self,
        actual: &'a Mp,
        _: &AssertionContext<'_, R>,
    ) -> Result<Self::Success<'a>, Self::Rejection<'a>> {
        let expected = borrow_for::<Mp::Value, _>(&self.value);
        match actual.get_key_value(self.key) {
            None => Err(None),
            Some((_, value)) if value.eq(expected) => Ok(()),
            Some((_, value)) => Err(Some((value, expected))),
        }
    }
}

impl<Mp: MapLookup<Q> + ?Sized, Q: ?Sized, E, R> ExpectationDiagnostics<Mp, R>
    for ContainsEntry<'_, Q, E>
where
    Mp::Value: PartialEq<E::View>,
    E: BorrowFor<Mp::Value>,
    R: ValueRenderer<Mp::Key>
        + ValueRenderer<Mp::Value>
        + ValueRenderer<Q>
        + ValueRenderer<E::View>,
{
    const KIND: FailureKind = FailureKind::Membership;

    fn explain<'a, Target>(
        &'a self,
        rejected: Option<(&'a Mp, Self::Rejection<'a>)>,
        failure: FailureBuilder<Target>,
        context: &AssertionContext<'_, R>,
    ) -> FailureBuilder<Target> {
        let render = context.render();
        match rejected {
            None => failure.relation("contains the entry").expected((
                render.value(self.key),
                render.value(borrow_for::<Mp::Value, _>(&self.value)),
            )),
            Some((actual, rejection)) => {
                let failure = failure.actual(render.map(actual));
                match rejection {
                    None => failure
                        .relation("does not contain key")
                        .expected(render.value(self.key)),
                    Some((value, expected)) => {
                        let mut child = context.isolated();
                        child.record_with(|context| {
                            let render = context.render();
                            FailureBuilder::detached::<Mp::Value>(FailureKind::Equality)
                                .actual(render.value(value))
                                .expected(render.value(expected))
                                .path([PathSegment::Key(
                                    render.value(self.key).into_rendered_compact(),
                                )])
                                .build()
                        });
                        child.into_evidence().explain(
                            failure.relation("does not contain the expected value at a key"),
                        )
                    }
                }
            }
        }
    }
}

/// Checks that a key is absent or maps to a different value, with one native lookup and one
/// unexpected-value borrow.
pub struct DoesNotContainEntry<'e, Q: ?Sized, E> {
    key: &'e Q,
    value: E,
}
impl<'e, Q: ?Sized, E> DoesNotContainEntry<'e, Q, E> {
    /// Borrows the query and owns the expected value.
    #[must_use]
    pub const fn new(key: &'e Q, value: E) -> Self {
        Self { key, value }
    }
}

impl<Mp: MapLookup<Q> + ?Sized, Q: ?Sized, E, R> Expectation<Mp, R>
    for DoesNotContainEntry<'_, Q, E>
where
    Mp::Value: PartialEq<E::View>,
    E: BorrowFor<Mp::Value>,
{
    type Success<'a>
        = ()
    where
        Self: 'a,
        Mp: 'a;
    type Rejection<'a>
        = &'a E::View
    where
        Self: 'a,
        Mp: 'a;

    fn evaluate<'a>(
        &'a self,
        actual: &'a Mp,
        _: &AssertionContext<'_, R>,
    ) -> Result<Self::Success<'a>, Self::Rejection<'a>> {
        let expected = borrow_for::<Mp::Value, _>(&self.value);
        if actual
            .get_key_value(self.key)
            .is_some_and(|(_, value)| value.eq(expected))
        {
            Err(expected)
        } else {
            Ok(())
        }
    }
}

impl<Mp: MapLookup<Q> + ?Sized, Q: ?Sized, E, R> ExpectationDiagnostics<Mp, R>
    for DoesNotContainEntry<'_, Q, E>
where
    Mp::Value: PartialEq<E::View>,
    E: BorrowFor<Mp::Value>,
    R: ValueRenderer<Mp::Key>
        + ValueRenderer<Mp::Value>
        + ValueRenderer<Q>
        + ValueRenderer<E::View>,
{
    const KIND: FailureKind = FailureKind::Membership;

    fn explain<'a, Target>(
        &'a self,
        rejected: Option<(&'a Mp, Self::Rejection<'a>)>,
        failure: FailureBuilder<Target>,
        context: &AssertionContext<'_, R>,
    ) -> FailureBuilder<Target> {
        let render = context.render();
        let expected = rejected.as_ref().map_or_else(
            || borrow_for::<Mp::Value, _>(&self.value),
            |(_, expected)| *expected,
        );
        let failure = match rejected {
            None => failure.relation("does not contain the entry"),
            Some((actual, _)) => failure
                .actual(render.map(actual))
                .relation("contains the entry"),
        };
        failure.unexpected((render.value(self.key), render.value(expected)))
    }
}

/// Retained missing query references from a membership rejection.
pub struct MissingKeysRejection<'a, Q: ?Sized> {
    missing: Vec<&'a Q>,
}

/// Requires each expected key query to resolve through native map lookup.
pub struct ContainsKeys<E, B = Vec<E>> {
    expected: B,
    operand: PhantomData<fn() -> E>,
}
impl<E, B: AsRef<[E]>> ContainsKeys<E, B> {
    /// Stores an array, slice, or vector of [repeatable expected data](crate#bulk-expected-data)
    /// without accessing its views.
    #[must_use]
    pub const fn new(expected: B) -> Self {
        Self {
            expected,
            operand: PhantomData,
        }
    }
}

// Keep the key projected in GAT-bearing impls so `Mp: 'a` carries its lifetime.
// Fully qualified views avoid a bound cycle without an independently inferred query parameter.
impl<Mp: Map + ?Sized, E, B, R> Expectation<Mp, R> for ContainsKeys<E, B>
where
    Mp: MapLookup<<E as BorrowFor<<Mp as Map>::Key>>::View>,
    E: BorrowFor<Mp::Key>,
    B: AsRef<[E]>,
{
    type Success<'a>
        = ()
    where
        Self: 'a,
        Mp: 'a;
    type Rejection<'a>
        = MissingKeysRejection<'a, E::View>
    where
        Self: 'a,
        Mp: 'a;

    fn evaluate<'a>(
        &'a self,
        actual: &'a Mp,
        _: &AssertionContext<'_, R>,
    ) -> Result<Self::Success<'a>, Self::Rejection<'a>> {
        let inputs = self.expected.as_ref();
        let mut missing = Vec::new();
        for key in inputs {
            let query = borrow_for::<Mp::Key, _>(key);
            if actual.get_key_value(query).is_none() {
                missing.push(query);
            }
        }
        if missing.is_empty() {
            Ok(())
        } else {
            Err(MissingKeysRejection { missing })
        }
    }
}

impl<Mp: Map<Key = K> + ?Sized, K, E, B, R> ExpectationDiagnostics<Mp, R> for ContainsKeys<E, B>
where
    Mp: MapLookup<E::View>,
    E: BorrowFor<K>,
    B: AsRef<[E]>,
    R: ValueRenderer<Mp::Key> + ValueRenderer<Mp::Value> + ValueRenderer<E::View>,
{
    const KIND: FailureKind = FailureKind::Membership;

    fn explain<'a, Target>(
        &'a self,
        rejected: Option<(&'a Mp, Self::Rejection<'a>)>,
        failure: FailureBuilder<Target>,
        context: &AssertionContext<'_, R>,
    ) -> FailureBuilder<Target> {
        let render = context.render();
        let expected = self.expected.as_ref();
        let failure = match rejected {
            None => failure.relation("contains all of"),
            Some((actual, MissingKeysRejection { missing, .. })) => failure
                .actual(render.map(actual))
                .relation("does not contain all of")
                .fact(Fact::labelled(
                    "Keys not found",
                    render.borrowed_values::<E::View, _>(&missing, GroupStyle::List),
                )),
        };
        failure.expected(render.borrowed_values::<E::View, _>(expected, GroupStyle::List))
    }
}

/// Retained missing keys, unexpected entries, and unequal values from an exact map comparison.
/// The fields remain private so diagnostics consume the original observations without repeating
/// lookup or value comparison. Missing and mismatched keys are selected query views, while
/// unexpected entries retain their stored keys and values.
pub struct ExactEntriesRejection<'a, K, V, EK: ?Sized, EV: ?Sized> {
    length: usize,
    missing: Vec<&'a EK>,
    unexpected: Vec<(&'a K, &'a V)>,
    mismatches: Vec<(&'a EK, &'a EV, &'a V)>,
}

/// Requires exact native key coverage and value equality, retaining every observation for
/// diagnostics.
pub struct ContainsExactlyEntries<EK, EV, B = Vec<(EK, EV)>> {
    expected: B,
    operands: PhantomData<fn() -> (EK, EV)>,
}
impl<EK, EV, B: AsRef<[(EK, EV)]>> ContainsExactlyEntries<EK, EV, B> {
    /// Stores [repeatable expected entries](crate#bulk-expected-data) without accessing their
    /// views.
    #[must_use]
    pub const fn new(expected: B) -> Self {
        Self {
            expected,
            operands: PhantomData,
        }
    }
}

impl<Mp: Map + ?Sized, EK, EV, B, R> Expectation<Mp, R> for ContainsExactlyEntries<EK, EV, B>
where
    Mp: MapLookup<<EK as BorrowFor<<Mp as Map>::Key>>::View>,
    EK: BorrowFor<Mp::Key>,
    Mp::Value: PartialEq<EV::View>,
    EV: BorrowFor<Mp::Value>,
    B: AsRef<[(EK, EV)]>,
{
    type Success<'a>
        = ()
    where
        Self: 'a,
        Mp: 'a;
    type Rejection<'a>
        = ExactEntriesRejection<'a, Mp::Key, Mp::Value, EK::View, EV::View>
    where
        Self: 'a,
        Mp: 'a;

    fn evaluate<'a>(
        &'a self,
        actual: &'a Mp,
        _: &AssertionContext<'_, R>,
    ) -> Result<Self::Success<'a>, Self::Rejection<'a>> {
        let expected = self.expected.as_ref();
        let length = actual.length();
        let mut missing = Vec::new();
        let mut mismatches = Vec::new();
        let mut found = FoundEntries::new();
        for (key, value) in expected {
            let key = borrow_for::<Mp::Key, _>(key);
            let expected_value = borrow_for::<Mp::Value, _>(value);
            match actual.get_key_value(key) {
                None => missing.push(key),
                Some((stored_key, value)) => {
                    found.record(stored_key);
                    if !value.eq(expected_value) {
                        mismatches.push((key, expected_value, value));
                    }
                }
            }
        }
        let unexpected = found.unexpected_entries(actual);
        if length == expected.len()
            && missing.is_empty()
            && unexpected.is_empty()
            && mismatches.is_empty()
        {
            Ok(())
        } else {
            Err(ExactEntriesRejection {
                length,
                missing,
                unexpected,
                mismatches,
            })
        }
    }
}

impl<Mp: Map<Key = K> + ?Sized, K, EK, EV, B, R> ExpectationDiagnostics<Mp, R>
    for ContainsExactlyEntries<EK, EV, B>
where
    Mp: MapLookup<EK::View>,
    EK: BorrowFor<K>,
    Mp::Value: PartialEq<EV::View>,
    EV: BorrowFor<Mp::Value>,
    B: AsRef<[(EK, EV)]>,
    R: ValueRenderer<Mp::Key>
        + ValueRenderer<Mp::Value>
        + ValueRenderer<EK::View>
        + ValueRenderer<usize>
        + ValueRenderer<EV::View>,
{
    const KIND: FailureKind = FailureKind::Equality;

    fn explain<'a, Target>(
        &'a self,
        rejected: Option<(&'a Mp, Self::Rejection<'a>)>,
        failure: FailureBuilder<Target>,
        context: &AssertionContext<'_, R>,
    ) -> FailureBuilder<Target> {
        let render = context.render();
        let expected = self.expected.as_ref();
        let failure = match rejected {
            None => failure.relation("contains exactly"),
            Some((actual, rejection)) => {
                let ExactEntriesRejection {
                    length,
                    missing,
                    unexpected,
                    mismatches,
                    ..
                } = rejection;
                let mut children = context.isolated_for_order(Mp::RENDERING_ORDER);
                for (key, expected, value) in mismatches {
                    children.record_with(|context| {
                        let render = context.render();
                        FailureBuilder::detached::<Mp::Value>(FailureKind::Equality)
                            .actual(render.value(value))
                            .expected(render.value(expected))
                            .path([PathSegment::Key(render.value(key).into_rendered_compact())])
                            .build()
                    });
                }
                let mut failure = failure
                    .actual(render.map(actual))
                    .relation("does not contain exactly");
                if length != expected.len() {
                    failure = failure
                        .fact(Fact::labelled("Actual length", render.value(&length)))
                        .fact(Fact::labelled(
                            "Expected length",
                            render.value(&expected.len()),
                        ));
                }
                if !missing.is_empty() {
                    failure = failure.fact(Fact::labelled(
                        "Keys not found",
                        render.borrowed_values::<EK::View, _>(&missing, GroupStyle::List),
                    ));
                }
                if !unexpected.is_empty() {
                    failure = failure.fact(Fact::labelled(
                        "Unexpected entries",
                        render.entry_list::<Mp::Key, Mp::Value, _, _, _>(
                            &unexpected,
                            Mp::RENDERING_ORDER,
                        ),
                    ));
                }
                children.into_evidence().explain(failure)
            }
        };
        failure.expected(
            render.entry_list::<EK::View, EV::View, _, _, _>(
                expected,
                RenderingOrder::PreserveIteration,
            ),
        )
    }
}

#[cfg(test)]
mod tests {
    mod child_paths {
        use super::super::*;
        use crate::{prelude::*, test_support::CustomValueRenderer};
        use alloc::collections::BTreeMap;
        use core::fmt;

        #[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
        struct Key {
            id: i32,
        }

        fn key_failures<R>(renderer: R, budget: RenderingBudget) -> AssertionFailures
        where
            R: ValueRenderer<Key> + ValueRenderer<i32> + ValueRenderer<usize>,
        {
            let actual = BTreeMap::from([(Key { id: 1 }, 1)]);
            assert_that!(actual)
                .with_renderer(renderer)
                .with_rendering_budget(budget)
                .with_location(false)
                .capture(|it| {
                    it.contains_entry(&Key { id: 1 }, 2)
                        .contains_exactly_entries([(Key { id: 1 }, 2)])
                        .contains_entry_matching(&Key { id: 1 }, matchers::eq(2))
                        .matches(entries_are![(Key { id: 1 }, matchers::eq(2))])
                        .matches(entries_are![])
                })
        }

        #[test]
        fn structured_keys_have_compact_child_headings() {
            let failures = key_failures(DebugRenderer, RenderingBudget::unlimited());
            assert_that!(failures).has_length(5);
            for (index, failure) in failures.iter().enumerate() {
                assert_that!(failure.children).has_length(1);
                let expected = if index == 4 {
                    indoc::formatdoc! {"
                    Nested failures:
                      - At [Key {{ id: 1 }}]:
                        has an unexpected key
                "}
                } else {
                    indoc::formatdoc! {"
                    Nested failures:
                      - At [Key {{ id: 1 }}]:
                        Expected: 2

                          Actual: 1
                "}
                };
                assert_that!(ToHumanReadableText::render_child(&failure.children[0]))
                    .is_equal_to(expected);
            }
        }

        #[test]
        fn compact_key_paths_preserve_custom_rendering_metadata_and_leaf_budgets() {
            struct AlternateAwareRenderer;
            impl<T: fmt::Debug + ?Sized> ValueRenderer<T> for AlternateAwareRenderer {
                fn fmt(&self, value: &T, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                    if f.alternate() {
                        write!(f, "pretty({value:#?})")
                    } else {
                        write!(f, "compact({value:?})")
                    }
                }
            }

            let compact = "compact(Key { id: 1 })";
            for maximum in [0, 8, usize::MAX] {
                let failures = key_failures(
                    AlternateAwareRenderer,
                    RenderingBudget::default().with_max_leaf_characters(maximum),
                );
                assert_that!(failures).has_length(5);
                for failure in &failures {
                    assert_that!(failure.children).has_length(1);
                    let [PathSegment::Key(key)] = failure.children[0].path.as_slice() else {
                        panic!("expected one key segment");
                    };
                    let retained = compact.len().min(maximum);
                    assert_that!(key.type_name()).is_equal_to(Some(core::any::type_name::<Key>()));
                    assert_that!(&key.body).is_equal_to(&crate::renderer::RenderedBody::Text {
                        text: compact[..retained].into(),
                        omitted_characters: compact.len() - retained,
                    });
                }
            }
        }

        #[test]
        fn equality_and_matching_share_budgeted_query_key_paths() {
            let actual = BTreeMap::from([(String::from("alpha"), 1)]);
            let failures = assert_that!(actual)
                .with_renderer(CustomValueRenderer)
                .with_rendering_budget(RenderingBudget::default().with_max_leaf_characters(8))
                .capture(|it| {
                    it.contains_entry("alpha", 2)
                        .contains_exactly_entries([("alpha", 2)])
                        .contains_entry_matching("alpha", matchers::eq(2))
                });
            assert_that!(failures).has_length(3);
            let matcher_path = &failures[2].children[0].path;
            for failure in &failures {
                assert_that!(failure.children).has_length(1);
                let child = &failure.children[0];
                assert_that!(child.facts).is_empty();
                assert_that!(child.path).is_equal_to(matcher_path);
                let [PathSegment::Key(key)] = child.path.as_slice() else {
                    panic!("expected one query key segment");
                };
                assert_that!(key.type_name).is_equal_to(Some(core::any::type_name::<str>()));
                assert_that!(&key.body).is_equal_to(&crate::renderer::RenderedBody::Text {
                    text: String::from("custom(\""),
                    omitted_characters: 7,
                });
            }
        }
    }

    mod evidence_budget {
        use super::super::ContainsExactlyEntries;
        use crate::{failure::PathSegment, prelude::*};
        use alloc::collections::BTreeMap;

        #[test]
        fn keyed_value_mismatches_respect_the_budget_and_count_omissions() {
            for limit in [0, 1, usize::MAX] {
                let failures = assert_that!(BTreeMap::from([("a", 1), ("b", 2)]))
                    .with_rendering_budget(RenderingBudget::default().with_max_items(limit))
                    .capture(|it| {
                        it.contains_entry("a", 9)
                            .contains_exactly_entries([("a", 9), ("b", 9)])
                    });

                assert_that!(failures).has_length(2);
                let retained = limit.min(1);
                assert_that!(failures[0].children).has_length(retained);
                assert_that!(failures[0].omitted_children).is_equal_to(1 - retained);
                let retained = limit.min(2);
                assert_that!(failures[1].children).has_length(retained);
                assert_that!(failures[1].omitted_children).is_equal_to(2 - retained);
                assert_that!(failures[1].facts).is_empty();
            }
        }

        #[test]
        fn keyed_value_mismatches_share_the_remaining_allowance_of_a_composition() {
            let maps = [
                BTreeMap::from([("a", 1), ("b", 2)]),
                BTreeMap::from([("a", 1), ("b", 2)]),
            ];
            let failures = assert_that!(maps)
                .with_rendering_budget(RenderingBudget::default().with_max_items(2))
                .capture(|it| {
                    it.matches(elements_are![
                        ContainsExactlyEntries::new([("a", 9), ("b", 9)]),
                        ContainsExactlyEntries::new([("a", 9), ("b", 9)]),
                    ])
                });

            assert_that!(failures).has_length(1);
            let [first, second] = failures[0].children.as_slice() else {
                panic!("expected one failure per element");
            };
            assert_that!(&first.path).contains_exactly([PathSegment::Index(0)]);
            assert_that!(first.children).has_length(2);
            // The first element's failure already occupies one of the two slots.
            assert_that!(&second.path).contains_exactly([PathSegment::Index(1)]);
            assert_that!(second.children).has_length(1);
            assert_that!(second.omitted_children).is_equal_to(1);
        }
    }
}
