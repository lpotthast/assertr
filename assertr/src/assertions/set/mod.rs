//! Set capability, relation assertions, and reusable relation expectations.
//!
//! The user-facing contract lives on [`SetLookup`], because this module itself is not public.

mod assertions;
mod imp;

use alloc::{boxed::Box, collections::BTreeSet};
use core::borrow::Borrow;

pub use assertions::SetAssertions;
pub use imp::{IsDisjointFrom, IsSubsetOf, IsSupersetOf};

#[cfg(feature = "std")]
use crate::renderer::RenderingOrder;
use crate::{
    assertions::collection::{Collection, StableOrder},
    renderer::CollectionPresentation,
};

/// Native lookup of a stored element by a borrowed query `Q`, for a collection of unique
/// elements. This is the capability behind the set relations.
///
/// Implementing `SetLookup` for your own set type, on top of [`Collection`], makes
/// [`SetAssertions`] available: `is_subset_of`, `is_superset_of`, and `is_disjoint_from`, plus the
/// [`IsSubsetOf`], [`IsSupersetOf`], and [`IsDisjointFrom`] matchers. The relations look the other
/// side's elements up in your set, so they accept any collection as the other side: another set,
/// an array, or a vector. `Q` defaults to the element type. Implement the trait generically over
/// `Q` when your native lookup accepts borrowed queries, such as `str` for `String` elements:
///
/// ```
/// use core::borrow::Borrow;
/// use std::collections::BTreeSet;
///
/// use assertr::{
///     assertions::{Collection, HasLength, SetLookup},
///     prelude::*,
///     renderer::CollectionPresentation,
/// };
///
/// #[derive(Debug)]
/// struct Tags(BTreeSet<String>);
///
/// impl HasLength for Tags {
///     fn length(&self) -> usize {
///         self.0.len()
///     }
/// }
///
/// impl Collection for Tags {
///     type Item = String;
///     const PRESENTATION: CollectionPresentation = CollectionPresentation::set();
///
///     fn elements(&self) -> impl Iterator<Item = &Self::Item> {
///         self.0.iter()
///     }
/// }
///
/// // The bounds are those of the native lookup, here `BTreeSet::get`.
/// impl<Q: Ord + ?Sized> SetLookup<Q> for Tags
/// where
///     String: Borrow<Q>,
/// {
///     fn get_element(&self, element: &Q) -> Option<&String> {
///         self.0.get(element)
///     }
/// }
///
/// let tags = Tags(BTreeSet::from([String::from("rust"), String::from("testing")]));
/// assert_that!(tags)
///     .contains("rust")
///     .is_subset_of(["rust", "testing", "docs"])
///     .is_disjoint_from(["python"]);
/// ```
///
/// `SetLookup` is not part of the prelude, so import it from
/// [`assertr::assertions`](crate::assertions). Shared and mutable references to a set are sets
/// too.
///
/// # Contract
///
/// Implementing this trait declares that the collection holds unique elements and that
/// [`get_element`](Self::get_element) finds the stored element equivalent to a query, with the
/// same equivalence that makes the elements unique, such as hashing or ordering. The returned
/// reference must point at the element that [`Collection::elements`] yields, because `is_subset_of`
/// identifies the elements it found by address. Rust cannot check this.
///
/// Set relations use this native lookup. Element assertions such as `contains` and
/// `contains_exactly_in_any_order` come from [`Collection`] and compare elements with
/// `PartialEq`, even on sets.
///
/// Order-sensitive assertions such as `contains_exactly` additionally require
/// [`StableOrder`](crate::assertions::StableOrder), which a set implements only when its type
/// guarantees its iteration order. A `BTreeSet` does, because it always yields its elements in
/// ascending order. A `HashSet` does not. Choose [`CollectionPresentation::set`] for set brackets
/// in diagnostics, and add `with_order(RenderingOrder::SortByRenderedText)` when iteration order
/// is arbitrary.
#[diagnostic::on_unimplemented(
    message = "the collection has no native set lookup for `{Q}`",
    label = "no set-lookup capability for `{Q}`",
    note = "set relations such as `is_subset_of` require `SetLookup`, which declares unique elements and native membership queries; element assertions such as `contains` need only `Collection`"
)]
pub trait SetLookup<Q: ?Sized = <Self as Collection>::Item>: Collection {
    /// The stored element equivalent to `element`, if any, found with the set's own lookup, such
    /// as hashing or ordering, rather than a linear scan over [`Collection::elements`].
    fn get_element(&self, element: &Q) -> Option<&Self::Item>;
}

impl<T> Collection for BTreeSet<T> {
    type Item = T;
    const PRESENTATION: CollectionPresentation = CollectionPresentation::set().with_type_hint(true);

    fn elements(&self) -> impl Iterator<Item = &T> {
        self.iter()
    }
}

/// A `BTreeSet` always yields its elements in ascending order, so their positions are
/// meaningful: `first` is the minimum, and `contains_exactly` expects the elements sorted.
impl<T> StableOrder for BTreeSet<T> {}

impl<T, Q> SetLookup<Q> for BTreeSet<T>
where
    T: Ord + Borrow<Q>,
    Q: Ord + ?Sized,
{
    fn get_element(&self, element: &Q) -> Option<&T> {
        BTreeSet::get(self, element)
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
impl<T, Q, S> SetLookup<Q> for std::collections::HashSet<T, S>
where
    T: core::hash::Hash + Eq + Borrow<Q>,
    Q: core::hash::Hash + Eq + ?Sized,
    S: core::hash::BuildHasher,
{
    fn get_element(&self, element: &Q) -> Option<&T> {
        std::collections::HashSet::get(self, element)
    }
}

/// Makes shared-reference subjects sets in their own right, mirroring the `Collection` impl for
/// `&C`.
impl<S, Q> SetLookup<Q> for &S
where
    S: SetLookup<Q> + ?Sized,
    Q: ?Sized,
{
    fn get_element(&self, element: &Q) -> Option<&S::Item> {
        S::get_element(self, element)
    }
}

/// Makes mutable-reference subjects sets, mirroring the shared-reference implementation.
impl<S, Q> SetLookup<Q> for &mut S
where
    S: SetLookup<Q> + ?Sized,
    Q: ?Sized,
{
    fn get_element(&self, element: &Q) -> Option<&S::Item> {
        S::get_element(self, element)
    }
}

/// Makes boxed subjects sets, mirroring the shared-reference implementation.
impl<S, Q> SetLookup<Q> for Box<S>
where
    S: SetLookup<Q> + ?Sized,
    Q: ?Sized,
{
    fn get_element(&self, element: &Q) -> Option<&S::Item> {
        S::get_element(self, element)
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
            assert_that!(actual.get_element(expected)).is_equal_to(Some(expected));
        }
        assert_that!(actual.get_element(&42)).is_none();
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
    fn a_btree_set_gets_order_sensitive_assertions_in_ascending_order() {
        let set = BTreeSet::from([3, 1, 2]);
        let it = assert_that!(set)
            .starts_with([1])
            .ends_with([3])
            .contains_contiguous([2, 3])
            .contains_exactly([1, 2, 3]);
        it.first().is_equal_to(1);
        it.last().is_equal_to(3);
    }

    #[test]
    fn a_btree_set_reports_the_position_of_an_unsorted_expectation() {
        assert_that!(|| {
            assert_that!(BTreeSet::from([1, 2, 3]))
                .with_location(false)
                .contains_exactly([1, 3, 2]);
        })
        .panics()
        .message()
        .is_equal_to(indoc::indoc! {"
            -------- assertr --------
            Expression: `BTreeSet::from([1, 2, 3])`

            Actual: BTreeSet {
                1,
                2,
                3,
            }

            does not contain exactly

            Expected: [
                1,
                3,
                2,
            ]

            Details:
              - Only the order of the elements differs.
            Nested failures:
              - At [1]:
                Expected: 3

                  Actual: 2
            -------- assertr --------
        "});
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
