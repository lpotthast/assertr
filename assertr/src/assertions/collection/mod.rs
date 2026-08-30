//! Element-collection assertions for slices, arrays, `Vec`, `VecDeque`, `LinkedList`, sets, and
//! custom collections.
//!
//! The family distinguishes order-free and order-sensitive operations:
//!
//! - [`CollectionAssertions`] applies to every [`Collection`], including sets.
//! - [`SequenceAssertions`] requires [`Sequence`], so order-sensitive calls on sets do not compile.
//!
//! Implement [`Collection`] and [`Sequence`] to make the collection assertions available on a
//! custom type. [`CollectionStyle`] controls how that type is rendered in diagnostics.

mod assertions;
pub(crate) mod imp;
mod sequence;

use alloc::collections::{LinkedList, VecDeque};
use alloc::vec::Vec;

pub use assertions::CollectionAssertions;
pub use sequence::SequenceAssertions;

/// The structural syntax used to render a [`Collection`] in assertion diagnostics.
///
/// This metadata is independent of [`Collection::TYPE_NAME`]. A named collection can use
/// list syntax (`Ring [1, 2]`). An unnamed collection can use set syntax (`{1, 2}`). Custom
/// set implementations normally select [`Set`](CollectionStyle::Set). Sequences normally keep
/// the default [`List`](CollectionStyle::List).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CollectionStyle {
    /// Render the collection with list delimiters, e.g. `[1, 2]`.
    List,
    /// Render the collection with set delimiters, e.g. `{1, 2}`.
    Set,
}

/// A collection whose elements can be inspected repeatedly by reference.
///
/// Implementing this trait makes [`CollectionAssertions`] available. Implement
/// [`HasLength`](crate::assertions::HasLength) as well for `is_empty` and `has_length`. This
/// implementor-facing trait is not re-exported from the prelude.
///
/// Element positions in diagnostics use iteration order. For an unordered subject, an index
/// identifies an iteration position rather than a stable logical position. [`Sequence`] marks
/// collections whose order is meaningful.
///
/// Assertr renders the collection structure. A custom [`ValueRenderer`](crate::ValueRenderer)
/// needs to render only [`Item`](Collection::Item).
pub trait Collection {
    /// The collection's element type.
    type Item;

    /// The structural syntax used to render this collection in assertion diagnostics.
    ///
    /// The default is [`CollectionStyle::List`]. Set-like implementations should override this
    /// with [`CollectionStyle::Set`]. This is independent of
    /// [`TYPE_NAME`](Self::TYPE_NAME), which controls only the optional prefix.
    const STYLE: CollectionStyle = CollectionStyle::List;

    /// The name prefixed to the rendered subject in failure messages, e.g. `Actual: HashSet {1, 2}`.
    ///
    /// `None`, the default, omits the prefix. `Some(name)` renders it before the structure selected
    /// by [`STYLE`](Self::STYLE), such as `Actual: Ring [1, 2]` or `Actual: Bag {1, 2}`. Built-in
    /// sequences use `None`. Built-in sets use their concrete type name.
    const TYPE_NAME: Option<&'static str> = None;

    /// The number of elements in this collection.
    fn length(&self) -> usize;

    /// The elements in iteration order.
    ///
    /// Must be repeatable. Every call must yield the same elements in the same order because some
    /// assertions make multiple, sometimes nested, passes.
    fn elements(&self) -> impl Iterator<Item = &Self::Item>;
}

/// A [`Collection`] whose iteration order is part of its semantics.
///
/// Both `Vec` and `HashSet` implement [`Collection`], but only `Vec` implements `Sequence` and gets
/// [`SequenceAssertions`]. This implementor-facing trait is not re-exported from the prelude.
///
/// A set is not a `Sequence`, so calling an order-sensitive assertion on one reports the missing
/// bound and recommends the order-free alternative:
///
/// ```compile_fail,E0277
/// use assertr::prelude::*;
/// use std::collections::BTreeSet;
///
/// assert_that!(BTreeSet::from([1, 2, 3])).contains_exactly([1, 2, 3]);
/// ```
///
/// Use the order-free assertion instead:
///
/// ```
/// use assertr::prelude::*;
/// use std::collections::BTreeSet;
///
/// assert_that!(BTreeSet::from([1, 2, 3])).contains_exactly_in_any_order([3, 2, 1]);
/// ```
#[diagnostic::on_unimplemented(
    message = "`{Self}` has no meaningful element order",
    label = "not an ordered collection",
    note = "order-sensitive assertions are available for `Sequence` types only. \
            use `contains_exactly_in_any_order` instead"
)]
pub trait Sequence: Collection {}

impl<T> Collection for [T] {
    type Item = T;

    fn length(&self) -> usize {
        <[T]>::len(self)
    }

    fn elements(&self) -> impl Iterator<Item = &T> {
        self.iter()
    }
}

impl<T> Sequence for [T] {}

impl<T, const N: usize> Collection for [T; N] {
    type Item = T;

    fn length(&self) -> usize {
        N
    }

    fn elements(&self) -> impl Iterator<Item = &T> {
        self.iter()
    }
}

impl<T, const N: usize> Sequence for [T; N] {}

impl<T> Collection for Vec<T> {
    type Item = T;

    fn length(&self) -> usize {
        Vec::len(self)
    }

    fn elements(&self) -> impl Iterator<Item = &T> {
        self.iter()
    }
}

impl<T> Sequence for Vec<T> {}

impl<T> Collection for VecDeque<T> {
    type Item = T;

    fn length(&self) -> usize {
        VecDeque::len(self)
    }

    fn elements(&self) -> impl Iterator<Item = &T> {
        self.iter()
    }
}

impl<T> Sequence for VecDeque<T> {}

impl<T> Collection for LinkedList<T> {
    type Item = T;

    fn length(&self) -> usize {
        LinkedList::len(self)
    }

    fn elements(&self) -> impl Iterator<Item = &T> {
        self.iter()
    }
}

impl<T> Sequence for LinkedList<T> {}

/// Makes shared-reference subjects such as `AssertThat<&[T]>` (the form `assert_that!` produces
/// for unsized targets) and `AssertThat<&Vec<T>>` collections in their own right.
impl<C> Collection for &C
where
    C: Collection + ?Sized,
{
    type Item = C::Item;

    const STYLE: CollectionStyle = C::STYLE;
    const TYPE_NAME: Option<&'static str> = C::TYPE_NAME;

    fn length(&self) -> usize {
        C::length(self)
    }

    fn elements(&self) -> impl Iterator<Item = &C::Item> {
        C::elements(self)
    }
}

impl<C> Sequence for &C where C: Sequence + ?Sized {}

#[cfg(test)]
mod tests {
    use crate::prelude::*;
    use alloc::collections::{LinkedList, VecDeque};
    use alloc::vec::Vec;

    use super::{Collection, CollectionStyle, Sequence};

    struct Holder {
        deque: VecDeque<i32>,
    }

    fn assert_collection_contract<C>(actual: &C, expected: &[i32])
    where
        C: Collection<Item = i32> + ?Sized,
    {
        assert_that!(C::STYLE).is_equal_to(CollectionStyle::List);
        assert_that!(C::TYPE_NAME).is_none();
        assert_that!(actual.length()).is_equal_to(expected.len());
        assert_that!(actual.elements().copied().collect::<Vec<_>>()).contains_exactly(expected);
    }

    fn require_sequence<C: Sequence + ?Sized>(_actual: &C) {}

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

        require_sequence(elements.as_slice());
        require_sequence(&elements);
        require_sequence(&vec);
        require_sequence(&deque);
        require_sequence(&list);
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
        require_sequence(&vec_ref);
        require_sequence(&deque_ref);

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
