//! Reusable native map expectations and their rejection evidence.

use super::{Map, MapKeyQuery, MapLookup};
use crate::{
    AssertionContext, AssertionFailure, Expectation, ExpectationDiagnostics, ValueRenderer,
    failure::{Fact, FailureBuilder, FailureKind, adapter::ToHumanReadableText},
    renderer::{GroupStyle, RenderingOrder},
};
use alloc::{collections::BTreeSet, string::String, vec::Vec};
use core::{borrow::Borrow, marker::PhantomData, ptr};

/// Whether diagnostics over `Mp`'s entries are sorted by their rendered text because the map has no
/// deterministic iteration order.
fn sorts_for_rendering<Mp: Map + ?Sized>() -> bool {
    Mp::RENDERING_ORDER == RenderingOrder::SortByRenderedText
}

/// Flattens the failures raised for the values under the expected keys into the children of one
/// failure. Every failure is already located at its key.
///
/// Keys keep the order the caller gave them. For a map whose rendering sorts entries by their text,
/// the keys are sorted the same way, so the nested failures and the rendered map agree in order. At
/// most `maximum` keys are kept. Returns the children and the number of omitted keys.
fn keyed_children<Mp: Map + ?Sized>(
    mut unsatisfied: Vec<Vec<AssertionFailure>>,
    maximum: usize,
) -> (Vec<AssertionFailure>, usize) {
    if sorts_for_rendering::<Mp>() {
        unsatisfied.sort_by_cached_key(|failures| {
            failures
                .iter()
                .map(|failure| ToHumanReadableText.render(failure).into_string())
                .collect::<String>()
        });
    }
    let omitted = unsatisfied.len().saturating_sub(maximum);
    unsatisfied.truncate(maximum);
    (unsatisfied.into_iter().flatten().collect(), omitted)
}

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

/// Checks key presence through native borrowed lookup, without imposing key equality bounds.
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

/// Checks map value membership using heterogeneous equality without key lookup.
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
    Mp::Value: PartialEq<E>,
{
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
        if actual.entries().any(|(_, value)| value.eq(&self.0)) {
            Ok(())
        } else {
            Err(())
        }
    }
}

impl<Mp: Map + ?Sized, E, R> ExpectationDiagnostics<Mp, R> for ContainsValue<E>
where
    Mp::Value: PartialEq<E>,
    R: ValueRenderer<Mp::Key> + ValueRenderer<Mp::Value> + ValueRenderer<E>,
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
            None => failure.relation("contains value"),
            Some((actual, ())) => failure
                .actual(render.map(actual))
                .relation("does not contain value"),
        };
        failure.expected(render.value(&self.0))
    }
}

/// Checks map value membership using heterogeneous equality without key lookup.
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
    Mp::Value: PartialEq<E>,
{
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
        if actual.entries().any(|(_, value)| value.eq(&self.0)) {
            Err(())
        } else {
            Ok(())
        }
    }
}

impl<Mp: Map + ?Sized, E, R> ExpectationDiagnostics<Mp, R> for DoesNotContainValue<E>
where
    Mp::Value: PartialEq<E>,
    R: ValueRenderer<Mp::Key> + ValueRenderer<Mp::Value> + ValueRenderer<E>,
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
            None => failure.relation("does not contain value"),
            Some((actual, ())) => failure
                .actual(render.map(actual))
                .relation("contains value"),
        };
        failure.unexpected(render.value(&self.0))
    }
}

/// Checks a map entry with one native lookup and one expected-value borrow.
pub struct ContainsEntry<'e, Q: ?Sized, E, B = E> {
    key: &'e Q,
    value: B,
    operand: PhantomData<fn() -> E>,
}
impl<'e, Q: ?Sized, E> ContainsEntry<'e, Q, E> {
    /// Borrows the query and owns the expected value.
    #[must_use]
    pub const fn new(key: &'e Q, value: E) -> Self {
        Self {
            key,
            value,
            operand: PhantomData,
        }
    }

    pub(super) fn with_storage<B: Borrow<E>>(key: &'e Q, value: B) -> ContainsEntry<'e, Q, E, B> {
        ContainsEntry {
            key,
            value,
            operand: PhantomData,
        }
    }
}

impl<Mp: MapLookup<Q> + ?Sized, Q: ?Sized, E, B, R> Expectation<Mp, R>
    for ContainsEntry<'_, Q, E, B>
where
    Mp::Value: PartialEq<E>,
    B: Borrow<E>,
{
    type Success<'a>
        = ()
    where
        Self: 'a,
        Mp: 'a;
    type Rejection<'a>
        = Option<(&'a Mp::Value, &'a E)>
    where
        Self: 'a,
        Mp: 'a;

    fn evaluate<'a>(
        &'a self,
        actual: &'a Mp,
        _: &AssertionContext<'_, R>,
    ) -> Result<Self::Success<'a>, Self::Rejection<'a>> {
        let expected = self.value.borrow();
        match actual.get_key_value(self.key) {
            None => Err(None),
            Some((_, value)) if value.eq(expected) => Ok(()),
            Some((_, value)) => Err(Some((value, expected))),
        }
    }
}

impl<Mp: MapLookup<Q> + ?Sized, Q: ?Sized, E, B, R> ExpectationDiagnostics<Mp, R>
    for ContainsEntry<'_, Q, E, B>
where
    Mp::Value: PartialEq<E>,
    B: Borrow<E>,
    R: ValueRenderer<Mp::Key> + ValueRenderer<Mp::Value> + ValueRenderer<Q> + ValueRenderer<E>,
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
            None => failure
                .relation("contains the entry")
                .expected((render.value(self.key), render.value(self.value.borrow()))),
            Some((actual, rejection)) => {
                let failure = failure.actual(render.map(actual));
                match rejection {
                    None => failure
                        .relation("does not contain key")
                        .expected(render.value(self.key)),
                    Some((value, expected)) => failure
                        .relation("does not contain the expected value at a key")
                        .child(
                            FailureBuilder::detached::<Mp::Value>(FailureKind::Equality)
                                .actual(render.value(value))
                                .expected(render.value(expected))
                                .build()
                                .located_at(Fact::key(render.value(self.key))),
                        ),
                }
            }
        }
    }
}

/// Checks a map entry with one native lookup and one expected-value borrow.
pub struct DoesNotContainEntry<'e, Q: ?Sized, E, B = E> {
    key: &'e Q,
    value: B,
    operand: PhantomData<fn() -> E>,
}
impl<'e, Q: ?Sized, E> DoesNotContainEntry<'e, Q, E> {
    /// Borrows the query and owns the expected value.
    #[must_use]
    pub const fn new(key: &'e Q, value: E) -> Self {
        Self {
            key,
            value,
            operand: PhantomData,
        }
    }

    pub(super) fn with_storage<B: Borrow<E>>(
        key: &'e Q,
        value: B,
    ) -> DoesNotContainEntry<'e, Q, E, B> {
        DoesNotContainEntry {
            key,
            value,
            operand: PhantomData,
        }
    }
}

impl<Mp: MapLookup<Q> + ?Sized, Q: ?Sized, E, B, R> Expectation<Mp, R>
    for DoesNotContainEntry<'_, Q, E, B>
where
    Mp::Value: PartialEq<E>,
    B: Borrow<E>,
{
    type Success<'a>
        = ()
    where
        Self: 'a,
        Mp: 'a;
    type Rejection<'a>
        = &'a E
    where
        Self: 'a,
        Mp: 'a;

    fn evaluate<'a>(
        &'a self,
        actual: &'a Mp,
        _: &AssertionContext<'_, R>,
    ) -> Result<Self::Success<'a>, Self::Rejection<'a>> {
        let expected = self.value.borrow();
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

impl<Mp: MapLookup<Q> + ?Sized, Q: ?Sized, E, B, R> ExpectationDiagnostics<Mp, R>
    for DoesNotContainEntry<'_, Q, E, B>
where
    Mp::Value: PartialEq<E>,
    B: Borrow<E>,
    R: ValueRenderer<Mp::Key> + ValueRenderer<Mp::Value> + ValueRenderer<Q> + ValueRenderer<E>,
{
    const KIND: FailureKind = FailureKind::Membership;

    fn explain<'a, Target>(
        &'a self,
        rejected: Option<(&'a Mp, Self::Rejection<'a>)>,
        failure: FailureBuilder<Target>,
        context: &AssertionContext<'_, R>,
    ) -> FailureBuilder<Target> {
        let render = context.render();
        let expected = rejected
            .as_ref()
            .map_or_else(|| self.value.borrow(), |(_, expected)| *expected);
        let failure = match rejected {
            None => failure.relation("does not contain the entry"),
            Some((actual, _)) => failure
                .actual(render.map(actual))
                .relation("contains the entry"),
        };
        failure.unexpected((render.value(self.key), render.value(expected)))
    }
}

/// Retained key queries and missing queries from a membership rejection.
pub struct MissingKeysRejection<'a, E> {
    expected: &'a [E],
    missing: Vec<&'a E>,
}

/// Requires each expected key query to resolve through native map lookup.
pub struct ContainsKeys<E, B = Vec<E>> {
    expected: B,
    operand: PhantomData<fn() -> E>,
}
impl<E, B: AsRef<[E]>> ContainsKeys<E, B> {
    /// Stores an array, slice, or vector of expected elements without converting it yet.
    #[must_use]
    pub const fn new(expected: B) -> Self {
        Self {
            expected,
            operand: PhantomData,
        }
    }
}

impl<Mp: Map + ?Sized, E, B, R> Expectation<Mp, R> for ContainsKeys<E, B>
where
    Mp: MapLookup<<E as MapKeyQuery<<Mp as Map>::Key>>::Query>,
    E: MapKeyQuery<Mp::Key>,
    B: AsRef<[E]>,
{
    type Success<'a>
        = ()
    where
        Self: 'a,
        Mp: 'a;
    type Rejection<'a>
        = MissingKeysRejection<'a, E>
    where
        Self: 'a,
        Mp: 'a;

    fn evaluate<'a>(
        &'a self,
        actual: &'a Mp,
        _: &AssertionContext<'_, R>,
    ) -> Result<Self::Success<'a>, Self::Rejection<'a>> {
        let expected = self.expected.as_ref();
        let missing = expected
            .iter()
            .filter(|key| {
                actual
                    .get_key_value(<E as MapKeyQuery<Mp::Key>>::as_query(*key))
                    .is_none()
            })
            .collect::<Vec<_>>();
        if missing.is_empty() {
            Ok(())
        } else {
            Err(MissingKeysRejection { expected, missing })
        }
    }
}

impl<Mp: Map + ?Sized, E, B, R> ExpectationDiagnostics<Mp, R> for ContainsKeys<E, B>
where
    Mp: MapLookup<<E as MapKeyQuery<<Mp as Map>::Key>>::Query>,
    E: MapKeyQuery<Mp::Key>,
    B: AsRef<[E]>,
    R: ValueRenderer<Mp::Key> + ValueRenderer<Mp::Value> + ValueRenderer<E>,
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
            || self.expected.as_ref(),
            |(_, rejection)| rejection.expected,
        );
        let failure = match rejected {
            None => failure.relation("contains all of"),
            Some((actual, MissingKeysRejection { missing, .. })) => failure
                .actual(render.map(actual))
                .relation("does not contain all of")
                .fact(Fact::labelled(
                    "Keys not found",
                    render.borrowed_values::<E, _>(&missing, GroupStyle::List),
                )),
        };
        failure.expected(render.borrowed_values::<E, _>(expected, GroupStyle::List))
    }
}

/// Retained missing keys, unexpected entries, and unequal values from an exact map comparison.
/// The fields remain private so diagnostics consume the original observations without repeating
/// lookup.
pub struct ExactEntriesRejection<'a, K, V, EK, EV> {
    expected: &'a [(EK, EV)],
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
    /// Stores expected entries without converting their storage yet.
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
    Mp: MapLookup<<EK as MapKeyQuery<<Mp as Map>::Key>>::Query>,
    EK: MapKeyQuery<Mp::Key>,
    Mp::Value: PartialEq<EV>,
    B: AsRef<[(EK, EV)]>,
{
    type Success<'a>
        = ()
    where
        Self: 'a,
        Mp: 'a;
    type Rejection<'a>
        = ExactEntriesRejection<'a, Mp::Key, Mp::Value, EK, EV>
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
        for (key, expected_value) in expected {
            match actual.get_key_value(key.as_query()) {
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
                expected,
                length,
                missing,
                unexpected,
                mismatches,
            })
        }
    }
}

impl<Mp: Map + ?Sized, EK, EV, B, R> ExpectationDiagnostics<Mp, R>
    for ContainsExactlyEntries<EK, EV, B>
where
    Mp: MapLookup<<EK as MapKeyQuery<<Mp as Map>::Key>>::Query>,
    EK: MapKeyQuery<Mp::Key>,
    Mp::Value: PartialEq<EV>,
    B: AsRef<[(EK, EV)]>,
    R: ValueRenderer<Mp::Key>
        + ValueRenderer<Mp::Value>
        + ValueRenderer<EK>
        + ValueRenderer<EV>
        + ValueRenderer<usize>,
{
    const KIND: FailureKind = FailureKind::Equality;

    fn explain<'a, Target>(
        &'a self,
        rejected: Option<(&'a Mp, Self::Rejection<'a>)>,
        failure: FailureBuilder<Target>,
        context: &AssertionContext<'_, R>,
    ) -> FailureBuilder<Target> {
        let render = context.render();
        let expected = rejected.as_ref().map_or_else(
            || self.expected.as_ref(),
            |(_, rejection)| rejection.expected,
        );
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
                let keys_with_unexpected_values = mismatches
                    .iter()
                    .map(|(key, _, _)| *key)
                    .collect::<Vec<_>>();
                let unexpected_values = mismatches
                    .iter()
                    .map(|(key, expected, value)| {
                        alloc::vec![
                            FailureBuilder::detached::<Mp::Value>(FailureKind::Equality)
                                .actual(render.value(*value))
                                .expected(render.value(*expected))
                                .build()
                                .located_at(Fact::key(render.value(*key)))
                        ]
                    })
                    .collect();
                let (children, omitted) =
                    keyed_children::<Mp>(unexpected_values, render.max_items());
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
                        render.borrowed_values::<EK, _>(&missing, GroupStyle::List),
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
                if !keys_with_unexpected_values.is_empty() {
                    failure = failure.fact(Fact::labelled(
                        "Keys with unexpected values",
                        render.borrowed_values::<EK, _>(
                            &keys_with_unexpected_values,
                            GroupStyle::List,
                        ),
                    ));
                }
                failure
                    .omitted(omitted, "unexpected value")
                    .children(children)
            }
        };
        failure.expected(
            render.entry_list::<EK, EV, _, _, _>(expected, RenderingOrder::PreserveIteration),
        )
    }
}
