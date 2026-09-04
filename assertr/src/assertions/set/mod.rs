//! Set assertions for `BTreeSet`, `HashSet`, and custom set types.
//!
//! A set implements [`Collection`] for order-free element assertions and [`SetLookup`] for subset,
//! superset, and disjointness relations. It does not implement
//! [`StableOrder`](crate::assertions::collection::StableOrder), even when its iteration happens to
//! be deterministic.
//!
//! Implement [`Collection`] and [`SetLookup`] for a custom set to make every set assertion
//! available.

mod assertions;
mod imp;

use alloc::collections::BTreeSet;

pub use assertions::SetAssertions;

#[cfg(feature = "std")]
use crate::renderer::RenderingOrder;
use crate::{assertions::collection::Collection, renderer::CollectionPresentation};

/// Native membership lookup capability for a set collection.
///
/// Implementing this trait declares that the collection has unique elements and can query
/// membership according to the same equivalence relation that enforces that uniqueness.
/// [`SetAssertions`] require this capability. This implementor-facing trait is not re-exported
/// from the prelude.
pub trait SetLookup: Collection {
    /// Whether `element` is a member, using the set's own lookup, such as hashing or ordering,
    /// rather than a linear scan over [`Collection::elements`].
    fn contains_element(&self, element: &Self::Item) -> bool;
}

impl<T> Collection for BTreeSet<T> {
    type Item = T;
    const PRESENTATION: CollectionPresentation = CollectionPresentation::set().with_type_hint();

    fn elements(&self) -> impl Iterator<Item = &T> {
        self.iter()
    }
}

impl<T: Ord> SetLookup for BTreeSet<T> {
    fn contains_element(&self, element: &T) -> bool {
        BTreeSet::contains(self, element)
    }
}

#[cfg(feature = "std")]
impl<T, S: core::hash::BuildHasher> Collection for std::collections::HashSet<T, S> {
    type Item = T;
    const PRESENTATION: CollectionPresentation = CollectionPresentation::set()
        .with_type_hint()
        .with_order(RenderingOrder::SortByRenderedText);

    fn elements(&self) -> impl Iterator<Item = &T> {
        self.iter()
    }
}

#[cfg(feature = "std")]
impl<T, S> SetLookup for std::collections::HashSet<T, S>
where
    T: core::hash::Hash + Eq,
    S: core::hash::BuildHasher,
{
    fn contains_element(&self, element: &T) -> bool {
        std::collections::HashSet::contains(self, element)
    }
}

/// Makes shared-reference subjects sets in their own right, mirroring the `Collection` impl for
/// `&C`.
impl<S> SetLookup for &S
where
    S: SetLookup + ?Sized,
{
    fn contains_element(&self, element: &S::Item) -> bool {
        S::contains_element(self, element)
    }
}

#[cfg(test)]
// Collection predicates receive elements by reference, including for small `Copy` element types.
#[allow(clippy::trivially_copy_pass_by_ref)]
mod tests {
    use alloc::collections::BTreeSet;
    use alloc::vec::Vec;

    use crate::prelude::*;

    use super::SetLookup;

    fn assert_set_contract<S>(actual: &S, expected: &[i32])
    where
        S: SetLookup<Item = i32> + ?Sized,
    {
        assert_that!(actual.length()).is_equal_to(expected.len());

        let mut elements = actual.elements().copied().collect::<Vec<_>>();
        elements.sort_unstable();
        assert_that!(elements).contains_exactly(expected);

        for expected in expected {
            assert_that!(actual.contains_element(expected)).is_true();
        }
        assert_that!(actual.contains_element(&42)).is_false();
    }

    fn is_one(value: &i32) -> bool {
        *value == 1
    }

    fn is_two(value: &i32) -> bool {
        *value == 2
    }

    fn is_three(value: &i32) -> bool {
        *value == 3
    }

    fn satisfies_one(it: AssertThat<i32, Capture>) {
        it.is_equal_to(1);
    }

    fn satisfies_two(it: AssertThat<i32, Capture>) {
        it.is_equal_to(2);
    }

    fn satisfies_three(it: AssertThat<i32, Capture>) {
        it.is_equal_to(3);
    }

    #[test]
    fn btree_set_adapter_follows_the_set_and_collection_contracts() {
        let set = BTreeSet::from([1, 2, 3]);
        let set_ref = &set;

        assert_set_contract(&set, &[1, 2, 3]);
        assert_set_contract(&set_ref, &[1, 2, 3]);
    }

    #[test]
    fn a_set_gets_every_order_free_collection_assertion_and_set_relation() {
        let predicates: [fn(&i32) -> bool; 3] = [is_three, is_one, is_two];
        let assertions: [fn(AssertThat<i32, Capture>); 3] =
            [satisfies_three, satisfies_one, satisfies_two];

        assert_that!(BTreeSet::from([1, 2, 3]))
            .contains(2)
            .contains_matching(is_two)
            .contains_satisfying(satisfies_two)
            .contains_all([1, 3])
            .does_not_contain(4)
            .does_not_contain_matching(|it: &i32| *it > 7)
            .does_not_contain_satisfying(|it| {
                it.is_equal_to(7);
            })
            .contains_exactly_in_any_order([3, 1, 2])
            .contains_exactly_in_any_order_matching(predicates)
            .contains_exactly_in_any_order_satisfying(assertions)
            .is_subset_of(BTreeSet::from([1, 2, 3, 4]))
            .is_superset_of(BTreeSet::from([1]))
            .is_disjoint_from(BTreeSet::from([9]));
    }

    #[test]
    fn borrowed_iteration_over_a_set_never_reports_offsets_as_indexes() {
        let failures = assert_that!(BTreeSet::from([1, 2, 3]))
            .with_location(false)
            .capture(|it| it.into_iter_does_not_contain(2));

        assert_that!(failures).has_length(1);
        assert_that!(failures[0].facts.as_slice())
            .does_not_contain_matching(|fact: &crate::Fact| fact.label == crate::Fact::INDEX);
    }

    #[test]
    fn collection_failures_include_the_btree_set_type_name() {
        let failures = assert_that!(BTreeSet::from([2]))
            .with_location(false)
            .capture(|it| it.contains(42));

        assert_that!(&failures).has_length(1);
        assert_that!(failures[0].to_string()).contains("Actual: BTreeSet {");
    }

    #[cfg(feature = "std")]
    #[test]
    fn hash_set_adapter_supports_custom_hashers_and_references() {
        use std::collections::HashSet;
        use std::hash::{BuildHasherDefault, DefaultHasher};

        let mut set: HashSet<i32, BuildHasherDefault<DefaultHasher>> =
            HashSet::with_hasher(BuildHasherDefault::default());
        set.extend([1, 2, 3]);
        let set_ref = &set;

        assert_set_contract(&set, &[1, 2, 3]);
        assert_set_contract(&set_ref, &[1, 2, 3]);
        assert_that!(set)
            .contains(2)
            .is_subset_of(BTreeSet::from([1, 2, 3, 4]));
    }
}
