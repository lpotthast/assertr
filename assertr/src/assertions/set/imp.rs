//! Reusable set relations and their rejection evidence.

use alloc::{collections::BTreeSet, vec::Vec};
use core::ptr;

use super::SetLookup;
use crate::{
    assertions::collection::Collection,
    expectation::{AssertionContext, Expectation, ExpectedOperand, operand_view},
    failure::{Fact, FailureBuilder, FailureKind},
    renderer::{RenderingOrder, ValueRenderer},
};

/// The relations, offending elements, and expected operand shared by all set relations.
struct SetRelation {
    holds: &'static str,
    fails: &'static str,
    offending: &'static str,
}

impl SetRelation {
    /// Describes the relation, or explains a rejection with its offending elements.
    fn explain<S, O, X, R>(
        &self,
        expected: &O,
        rejected: Option<(&S, &[&X], RenderingOrder)>,
        failure: FailureBuilder,
        context: &AssertionContext<'_, R>,
    ) -> FailureBuilder
    where
        S: Collection + ?Sized,
        O: Collection,
        X: ?Sized,
        R: ValueRenderer<S::Item> + ValueRenderer<O::Item> + ValueRenderer<X>,
    {
        let render = context.render();
        let failure = match rejected {
            None => failure.relation(self.holds),
            Some((actual, offending, order)) => failure
                .actual(render.collection(actual))
                .relation(self.fails)
                .fact(Fact::labelled(
                    self.offending,
                    render.borrowed_values::<X, _>(&offending, order),
                )),
        };
        failure.expected(render.collection(expected))
    }
}

/// The native lookup query an operand element selects for the subject's element type.
fn query<S: Collection + ?Sized, E: ExpectedOperand<S::Item>>(element: &E) -> &E::View {
    operand_view::<S::Item, _>(element)
}

const SUBSET: SetRelation = SetRelation {
    holds: "is a subset of",
    fails: "is not a subset of",
    offending: "Elements not in expected",
};

const SUPERSET: SetRelation = SetRelation {
    holds: "is a superset of",
    fails: "is not a superset of",
    offending: "Elements not in actual",
};

const DISJOINT: SetRelation = SetRelation {
    holds: "is disjoint from",
    fails: "is not disjoint from",
    offending: "Overlapping elements",
};

/// Declares a set relation expectation owning its operand.
macro_rules! set_relation_type {
    ($(#[$attr:meta])* $name:ident) => {
        $(#[$attr])*
        #[derive(Debug, Clone)]
        pub struct $name<O>(O);

        impl<O> $name<O> {
            /// Owns the other side of the relation: a set, an array, a slice, a vector, or any
            /// other collection.
            #[must_use]
            pub const fn new(other: O) -> Self {
                Self(other)
            }
        }
    };
}

set_relation_type!(
    /// Checks that a set is a subset of a collection, looking each of the collection's elements up
    /// in the set.
    ///
    /// The expected superset can be any [`Collection`] whose elements select a borrowed view of
    /// the set's element type through [`BorrowFor`](crate::borrow_for::BorrowFor), such as another
    /// set, an array, or a vector. The set's native [`SetLookup`] finds the stored elements
    /// the expected collection names. A rejection lists the subject elements no expected
    /// element named, in the subject's presentation order.
    /// [`SetAssertions::is_subset_of`](crate::assertions::SetAssertions::is_subset_of) executes
    /// this same definition.
    ///
    /// ```
    /// use std::collections::BTreeSet;
    ///
    /// use assertr::{matchers::set::IsSubsetOf, prelude::*};
    ///
    /// assert_that!(BTreeSet::from([1, 2])).matches(IsSubsetOf::new(BTreeSet::from([1, 2, 3])));
    /// assert_that!(BTreeSet::from([1, 2])).matches(IsSubsetOf::new([1, 2, 3]));
    /// ```
    IsSubsetOf
);

set_relation_type!(
    /// Checks that a set is a superset of a collection, looking each of the collection's elements
    /// up in the set.
    ///
    /// The expected subset can be any [`Collection`] whose elements select a borrowed view of the
    /// set's element type through [`BorrowFor`](crate::borrow_for::BorrowFor). A rejection lists
    /// the expected elements missing from the subject, in the expected collection's
    /// presentation order.
    /// [`SetAssertions::is_superset_of`](crate::assertions::SetAssertions::is_superset_of)
    /// executes this same definition.
    ///
    /// ```
    /// use std::collections::BTreeSet;
    ///
    /// use assertr::{matchers::set::IsSupersetOf, prelude::*};
    ///
    /// assert_that!(BTreeSet::from([1, 2, 3])).matches(IsSupersetOf::new(BTreeSet::from([3])));
    /// assert_that!(BTreeSet::from([1, 2, 3])).matches(IsSupersetOf::new(vec![1, 3]));
    /// ```
    IsSupersetOf
);

set_relation_type!(
    /// Checks that a set shares no element with a collection, looking each of the collection's
    /// elements up in the set.
    ///
    /// The other collection can be any [`Collection`] whose elements select a borrowed view of the
    /// set's element type through [`BorrowFor`](crate::borrow_for::BorrowFor). A rejection lists
    /// the overlapping elements as stored in the subject, in the subject's presentation order.
    /// [`SetAssertions::is_disjoint_from`](crate::assertions::SetAssertions::is_disjoint_from)
    /// executes this same definition.
    ///
    /// ```
    /// use std::collections::BTreeSet;
    ///
    /// use assertr::{matchers::set::IsDisjointFrom, prelude::*};
    ///
    /// assert_that!(BTreeSet::from([1, 2])).matches(IsDisjointFrom::new(BTreeSet::from([3])));
    /// assert_that!(BTreeSet::from([1, 2])).matches(IsDisjointFrom::new([3, 4]));
    /// ```
    IsDisjointFrom
);

// Keep the element projected in GAT-bearing impls so `S: 'a` carries its lifetime. Fully qualified
// views avoid a bound cycle without an independently inferred query parameter.
impl<S, O, R> Expectation<S, R> for IsSubsetOf<O>
where
    S: Collection
        + SetLookup<<<O as Collection>::Item as ExpectedOperand<<S as Collection>::Item>>::View>
        + ?Sized,
    O: Collection,
    O::Item: ExpectedOperand<S::Item>,
    R: ValueRenderer<S::Item> + ValueRenderer<O::Item>,
{
    type Success<'a>
        = ()
    where
        Self: 'a,
        S: 'a;
    type Rejection<'a>
        = Vec<&'a S::Item>
    where
        Self: 'a,
        S: 'a;

    fn evaluate<'a>(
        &'a self,
        actual: &'a S,
        _: &AssertionContext<'_, R>,
    ) -> Result<Self::Success<'a>, Self::Rejection<'a>> {
        // `SetLookup` answers queries from the other side, so the stored elements it finds are
        // remembered by address, which the lookup contract makes stable. Every element not found
        // is missing from the expected superset. A set holds at most one zero-sized element, so
        // shared addresses of zero-sized elements stay unambiguous.
        let found = self
            .0
            .elements()
            .filter_map(|element| actual.get_element(query::<S, _>(element)))
            .map(ptr::from_ref)
            .collect::<BTreeSet<_>>();
        let missing = actual
            .elements()
            .filter(|element| !found.contains(&ptr::from_ref(*element)))
            .collect::<Vec<_>>();
        if missing.is_empty() {
            Ok(())
        } else {
            Err(missing)
        }
    }

    const KIND: FailureKind = FailureKind::Membership;

    fn explain<'a>(
        &'a self,
        rejected: Option<(&'a S, Self::Rejection<'a>)>,
        failure: FailureBuilder,
        context: &AssertionContext<'_, R>,
    ) -> FailureBuilder {
        let rejected = rejected
            .as_ref()
            .map(|(actual, missing)| (*actual, missing.as_slice(), S::PRESENTATION.order()));
        SUBSET.explain::<S, O, S::Item, R>(&self.0, rejected, failure, context)
    }
}

impl<S, O, R> Expectation<S, R> for IsSupersetOf<O>
where
    S: Collection
        + SetLookup<<<O as Collection>::Item as ExpectedOperand<<S as Collection>::Item>>::View>
        + ?Sized,
    O: Collection,
    O::Item: ExpectedOperand<S::Item>,
    R: ValueRenderer<S::Item> + ValueRenderer<O::Item>,
{
    type Success<'a>
        = ()
    where
        Self: 'a,
        S: 'a;
    type Rejection<'a>
        = Vec<&'a O::Item>
    where
        Self: 'a,
        S: 'a;

    fn evaluate<'a>(
        &'a self,
        actual: &'a S,
        _: &AssertionContext<'_, R>,
    ) -> Result<Self::Success<'a>, Self::Rejection<'a>> {
        let missing = self
            .0
            .elements()
            .filter(|element| actual.get_element(query::<S, _>(*element)).is_none())
            .collect::<Vec<_>>();
        if missing.is_empty() {
            Ok(())
        } else {
            Err(missing)
        }
    }

    const KIND: FailureKind = FailureKind::Membership;

    fn explain<'a>(
        &'a self,
        rejected: Option<(&'a S, Self::Rejection<'a>)>,
        failure: FailureBuilder,
        context: &AssertionContext<'_, R>,
    ) -> FailureBuilder {
        let rejected = rejected
            .as_ref()
            .map(|(actual, missing)| (*actual, missing.as_slice(), O::PRESENTATION.order()));
        SUPERSET.explain::<S, O, O::Item, R>(&self.0, rejected, failure, context)
    }
}

impl<S, O, R> Expectation<S, R> for IsDisjointFrom<O>
where
    S: Collection
        + SetLookup<<<O as Collection>::Item as ExpectedOperand<<S as Collection>::Item>>::View>
        + ?Sized,
    O: Collection,
    O::Item: ExpectedOperand<S::Item>,
    R: ValueRenderer<S::Item> + ValueRenderer<O::Item>,
{
    type Success<'a>
        = ()
    where
        Self: 'a,
        S: 'a;
    type Rejection<'a>
        = Vec<&'a S::Item>
    where
        Self: 'a,
        S: 'a;

    fn evaluate<'a>(
        &'a self,
        actual: &'a S,
        _: &AssertionContext<'_, R>,
    ) -> Result<Self::Success<'a>, Self::Rejection<'a>> {
        // Repeated elements of a list operand find the same stored element, which is reported
        // once.
        let overlapping = self
            .0
            .elements()
            .filter_map(|element| actual.get_element(query::<S, _>(element)))
            .map(ptr::from_ref)
            .collect::<BTreeSet<_>>();
        if overlapping.is_empty() {
            return Ok(());
        }
        Err(actual
            .elements()
            .filter(|element| overlapping.contains(&ptr::from_ref(*element)))
            .collect())
    }

    const KIND: FailureKind = FailureKind::Membership;

    fn explain<'a>(
        &'a self,
        rejected: Option<(&'a S, Self::Rejection<'a>)>,
        failure: FailureBuilder,
        context: &AssertionContext<'_, R>,
    ) -> FailureBuilder {
        let rejected = rejected.as_ref().map(|(actual, overlapping)| {
            (*actual, overlapping.as_slice(), S::PRESENTATION.order())
        });
        DISJOINT.explain::<S, O, S::Item, R>(&self.0, rejected, failure, context)
    }
}
