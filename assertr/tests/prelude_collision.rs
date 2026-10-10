//! Coverage for collision-prone names. Capability traits, including `HasLength`, remain in their
//! own modules, so their bare names stay usable next to other glob-imported preludes. Downstream
//! capability implementations are covered in `custom_assertions.rs`.

use assertr::prelude::*;

/// Stand-in for a downstream prelude exporting its own collection names.
mod downstream_prelude {
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

    pub struct HasLength {
        pub size: usize,
    }
}

use downstream_prelude::*;

// `Collection` is the name of assertr's collection extension trait, but it is not re-exported
// from the prelude, so the bare name stays unambiguous next to another glob import.
fn size_of(collection: &Collection) -> usize {
    collection.size
}

#[test]
fn bare_collection_name_stays_usable_next_to_a_second_glob_imported_prelude() {
    assert_that!(size_of(&Collection { size: 3 })).is_equal_to(3);
}

#[test]
fn the_collection_assertions_work_without_the_collection_trait_in_scope() {
    // Only `CollectionAssertions` comes from the prelude. `Collection` itself is never named.
    assert_that!(vec![1, 2, 3])
        .contains(2)
        .contains_exactly([1, 2, 3]);
}

/// The capability traits behind the length, set, and map families are as collision-prone as
/// `Collection`, so they are kept out of the prelude for the same reason. A downstream `HasLength`,
/// `Set`, or `Map` must stay usable as a bare name next to a glob-imported `assertr::prelude::*`.
#[test]
fn bare_capability_names_stay_usable_next_to_a_second_glob_imported_prelude() {
    fn size_of_sequence(value: &Sequence) -> usize {
        value.size
    }
    fn size_of_set(value: &Set) -> usize {
        value.size
    }
    fn size_of_map(value: &Map) -> usize {
        value.size
    }
    fn size_of_length(value: &HasLength) -> usize {
        value.size
    }

    assert_that!(size_of_sequence(&Sequence { size: 1 })).is_equal_to(1);
    assert_that!(size_of_set(&Set { size: 2 })).is_equal_to(2);
    assert_that!(size_of_map(&Map { size: 3 })).is_equal_to(3);
    assert_that!(size_of_length(&HasLength { size: 4 })).is_equal_to(4);
}

mod matcher_names {
    mod foreign_prelude {
        pub struct ConstraintDescription;
        pub struct Matcher;
        pub fn eq() -> bool {
            true
        }
        pub fn anything() -> bool {
            true
        }
    }
    #[test]
    fn matcher_names_remain_available_to_other_preludes() {
        use assertr::prelude::*;
        use foreign_prelude::*;
        let _ = (ConstraintDescription, Matcher);
        assert_that!(eq() && anything()).is_true();
        assert_that!(1).matches(matchers::eq(1));
    }
}
