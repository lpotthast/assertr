//! Element-collection capabilities, assertion families, and reusable expectations.
//!
//! The user-facing contracts live on the capability traits [`Collection`], [`StableOrder`], and
//! [`RandomAccess`], because this module itself is not public.

mod assertions;
mod each;
mod elements_are;
mod elements_are_in_any_order;
mod extract;
mod identity;
pub(crate) mod matching;
mod random_access;
mod stable_order;
mod value;

use alloc::{
    collections::{BinaryHeap, LinkedList, VecDeque},
    vec::Vec,
};

pub use assertions::CollectionAssertions;
pub use each::{Each, each};
pub use elements_are::{
    ElementsAre, contains_contiguous_elements, elements_are, ends_with_elements,
    starts_with_elements,
};
pub use elements_are_in_any_order::{ElementsAreInAnyOrder, elements_are_in_any_order};
pub use extract::{CollectionExtractAssertions, HasSingle};
pub use identity::{
    ContainsExactlySameInstances, ContainsExactlySameInstancesInAnyOrder, ContainsSameInstanceAs,
    DoesNotContainSameInstanceAs, ExactIdentityRejection, IdentityMembershipRejection,
    UnorderedIdentityRejection,
};
pub use matching::{
    ContainsMatching, DoesNotContainMatching, contains_matching, does_not_contain_matching,
};
pub use random_access::{HasElementAt, RandomAccessExtractAssertions};
pub use stable_order::{HasFirst, HasLast, StableOrderAssertions, StableOrderExtractAssertions};
pub use value::{
    Contains, ContainsAll, ContainsContiguous, ContainsExactly, ContainsExactlyInAnyOrder,
    DoesNotContain, EndsWith, ExactElementsRejection, MissingElementsRejection,
    PositionalRejection, StartsWith,
};

use crate::{
    assertions::HasLength,
    renderer::{CollectionPresentation, RenderingOrder},
};

/// Where an expected sequence must occur within an ordered subject.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Placement {
    /// Every position of the subject, in order.
    Exact,
    /// The leading positions.
    Prefix,
    /// The trailing positions, aligned with the subject's end.
    Suffix,
    /// Any contiguous window.
    Contiguous,
}

/// A collection whose elements can be inspected repeatedly by reference, the capability behind
/// the order-free element assertions.
///
/// Implementing `Collection` for your own type makes
/// [`CollectionAssertions`](crate::assertions::CollectionAssertions) available on it: `contains`,
/// `does_not_contain`, `contains_all`, `contains_exactly_in_any_order`, their `_matching` and
/// `_satisfying` variants, and the identity checks such as `contains_same_instance_as`. In panic
/// mode, [`CollectionExtractAssertions`](crate::assertions::CollectionExtractAssertions) adds
/// `single`. Element matchers such as [`each`](crate::matchers::each) and
/// [`contains_matching`](crate::matchers::contains_matching) work on it too. The [`HasLength`]
/// supertrait adds `is_empty`, `is_not_empty`, and `has_length`.
///
/// ```
/// use assertr::{
///     assertions::{Collection, HasLength},
///     prelude::*,
///     renderer::CollectionPresentation,
/// };
///
/// #[derive(Debug)]
/// struct Inventory {
///     items: Vec<&'static str>,
/// }
///
/// impl HasLength for Inventory {
///     fn length(&self) -> usize {
///         self.items.len()
///     }
/// }
///
/// impl Collection for Inventory {
///     type Item = &'static str;
///     // An inventory is a bag: its order carries no meaning.
///     const PRESENTATION: CollectionPresentation = CollectionPresentation::list();
///
///     fn elements(&self) -> impl Iterator<Item = &Self::Item> {
///         self.items.iter()
///     }
/// }
///
/// let inventory = Inventory { items: vec!["rope", "lamp", "rope"] };
/// assert_that!(inventory)
///     .contains("lamp")
///     .does_not_contain("map")
///     .contains_exactly_in_any_order(["rope", "rope", "lamp"])
///     .has_length(3);
/// ```
///
/// `Collection` is not part of the prelude, so import it from
/// [`assertr::assertions`](crate::assertions) to implement it. Shared and mutable references to a
/// collection are collections too, so `assert_that!(&inventory)` works as well.
///
/// # Contract
///
/// - [`elements`](Self::elements) must yield the same elements in the same order on every call,
///   because some assertions traverse the collection several times, sometimes in nested passes.
/// - [`HasLength::length`] must equal the number of elements that `elements` yields.
///
/// Rust cannot check either requirement. Violating them makes outcomes and diagnostics
/// unreliable.
///
/// # Order and positions
///
/// A plain `Collection` is order-free. Its iteration offsets are never reported as element
/// positions, and order-sensitive assertions such as `contains_exactly` or `starts_with` do not
/// compile for it. Implement [`StableOrder`] as well when the order is part of your collection's
/// meaning, as in a list, and [`RandomAccess`] when you can also access any position in constant
/// time. Sets implement `Collection` and [`SetLookup`](crate::assertions::SetLookup), but never
/// `StableOrder`, even when their iteration is deterministic:
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
/// # Rendering
///
/// Assertr renders the collection structure itself, so a custom
/// [`ValueRenderer`](crate::renderer::ValueRenderer) needs to render only
/// [`Item`](Collection::Item) for the element assertions. The length assertions render the whole
/// subject and therefore need a renderer for the collection type itself. With the default
/// renderer, a derived `Debug` covers both. [`PRESENTATION`](Self::PRESENTATION) selects the
/// brackets, type hint, and ordering that diagnostics use.
pub trait Collection: HasLength {
    /// The collection's element type.
    type Item;

    /// How this collection is presented in diagnostics: list or set brackets, whether the type
    /// name is shown, and whether elements are sorted by their rendered text.
    ///
    /// Use [`CollectionPresentation::list`] or [`CollectionPresentation::set`]. Sort by rendered
    /// text with [`RenderingOrder::SortByRenderedText`] when iteration order is arbitrary, as in
    /// a hash-based collection, so reports stay stable between runs.
    ///
    /// This metadata only affects diagnostics. It cannot grant positional assertions. Implement
    /// [`StableOrder`] or [`RandomAccess`] separately when the collection provides those
    /// capabilities.
    const PRESENTATION: CollectionPresentation;

    /// The elements in iteration order.
    ///
    /// Must be repeatable. Every call must yield the same elements in the same order because some
    /// assertions make multiple, sometimes nested, passes.
    fn elements(&self) -> impl Iterator<Item = &Self::Item>;
}

/// A [`Collection`] whose iteration order defines stable, meaningful element positions, the
/// capability behind the order-sensitive assertions.
///
/// Implementing `StableOrder` for your own collection makes
/// [`StableOrderAssertions`](crate::assertions::StableOrderAssertions) available: `starts_with`,
/// `ends_with`, `contains_contiguous`, `contains_exactly`, their `_matching` and `_satisfying`
/// variants, and `contains_exactly_same_instances`. In panic mode,
/// [`StableOrderExtractAssertions`](crate::assertions::StableOrderExtractAssertions) adds
/// `first` and `last`. Failures name the index of a mismatching element.
/// The trait has no methods. It only declares that [`Collection::elements`] yields elements in
/// their meaningful order.
///
/// ```
/// use assertr::{
///     assertions::{Collection, HasLength, StableOrder},
///     prelude::*,
///     renderer::CollectionPresentation,
/// };
///
/// #[derive(Debug)]
/// struct Playlist {
///     tracks: Vec<&'static str>,
/// }
///
/// impl HasLength for Playlist {
///     fn length(&self) -> usize {
///         self.tracks.len()
///     }
/// }
///
/// impl Collection for Playlist {
///     type Item = &'static str;
///     const PRESENTATION: CollectionPresentation = CollectionPresentation::list();
///
///     fn elements(&self) -> impl Iterator<Item = &Self::Item> {
///         self.tracks.iter()
///     }
/// }
///
/// // The track order is part of what a playlist is.
/// impl StableOrder for Playlist {}
///
/// let playlist = Playlist { tracks: vec!["Intro", "Theme", "Outro"] };
/// assert_that!(playlist)
///     .starts_with(["Intro"])
///     .contains_exactly(["Intro", "Theme", "Outro"])
///     .last()
///     .is_equal_to("Outro");
/// ```
///
/// `StableOrder` is not part of the prelude, so import it from
/// [`assertr::assertions`](crate::assertions).
///
/// # When to implement it
///
/// "Stable" means that order is part of the collection's value semantics, so two collections
/// with the same elements in a different order are different values. Deterministic iteration
/// alone does not qualify. A [`alloc::collections::BTreeSet`] always iterates in sorted order,
/// but that order does not make its elements positional, so it does not implement this trait.
///
/// The capability does not promise efficient access to an arbitrary position. [`LinkedList`]
/// therefore has stable order even though it does not implement [`RandomAccess`].
///
/// Presentation metadata cannot grant this capability:
///
/// ```compile_fail,E0277
/// use assertr::assertions::{Collection, HasLength, StableOrder};
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

/// A [`StableOrder`] collection supporting constant-time access to an element by position, the
/// capability behind indexed extraction.
///
/// Implementing `RandomAccess` for your own collection makes
/// [`RandomAccessExtractAssertions`](crate::assertions::RandomAccessExtractAssertions) available,
/// whose `at` checks that an index is in bounds and continues with the element there.
/// Traversal-based sequence assertions need only [`StableOrder`].
///
/// ```
/// use assertr::{
///     assertions::{Collection, HasLength, RandomAccess, StableOrder},
///     prelude::*,
///     renderer::CollectionPresentation,
/// };
///
/// #[derive(Debug)]
/// struct Grid {
///     cells: Vec<u8>,
/// }
///
/// impl HasLength for Grid {
///     fn length(&self) -> usize {
///         self.cells.len()
///     }
/// }
///
/// impl Collection for Grid {
///     type Item = u8;
///     const PRESENTATION: CollectionPresentation = CollectionPresentation::list();
///
///     fn elements(&self) -> impl Iterator<Item = &u8> {
///         self.cells.iter()
///     }
/// }
///
/// impl StableOrder for Grid {}
///
/// impl RandomAccess for Grid {
///     fn element_at(&self, index: usize) -> Option<&u8> {
///         self.cells.get(index)
///     }
/// }
///
/// assert_that!(Grid { cells: vec![0, 7, 0] }).at(1).is_equal_to(7);
/// ```
///
/// `RandomAccess` is not part of the prelude, so import it from
/// [`assertr::assertions`](crate::assertions). [`element_at`](Self::element_at) must return the
/// same element that [`Collection::elements`] yields at that position.
///
/// A linked list has stable positions but no random access:
///
/// ```compile_fail,E0277
/// use std::collections::LinkedList;
/// use assertr::assertions::RandomAccess;
///
/// fn requires_random_access<C: RandomAccess>() {}
/// requires_random_access::<LinkedList<i32>>();
/// ```
#[diagnostic::on_unimplemented(
    message = "the collection does not support constant-time access by position",
    label = "no random-access capability",
    note = "indexed extraction such as `at` requires `RandomAccess`; traversal-based sequence assertions need only `StableOrder`"
)]
pub trait RandomAccess: StableOrder {
    /// Returns the element at the zero-based `index`, or `None` when `index` is out of bounds.
    ///
    /// Must run in constant time and agree with the position of that element in
    /// [`Collection::elements`].
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

/// A heap iterates in its internal layout order, which is neither insertion nor priority order, so
/// it is an order-free bag with arbitrary iteration.
impl<T> Collection for BinaryHeap<T> {
    type Item = T;
    const PRESENTATION: CollectionPresentation = CollectionPresentation::list()
        .with_type_hint(true)
        .with_order(RenderingOrder::SortByRenderedText);

    fn elements(&self) -> impl Iterator<Item = &T> {
        self.iter()
    }
}

/// Makes shared-reference subjects such as `AssertThat<&[T]>` (the form `assert_that!` produces for
/// unsized targets) and `AssertThat<&Vec<T>>` collections in their own right.
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

/// Makes mutable-reference subjects such as `AssertThat<&mut Vec<T>>` collections, mirroring the
/// shared-reference implementation.
impl<C> Collection for &mut C
where
    C: Collection + ?Sized,
{
    type Item = C::Item;
    const PRESENTATION: CollectionPresentation = C::PRESENTATION;

    fn elements(&self) -> impl Iterator<Item = &C::Item> {
        C::elements(self)
    }
}

impl<C> StableOrder for &mut C where C: StableOrder + ?Sized {}

impl<C> RandomAccess for &mut C
where
    C: RandomAccess + ?Sized,
{
    fn element_at(&self, index: usize) -> Option<&Self::Item> {
        C::element_at(self, index)
    }
}

#[cfg(test)]
mod tests {
    use alloc::{
        collections::{BinaryHeap, LinkedList, VecDeque},
        vec::Vec,
    };

    use super::{RandomAccess, StableOrder};
    use crate::prelude::*;

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
        assert_that!(front).is_not_empty();
        assert_that!(back).is_not_empty();
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

        assert_that!(crate::assertions::HasLength::length(&heap)).is_equal_to(3);
        assert_that!(&heap)
            .contains(3)
            .does_not_contain(4)
            .contains_exactly_in_any_order([1, 2, 3])
            .has_length(3);

        let failures = assert_that!(&heap)
            .with_location(false)
            .capture(|it| it.contains(4));
        assert_that!(failures[0].to_string()).contains(indoc::indoc! {"
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

    #[test]
    fn mutable_reference_adapters_forward_collection_and_sequence_contracts() {
        let mut vec = vec![1, 2, 3];
        let mut deque = split_deque([1, 2, 3]);
        let mut list = [1, 2, 3].into_iter().collect::<LinkedList<_>>();

        assert_collection_contract(&&mut vec, &[1, 2, 3]);
        assert_random_access_contract(&&mut vec, &[1, 2, 3]);
        assert_random_access_contract(&&mut deque, &[1, 2, 3]);
        assert_collection_contract(&&mut list, &[1, 2, 3]);

        assert_that_owned!(&mut vec)
            .contains(1)
            .starts_with([1, 2])
            .contains_exactly_in_any_order([3, 2, 1])
            .has_length(3);
        assert_that_owned!(&mut list).ends_with([2, 3]);
        assert_that_owned!(&mut vec).at(2).is_equal_to(3);
    }

    #[test]
    fn mutable_reference_subjects_implement_every_family_without_renderer_support() {
        use crate::test_support::{NoRenderer, assert_trait_impl};

        assert_trait_impl!(
            AssertThat<'static, &'static mut Vec<i32>, Panic, NoRenderer>
                => CollectionAssertions<i32, NoRenderer>
        );
        assert_trait_impl!(
            AssertThat<'static, &'static mut Vec<i32>, Panic, NoRenderer>
                => StableOrderAssertions<i32, NoRenderer>
        );
        assert_trait_impl!(
            AssertThat<'static, &'static mut Vec<i32>, Panic, NoRenderer>
                => RandomAccessExtractAssertions<'static, i32, NoRenderer>
        );
        assert_trait_impl!(
            AssertThat<'static, &'static mut alloc::collections::BTreeSet<i32>, Panic, NoRenderer>
                => SetAssertions<i32, NoRenderer>
        );
        assert_trait_impl!(
            AssertThat<'static, &'static mut alloc::collections::BTreeMap<i32, i32>, Panic, NoRenderer>
                => MapAssertions<i32, i32, NoRenderer>
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
