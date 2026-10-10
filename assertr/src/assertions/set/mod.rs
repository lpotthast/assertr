//! Set capability, relation assertions, and reusable relation expectations.
//!
//! The user-facing contract lives on [`SetLookup`], because this module itself is not public.

mod assertions;
mod imp;

use alloc::{boxed::Box, collections::BTreeSet};

pub use assertions::SetAssertions;
pub use imp::{IsDisjointFrom, IsSubsetOf, IsSupersetOf};

#[cfg(feature = "std")]
use crate::renderer::RenderingOrder;
use crate::{assertions::collection::Collection, renderer::CollectionPresentation};

/// Native membership lookup for a collection of unique elements, the capability behind the set
/// relations.
///
/// Implementing `SetLookup` for your own set type, on top of [`Collection`], makes
/// [`SetAssertions`] available: `is_subset_of`, `is_superset_of`, and `is_disjoint_from`, plus the
/// [`IsSubsetOf`], [`IsSupersetOf`], and [`IsDisjointFrom`] matchers. The relations accept any
/// other `SetLookup` type with the same element type, so your set can be compared with a
/// `BTreeSet` or a `HashSet`.
///
/// ```
/// use std::collections::BTreeSet;
///
/// use assertr::{
///     assertions::{Collection, HasLength, SetLookup},
///     prelude::*,
///     renderer::CollectionPresentation,
/// };
///
/// #[derive(Debug)]
/// struct Tags(BTreeSet<&'static str>);
///
/// impl HasLength for Tags {
///     fn length(&self) -> usize {
///         self.0.len()
///     }
/// }
///
/// impl Collection for Tags {
///     type Item = &'static str;
///     const PRESENTATION: CollectionPresentation = CollectionPresentation::set();
///
///     fn elements(&self) -> impl Iterator<Item = &Self::Item> {
///         self.0.iter()
///     }
/// }
///
/// impl SetLookup for Tags {
///     fn contains_element(&self, element: &&'static str) -> bool {
///         self.0.contains(element)
///     }
/// }
///
/// let tags = Tags(BTreeSet::from(["rust", "testing"]));
/// assert_that!(tags)
///     .contains("rust")
///     .is_subset_of(BTreeSet::from(["rust", "testing", "docs"]))
///     .is_disjoint_from(BTreeSet::from(["python"]));
/// ```
///
/// `SetLookup` is not part of the prelude, so import it from
/// [`assertr::assertions`](crate::assertions). Shared and mutable references to a set are sets
/// too.
///
/// # Contract
///
/// Implementing this trait declares that the collection holds unique elements and that
/// [`contains_element`](Self::contains_element) queries membership with the same equivalence that
/// makes them unique, such as hashing or ordering. Rust cannot check this.
///
/// Set relations use this native lookup. Element assertions such as `contains` and
/// `contains_exactly_in_any_order` come from [`Collection`] and compare elements with
/// `PartialEq`, even on sets.
///
/// A set never implements [`StableOrder`](crate::assertions::StableOrder), even when its
/// iteration happens to be deterministic, so order-sensitive assertions such as
/// `contains_exactly` do not compile for it. Choose
/// [`CollectionPresentation::set`] for set brackets in diagnostics, and add
/// `with_order(RenderingOrder::SortByRenderedText)` when iteration order is arbitrary.
#[diagnostic::on_unimplemented(
    message = "the collection has no native set membership lookup",
    label = "no set-lookup capability",
    note = "set relations such as `is_subset_of` require `SetLookup`, which declares unique elements and native membership queries; element assertions such as `contains` need only `Collection`"
)]
pub trait SetLookup: Collection {
    /// Whether `element` is a member, using the set's own lookup, such as hashing or ordering,
    /// rather than a linear scan over [`Collection::elements`].
    fn contains_element(&self, element: &Self::Item) -> bool;
}

impl<T> Collection for BTreeSet<T> {
    type Item = T;
    const PRESENTATION: CollectionPresentation = CollectionPresentation::set().with_type_hint(true);

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
impl<T, S> Collection for std::collections::HashSet<T, S> {
    type Item = T;
    const PRESENTATION: CollectionPresentation = CollectionPresentation::set()
        .with_type_hint(true)
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

/// Makes mutable-reference subjects sets, mirroring the shared-reference implementation.
impl<S> SetLookup for &mut S
where
    S: SetLookup + ?Sized,
{
    fn contains_element(&self, element: &S::Item) -> bool {
        S::contains_element(self, element)
    }
}

/// Makes boxed subjects sets, mirroring the shared-reference implementation.
impl<S> SetLookup for Box<S>
where
    S: SetLookup + ?Sized,
{
    fn contains_element(&self, element: &S::Item) -> bool {
        S::contains_element(self, element)
    }
}

#[cfg(test)]
mod tests {
    use alloc::{boxed::Box, collections::BTreeSet, vec::Vec};
    #[cfg(feature = "std")]
    use std::{
        collections::HashSet,
        hash::{BuildHasherDefault, DefaultHasher},
    };

    use super::SetLookup;
    use crate::prelude::*;

    fn assert_set_contract<S>(actual: &S, expected: &[i32])
    where
        S: SetLookup<Item = i32> + ?Sized,
    {
        assert_that!(actual.length()).is_equal_to(expected.len());

        assert_that!(actual.elements().copied().collect::<Vec<_>>())
            .contains_exactly_in_any_order(expected);

        for expected in expected {
            assert_that!(actual.contains_element(expected)).is_true();
        }
        assert_that!(actual.contains_element(&42)).is_false();
    }

    fn eq_to(expected: i32) -> impl Fn(&i32) -> bool {
        move |value| *value == expected
    }

    fn is(expected: i32) -> impl Fn(AssertThat<'_, i32, Capture>) {
        move |it| {
            it.is_equal_to(expected);
        }
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
        assert_that!(BTreeSet::from([1, 2, 3]))
            .contains(2)
            .contains_matching(matchers::predicate(eq_to(2)))
            .contains_satisfying(is(2))
            .contains_all([1, 3])
            .does_not_contain(4)
            .does_not_contain_matching(matchers::predicate(|it: &i32| *it > 7))
            .does_not_contain_satisfying(|it| {
                it.is_equal_to(7);
            })
            .contains_exactly_in_any_order([3, 1, 2])
            .contains_exactly_in_any_order_matching([3, 1, 2].map(eq_to).map(matchers::predicate))
            .contains_exactly_in_any_order_satisfying([3, 1, 2].map(is))
            .is_subset_of(BTreeSet::from([1, 2, 3, 4]))
            .is_superset_of(BTreeSet::from([1]))
            .is_disjoint_from(BTreeSet::from([9]));
    }

    #[test]
    fn collection_failures_include_the_btree_set_type_name() {
        let failures = assert_that!(BTreeSet::from([2]))
            .with_location(false)
            .capture(|it| it.contains(42));

        assert_that!(&failures).contains_exactly_satisfying([
            |element: AssertThat<AssertionFailure, Capture>| {
                element
                    .derive_owned(ToString::to_string)
                    .contains("Actual: BTreeSet {");
            },
        ]);
    }

    #[test]
    fn mutable_reference_adapter_follows_the_set_contract() {
        let mut set = BTreeSet::from([1, 2, 3]);

        assert_set_contract(&&mut set, &[1, 2, 3]);
        assert_that_owned!(&mut set)
            .contains(2)
            .is_subset_of(BTreeSet::from([1, 2, 3, 4]))
            .is_disjoint_from(BTreeSet::from([9]));
    }

    #[test]
    fn boxed_adapter_follows_the_set_contract() {
        let set = Box::new(BTreeSet::from([1, 2, 3]));

        assert_set_contract(&set, &[1, 2, 3]);
        assert_that!(set)
            .contains(2)
            .is_subset_of(BTreeSet::from([1, 2, 3, 4]))
            .is_disjoint_from(BTreeSet::from([9]));
    }

    #[cfg(feature = "std")]
    #[test]
    fn element_assertions_on_a_hash_set_need_no_hasher_bound() {
        fn helper<S>(set: &HashSet<i32, S>) {
            assert_that!(set)
                .contains(1)
                .does_not_contain(4)
                .has_length(3);
        }

        helper(&HashSet::from([1, 2, 3]));
    }

    #[cfg(feature = "std")]
    #[test]
    fn hash_set_adapter_supports_custom_hashers_and_references() {
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
