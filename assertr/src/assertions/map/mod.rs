//! Map assertions for `BTreeMap`, `HashMap`, and custom map types.
//!
//! [`MapAssertions`] is blanket-implemented for every [`Map`]. Maps have their own family because
//! their entries are key/value pairs rather than plain collection elements.
//!
//! Implement [`Map`] and [`MapLookup`] for a custom map. Bulk operands select their query view
//! through [`BorrowFor`](crate::borrow_for::BorrowFor) with the stored key as context.
//! See [`MapAssertions`] for operand registration and native lookup requirements.

mod assertions;
mod entries_are;
mod entry;
mod entry_matcher_list;
mod imp;
mod matching;

use alloc::collections::BTreeMap;
use core::borrow::Borrow;

pub use assertions::MapAssertions;
pub use entries_are::{EntriesAre, entries_are};
pub use entry::{Entry, EntryRejection, entry};
pub use entry_matcher_list::{EntryMatcherList, entry_matchers};
pub(crate) use imp::FoundEntries;
pub use imp::{
    ContainsEntry, ContainsExactlyEntries, ContainsKey, ContainsKeys, ContainsValue,
    DoesNotContainEntry, DoesNotContainKey, DoesNotContainValue,
};
pub use matching::{ContainsEntryMatching, ContainsValueMatching};

use crate::{assertions::HasLength, renderer::RenderingOrder};

/// A keyed collection supporting iteration over its entries.
///
/// Implementing this trait makes iteration-based [`MapAssertions`] available. Implement
/// [`MapLookup`] for key queries. The prelude does not re-export this implementor-facing trait.
///
/// Assertr renders map syntax. A custom [`ValueRenderer`](crate::renderer::ValueRenderer) needs to
/// render only [`Key`](Map::Key) and [`Value`](Map::Value).
pub trait Map: HasLength {
    /// The map's key type.
    type Key;

    /// The map's value type.
    type Value;

    /// Whether diagnostics preserve iteration order or sort entries by rendered text.
    ///
    /// This affects presentation only. It does not change matching or lookup behavior.
    const RENDERING_ORDER: RenderingOrder;

    /// The entries of this map.
    ///
    /// Must be repeatable. Every call must yield the same entries because some assertions make
    /// multiple passes. References must point at the stored keys and values returned by
    /// [`MapLookup::get_key_value`].
    fn entries(&self) -> impl Iterator<Item = (&Self::Key, &Self::Value)>;
}

/// Native lookup of a [`Map`] by a borrowed key view `Q`, carrying the map's own lookup bounds.
///
/// Every assertion that queries a key (`contains_key`, `contains_entry`, `contains_keys`, the
/// `contains_exactly_entries` family, and their negatives) requires the subject to implement
/// `MapLookup<Q>` for the query type `Q`. `Q` may be an unsized borrowed view of [`Map::Key`], such
/// as `str` for a `String` key, and is compared according to the contract of [`Borrow`].
///
/// The bounds live on the implementation, not on the trait, so each map demands exactly what its
/// native lookup needs: `Q: Hash + Eq` for a `HashMap`, `Q: Ord` for a `BTreeMap`. A key type only
/// needs to satisfy its own map's requirements. A custom map can implement this trait once,
/// generically over `Q`, by delegating to its native lookup:
///
/// ```
/// use core::borrow::Borrow;
/// use std::collections::BTreeMap;
///
/// use assertr::assertions::HasLength;
/// use assertr::assertions::{Map, MapLookup};
/// use assertr::renderer::RenderingOrder;
///
/// struct Config(BTreeMap<String, i32>);
///
/// # impl HasLength for Config {
/// #     fn length(&self) -> usize { self.0.len() }
/// # }
/// # impl Map for Config {
/// #     type Key = String;
/// #     type Value = i32;
/// #     const RENDERING_ORDER: RenderingOrder = RenderingOrder::PreserveIteration;
/// #     fn entries(&self) -> impl Iterator<Item = (&String, &i32)> { self.0.iter() }
/// # }
/// impl<Q> MapLookup<Q> for Config
/// where
///     Q: Ord + ?Sized,
///     String: Borrow<Q>,
/// {
///     fn get_key_value(&self, key: &Q) -> Option<(&String, &i32)> {
///         self.0.get_key_value(key)
///     }
/// }
/// ```
///
/// The returned references must point at the entry's *stored* key and value, the same ones
/// [`Map::entries`] yields. The exact-entry assertions rely on that identity to tell expected
/// entries from unexpected ones without requiring `Hash` or `Ord` on the key type.
///
/// Like [`Map`], this trait is not re-exported from the prelude.
pub trait MapLookup<Q: ?Sized>: Map {
    /// The stored key and value under `key`, if any.
    fn get_key_value(&self, key: &Q) -> Option<(&Self::Key, &Self::Value)>;
}

impl<K, V> Map for BTreeMap<K, V> {
    type Key = K;
    type Value = V;
    const RENDERING_ORDER: RenderingOrder = RenderingOrder::PreserveIteration;

    fn entries(&self) -> impl Iterator<Item = (&K, &V)> {
        self.iter()
    }
}

impl<K, Q, V> MapLookup<Q> for BTreeMap<K, V>
where
    K: Ord + Borrow<Q>,
    Q: Ord + ?Sized,
{
    fn get_key_value(&self, key: &Q) -> Option<(&K, &V)> {
        BTreeMap::get_key_value(self, key)
    }
}

#[cfg(feature = "std")]
impl<K, V, S> Map for std::collections::HashMap<K, V, S> {
    type Key = K;
    type Value = V;
    const RENDERING_ORDER: RenderingOrder = RenderingOrder::SortByRenderedText;

    fn entries(&self) -> impl Iterator<Item = (&K, &V)> {
        self.iter()
    }
}

#[cfg(feature = "std")]
impl<K, Q, V, S> MapLookup<Q> for std::collections::HashMap<K, V, S>
where
    K: core::hash::Hash + Eq + Borrow<Q>,
    Q: core::hash::Hash + Eq + ?Sized,
    S: core::hash::BuildHasher,
{
    fn get_key_value(&self, key: &Q) -> Option<(&K, &V)> {
        std::collections::HashMap::get_key_value(self, key)
    }
}

/// Makes shared-reference subjects maps in their own right, mirroring the `Collection` impl for
/// `&C`.
impl<M> Map for &M
where
    M: Map + ?Sized,
{
    type Key = M::Key;
    type Value = M::Value;
    const RENDERING_ORDER: RenderingOrder = M::RENDERING_ORDER;

    fn entries(&self) -> impl Iterator<Item = (&M::Key, &M::Value)> {
        M::entries(self)
    }
}

impl<M, Q> MapLookup<Q> for &M
where
    M: MapLookup<Q> + ?Sized,
    Q: ?Sized,
{
    fn get_key_value(&self, key: &Q) -> Option<(&M::Key, &M::Value)> {
        M::get_key_value(self, key)
    }
}

/// Makes mutable-reference subjects maps, mirroring the shared-reference implementation.
impl<M> Map for &mut M
where
    M: Map + ?Sized,
{
    type Key = M::Key;
    type Value = M::Value;
    const RENDERING_ORDER: RenderingOrder = M::RENDERING_ORDER;

    fn entries(&self) -> impl Iterator<Item = (&M::Key, &M::Value)> {
        M::entries(self)
    }
}

impl<M, Q> MapLookup<Q> for &mut M
where
    M: MapLookup<Q> + ?Sized,
    Q: ?Sized,
{
    fn get_key_value(&self, key: &Q) -> Option<(&M::Key, &M::Value)> {
        M::get_key_value(self, key)
    }
}

/// Instrumented maps, keys, and operands that record every observation in one shared log.
#[cfg(test)]
pub(super) mod fixture {
    use alloc::{string::String, vec::Vec};
    use core::{
        borrow::Borrow,
        cell::{Cell, RefCell},
        cmp::Ordering,
        fmt,
        hash::{Hash, Hasher},
    };

    use super::{Map, MapLookup};
    use crate::{
        assertions::HasLength,
        borrow_for::BorrowFor,
        renderer::{RenderingOrder, ValueRenderer},
        test_support::{StrOperand, StringRenderer},
    };

    /// Observations in the order they happened.
    pub(crate) type Events = RefCell<Vec<&'static str>>;

    /// Counts the recorded occurrences of `event`.
    pub(crate) fn count(events: &Events, event: &str) -> usize {
        events
            .borrow()
            .iter()
            .filter(|recorded| **recorded == event)
            .count()
    }

    /// Takes the recorded native lookups and value comparisons, clearing the log.
    pub(crate) fn take_observations(events: &Events) -> Vec<&'static str> {
        events
            .take()
            .into_iter()
            .filter(|event| matches!(*event, "lookup" | "compare"))
            .collect()
    }

    /// A stored value whose comparisons with a `str` view record "compare".
    #[derive(Debug)]
    pub(crate) struct Value<'a> {
        text: &'static str,
        events: &'a Events,
    }

    impl PartialEq<str> for Value<'_> {
        fn eq(&self, expected: &str) -> bool {
            self.events.borrow_mut().push("compare");
            self.text == expected
        }
    }

    impl ValueRenderer<Value<'_>> for StringRenderer {
        fn fmt(&self, value: &Value<'_>, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "{:?}", value.text)
        }
    }

    impl<F: Fn()> BorrowFor<Value<'_>> for StrOperand<F> {
        type View = str;
    }

    /// An expected value whose borrows record "value".
    pub(crate) fn value(events: &Events, text: &'static str) -> StrOperand<impl Fn()> {
        StrOperand {
            value: text,
            observe: move || events.borrow_mut().push("value"),
        }
    }

    /// A map with `String` keys whose native lookups record "lookup".
    pub(crate) struct ObservedMap<'a> {
        entries: Vec<(String, Value<'a>)>,
        events: &'a Events,
    }

    impl<'a> ObservedMap<'a> {
        pub(crate) fn new(events: &'a Events, entries: &[(&str, &'static str)]) -> Self {
            let entries = entries
                .iter()
                .map(|&(key, text)| (String::from(key), Value { text, events }))
                .collect();
            Self { entries, events }
        }
    }

    impl HasLength for ObservedMap<'_> {
        fn length(&self) -> usize {
            self.entries.len()
        }
    }

    impl<'a> Map for ObservedMap<'a> {
        type Key = String;
        type Value = Value<'a>;
        const RENDERING_ORDER: RenderingOrder = RenderingOrder::PreserveIteration;

        fn entries(&self) -> impl Iterator<Item = (&String, &Value<'a>)> {
            self.entries.iter().map(|(key, value)| (key, value))
        }
    }

    impl MapLookup<str> for ObservedMap<'_> {
        fn get_key_value(&self, query: &str) -> Option<(&String, &Self::Value)> {
            self.events.borrow_mut().push("lookup");
            self.entries().find(|(key, _)| *key == query)
        }
    }

    /// A key operand whose borrows record "key". After its first borrow, a non-repeatable query
    /// resolves to "later" instead.
    pub(crate) struct Query<'a> {
        first: &'a str,
        borrows: Cell<usize>,
        repeatable: bool,
        events: &'a Events,
    }

    impl<'a> Query<'a> {
        pub(crate) fn new(first: &'a str, events: &'a Events) -> Self {
            Self {
                first,
                borrows: Cell::new(0),
                repeatable: true,
                events,
            }
        }

        pub(crate) fn once(first: &'a str, events: &'a Events) -> Self {
            Self {
                repeatable: false,
                ..Self::new(first, events)
            }
        }
    }

    impl Borrow<str> for Query<'_> {
        fn borrow(&self) -> &str {
            self.events.borrow_mut().push("key");
            let previous = self.borrows.replace(self.borrows.get() + 1);
            if self.repeatable || previous == 0 {
                self.first
            } else {
                "later"
            }
        }
    }

    impl BorrowFor<String> for Query<'_> {
        type View = str;
    }

    /// Bulk expected data whose slice accesses record "container".
    pub(crate) struct Inputs<'a, T> {
        pub(crate) values: Vec<T>,
        pub(crate) events: &'a Events,
    }

    impl<T> AsRef<[T]> for Inputs<'_, T> {
        fn as_ref(&self) -> &[T] {
            self.events.borrow_mut().push("container");
            &self.values
        }
    }

    /// A key for standard maps whose equality, ordering, and hashing record "eq", "cmp", and
    /// "hash".
    #[derive(Clone, Debug)]
    pub(crate) struct CountingKey<'a> {
        pub(crate) value: i32,
        pub(crate) events: &'a Events,
    }

    impl PartialEq for CountingKey<'_> {
        fn eq(&self, other: &Self) -> bool {
            self.events.borrow_mut().push("eq");
            self.value == other.value
        }
    }

    impl Eq for CountingKey<'_> {}

    impl PartialOrd for CountingKey<'_> {
        fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
            Some(self.cmp(other))
        }
    }

    impl Ord for CountingKey<'_> {
        fn cmp(&self, other: &Self) -> Ordering {
            self.events.borrow_mut().push("cmp");
            self.value.cmp(&other.value)
        }
    }

    impl Hash for CountingKey<'_> {
        fn hash<H: Hasher>(&self, state: &mut H) {
            self.events.borrow_mut().push("hash");
            self.value.hash(state);
        }
    }
}

#[cfg(test)]
mod tests {
    use alloc::{
        borrow::Cow, boxed::Box, collections::BTreeMap, rc::Rc, string::String, sync::Arc, vec::Vec,
    };

    use super::{
        Map, MapLookup, RenderingOrder,
        fixture::{CountingKey, Events, count},
    };
    use crate::{
        assertions::HasLength,
        matchers::{entry, entry_matchers, predicate},
        prelude::*,
    };

    fn assert_map_contract<M>(actual: &M, arbitrary_iteration: bool)
    where
        M: Map<Key = String, Value = i32> + MapLookup<str> + MapLookup<String> + ?Sized,
    {
        assert_that!(M::RENDERING_ORDER == RenderingOrder::SortByRenderedText)
            .is_equal_to(arbitrary_iteration);
        assert_that!(actual.length()).is_equal_to(2);
        assert_that!(actual.get_key_value("alpha")).is_equal_to(Some((&String::from("alpha"), &1)));
        assert_that!(actual.get_key_value(&String::from("beta")))
            .is_equal_to(Some((&String::from("beta"), &2)));
        assert_that!(actual.get_key_value("missing")).is_none();

        // The lookup hands out the stored entry, not a copy: the exact-entry assertions rely on it.
        let (stored_key, stored_value) = actual.entries().find(|(key, _)| *key == "alpha").unwrap();
        let (found_key, found_value) = actual.get_key_value("alpha").unwrap();
        assert_that!(core::ptr::eq(stored_key, found_key)).is_true();
        assert_that!(core::ptr::eq(stored_value, found_value)).is_true();

        let mut entries = actual
            .entries()
            .map(|(key, value)| (key.as_str(), *value))
            .collect::<Vec<_>>();
        entries.sort_unstable();
        assert_that!(entries).contains_exactly([("alpha", 1), ("beta", 2)]);
    }

    fn counting_keys(events: &Events, count: i32) -> impl Iterator<Item = CountingKey<'_>> {
        (0..count).map(move |value| CountingKey { value, events })
    }

    #[test]
    fn btree_map_adapter_follows_the_map_contract_for_values_and_references() {
        let mut map = BTreeMap::from([(String::from("alpha"), 1), (String::from("beta"), 2)]);
        assert_map_contract(&map, false);
        assert_map_contract(&&map, false);
        assert_map_contract(&&mut map, false);
        assert_that_owned!(&mut map)
            .contains_key("alpha")
            .contains_value(2)
            .contains_entry("beta", 2)
            .contains_exactly_entries([("alpha", 1), ("beta", 2)]);
    }

    #[test]
    fn iteration_based_assertions_need_no_lookup_bounds_on_the_key() {
        #[derive(Debug)]
        struct Opaque;

        // A `BTreeMap` cannot be filled without `Ord`, but it can still be inspected.
        assert_that!(BTreeMap::<Opaque, i32>::new())
            .does_not_contain_value(1)
            .is_empty();
    }

    #[cfg(feature = "std")]
    #[test]
    fn iteration_based_assertions_on_a_hash_map_need_no_hasher_bound() {
        use std::collections::HashMap;

        fn helper<S>(map: &HashMap<&str, i32, S>) {
            assert_that!(map).contains_value(1).has_length(1);
        }

        helper(&HashMap::from([("a", 1)]));
    }

    #[test]
    fn bulk_key_queries_preserve_standard_borrow_key_forms_without_turbofish() {
        let map = BTreeMap::from([(String::from("alpha"), 1)]);
        let shared = String::from("alpha");
        let mut mutable = String::from("alpha");

        assert_that!(map)
            .contains_keys([String::from("alpha")])
            .contains_keys([&shared])
            .contains_keys([&mut mutable])
            .contains_keys([Box::new(String::from("alpha"))])
            .contains_keys([Rc::new(String::from("alpha"))])
            .contains_keys([Arc::new(String::from("alpha"))])
            .contains_keys([Cow::<String>::Owned(String::from("alpha"))]);
    }

    #[test]
    fn custom_map_keys_do_not_need_to_implement_eq() {
        #[derive(Debug)]
        struct Key(u8);

        #[derive(Debug)]
        struct NonEqMap(Vec<(Key, i32)>);

        impl Map for NonEqMap {
            type Key = Key;
            type Value = i32;
            const RENDERING_ORDER: RenderingOrder = RenderingOrder::PreserveIteration;

            fn entries(&self) -> impl Iterator<Item = (&Key, &i32)> {
                self.0.iter().map(|(key, value)| (key, value))
            }
        }

        impl HasLength for NonEqMap {
            fn length(&self) -> usize {
                self.0.len()
            }
        }

        impl MapLookup<Key> for NonEqMap {
            fn get_key_value(&self, expected: &Key) -> Option<(&Key, &i32)> {
                self.0
                    .iter()
                    .find(|(key, _)| key.0 == expected.0)
                    .map(|(key, value)| (key, value))
            }
        }

        let map = NonEqMap(Vec::from([(Key(1), 10)]));
        assert_that!(map).contains_exactly_entries([(Key(1), 10)]);
    }

    #[test]
    #[allow(clippy::mutable_key_type)]
    fn btree_map_adapter_uses_ordered_lookup_instead_of_scanning_entries() {
        let events = Events::default();
        let map = counting_keys(&events, 8)
            .map(|key| (key, 0))
            .collect::<BTreeMap<_, _>>();
        let missing = CountingKey {
            value: 42,
            events: &events,
        };
        events.take();

        assert_that!(MapLookup::<CountingKey>::get_key_value(&map, &missing)).is_none();
        assert_that!(count(&events, "cmp")).is_not_equal_to(0);
        assert_that!(count(&events, "eq") + count(&events, "hash")).is_equal_to(0);
    }

    #[cfg(feature = "std")]
    #[test]
    #[allow(clippy::mutable_key_type)]
    fn hash_map_adapter_uses_hashed_lookup_instead_of_scanning_entries() {
        use std::{
            collections::HashMap,
            hash::{BuildHasherDefault, DefaultHasher},
        };

        let events = Events::default();
        let map = counting_keys(&events, 8)
            .map(|key| (key, 0))
            .collect::<HashMap<_, _, BuildHasherDefault<DefaultHasher>>>();
        let missing = CountingKey {
            value: 42,
            events: &events,
        };
        events.take();

        assert_that!(MapLookup::<CountingKey>::get_key_value(&map, &missing)).is_none();
        assert_that!(count(&events, "hash")).is_not_equal_to(0);
        assert_that!(count(&events, "cmp")).is_equal_to(0);
    }

    #[test]
    #[allow(clippy::mutable_key_type)]
    fn exact_entry_assertions_do_not_compare_every_actual_and_expected_key() {
        let events = Events::default();
        let map = counting_keys(&events, 32)
            .map(|key| {
                let value = key.value;
                (key, value)
            })
            .collect::<BTreeMap<_, _>>();
        let linear_comparison_bound = map.len() * 2;
        let keys = || map.keys().cloned().collect::<Vec<_>>();
        let expected = map
            .iter()
            .map(|(key, value)| (key.clone(), *value))
            .collect::<Vec<_>>();
        let matchers = entry_matchers(
            keys()
                .into_iter()
                .map(|key| (key, predicate(|value: &i32| *value >= 0))),
        );
        let assertions = keys()
            .into_iter()
            .map(|key| {
                (key, |it: AssertThat<i32, Capture>| {
                    it.is_not_equal_to(-1);
                })
            })
            .collect::<Vec<_>>();

        events.take();
        assert_that!(map).contains_exactly_entries(expected);
        assert_that!(count(&events, "eq")).is_less_than(linear_comparison_bound);
        events.take();
        assert_that!(map).contains_exactly_entries_matching(matchers);
        assert_that!(count(&events, "eq")).is_less_than(linear_comparison_bound);
        events.take();
        assert_that!(map).contains_exactly_entries_satisfying(assertions);
        assert_that!(count(&events, "eq")).is_less_than(linear_comparison_bound);
    }

    /// Key types with only the map's own lookup bounds: `Ord` but not `Hash` for a `BTreeMap`,
    /// and `Hash` but not `Ord` for a `HashMap`. Every key-querying assertion must be available.
    #[test]
    fn standard_maps_look_up_keys_with_only_their_own_bounds() {
        fn check<Mp, K>(map: Mp, key: fn(u32) -> K)
        where
            Mp: Map<Key = K, Value = i32> + MapLookup<K> + core::fmt::Debug,
            K: core::fmt::Debug,
        {
            #[allow(clippy::trivially_copy_pass_by_ref)]
            fn is_positive(value: &i32) -> bool {
                *value > 0
            }

            fn satisfies_positive(it: AssertThat<i32, Capture>) {
                it.is_greater_than(0);
            }

            assert_that!(map)
                .contains_key(&key(1))
                .does_not_contain_key(&key(3))
                .contains_entry(&key(1), 1)
                .contains_entry_satisfying(&key(1), satisfies_positive)
                .does_not_contain_entry(&key(1), 2)
                .contains_keys([key(1), key(2)])
                .contains_exactly_entries([(key(1), 1), (key(2), 2)])
                .contains_exactly_entries_matching([
                    entry(key(1), predicate(is_positive)),
                    entry(key(2), predicate(is_positive)),
                ])
                .contains_exactly_entries_satisfying([
                    (key(1), satisfies_positive),
                    (key(2), satisfies_positive),
                ]);
        }

        #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
        struct OrdOnlyKey(u32);
        check(
            BTreeMap::from([(OrdOnlyKey(1), 1), (OrdOnlyKey(2), 2)]),
            OrdOnlyKey,
        );

        #[cfg(feature = "std")]
        {
            #[derive(Debug, Clone, PartialEq, Eq, Hash)]
            struct HashOnlyKey(u32);
            check(
                std::collections::HashMap::from([(HashOnlyKey(1), 1), (HashOnlyKey(2), 2)]),
                HashOnlyKey,
            );
        }
    }

    #[cfg(feature = "std")]
    #[test]
    fn hash_map_adapter_supports_custom_hashers_and_references() {
        use std::{
            collections::HashMap,
            hash::{BuildHasherDefault, DefaultHasher},
        };

        let mut map: HashMap<String, i32, BuildHasherDefault<DefaultHasher>> =
            HashMap::with_hasher(BuildHasherDefault::default());
        map.insert(String::from("alpha"), 1);
        map.insert(String::from("beta"), 2);
        assert_map_contract(&map, true);
        assert_map_contract(&&map, true);
        assert_that!(map)
            .contains_key("alpha")
            .contains_value(2)
            .contains_exactly_entries([(String::from("alpha"), 1), (String::from("beta"), 2)]);
    }
}
