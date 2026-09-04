//! Element-collection assertions for slices, arrays, `Vec`, `VecDeque`, `LinkedList`, `BinaryHeap`,
//! sets, and custom collections.
//!
//! The family distinguishes order-free and order-sensitive operations:
//!
//! - [`CollectionAssertions`] applies to every [`Collection`], including sets.
//! - [`StableOrderAssertions`] requires [`StableOrder`], so order-sensitive calls on sets do not
//!   compile.
//! - [`StableOrderExtractAssertions`] borrows elements selected from a stable order.
//! - [`RandomAccessExtractAssertions`] requires [`RandomAccess`] for indexed extraction.
//!
//! Implement [`HasLength`] and [`Collection`] to make collection assertions available on a custom
//! type. Implement [`StableOrder`] only when iteration defines meaningful positions, and
//! [`RandomAccess`] only when arbitrary positions can be accessed in constant time.

mod assertions;
pub(crate) mod imp;
mod random_access;
mod stable_order;

use alloc::collections::{BinaryHeap, LinkedList, VecDeque};
use alloc::vec::Vec;

use crate::{
    assertions::HasLength,
    renderer::{CollectionPresentation, RenderingOrder},
};

pub use assertions::CollectionAssertions;
pub use random_access::RandomAccessExtractAssertions;
pub use stable_order::{StableOrderAssertions, StableOrderExtractAssertions};

/// A collection whose elements can be inspected repeatedly by reference.
///
/// Implementing this trait makes [`CollectionAssertions`] available. Its [`HasLength`] supertrait
/// also provides `is_empty` and `has_length`. This implementor-facing trait is not re-exported
/// from the prelude.
///
/// Indexed assertions and indexed diagnostics require [`StableOrder`]. Bags and sets have no
/// indexes in assertr's model; their iteration offsets are never exposed as element positions. A
/// set therefore cannot call an order-sensitive assertion:
///
/// ```compile_fail,E0277
/// use assertr::prelude::*;
/// use std::collections::BTreeSet;
///
/// assert_that!(BTreeSet::from([1, 2, 3])).contains_exactly([1, 2, 3]);
/// ```
///
/// Use an order-free assertion such as `contains_exactly_in_any_order` instead.
///
/// Assertr renders the collection structure. A custom [`ValueRenderer`](crate::ValueRenderer)
/// needs to render only [`Item`](Collection::Item).
pub trait Collection: HasLength {
    /// The collection's element type.
    type Item;

    /// How this collection is presented in diagnostics.
    ///
    /// This metadata cannot grant positional assertions. Implement [`StableOrder`] or
    /// [`RandomAccess`] separately when the collection provides those capabilities.
    const PRESENTATION: CollectionPresentation;

    /// The elements in iteration order.
    ///
    /// Must be repeatable. Every call must yield the same elements in the same order because some
    /// assertions make multiple, sometimes nested, passes.
    fn elements(&self) -> impl Iterator<Item = &Self::Item>;
}

/// A [`Collection`] whose iteration order defines stable, meaningful element positions.
///
/// This capability unlocks order-sensitive assertions and index-bearing diagnostics. It does not
/// promise efficient access to an arbitrary position. [`LinkedList`] therefore has stable order
/// even though it does not implement [`RandomAccess`]. "Stable" means that order is part of the
/// collection's value semantics; deterministic iteration alone does not qualify, so a
/// [`alloc::collections::BTreeSet`] does not implement this trait.
///
/// Presentation metadata cannot grant this capability:
///
/// ```compile_fail,E0277
/// use assertr::assertions::{HasLength, collection::{Collection, StableOrder}};
/// use assertr::renderer::CollectionPresentation;
///
/// struct Deterministic(Vec<i32>);
///
/// impl HasLength for Deterministic {
///     fn length(&self) -> usize { self.0.len() }
/// }
///
/// impl Collection for Deterministic {
///     type Item = i32;
///     const PRESENTATION: CollectionPresentation = CollectionPresentation::list();
///
///     fn elements(&self) -> impl Iterator<Item = &i32> { self.0.iter() }
/// }
///
/// fn requires_stable_order<C: StableOrder>() {}
/// requires_stable_order::<Deterministic>();
/// ```
#[diagnostic::on_unimplemented(
    message = "the collection has no stable, meaningful element order",
    label = "no stable-order capability",
    note = "order-sensitive assertions require `StableOrder`; use an order-free assertion such as `contains_exactly_in_any_order` instead"
)]
pub trait StableOrder: Collection {}

/// A [`StableOrder`] collection supporting constant-time access to an element by position.
///
/// APIs that retrieve a position directly require this capability. Traversal-based sequence
/// assertions need only [`StableOrder`].
///
/// A linked list has stable positions but no random access:
///
/// ```compile_fail,E0277
/// use std::collections::LinkedList;
/// use assertr::assertions::collection::RandomAccess;
///
/// fn requires_random_access<C: RandomAccess>() {}
/// requires_random_access::<LinkedList<i32>>();
/// ```
pub trait RandomAccess: StableOrder {
    /// Returns the element at `index`, or `None` when `index` is out of bounds.
    fn element_at(&self, index: usize) -> Option<&Self::Item>;
}

impl<T> Collection for [T] {
    type Item = T;
    const PRESENTATION: CollectionPresentation = CollectionPresentation::list();

    fn elements(&self) -> impl Iterator<Item = &T> {
        self.iter()
    }
}

impl<T> StableOrder for [T] {}

impl<T> RandomAccess for [T] {
    fn element_at(&self, index: usize) -> Option<&T> {
        self.get(index)
    }
}

impl<T, const N: usize> Collection for [T; N] {
    type Item = T;
    const PRESENTATION: CollectionPresentation = CollectionPresentation::list();

    fn elements(&self) -> impl Iterator<Item = &T> {
        self.iter()
    }
}

impl<T, const N: usize> StableOrder for [T; N] {}

impl<T, const N: usize> RandomAccess for [T; N] {
    fn element_at(&self, index: usize) -> Option<&T> {
        self.as_slice().get(index)
    }
}

impl<T> Collection for Vec<T> {
    type Item = T;
    const PRESENTATION: CollectionPresentation = CollectionPresentation::list();

    fn elements(&self) -> impl Iterator<Item = &T> {
        self.iter()
    }
}

impl<T> StableOrder for Vec<T> {}

impl<T> RandomAccess for Vec<T> {
    fn element_at(&self, index: usize) -> Option<&T> {
        self.as_slice().get(index)
    }
}

impl<T> Collection for VecDeque<T> {
    type Item = T;
    const PRESENTATION: CollectionPresentation = CollectionPresentation::list();

    fn elements(&self) -> impl Iterator<Item = &T> {
        self.iter()
    }
}

impl<T> StableOrder for VecDeque<T> {}

impl<T> RandomAccess for VecDeque<T> {
    fn element_at(&self, index: usize) -> Option<&T> {
        VecDeque::get(self, index)
    }
}

impl<T> Collection for LinkedList<T> {
    type Item = T;
    const PRESENTATION: CollectionPresentation = CollectionPresentation::list();

    fn elements(&self) -> impl Iterator<Item = &T> {
        self.iter()
    }
}

impl<T> StableOrder for LinkedList<T> {}

/// A heap iterates in its internal layout order, which is neither insertion nor priority order,
/// so it is an order-free bag with arbitrary iteration.
impl<T> Collection for BinaryHeap<T> {
    type Item = T;
    const PRESENTATION: CollectionPresentation = CollectionPresentation::list()
        .with_type_hint()
        .with_order(RenderingOrder::SortByRenderedText);

    fn elements(&self) -> impl Iterator<Item = &T> {
        self.iter()
    }
}

/// Makes shared-reference subjects such as `AssertThat<&[T]>` (the form `assert_that!` produces
/// for unsized targets) and `AssertThat<&Vec<T>>` collections in their own right.
impl<C> Collection for &C
where
    C: Collection + ?Sized,
{
    type Item = C::Item;
    const PRESENTATION: CollectionPresentation = C::PRESENTATION;

    fn elements(&self) -> impl Iterator<Item = &C::Item> {
        C::elements(self)
    }
}

impl<C> StableOrder for &C where C: StableOrder + ?Sized {}

impl<C> RandomAccess for &C
where
    C: RandomAccess + ?Sized,
{
    fn element_at(&self, index: usize) -> Option<&Self::Item> {
        C::element_at(self, index)
    }
}

#[cfg(test)]
mod tests {
    use crate::prelude::*;
    use alloc::collections::{BinaryHeap, LinkedList, VecDeque};
    use alloc::vec::Vec;

    use super::{RandomAccess, StableOrder};

    struct Holder {
        deque: VecDeque<i32>,
    }

    fn assert_collection_contract<C>(actual: &C, expected: &[i32])
    where
        C: StableOrder<Item = i32> + ?Sized,
    {
        assert_that!(actual.length()).is_equal_to(expected.len());
        assert_that!(actual.elements().copied().collect::<Vec<_>>()).contains_exactly(expected);
    }

    fn assert_random_access_contract<C>(actual: &C, expected: &[i32])
    where
        C: RandomAccess<Item = i32> + ?Sized,
    {
        for (index, expected) in expected.iter().enumerate() {
            assert_that!(actual.element_at(index)).is_equal_to(Some(expected));
        }
        assert_that!(actual.element_at(expected.len())).is_none();
    }

    fn split_deque(elements: [i32; 3]) -> VecDeque<i32> {
        let mut deque = VecDeque::with_capacity(elements.len() + 1);
        deque.push_back(elements[1]);
        deque.push_back(elements[2]);
        deque.push_front(elements[0]);

        let (front, back) = deque.as_slices();
        assert_that!(front.is_empty()).is_false();
        assert_that!(back.is_empty()).is_false();
        assert_that!(deque.iter().copied().collect::<Vec<_>>()).contains_exactly(elements);
        deque
    }

    #[test]
    fn built_in_sequence_adapters_follow_the_collection_contract() {
        let elements = [1, 2, 3];
        let vec = elements.to_vec();
        let deque = elements.into_iter().collect::<VecDeque<_>>();
        let list = elements.into_iter().collect::<LinkedList<_>>();

        assert_collection_contract(elements.as_slice(), &elements);
        assert_collection_contract(&elements, &elements);
        assert_collection_contract(&vec, &elements);
        assert_collection_contract(&deque, &elements);
        assert_collection_contract(&list, &elements);
    }

    #[test]
    fn indexable_sequence_adapters_follow_the_random_access_contract() {
        let elements = [1, 2, 3];
        let vec = elements.to_vec();
        let deque = elements.into_iter().collect::<VecDeque<_>>();
        let vec_ref = &vec;

        assert_random_access_contract(elements.as_slice(), &elements);
        assert_random_access_contract(&elements, &elements);
        assert_random_access_contract(&vec, &elements);
        assert_random_access_contract(&deque, &elements);
        assert_random_access_contract(&vec_ref, &elements);
    }

    #[test]
    fn binary_heap_is_an_order_free_bag_with_arbitrary_iteration() {
        let heap = BinaryHeap::from([2, 3, 1]);

        assert_that!(heap.length()).is_equal_to(3);
        assert_that!(&heap)
            .contains(3)
            .does_not_contain(4)
            .contains_exactly_in_any_order([1, 2, 3])
            .has_length(3);

        let failures = assert_that!(&heap)
            .with_location(false)
            .capture(|it| it.contains(4));
        assert_that!(TextReporter.report(&failures[0])).contains(indoc::indoc! {"
            Actual: BinaryHeap [
                1,
                2,
                3,
            ] (sorted for rendering)
        "});
    }

    #[test]
    fn a_physically_split_vec_deque_uses_its_logical_iteration_order() {
        let deque = split_deque([1, 2, 3]);

        assert_collection_contract(&deque, &[1, 2, 3]);
        assert_that!(deque).contains(2).contains_exactly([1, 2, 3]);
    }

    #[test]
    fn shared_reference_adapters_forward_collection_and_sequence_contracts() {
        let vec = vec![1, 2, 3];
        let deque = split_deque([1, 2, 3]);
        let vec_ref = &vec;
        let deque_ref = &deque;

        assert_collection_contract(&vec_ref, &[1, 2, 3]);
        assert_collection_contract(&deque_ref, &[1, 2, 3]);

        assert_that!(Holder { deque }).satisfies_ref(
            |holder| &holder.deque,
            |deque| {
                deque.contains(2).contains_exactly([1, 2, 3]);
            },
        );
    }

    #[cfg(feature = "fluent")]
    #[test]
    fn fluent_entry_point_borrows_a_mutable_reference_pointee() {
        fn check(values: &mut Vec<i32>) {
            values
                .must()
                .contain(2)
                .contain_exactly([1, 2, 3])
                .have_length(3);
        }

        check(&mut vec![1, 2, 3]);
    }
}
