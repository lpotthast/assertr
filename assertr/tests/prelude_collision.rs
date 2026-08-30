//! Coverage for the collision-prone extension-point names: the assertr prelude exports neither
//! `Condition` (renamed to `AssertrCondition`) nor `Collection` (the collection extension trait,
//! which is reachable only through its own module), so both bare names stay usable next to other
//! glob-imported preludes and next to local definitions.

use assertr::prelude::*;

/// Stand-in for a downstream prelude (e.g. `bevy::prelude`) exporting its own `Condition` and
/// `Collection` items.
mod downstream_prelude {
    pub trait Condition {
        fn holds(&self) -> bool;
    }

    impl Condition for bool {
        fn holds(&self) -> bool {
            *self
        }
    }

    pub struct Collection {
        pub size: usize,
    }

    pub struct Sequence {
        pub size: usize,
    }

    pub struct Set {
        pub size: usize,
    }

    pub struct Map {
        pub size: usize,
    }
}

use downstream_prelude::*;

// Using the bare name compiles only while the assertr prelude does not also glob-export a
// `Condition`; two glob imports providing the same name would make it ambiguous here.
fn evaluate(condition: &dyn Condition) -> bool {
    condition.holds()
}

#[test]
fn bare_condition_name_stays_usable_next_to_a_second_glob_imported_prelude() {
    assert_that!(evaluate(&true)).is_true();
}

#[test]
fn locally_defined_condition_type_coexists_with_the_prelude() {
    struct Condition {
        active: bool,
    }

    let condition = Condition { active: true };
    assert_that!(condition.active).is_true();
}

#[test]
fn assertr_conditions_remain_usable_alongside_a_foreign_condition_trait() {
    struct IsPositive;

    impl AssertrCondition<i32> for IsPositive {
        type Error = String;

        fn test(&self, value: &i32) -> Result<(), Self::Error> {
            if *value > 0 {
                Ok(())
            } else {
                Err(format!("{value} is not positive!"))
            }
        }
    }

    assert_that!(42).is(IsPositive);
}

// Same reasoning as `evaluate` above: `Collection` is the name of assertr's collection extension
// trait, but it is not re-exported from the prelude, so the bare name stays unambiguous.
fn size_of(collection: &Collection) -> usize {
    collection.size
}

#[test]
fn bare_collection_name_stays_usable_next_to_a_second_glob_imported_prelude() {
    assert_that!(size_of(&Collection { size: 3 })).is_equal_to(3);
}

#[test]
fn the_collection_assertions_work_without_the_collection_trait_in_scope() {
    // Only `CollectionAssertions` comes from the prelude; `Collection` itself is never named.
    assert_that!(vec![1, 2, 3])
        .contains(2)
        .contains_exactly([1, 2, 3]);
}

#[test]
fn a_custom_collection_gets_every_collection_assertion() {
    use assertr::assertions::HasLength;
    use assertr::assertions::collection::{
        Collection as AssertrCollection, CollectionStyle, Sequence as AssertrSequence,
    };

    /// A downstream collection type, implementing only the extension traits.
    #[derive(Debug)]
    struct Ring(Vec<i32>);

    impl AssertrCollection for Ring {
        type Item = i32;

        const TYPE_NAME: Option<&'static str> = Some("Ring");

        fn length(&self) -> usize {
            self.0.len()
        }

        fn elements(&self) -> impl Iterator<Item = &i32> {
            self.0.iter()
        }
    }

    impl AssertrSequence for Ring {}

    impl HasLength for Ring {
        fn length(&self) -> usize {
            self.0.len()
        }
    }

    let ring = Ring(vec![1, 2, 3]);
    assert_that!(ring)
        .contains(2)
        .does_not_contain(4)
        .contains_exactly([1, 2, 3])
        .contains_exactly_in_any_order([3, 2, 1])
        .has_length(3);

    assert_that!(Ring::STYLE).is_equal_to(CollectionStyle::List);
    let failures = assert_that!(Ring(vec![1, 2, 3]))
        .with_location(false)
        .capture(|it| it.contains(4));
    assert_that!(&failures).has_length(1);
    assert_that!(failures[0].to_string()).contains("Actual: Ring [");

    #[cfg(feature = "fluent")]
    {
        let mut ring = Ring(vec![1, 2, 3]);
        (&mut ring)
            .must()
            .contain(2)
            .contain_exactly([1, 2, 3])
            .have_length(3);
    }
}

/// The extension traits behind the set and map families are as collision-prone as `Collection`,
/// so they are kept out of the prelude for the same reason: a downstream `Set` or `Map` must stay
/// usable as a bare name next to a glob-imported `assertr::prelude::*`.
#[test]
fn bare_set_and_map_names_stay_usable_next_to_a_second_glob_imported_prelude() {
    fn size_of_sequence(value: &Sequence) -> usize {
        value.size
    }
    fn size_of_set(value: &Set) -> usize {
        value.size
    }
    fn size_of_map(value: &Map) -> usize {
        value.size
    }

    assert_that!(size_of_sequence(&Sequence { size: 1 })).is_equal_to(1);
    assert_that!(size_of_set(&Set { size: 2 })).is_equal_to(2);
    assert_that!(size_of_map(&Map { size: 3 })).is_equal_to(3);
}

#[test]
fn a_custom_set_gets_every_set_and_collection_assertion() {
    use assertr::assertions::HasLength;
    use assertr::assertions::collection::{Collection as AssertrCollection, CollectionStyle};
    use assertr::assertions::set::Set as AssertrSet;

    /// A downstream set type, implementing only the extension traits.
    #[derive(Debug)]
    struct Bag(Vec<i32>);

    impl AssertrCollection for Bag {
        type Item = i32;

        const STYLE: CollectionStyle = CollectionStyle::Set;

        fn length(&self) -> usize {
            self.0.len()
        }

        fn elements(&self) -> impl Iterator<Item = &i32> {
            self.0.iter()
        }
    }

    impl AssertrSet for Bag {
        fn contains_element(&self, element: &i32) -> bool {
            self.0.contains(element)
        }
    }

    impl HasLength for Bag {
        fn length(&self) -> usize {
            self.0.len()
        }
    }

    let bag = Bag(vec![1, 2, 3]);
    assert_that!(bag)
        .contains(2)
        .does_not_contain(4)
        .contains_all([1, 3])
        .contains_exactly_in_any_order([3, 2, 1])
        .is_subset_of(Bag(vec![1, 2, 3, 4]))
        .is_superset_of(Bag(vec![1]))
        .is_disjoint_from(Bag(vec![9]))
        .has_length(3);

    assert_that!(Bag::TYPE_NAME).is_none();
    let failures = assert_that!(Bag(vec![1, 2, 3]))
        .with_location(false)
        .capture(|it| it.contains(4));
    assert_that!(&failures).has_length(1);
    assert_that!(failures[0].to_string()).contains("Actual: {");

    let relation_failures = assert_that!(Bag(vec![1, 2]))
        .with_location(false)
        .capture(|it| it.is_subset_of(Bag(vec![1])));
    assert_that!(&relation_failures).has_length(1);
    assert_that!(relation_failures[0].to_string())
        .contains("Actual: {")
        .contains("Expected superset: {");

    #[cfg(feature = "fluent")]
    {
        let mut bag = Bag(vec![1, 2, 3]);
        (&mut bag)
            .must()
            .contain(2)
            .be_subset_of(Bag(vec![1, 2, 3, 4]))
            .have_length(3);
    }
}

#[test]
fn a_custom_map_gets_every_map_assertion() {
    use core::borrow::Borrow;
    use std::collections::BTreeMap;

    use assertr::assertions::HasLength;
    use assertr::assertions::map::{Map as AssertrMap, MapLookup as AssertrMapLookup};

    /// A downstream map type, implementing only the extension traits.
    #[derive(Debug)]
    struct Config(BTreeMap<String, i32>);

    impl AssertrMap for Config {
        type Key = String;
        type Value = i32;

        const TYPE_NAME: Option<&'static str> = Some("Config");

        fn length(&self) -> usize {
            self.0.len()
        }

        fn entries(&self) -> impl Iterator<Item = (&String, &i32)> {
            self.0.iter()
        }
    }

    /// One generic impl carrying the wrapped map's own lookup bounds.
    impl<Q> AssertrMapLookup<Q> for Config
    where
        Q: Ord + ?Sized,
        String: Borrow<Q>,
    {
        fn get_key_value(&self, key: &Q) -> Option<(&String, &i32)> {
            self.0.get_key_value(key)
        }
    }

    impl HasLength for Config {
        fn length(&self) -> usize {
            self.0.len()
        }
    }

    #[allow(clippy::trivially_copy_pass_by_ref)]
    fn is_three(value: &i32) -> bool {
        *value == 3
    }

    fn satisfies_three(it: AssertThat<i32, Capture>) {
        it.is_equal_to(3);
    }

    let config = Config(BTreeMap::from([("retries".to_owned(), 3)]));
    assert_that!(config)
        .contains_key("retries")
        .does_not_contain_key("timeout")
        .contains_value(3)
        .does_not_contain_value(9)
        .contains_entry("retries", 3)
        .contains_entry_satisfying("retries", satisfies_three)
        .contains_keys(["retries"])
        .contains_exactly_entries([("retries", 3)])
        .contains_exactly_entries_matching([("retries", is_three)])
        .contains_exactly_entries_satisfying([("retries", satisfies_three)])
        .has_length(1);

    #[cfg(feature = "fluent")]
    {
        let mut config = Config(BTreeMap::from([("retries".to_owned(), 3)]));
        (&mut config)
            .must()
            .contain_key("retries")
            .contain_entry("retries", 3)
            .have_length(1);
    }
}
