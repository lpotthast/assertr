//! Algorithms and diagnostics shared by every map assertion.
//!
//! The public [`MapAssertions`](super::MapAssertions) methods are thin wrappers around these
//! functions, so every map type produces identical failure messages.

use alloc::collections::BTreeSet;
use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;
use core::fmt::Write;
use core::ptr;
use indoc::writedoc;

use super::{Map, MapKeyQuery, MapLookup};
use crate::renderer::{GroupStyle, RenderingOrder, omission};
use crate::util::failure::join_failures;
use crate::{AssertThat, AssertrPartialEq, Mode, ValueRenderer, mode::Capture};

/// The value stored under `key`, if any.
fn value_of<'m, Mp, Q>(actual: &'m Mp, key: &Q) -> Option<&'m Mp::Value>
where
    Mp: MapLookup<Q>,
    Q: ?Sized,
{
    actual.get_key_value(key).map(|(_, value)| value)
}

/// The stored entries the expected keys resolved to, identified by the address of their stored
/// key.
///
/// The exact-entry assertions have to report the actual entries that no expectation named. The
/// only lookup direction [`MapLookup`] offers is expected key to stored entry, so instead of
/// indexing the expected keys (which would require `Hash` or `Ord` on the key type) the
/// entries hit by the lookups are remembered by identity, and every entry [`Map::entries`] yields
/// that was not hit is unexpected. This is what the "same stored key" contract of
/// [`MapLookup::get_key_value`] exists for. Duplicate expected keys resolve to the same entry and
/// are therefore harmless. A zero-sized key type gives every key the same address, but a map with
/// such a key holds at most one entry, so identity stays unambiguous.
struct FoundEntries<K>(BTreeSet<*const K>);

impl<K> FoundEntries<K> {
    fn new() -> Self {
        Self(BTreeSet::new())
    }

    fn record(&mut self, stored_key: &K) {
        self.0.insert(ptr::from_ref(stored_key));
    }

    fn unexpected_entries<'m, Mp>(&self, actual: &'m Mp) -> Vec<(&'m K, &'m Mp::Value)>
    where
        Mp: Map<Key = K>,
    {
        actual
            .entries()
            .filter(|(actual_key, _)| !self.0.contains(&ptr::from_ref(*actual_key)))
            .collect()
    }
}

/// Asserts that `expected` is a key of the subject, returning whether it is.
///
/// The boolean is what makes [`assert_contains_entry`] a *partial* composer: it needs to know
/// whether the key check already raised a failure, so it can skip its own diagnostic rather than
/// reporting the same missing key twice.
///
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

    let rendered_actual = this.render().map(actual);
    let expected = this.render().value(expected);
    this.fail(|w: &mut String| {
        writedoc! {w, r"
            Actual: {rendered_actual:#?}

            does not contain expected key: {expected:#?}
        "}
    });
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
        let rendered_actual = this.render().map(actual);
        let not_expected = this.render().value(not_expected);
        this.fail(|w: &mut String| {
            writedoc! {w, r"
                Actual: {rendered_actual:#?}

                contains unexpected key: {not_expected:#?}
            "}
        });
    }
}

#[track_caller]
pub(crate) fn assert_contains_value<Mp, E, M, R>(this: &AssertThat<'_, Mp, M, R>, expected: &E)
where
    Mp: Map,
    Mp::Value: AssertrPartialEq<E, R>,
    M: Mode,
    R: ValueRenderer<Mp::Key> + ValueRenderer<Mp::Value> + ValueRenderer<E>,
{
    this.track_assertion();
    let actual = this.actual();

    if !actual.entries().any(|(_key, value)| {
        let mut ctx = this.eq_context();
        <_ as AssertrPartialEq<_, R>>::eq(value, expected, Some(&mut ctx))
    }) {
        let rendered_actual = this.render().map(actual);
        let expected = this.render().value(expected);
        this.fail(|w: &mut String| {
            writedoc! {w, r"
                Actual: {rendered_actual:#?}

                does not contain expected value: {expected:#?}
            "}
        });
    }
}

#[track_caller]
pub(crate) fn assert_does_not_contain_value<Mp, E, M, R>(
    this: &AssertThat<'_, Mp, M, R>,
    not_expected: &E,
) where
    Mp: Map,
    Mp::Value: AssertrPartialEq<E, R>,
    M: Mode,
    R: ValueRenderer<Mp::Key> + ValueRenderer<Mp::Value> + ValueRenderer<E>,
{
    this.track_assertion();
    let actual = this.actual();

    if actual.entries().any(|(_key, value)| {
        let mut ctx = this.eq_context();
        <_ as AssertrPartialEq<_, R>>::eq(value, not_expected, Some(&mut ctx))
    }) {
        let rendered_actual = this.render().map(actual);
        let not_expected = this.render().value(not_expected);
        this.fail(|w: &mut String| {
            writedoc! {w, r"
                Actual: {rendered_actual:#?}

                contains unexpected value: {not_expected:#?}
            "}
        });
    }
}

/// The one *partial* composer in the crate: it delegates the key check to
/// [`assert_contains_key`], which tracks and reports on its own, then performs the value comparison
/// as part of that same assertion. One tracked assertion, at most one failure.
///
#[track_caller]
pub(crate) fn assert_contains_entry<Mp, E, Q, M, R>(
    this: &AssertThat<'_, Mp, M, R>,
    key: &Q,
    value: &E,
) where
    Mp: MapLookup<Q>,
    Q: ?Sized,
    Mp::Value: AssertrPartialEq<E, R>,
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

    let mut ctx = this.eq_context();
    if !AssertrPartialEq::eq(actual_value, value, Some(&mut ctx)) {
        let mut details = Vec::new();
        if !ctx.differences.differences.is_empty() {
            details.push(format!("Differences: {:#?}", ctx.differences));
        }
        let rendered_actual = this.render().map(actual);
        let expected_key = this.render().value(key);
        let expected_value = this.render().value(value);
        let actual_value = this.render().value(actual_value);
        this.fail_with_details(details, |w: &mut String| {
            writedoc! {w, r"
                Actual: {rendered_actual:#?}

                does not contain expected value at key: {expected_key:#?}

                Expected value: {expected_value:#?}
                  Actual value: {actual_value:#?}
                ",
            }
        });
    }
}

#[track_caller]
pub(crate) fn assert_contains_entry_satisfying<Mp, A, Q, M, R>(
    this: &AssertThat<'_, Mp, M, R>,
    key: &Q,
    assertions: &A,
) where
    Mp: MapLookup<Q>,
    Q: ?Sized,
    A: for<'a> Fn(AssertThat<'a, Mp::Value, Capture, R>),
    M: Mode,
    R: ValueRenderer<Mp::Key> + ValueRenderer<Mp::Value> + ValueRenderer<Q> + Clone,
{
    this.track_assertion();
    let actual = this.actual();

    let Some(value) = value_of(actual, key) else {
        let rendered_actual = this.render().map(actual);
        let expected_key = this.render().value(key);
        this.fail(|w: &mut String| {
            writedoc! {w, r"
                Actual: {rendered_actual:#?}

                does not contain expected key: {expected_key:#?}
            "}
        });
        return;
    };

    let failures = this.collect_element_failures(value, assertions);
    if !failures.is_empty() {
        let expected_key = this.render().value(key);
        let details = alloc::vec![format!(
            "Value at key {expected_key:#?} does not satisfy the assertions:\n{}",
            join_failures(&failures, this.render().max_items())
        )];
        let rendered_actual = this.render().map(actual);
        this.fail_with_details(details, |w: &mut String| {
            writedoc! {w, r"
                Actual: {rendered_actual:#?}

                does not contain an entry satisfying the assertions at key: {expected_key:#?}
            "}
        });
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
    Mp::Value: AssertrPartialEq<E, R>,
    M: Mode,
    R: ValueRenderer<Mp::Key> + ValueRenderer<Mp::Value> + ValueRenderer<Q> + ValueRenderer<E>,
{
    this.track_assertion();
    let actual = this.actual();

    if value_of(actual, key).is_some_and(|actual_value| {
        let mut ctx = this.eq_context();
        <_ as AssertrPartialEq<_, R>>::eq(actual_value, value, Some(&mut ctx))
    }) {
        let rendered_actual = this.render().map(actual);
        let unexpected_key_rendered = this.render().value(key);
        let unexpected_value_rendered = this.render().value(value);
        this.fail(|w: &mut String| {
            writedoc! {w, r"
                Actual: {rendered_actual:#?}

                contains unexpected entry at key: {unexpected_key_rendered:#?}

                Unexpected value: {unexpected_value_rendered:#?}
            "}
        });
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
        let expected_refs: Vec<&E> = expected.iter().collect();
        let rendered_actual = this.render().map(actual);
        let expected_rendered = this
            .render()
            .borrowed_values::<E, _>(expected_refs.as_slice(), GroupStyle::List);
        let keys_not_found_rendered = this
            .render()
            .borrowed_values::<E, _>(keys_not_found.as_slice(), GroupStyle::List);
        this.fail(|w: &mut String| {
            writedoc! {w, r"
                Actual: {rendered_actual:#?}

                does not contain all expected keys

                Expected keys: {expected_rendered:#?}

                Keys not found: {keys_not_found_rendered:#?}
            "}
        });
    }
}

#[track_caller]
pub(crate) fn assert_contains_exactly_entries<Mp, K, EK, EV, M, R>(
    this: &AssertThat<'_, Mp, M, R>,
    expected: &[(EK, EV)],
) where
    Mp: Map<Key = K> + MapLookup<<EK as MapKeyQuery<K>>::Query>,
    EK: MapKeyQuery<K>,
    Mp::Value: AssertrPartialEq<EV, R>,
    M: Mode,
    R: ValueRenderer<Mp::Key> + ValueRenderer<Mp::Value> + ValueRenderer<EK> + ValueRenderer<EV>,
{
    this.track_assertion();
    let actual = this.actual();
    let actual_length = actual.length();
    let same_length = actual_length == expected.len();

    let mut details = Vec::new();
    let maximum = this.render().max_items();
    let mut number_of_difference_details = 0_usize;
    let mut keys_not_found = Vec::new();
    let mut keys_with_unexpected_values = Vec::new();
    let mut found = FoundEntries::new();

    for (expected_key, expected_value) in expected {
        match actual.get_key_value(<EK as MapKeyQuery<K>>::as_query(expected_key)) {
            None => keys_not_found.push(expected_key),
            Some((stored_key, actual_value)) => {
                found.record(stored_key);
                let mut ctx = this.eq_context();
                if !AssertrPartialEq::eq(actual_value, expected_value, Some(&mut ctx)) {
                    keys_with_unexpected_values.push(expected_key);
                    if !ctx.differences.differences.is_empty() {
                        number_of_difference_details += 1;
                        if number_of_difference_details <= maximum {
                            let expected_key_rendered = this.render().value(expected_key);
                            details.push(format!(
                                "Differences at key {expected_key_rendered:#?}: {:#?}",
                                ctx.differences
                            ));
                        }
                    }
                }
            }
        }
    }

    let unexpected_entries = found.unexpected_entries(actual);

    if !same_length
        || !keys_not_found.is_empty()
        || !unexpected_entries.is_empty()
        || !keys_with_unexpected_values.is_empty()
    {
        let omitted = number_of_difference_details.saturating_sub(maximum);
        if omitted != 0 {
            details.push(omission(omitted, "entry difference"));
        }
        if !same_length {
            details.push(format!(
                "Number of entries ({actual_length}) does not match number of expected entries ({})!",
                expected.len()
            ));
        }
        if !keys_not_found.is_empty() {
            let keys_not_found_rendered = this
                .render()
                .borrowed_values::<EK, _>(keys_not_found.as_slice(), GroupStyle::List);
            details.push(format!("Keys not found: {keys_not_found_rendered:#?}"));
        }
        if !unexpected_entries.is_empty() {
            let unexpected_entries_rendered =
                this.render().entry_list::<Mp::Key, Mp::Value, _, _, _>(
                    &unexpected_entries,
                    Mp::RENDERING_ORDER == RenderingOrder::SortByRenderedText,
                );
            details.push(format!(
                "Unexpected entries: {unexpected_entries_rendered:#?}"
            ));
        }
        if !keys_with_unexpected_values.is_empty() {
            let keys_with_unexpected_values_rendered = this
                .render()
                .borrowed_values::<EK, _>(keys_with_unexpected_values.as_slice(), GroupStyle::List);
            details.push(format!(
                "Keys with unexpected values: {keys_with_unexpected_values_rendered:#?}"
            ));
        }

        let expected_rendered = this.render().entry_list::<EK, EV, _, _, _>(expected, false);
        let rendered_actual = this.render().map(actual);

        this.fail_with_details(details, |w: &mut String| {
            writedoc! {w, r"
                Actual: {rendered_actual:#?}

                does not exactly contain expected entries

                Expected entries: {expected_rendered:#?}
            "}
        });
    }
}

#[track_caller]
pub(crate) fn assert_contains_exactly_entries_matching<Mp, K, EK, P, M, R>(
    this: &AssertThat<'_, Mp, M, R>,
    predicates: &[(EK, P)],
) where
    Mp: Map<Key = K> + MapLookup<<EK as MapKeyQuery<K>>::Query>,
    EK: MapKeyQuery<K>,
    P: Fn(&Mp::Value) -> bool,
    M: Mode,
    R: ValueRenderer<Mp::Key> + ValueRenderer<Mp::Value> + ValueRenderer<EK>,
{
    this.track_assertion();
    let actual = this.actual();
    let actual_length = actual.length();
    let same_length = actual_length == predicates.len();

    let mut keys_not_found = Vec::new();
    let mut keys_not_matching = Vec::new();
    let mut found = FoundEntries::new();
    for (expected_key, predicate) in predicates {
        match actual.get_key_value(<EK as MapKeyQuery<K>>::as_query(expected_key)) {
            None => keys_not_found.push(expected_key),
            Some((stored_key, value)) => {
                found.record(stored_key);
                if !predicate(value) {
                    keys_not_matching.push(expected_key);
                }
            }
        }
    }

    let unexpected_entries = found.unexpected_entries(actual);

    if same_length
        && keys_not_found.is_empty()
        && unexpected_entries.is_empty()
        && keys_not_matching.is_empty()
    {
        return;
    }

    let mut details = Vec::new();
    if !same_length {
        details.push(format!(
            "Number of entries ({actual_length}) does not match number of predicates ({})!",
            predicates.len()
        ));
    }
    if !keys_not_found.is_empty() {
        details.push(format!(
            "Keys not found: {:#?}",
            this.render()
                .borrowed_values::<EK, _>(keys_not_found.as_slice(), GroupStyle::List)
        ));
    }
    if !unexpected_entries.is_empty() {
        let rendered = this.render().entry_list::<Mp::Key, Mp::Value, _, _, _>(
            &unexpected_entries,
            Mp::RENDERING_ORDER == RenderingOrder::SortByRenderedText,
        );
        details.push(format!("Unexpected entries: {rendered:#?}"));
    }
    if !keys_not_matching.is_empty() {
        details.push(format!(
            "Keys with values not matching their predicates: {:#?}",
            this.render()
                .borrowed_values::<EK, _>(keys_not_matching.as_slice(), GroupStyle::List)
        ));
    }

    let expected_keys = predicates
        .iter()
        .map(|(expected_key, _)| expected_key)
        .collect::<Vec<_>>();
    let expected_keys = this
        .render()
        .borrowed_values::<EK, _>(expected_keys.as_slice(), GroupStyle::List);
    let rendered_actual = this.render().map(actual);
    this.fail_with_details(details, |w: &mut String| {
        writedoc! {w, r"
            Actual: {rendered_actual:#?}

            does not exactly contain entries matching the predicates

            Expected keys: {expected_keys:#?}
        "}
    });
}

#[track_caller]
pub(crate) fn assert_contains_exactly_entries_satisfying<Mp, K, EK, A, M, R>(
    this: &AssertThat<'_, Mp, M, R>,
    assertions: &[(EK, A)],
) where
    Mp: Map<Key = K> + MapLookup<<EK as MapKeyQuery<K>>::Query>,
    EK: MapKeyQuery<K>,
    A: for<'a> Fn(AssertThat<'a, Mp::Value, Capture, R>),
    M: Mode,
    R: ValueRenderer<Mp::Key> + ValueRenderer<Mp::Value> + ValueRenderer<EK> + Clone,
{
    this.track_assertion();
    let actual = this.actual();
    let actual_length = actual.length();
    let same_length = actual_length == assertions.len();

    let maximum = this.render().max_items();
    let mut keys_not_found = Vec::new();
    let mut unsatisfied = Vec::new();
    let mut number_of_unsatisfied_values = 0_usize;
    let mut found = FoundEntries::new();
    for (expected_key, value_assertions) in assertions {
        match actual.get_key_value(<EK as MapKeyQuery<K>>::as_query(expected_key)) {
            None => keys_not_found.push(expected_key),
            Some((stored_key, value)) => {
                found.record(stored_key);
                let failures = this.collect_element_failures(value, value_assertions);
                if !failures.is_empty() {
                    number_of_unsatisfied_values += 1;
                    if unsatisfied.len() < maximum {
                        unsatisfied.push((expected_key, failures));
                    }
                }
            }
        }
    }

    let unexpected_entries = found.unexpected_entries(actual);

    if same_length
        && keys_not_found.is_empty()
        && unexpected_entries.is_empty()
        && number_of_unsatisfied_values == 0
    {
        return;
    }

    let mut details = Vec::new();
    if !same_length {
        details.push(format!(
            "Number of entries ({actual_length}) does not match number of assertions ({})!",
            assertions.len()
        ));
    }
    if !keys_not_found.is_empty() {
        details.push(format!(
            "Keys not found: {:#?}",
            this.render()
                .borrowed_values::<EK, _>(keys_not_found.as_slice(), GroupStyle::List)
        ));
    }
    if !unexpected_entries.is_empty() {
        let rendered = this.render().entry_list::<Mp::Key, Mp::Value, _, _, _>(
            &unexpected_entries,
            Mp::RENDERING_ORDER == RenderingOrder::SortByRenderedText,
        );
        details.push(format!("Unexpected entries: {rendered:#?}"));
    }
    for (expected_key, failures) in unsatisfied {
        let expected_key = this.render().value(expected_key);
        details.push(format!(
            "Value at key {expected_key:#?} does not satisfy its assertions:\n{}",
            join_failures(&failures, this.render().max_items())
        ));
    }
    let omitted = number_of_unsatisfied_values.saturating_sub(maximum);
    if omitted != 0 {
        details.push(omission(omitted, "unsatisfied value"));
    }

    let expected_keys = assertions
        .iter()
        .map(|(expected_key, _)| expected_key)
        .collect::<Vec<_>>();
    let expected_keys = this
        .render()
        .borrowed_values::<EK, _>(expected_keys.as_slice(), GroupStyle::List);
    let rendered_actual = this.render().map(actual);
    this.fail_with_details(details, |w: &mut String| {
        writedoc! {w, r"
            Actual: {rendered_actual:#?}

            does not exactly contain entries satisfying the assertions

            Expected keys: {expected_keys:#?}
        "}
    });
}
