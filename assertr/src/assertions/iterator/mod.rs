//! Shared streaming implementation for direct and borrowed iterator assertions.
//!
//! Every assertion consumes only as much of the iterator as it needs, keeps a bounded preview of
//! the consumed elements, and raises its failure through the crate-internal failure builder. The
//! preview becomes the failure's actual value, and what the scan learned about consumption
//! becomes its facts.

mod cardinality;
mod membership;
mod positional;
mod unordered;

use alloc::{collections::VecDeque, vec::Vec};
use core::borrow::Borrow;

use crate::{
    AssertThat, AssertionFailure, AssertrPartialEq, Mode, ValueRenderer,
    failure::{Fact, FailureBuilder, FailureKind, FailureTarget},
    mode::Capture,
    renderer::{GroupStyle, RenderedValues},
    util::matching::match_bipartite,
};

pub(crate) use cardinality::{assert_has_length, assert_is_empty, assert_is_not_empty};
pub(crate) use membership::{
    assert_contains, assert_contains_all, assert_contains_matching, assert_contains_satisfying,
    assert_does_not_contain, assert_does_not_contain_matching, assert_does_not_contain_satisfying,
};
pub(crate) use positional::{
    assert_contains_contiguous, assert_contains_contiguous_matching,
    assert_contains_contiguous_satisfying, assert_contains_exactly,
    assert_contains_exactly_matching, assert_contains_exactly_satisfying, assert_ends_with,
    assert_ends_with_matching, assert_ends_with_satisfying, assert_starts_with,
    assert_starts_with_matching, assert_starts_with_satisfying,
};
pub(crate) use unordered::{
    assert_contains_exactly_in_any_order, assert_contains_exactly_in_any_order_matching,
    assert_contains_exactly_in_any_order_satisfying,
};

const PREVIEW_CAPACITY: usize = 16;

struct Preview<Item> {
    items: Vec<Item>,
    consumed: usize,
}

impl<Item> Preview<Item> {
    fn omitted(&self) -> usize {
        self.consumed.saturating_sub(self.items.len())
    }

    fn start_index(&self) -> usize {
        self.omitted()
    }

    /// The retained elements, rendered as the failure's actual value.
    fn rendered<'a, T, S, M: Mode, R>(
        &'a self,
        this: &'a AssertThat<'_, S, M, R>,
    ) -> RenderedValues<'a, T, Vec<Item>, R>
    where
        Item: Borrow<T>,
        R: ValueRenderer<T>,
    {
        this.render()
            .borrowed_values::<T, _>(&self.items, GroupStyle::List)
    }

    /// Attaches what the scan learned about consumption: how many elements were consumed, whether
    /// the preview had to drop earlier ones, and the index of the element that decided the
    /// assertion, if the caller reports positions.
    fn facts<S: FailureTarget>(
        &self,
        failure: FailureBuilder<S>,
        decisive_index: Option<usize>,
    ) -> FailureBuilder<S> {
        let mut failure = failure.fact("Consumed elements", self.consumed);
        let omitted = self.omitted();
        if omitted == 1 {
            failure = failure.note(format_args!(
                "The preview shows the last {} consumed elements. 1 earlier element was omitted.",
                self.items.len()
            ));
        } else if omitted > 1 {
            failure = failure.note(format_args!(
                "The preview shows the last {} consumed elements. {omitted} earlier elements were omitted.",
                self.items.len()
            ));
        }
        if let Some(index) = decisive_index {
            failure = failure.fact("Decisive index", index);
        }
        failure
    }
}

/// Whether the position of an element within the iteration is meaningful to the caller.
///
/// Direct iterator assertions report yield positions. Borrowed `into_iter_*` assertions run over
/// an arbitrary traversal and never mention positions.
#[derive(Clone, Copy)]
pub(crate) enum PositionReporting {
    YieldOrder,
    Unavailable,
}

impl PositionReporting {
    const fn index(self, index: usize) -> Option<usize> {
        match self {
            Self::YieldOrder => Some(index),
            Self::Unavailable => None,
        }
    }

    /// Tags a child raised for the element at `index` with its position, if positions are
    /// meaningful.
    fn locate(self, failure: AssertionFailure, index: usize) -> AssertionFailure {
        match self {
            Self::YieldOrder => failure.located_at(Fact::index(index)),
            Self::Unavailable => failure,
        }
    }
}

/// The reference value of a membership failure, and whether the assertion looked for it or
/// asserted its absence.
enum Reference<'a, E: ?Sized> {
    Expected(&'a E),
    Unexpected(&'a E),
}

// Implemented by hand: a derive would demand `E: Copy`, but only references are held.
impl<E: ?Sized> Clone for Reference<'_, E> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<E: ?Sized> Copy for Reference<'_, E> {}

/// The failures of the elements that did not satisfy a positional criterion, each with the
/// element's index in yield order.
type UnsatisfiedElements = Vec<(usize, Vec<AssertionFailure>)>;

struct Tail<Item> {
    items: VecDeque<Item>,
    consumed: usize,
}

impl<Item> Tail<Item> {
    fn new() -> Self {
        Self {
            items: VecDeque::new(),
            consumed: 0,
        }
    }
    fn push(&mut self, item: Item) {
        self.consumed += 1;
        if self.items.len() == PREVIEW_CAPACITY {
            let _ = self.items.pop_front();
        }
        self.items.push_back(item);
    }
    fn finish(self) -> Preview<Item> {
        Preview {
            items: self.items.into_iter().collect(),
            consumed: self.consumed,
        }
    }
}

fn exact_size_hint<I: Iterator>(iterator: &I) -> Option<usize> {
    let (lower, upper) = iterator.size_hint();
    (upper == Some(lower)).then_some(lower)
}

/// Flattens the failures of unsatisfied elements into children, each located at its index in
/// yield order. At most `maximum` elements are kept. Returns the children and the number of
/// omitted elements.
fn indexed_children(
    mut unsatisfied: UnsatisfiedElements,
    maximum: usize,
) -> (Vec<AssertionFailure>, usize) {
    let omitted = unsatisfied.len().saturating_sub(maximum);
    unsatisfied.truncate(maximum);
    let children = unsatisfied
        .into_iter()
        .flat_map(|(index, failures)| {
            failures
                .into_iter()
                .map(move |failure| failure.located_at(Fact::index(index)))
        })
        .collect();
    (children, omitted)
}

/// A child failure for an element that did not equal its expected counterpart.
fn unequal_element<T, E, S, M: Mode, R>(
    this: &AssertThat<'_, S, M, R>,
    element: &T,
    expected: &E,
) -> AssertionFailure
where
    R: ValueRenderer<T> + ValueRenderer<E>,
{
    FailureBuilder::detached::<T>(FailureKind::Equality)
        .actual(this.render().value(element))
        .expected(this.render().value(expected))
        .build()
}

/// A child failure for an element that did not match its predicate.
fn unmatched_element<T, S, M: Mode, R>(
    this: &AssertThat<'_, S, M, R>,
    element: &T,
) -> AssertionFailure
where
    R: ValueRenderer<T>,
{
    FailureBuilder::detached::<T>(FailureKind::Predicate)
        .actual(this.render().value(element))
        .relation("does not match its predicate")
        .build()
}
