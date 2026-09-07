//! Algorithms and diagnostics shared by every map assertion.
//!
//! The public [`MapAssertions`](super::MapAssertions) methods are thin wrappers around these
//! functions, so every map type produces identical failure messages.

use alloc::collections::BTreeSet;
use alloc::string::String;
use alloc::vec::Vec;
use core::ptr;

use super::{Map, MapKeyQuery, MapLookup};
use crate::failure::{Fact, FailureBuilder, FailureKind, adapter::ToHumanReadableText};
use crate::renderer::{GroupStyle, RenderingOrder};
use crate::{AssertThat, AssertionFailure, Mode, ValueRenderer};

/// The value stored under `key`, if any.
fn value_of<'m, Mp, Q>(actual: &'m Mp, key: &Q) -> Option<&'m Mp::Value>
where
    Mp: MapLookup<Q>,
    Q: ?Sized,
{
    actual.get_key_value(key).map(|(_, value)| value)
}

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
        Mp: Map<Key = K>,
    {
        actual
            .entries()
            .filter(|(actual_key, _)| !self.contains(actual_key))
            .collect()
    }
}

/// Asserts that `expected` is a key of the subject, returning whether it is.
///
/// The boolean is what makes [`assert_contains_entry`] a *partial* composer: it needs to know
/// whether the key check already raised a failure, so it can skip its own diagnostic rather than
/// reporting the same missing key twice.
#[track_caller]
pub(crate) fn assert_contains_key<Mp, Q, M, R>(
    this: &AssertThat<'_, Mp, M, R>,
    expected: &Q,
) -> bool
where
    Mp: MapLookup<Q>,
    Q: ?Sized,
    M: Mode,
    R: ValueRenderer<Mp::Key> + ValueRenderer<Mp::Value> + ValueRenderer<Q>,
{
    this.track_assertion();
    let actual = this.actual();

    if value_of(actual, expected).is_some() {
        return true;
    }

    this.failure(FailureKind::Membership)
        .actual(this.render().map(actual))
        .relation("does not contain key")
        .expected(this.render().value(expected))
        .raise();
    false
}

#[track_caller]
pub(crate) fn assert_does_not_contain_key<Mp, Q, M, R>(
    this: &AssertThat<'_, Mp, M, R>,
    not_expected: &Q,
) where
    Mp: MapLookup<Q>,
    Q: ?Sized,
    M: Mode,
    R: ValueRenderer<Mp::Key> + ValueRenderer<Mp::Value> + ValueRenderer<Q>,
{
    this.track_assertion();
    let actual = this.actual();

    if value_of(actual, not_expected).is_some() {
        this.failure(FailureKind::Membership)
            .actual(this.render().map(actual))
            .relation("contains key")
            .unexpected(this.render().value(not_expected))
            .raise();
    }
}

#[track_caller]
pub(crate) fn assert_contains_value<Mp, E, M, R>(this: &AssertThat<'_, Mp, M, R>, expected: &E)
where
    Mp: Map,
    Mp::Value: PartialEq<E>,
    M: Mode,
    R: ValueRenderer<Mp::Key> + ValueRenderer<Mp::Value> + ValueRenderer<E>,
{
    this.track_assertion();
    let actual = this.actual();

    if !actual
        .entries()
        .any(|(_key, value)| crate::matchers::equals(value, expected))
    {
        this.failure(FailureKind::Membership)
            .actual(this.render().map(actual))
            .relation("does not contain value")
            .expected(this.render().value(expected))
            .raise();
    }
}

#[track_caller]
pub(crate) fn assert_does_not_contain_value<Mp, E, M, R>(
    this: &AssertThat<'_, Mp, M, R>,
    not_expected: &E,
) where
    Mp: Map,
    Mp::Value: PartialEq<E>,
    M: Mode,
    R: ValueRenderer<Mp::Key> + ValueRenderer<Mp::Value> + ValueRenderer<E>,
{
    this.track_assertion();
    let actual = this.actual();

    if actual
        .entries()
        .any(|(_key, value)| crate::matchers::equals(value, not_expected))
    {
        this.failure(FailureKind::Membership)
            .actual(this.render().map(actual))
            .relation("contains value")
            .unexpected(this.render().value(not_expected))
            .raise();
    }
}

/// The one *partial* composer in the crate: it delegates the key check to [`assert_contains_key`],
/// which tracks and reports on its own, then performs the value comparison as part of that same
/// assertion. One tracked assertion, at most one failure.
#[track_caller]
pub(crate) fn assert_contains_entry<Mp, E, Q, M, R>(
    this: &AssertThat<'_, Mp, M, R>,
    key: &Q,
    value: &E,
) where
    Mp: MapLookup<Q>,
    Q: ?Sized,
    Mp::Value: PartialEq<E>,
    M: Mode,
    R: ValueRenderer<Mp::Key> + ValueRenderer<Mp::Value> + ValueRenderer<Q> + ValueRenderer<E>,
{
    // Note: This already calls `this.track_assertion()`!
    let key_present = assert_contains_key(this, key);

    if !key_present {
        // Ignored: `assert_contains_key` already reported the missing key.
        return;
    }

    let actual = this.actual();
    let Some(actual_value) = value_of(actual, key) else {
        return;
    };

    if !crate::matchers::equals(actual_value, value) {
        let unexpected_value = FailureBuilder::detached::<Mp::Value>(FailureKind::Equality)
            .actual(this.render().value(actual_value))
            .expected(this.render().value(value));

        this.failure(FailureKind::Membership)
            .actual(this.render().map(actual))
            .relation("does not contain the expected value at a key")
            .child(
                unexpected_value
                    .build()
                    .located_at(Fact::key(this.render().value(key))),
            )
            .raise();
    }
}

#[track_caller]
pub(crate) fn assert_does_not_contain_entry<Mp, E, Q, M, R>(
    this: &AssertThat<'_, Mp, M, R>,
    key: &Q,
    value: &E,
) where
    Mp: MapLookup<Q>,
    Q: ?Sized,
    Mp::Value: PartialEq<E>,
    M: Mode,
    R: ValueRenderer<Mp::Key> + ValueRenderer<Mp::Value> + ValueRenderer<Q> + ValueRenderer<E>,
{
    this.track_assertion();
    let actual = this.actual();

    if value_of(actual, key)
        .is_some_and(|actual_value| crate::matchers::equals(actual_value, value))
    {
        this.failure(FailureKind::Membership)
            .actual(this.render().map(actual))
            .relation("contains the entry")
            .unexpected((this.render().value(key), this.render().value(value)))
            .raise();
    }
}

#[track_caller]
pub(crate) fn assert_contains_keys<Mp, K, E, M, R>(this: &AssertThat<'_, Mp, M, R>, expected: &[E])
where
    Mp: Map<Key = K> + MapLookup<<E as MapKeyQuery<K>>::Query>,
    E: MapKeyQuery<K>,
    M: Mode,
    R: ValueRenderer<Mp::Key> + ValueRenderer<Mp::Value> + ValueRenderer<E>,
{
    this.track_assertion();
    let actual = this.actual();

    let keys_not_found = expected
        .iter()
        .filter(|expected_key| {
            value_of(actual, <E as MapKeyQuery<K>>::as_query(*expected_key)).is_none()
        })
        .collect::<Vec<_>>();

    if !keys_not_found.is_empty() {
        this.failure(FailureKind::Membership)
            .actual(this.render().map(actual))
            .relation("does not contain all of")
            .expected(
                this.render()
                    .borrowed_values::<E, _>(expected, GroupStyle::List),
            )
            .fact(Fact::labelled(
                "Keys not found",
                this.render()
                    .borrowed_values::<E, _>(keys_not_found.as_slice(), GroupStyle::List),
            ))
            .raise();
    }
}

#[track_caller]
pub(crate) fn assert_contains_exactly_entries<Mp, K, EK, EV, M, R>(
    this: &AssertThat<'_, Mp, M, R>,
    expected: &[(EK, EV)],
) where
    Mp: Map<Key = K> + MapLookup<<EK as MapKeyQuery<K>>::Query>,
    EK: MapKeyQuery<K>,
    Mp::Value: PartialEq<EV>,
    M: Mode,
    R: ValueRenderer<Mp::Key>
        + ValueRenderer<Mp::Value>
        + ValueRenderer<EK>
        + ValueRenderer<EV>
        + ValueRenderer<usize>,
{
    this.track_assertion();
    let actual = this.actual();
    let actual_length = actual.length();
    let same_length = actual_length == expected.len();

    let mut keys_not_found = Vec::new();
    let mut keys_with_unexpected_values = Vec::new();
    let mut unexpected_values = Vec::new();
    let mut found = FoundEntries::new();

    for (expected_key, expected_value) in expected {
        match actual.get_key_value(<EK as MapKeyQuery<K>>::as_query(expected_key)) {
            None => keys_not_found.push(expected_key),
            Some((stored_key, actual_value)) => {
                found.record(stored_key);

                if !crate::matchers::equals(actual_value, expected_value) {
                    keys_with_unexpected_values.push(expected_key);
                    let unexpected_value =
                        FailureBuilder::detached::<Mp::Value>(FailureKind::Equality)
                            .actual(this.render().value(actual_value))
                            .expected(this.render().value(expected_value));

                    unexpected_values.push(alloc::vec![
                        unexpected_value
                            .build()
                            .located_at(Fact::key(this.render().value(expected_key)))
                    ]);
                }
            }
        }
    }

    let unexpected_entries = found.unexpected_entries(actual);

    if same_length
        && keys_not_found.is_empty()
        && unexpected_entries.is_empty()
        && keys_with_unexpected_values.is_empty()
    {
        return;
    }

    let (children, omitted) = keyed_children::<Mp>(unexpected_values, this.render().max_items());
    let mut failure = this
        .failure(FailureKind::Equality)
        .actual(this.render().map(actual))
        .relation("does not contain exactly")
        .expected(this.render().entry_list::<EK, EV, _, _, _>(expected, false));
    if !same_length {
        failure = failure
            .fact(Fact::labelled(
                "Actual length",
                this.render().value(&actual_length),
            ))
            .fact(Fact::labelled(
                "Expected length",
                this.render().value(&expected.len()),
            ));
    }
    if !keys_not_found.is_empty() {
        failure = failure.fact(Fact::labelled(
            "Keys not found",
            this.render()
                .borrowed_values::<EK, _>(keys_not_found.as_slice(), GroupStyle::List),
        ));
    }
    if !unexpected_entries.is_empty() {
        failure = failure.fact(Fact::labelled(
            "Unexpected entries",
            this.render().entry_list::<Mp::Key, Mp::Value, _, _, _>(
                &unexpected_entries,
                sorts_for_rendering::<Mp>(),
            ),
        ));
    }
    if !keys_with_unexpected_values.is_empty() {
        failure = failure.fact(Fact::labelled(
            "Keys with unexpected values",
            this.render()
                .borrowed_values::<EK, _>(keys_with_unexpected_values.as_slice(), GroupStyle::List),
        ));
    }
    failure
        .omitted(omitted, "unexpected value")
        .children(children)
        .raise();
}
