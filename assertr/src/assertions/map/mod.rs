//! Map assertions for `BTreeMap`, `HashMap`, and custom map types.
//!
//! [`MapAssertions`] is blanket-implemented for every [`Map`]. Maps have their own family because
//! their entries are key/value pairs rather than plain collection elements.
//!
//! Implement [`Map`] and [`MapLookup`] for a custom map. Implement [`MapKeyQuery`] only for custom
//! expected-key adapters used by bulk key assertions.

mod assertions;
mod imp;

use alloc::borrow::{Cow, ToOwned};
use alloc::boxed::Box;
use alloc::collections::BTreeMap;
use alloc::rc::Rc;
use alloc::string::String;
use alloc::sync::Arc;
use core::borrow::Borrow;

use crate::{assertions::HasLength, renderer::RenderingOrder};

pub use assertions::MapAssertions;

/// A keyed collection supporting iteration over its entries.
///
/// Implementing this trait makes iteration-based [`MapAssertions`] available. Implement
/// [`MapLookup`] for key queries. The prelude does not re-export this implementor-facing trait.
///
/// Assertr renders map syntax. A custom [`ValueRenderer`](crate::ValueRenderer) needs to render
/// only [`Key`](Map::Key) and [`Value`](Map::Value).
pub trait Map: HasLength {
    /// The map's key type.
    type Key;

    /// The map's value type.
    type Value;

    /// Whether diagnostics preserve iteration order or sort entries by rendered text.
    ///
    /// This affects presentation only; it does not change matching or lookup behavior.
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
/// `MapLookup<Q>` for the query type `Q`. `Q` may be an unsized borrowed view of
/// [`Map::Key`], such as `str` for a `String` key, and is compared according to the contract of
/// [`Borrow`].
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
/// use assertr::assertions::map::{Map, MapLookup};
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

/// Adapts one expected bulk key `E` to the query type a [`MapLookup`] implementation accepts.
///
/// Rust can infer `Q` from the `&Q` argument of a single-key assertion. It cannot infer the same
/// type from `E: Borrow<Q>` in a bulk assertion because every `E` also implements `Borrow<E>`.
/// This associated type keeps `Q` out of the assertion method's generic arguments, so existing
/// calls remain inference-friendly while bulk methods can use native borrowed lookup.
///
/// The standard `Borrow<K>` input forms (`K`, `&K`, `&mut K`, `Box<K>`, `Rc<K>`, `Arc<K>`, and
/// `Cow<K>`) are implemented generically. String keys additionally accept the corresponding
/// `str` views, so both of these compile without a turbofish:
///
/// ```
/// use std::collections::BTreeMap;
///
/// use assertr::prelude::*;
///
/// let map = BTreeMap::from([(String::from("a"), 1)]);
/// assert_that!(&map).contains_keys(["a"]);
/// assert_that!(map).contains_exactly_entries([("a", 1)]);
/// ```
///
/// A custom expected-key wrapper or borrowed view implements this trait with the stored key as
/// `K` and the map's [`MapLookup`] key as [`Query`](MapKeyQuery::Query). The trait is intentionally
/// not re-exported from the prelude. Only adapter authors need to name it.
///
/// ```
/// use assertr::assertions::map::MapKeyQuery;
///
/// struct UserKey(String);
///
/// impl MapKeyQuery<UserKey> for &str {
///     type Query = str;
///
///     fn as_query(&self) -> &str {
///         self
///     }
/// }
/// ```
pub trait MapKeyQuery<K: ?Sized> {
    /// The key view passed to [`MapLookup`].
    type Query: ?Sized;

    /// Borrows this expected key as its lookup query.
    fn as_query(&self) -> &Self::Query;
}

impl<K: ?Sized> MapKeyQuery<K> for K {
    type Query = K;

    fn as_query(&self) -> &K {
        self
    }
}

impl<K: ?Sized> MapKeyQuery<K> for &K {
    type Query = K;

    fn as_query(&self) -> &K {
        self
    }
}

impl<K: ?Sized> MapKeyQuery<K> for &mut K {
    type Query = K;

    fn as_query(&self) -> &K {
        self
    }
}

impl<K: ?Sized> MapKeyQuery<K> for Box<K> {
    type Query = K;

    fn as_query(&self) -> &K {
        self
    }
}

impl<K: ?Sized> MapKeyQuery<K> for Rc<K> {
    type Query = K;

    fn as_query(&self) -> &K {
        self
    }
}

impl<K: ?Sized> MapKeyQuery<K> for Arc<K> {
    type Query = K;

    fn as_query(&self) -> &K {
        self
    }
}

impl<K> MapKeyQuery<K> for Cow<'_, K>
where
    K: ToOwned + ?Sized,
{
    type Query = K;

    fn as_query(&self) -> &K {
        <Self as Borrow<K>>::borrow(self)
    }
}

impl MapKeyQuery<String> for &str {
    type Query = str;

    fn as_query(&self) -> &str {
        self
    }
}

impl MapKeyQuery<String> for &mut str {
    type Query = str;

    fn as_query(&self) -> &str {
        self
    }
}

impl MapKeyQuery<String> for Box<str> {
    type Query = str;

    fn as_query(&self) -> &str {
        self
    }
}

impl MapKeyQuery<String> for Rc<str> {
    type Query = str;

    fn as_query(&self) -> &str {
        self
    }
}

impl MapKeyQuery<String> for Arc<str> {
    type Query = str;

    fn as_query(&self) -> &str {
        self
    }
}

impl MapKeyQuery<String> for Cow<'_, str> {
    type Query = str;

    fn as_query(&self) -> &str {
        <Self as Borrow<str>>::borrow(self)
    }
}

impl<K: Ord, V> Map for BTreeMap<K, V> {
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
impl<K, V, S> Map for std::collections::HashMap<K, V, S>
where
    K: core::hash::Hash + Eq,
    S: core::hash::BuildHasher,
{
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

#[cfg(test)]
mod tests {
    use alloc::borrow::Cow;
    use alloc::boxed::Box;
    use alloc::collections::BTreeMap;
    use alloc::rc::Rc;
    use alloc::string::String;
    use alloc::sync::Arc;
    use alloc::vec::Vec;
    use core::cell::Cell;
    use core::cmp::Ordering;
    use core::hash::{Hash, Hasher};

    use crate::prelude::*;

    use crate::assertions::HasLength;

    use super::{Map, MapLookup, RenderingOrder};

    #[derive(Debug, Default)]
    struct LookupCounts {
        equality: Cell<usize>,
        hashing: Cell<usize>,
        ordering: Cell<usize>,
    }

    impl LookupCounts {
        fn reset(&self) {
            self.equality.set(0);
            self.hashing.set(0);
            self.ordering.set(0);
        }
    }

    #[derive(Clone, Debug)]
    struct CountingKey {
        value: i32,
        counts: Rc<LookupCounts>,
    }

    impl CountingKey {
        fn new(value: i32, counts: &Rc<LookupCounts>) -> Self {
            Self {
                value,
                counts: Rc::clone(counts),
            }
        }
    }

    impl PartialEq for CountingKey {
        fn eq(&self, other: &Self) -> bool {
            self.counts.equality.update(|count| count + 1);
            self.value == other.value
        }
    }

    impl Eq for CountingKey {}

    impl PartialOrd for CountingKey {
        fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
            Some(self.cmp(other))
        }
    }

    impl Ord for CountingKey {
        fn cmp(&self, other: &Self) -> Ordering {
            self.counts.ordering.update(|count| count + 1);
            self.value.cmp(&other.value)
        }
    }

    impl Hash for CountingKey {
        fn hash<H: Hasher>(&self, state: &mut H) {
            self.counts.hashing.update(|count| count + 1);
            self.value.hash(state);
        }
    }

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
    fn btree_map_adapter_follows_the_map_contract_for_values_and_references() {
        let map = BTreeMap::from([(String::from("alpha"), 1), (String::from("beta"), 2)]);
        let map_ref = &map;

        assert_map_contract(&map, false);
        assert_map_contract(&map_ref, false);
    }

    #[test]
    #[allow(clippy::mutable_key_type)]
    fn btree_map_adapter_uses_ordered_lookup_instead_of_scanning_entries() {
        let counts = Rc::new(LookupCounts::default());
        let map = (0..8)
            .map(|value| (CountingKey::new(value, &counts), value))
            .collect::<BTreeMap<_, _>>();
        let missing = CountingKey::new(42, &counts);
        counts.reset();

        assert_that!(
            <BTreeMap<CountingKey, i32> as MapLookup<CountingKey>>::get_key_value(&map, &missing)
        )
        .is_none();
        assert_that!(counts.ordering.get()).is_not_equal_to(0);
        assert_that!(counts.equality.get()).is_equal_to(0);
        assert_that!(counts.hashing.get()).is_equal_to(0);
    }

    /// A key type that is `Ord` but not `Hash`: the ordinary case for a hand-written `BTreeMap`
    /// key. Every key-querying assertion must be available with the map's own bounds alone.
    #[test]
    fn btree_map_adapter_looks_up_keys_that_only_implement_ord() {
        #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
        struct OrdOnlyKey(u32);

        #[allow(clippy::trivially_copy_pass_by_ref)]
        fn is_positive(value: &i32) -> bool {
            *value > 0
        }

        fn satisfies_positive(it: AssertThat<i32, Capture>) {
            it.is_greater_than(0);
        }

        let map = BTreeMap::from([(OrdOnlyKey(1), 1), (OrdOnlyKey(2), 2)]);
        assert_that!(map)
            .contains_key(&OrdOnlyKey(1))
            .does_not_contain_key(&OrdOnlyKey(3))
            .contains_entry(&OrdOnlyKey(1), 1)
            .contains_entry_satisfying(&OrdOnlyKey(1), satisfies_positive)
            .does_not_contain_entry(&OrdOnlyKey(1), 2)
            .contains_keys([OrdOnlyKey(1), OrdOnlyKey(2)])
            .contains_exactly_entries([(OrdOnlyKey(1), 1), (OrdOnlyKey(2), 2)])
            .contains_exactly_entries_matching([
                (OrdOnlyKey(1), is_positive),
                (OrdOnlyKey(2), is_positive),
            ])
            .contains_exactly_entries_satisfying([
                (OrdOnlyKey(1), satisfies_positive),
                (OrdOnlyKey(2), satisfies_positive),
            ]);
    }

    #[test]
    #[allow(clippy::mutable_key_type, clippy::trivially_copy_pass_by_ref)]
    fn exact_entry_assertions_do_not_compare_every_actual_and_expected_key() {
        fn matches(value: &i32) -> bool {
            *value >= 0
        }

        fn satisfies(it: AssertThat<i32, Capture>) {
            it.is_not_equal_to(-1);
        }

        let counts = Rc::new(LookupCounts::default());
        let map = (0..32)
            .map(|value| (CountingKey::new(value, &counts), value))
            .collect::<BTreeMap<_, _>>();
        let linear_comparison_bound = map.len() * 2;

        let expected = map
            .iter()
            .map(|(key, value)| (key.clone(), *value))
            .collect::<Vec<_>>();
        counts.reset();
        assert_that!(map).contains_exactly_entries(expected);
        assert_that!(counts.equality.get()).is_less_than(linear_comparison_bound);

        let predicates = map
            .keys()
            .map(|key| (key.clone(), matches as fn(&i32) -> bool))
            .collect::<Vec<_>>();
        counts.reset();
        assert_that!(map).contains_exactly_entries_matching(predicates);
        assert_that!(counts.equality.get()).is_less_than(linear_comparison_bound);

        let assertions = map
            .keys()
            .map(|key| (key.clone(), satisfies as fn(AssertThat<i32, Capture>)))
            .collect::<Vec<_>>();
        counts.reset();
        assert_that!(map).contains_exactly_entries_satisfying(assertions);
        assert_that!(counts.equality.get()).is_less_than(linear_comparison_bound);
    }

    #[cfg(feature = "std")]
    #[test]
    fn hash_map_adapter_supports_custom_hashers_and_references() {
        use std::collections::HashMap;
        use std::hash::{BuildHasherDefault, DefaultHasher};

        let mut map: HashMap<String, i32, BuildHasherDefault<DefaultHasher>> =
            HashMap::with_hasher(BuildHasherDefault::default());
        map.insert(String::from("alpha"), 1);
        map.insert(String::from("beta"), 2);
        let map_ref = &map;

        assert_map_contract(&map, true);
        assert_map_contract(&map_ref, true);
        assert_that!(map)
            .contains_key("alpha")
            .contains_value(2)
            .contains_exactly_entries([(String::from("alpha"), 1), (String::from("beta"), 2)]);
    }

    #[cfg(feature = "std")]
    #[test]
    #[allow(clippy::mutable_key_type)]
    fn hash_map_adapter_uses_hashed_lookup_instead_of_scanning_entries() {
        use std::collections::HashMap;
        use std::hash::{BuildHasherDefault, DefaultHasher};

        let counts = Rc::new(LookupCounts::default());
        let map = (0..8)
            .map(|value| (CountingKey::new(value, &counts), value))
            .collect::<HashMap<_, _, BuildHasherDefault<DefaultHasher>>>();
        let missing = CountingKey::new(42, &counts);
        counts.reset();

        assert_that!(
            <HashMap<CountingKey, i32, BuildHasherDefault<DefaultHasher>> as MapLookup<
                CountingKey,
            >>::get_key_value(&map, &missing)
        )
        .is_none();
        assert_that!(counts.hashing.get()).is_not_equal_to(0);
        assert_that!(counts.ordering.get()).is_equal_to(0);
    }

    /// A key type that is `Hash` but not `Ord`: the ordinary case for a hand-written `HashMap`
    /// key. Every key-querying assertion must be available with the map's own bounds alone.
    #[cfg(feature = "std")]
    #[test]
    fn hash_map_adapter_looks_up_keys_that_only_implement_hash() {
        use std::collections::HashMap;

        #[derive(Debug, Clone, PartialEq, Eq, Hash)]
        struct HashOnlyKey(u32);

        #[allow(clippy::trivially_copy_pass_by_ref)]
        fn is_positive(value: &i32) -> bool {
            *value > 0
        }

        fn satisfies_positive(it: AssertThat<i32, Capture>) {
            it.is_greater_than(0);
        }

        let map = HashMap::from([(HashOnlyKey(1), 1), (HashOnlyKey(2), 2)]);
        assert_that!(map)
            .contains_key(&HashOnlyKey(1))
            .does_not_contain_key(&HashOnlyKey(3))
            .contains_entry(&HashOnlyKey(1), 1)
            .contains_entry_satisfying(&HashOnlyKey(1), satisfies_positive)
            .does_not_contain_entry(&HashOnlyKey(1), 2)
            .contains_keys([HashOnlyKey(1), HashOnlyKey(2)])
            .contains_exactly_entries([(HashOnlyKey(1), 1), (HashOnlyKey(2), 2)])
            .contains_exactly_entries_matching([
                (HashOnlyKey(1), is_positive),
                (HashOnlyKey(2), is_positive),
            ])
            .contains_exactly_entries_satisfying([
                (HashOnlyKey(1), satisfies_positive),
                (HashOnlyKey(2), satisfies_positive),
            ]);
    }
}
