//! Projections of a map onto order-free collection views of its keys or values.

use core::fmt;

use super::{Map, MapLookup};
use crate::{
    AssertThat, Mode,
    assertions::{HasLength, SetLookup, collection::Collection},
    renderer::{CollectionPresentation, DebugRenderer},
};

/// An order-free collection view of a map's stored keys, created by
/// [`MapProjectionAssertions::keys`].
///
/// The view implements [`Collection`], so every order-free collection assertion and matcher
/// applies to the keys. Keys are unique, so a map with native key lookup also makes the view a
/// [`SetLookup`] set, which enables set relations such as `is_subset_of`. The view never
/// implements [`StableOrder`](crate::assertions::StableOrder), even for a `BTreeMap`, because a
/// map's iteration order does not give its keys meaningful positions. Diagnostics present the keys
/// as a set in the map's [`RENDERING_ORDER`](Map::RENDERING_ORDER).
pub struct MapKeys<'m, Mp: ?Sized>(&'m Mp);

/// An order-free collection view of a map's stored values, created by
/// [`MapProjectionAssertions::values`].
///
/// The view implements [`Collection`], so every order-free collection assertion and matcher
/// applies to the values. Values may repeat, and their multiplicity is preserved, so
/// `contains_exactly_in_any_order` counts duplicates. Diagnostics present the values as a list in
/// the map's [`RENDERING_ORDER`](Map::RENDERING_ORDER).
pub struct MapValues<'m, Mp: ?Sized>(&'m Mp);

impl<Mp: Map + ?Sized> fmt::Debug for MapKeys<'_, Mp>
where
    Mp::Key: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_set().entries(self.elements()).finish()
    }
}

impl<Mp: Map + ?Sized> fmt::Debug for MapValues<'_, Mp>
where
    Mp::Value: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.elements()).finish()
    }
}

impl<Mp: Map + ?Sized> HasLength for MapKeys<'_, Mp> {
    fn length(&self) -> usize {
        self.0.length()
    }
}

impl<Mp: Map + ?Sized> HasLength for MapValues<'_, Mp> {
    fn length(&self) -> usize {
        self.0.length()
    }
}

impl<Mp: Map + ?Sized> Collection for MapKeys<'_, Mp> {
    type Item = Mp::Key;
    const PRESENTATION: CollectionPresentation =
        CollectionPresentation::set().with_order(Mp::RENDERING_ORDER);

    fn elements(&self) -> impl Iterator<Item = &Mp::Key> {
        self.0.entries().map(|(key, _)| key)
    }
}

impl<Mp: Map + ?Sized> Collection for MapValues<'_, Mp> {
    type Item = Mp::Value;
    const PRESENTATION: CollectionPresentation =
        CollectionPresentation::list().with_order(Mp::RENDERING_ORDER);

    fn elements(&self) -> impl Iterator<Item = &Mp::Value> {
        self.0.entries().map(|(_, value)| value)
    }
}

/// Membership uses the map's native key lookup, the same equivalence that keeps its keys unique.
impl<Mp> SetLookup for MapKeys<'_, Mp>
where
    Mp: MapLookup<<Mp as Map>::Key> + ?Sized,
{
    fn contains_element(&self, element: &Mp::Key) -> bool {
        self.0.get_key_value(element).is_some()
    }
}

/// Projections of a map onto collection views of its keys or values.
///
/// Each method borrows the assertion chain and returns a chain over a view, like
/// [`AssertThat::derive_owned`]. The collection, length, and set families then apply to the keys
/// or values, and failures propagate to the original chain. Projecting does not count as an
/// assertion and works in every mode.
///
/// ```
/// use assertr::{matchers::{each, gt}, prelude::*};
/// use std::collections::{BTreeMap, BTreeSet};
///
/// let limits = BTreeMap::from([("retries", 3), ("timeout", 30), ("backoff", 3)]);
/// let it = assert_that!(limits);
/// it.keys()
///     .contains_exactly_in_any_order(["timeout", "retries", "backoff"])
///     .is_subset_of(BTreeSet::from(["backoff", "retries", "timeout", "verbose"]));
/// it.values()
///     .matches(each(gt(0)))
///     .contains_exactly_in_any_order([3, 3, 30]);
/// ```
///
/// Neither view has positions, so order-sensitive assertions do not compile on it:
///
/// ```compile_fail,E0599
/// use assertr::prelude::*;
/// use std::collections::BTreeMap;
///
/// assert_that!(BTreeMap::from([("a", 1)])).keys().contains_exactly(["a"]);
/// ```
#[cfg_attr(feature = "fluent", assertr_macros::fluent_aliases)]
pub trait MapProjectionAssertions<'t, Mp: ?Sized, M: Mode, R = DebugRenderer> {
    /// Returns an assertion over an order-free collection view of the map's stored keys.
    fn keys(&'t self) -> AssertThat<'t, MapKeys<'t, Mp>, M, R>
    where
        R: Clone;

    /// Returns an assertion over an order-free collection view of the map's stored values.
    fn values(&'t self) -> AssertThat<'t, MapValues<'t, Mp>, M, R>
    where
        R: Clone;
}

impl<'t, Mp: Map, M: Mode, R> MapProjectionAssertions<'t, Mp, M, R> for AssertThat<'t, Mp, M, R> {
    fn keys(&'t self) -> AssertThat<'t, MapKeys<'t, Mp>, M, R>
    where
        R: Clone,
    {
        self.derive_owned(MapKeys)
    }

    fn values(&'t self) -> AssertThat<'t, MapValues<'t, Mp>, M, R>
    where
        R: Clone,
    {
        self.derive_owned(MapValues)
    }
}

#[cfg(test)]
mod tests {
    use alloc::collections::{BTreeMap, BTreeSet};

    use crate::prelude::*;

    #[cfg(feature = "fluent")]
    mod fluent_aliases {
        use super::*;

        #[test]
        fn are_as_expected() {
            // Projections are already imperative and keep their spelling.
            let map = BTreeMap::from([("a", 1)]);
            map.must().keys().contain("a");
            map.must().values().contain(1);
        }
    }

    mod renderer_contract {
        use super::*;
        use crate::{
            assertions::map::{MapKeys, MapValues},
            test_support::{NoRenderer, assert_trait_impl},
        };

        #[test]
        fn trait_is_implemented_without_renderer_support() {
            assert_trait_impl!(
                AssertThat<'static, BTreeMap<i32, i32>, Panic, NoRenderer>
                    => MapProjectionAssertions<'static, BTreeMap<i32, i32>, Panic, NoRenderer>
            );
            assert_trait_impl!(
                AssertThat<'static, MapKeys<'static, BTreeMap<i32, i32>>, Panic, NoRenderer>
                    => CollectionAssertions<i32, NoRenderer>
            );
            assert_trait_impl!(
                AssertThat<'static, MapValues<'static, BTreeMap<i32, i32>>, Panic, NoRenderer>
                    => CollectionAssertions<i32, NoRenderer>
            );
        }

        #[test]
        fn views_need_no_lookup_bounds_on_the_key() {
            #[derive(Debug)]
            struct Opaque;

            // A `BTreeMap` cannot be filled without `Ord`, but its keys and values can be
            // inspected.
            let map = BTreeMap::<Opaque, i32>::new();
            let it = assert_that!(map);
            it.keys().is_empty();
            it.values().does_not_contain(1);
        }
    }

    mod keys {
        use indoc::formatdoc;

        use super::*;

        #[test]
        fn caller_location_is_as_expected() {
            assert_caller_location!(
                assert_that!(BTreeMap::from([("a", 1)])).keys(),
                contains("b")
            );
        }

        #[test]
        fn supports_order_free_collection_and_set_assertions() {
            let map = BTreeMap::from([("a", 1), ("b", 2)]);
            assert_that!(map)
                .keys()
                .contains("a")
                .does_not_contain("c")
                .contains_exactly_in_any_order(["b", "a"])
                .has_length(2)
                .is_subset_of(BTreeSet::from(["a", "b", "c"]))
                .is_disjoint_from(BTreeSet::from(["c"]));
        }

        #[test]
        fn failures_propagate_to_the_map_chain() {
            let failures = assert_that!(BTreeMap::from([("a", 1)]))
                .with_location(false)
                .capture(|it| {
                    it.keys().contains("b");
                    it
                });
            assert_that!(failures).has_length(1);
        }

        #[test]
        fn present_keys_as_a_set_in_the_maps_rendering_order() {
            assert_that!(|| {
                let map = BTreeMap::from([("a", 1), ("b", 2)]);
                assert_that!(map).with_location(false).keys().contains("c");
            })
            .panics()
            .has_message()
            .is_equal_to(formatdoc! {r#"
                -------- assertr --------
                Actual: {{
                    "a",
                    "b",
                }}

                does not contain

                Expected: "c"
                -------- assertr --------
            "#});
        }

        #[cfg(feature = "std")]
        #[test]
        fn sort_hash_map_keys_by_rendered_text() {
            let map = std::collections::HashMap::from([("b", 2), ("c", 3), ("a", 1)]);
            let failures = assert_that!(map).with_location(false).capture(|it| {
                it.keys().contains("d");
                it
            });
            assert_that!(failures[0].to_string())
                .contains("Actual: {\n    \"a\",\n    \"b\",\n    \"c\",\n}");
        }
    }

    mod values {
        use super::*;
        use crate::matchers::{each, gt};

        #[test]
        fn caller_location_is_as_expected() {
            assert_caller_location!(
                assert_that!(BTreeMap::from([("a", 1)])).values(),
                contains(2)
            );
        }

        #[test]
        fn supports_order_free_collection_assertions_and_preserves_duplicates() {
            let map = BTreeMap::from([("a", 1), ("b", 1), ("c", 2)]);
            assert_that!(map)
                .values()
                .contains(2)
                .matches(each(gt(0)))
                .contains_exactly_in_any_order([1, 2, 1])
                .has_length(3);
            let failures = assert_that!(map).capture(|it| {
                it.values().contains_exactly_in_any_order([1, 2]);
                it
            });
            assert_that!(failures).has_length(1);
        }
    }
}
