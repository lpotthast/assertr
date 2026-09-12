//! Shared streaming implementation for direct and borrowed iterator assertions.
//!
//! Each scan consumes only as much of the iterator as it needs and retains a bounded preview
//! or owned child evidence for explanation. The chain executor tracks and raises failures. The
//! equality preview becomes the failure's actual value. Matcher previews retain selected leaf
//! evidence. What the scan learned about consumption becomes its facts.

mod cardinality;
mod membership;
mod positional;
mod unordered;

#[cfg(test)]
mod tests;

use crate::assertions::core::partial_eq::EqualToRef;
use alloc::{collections::VecDeque, vec::Vec};
use core::borrow::Borrow;
use core::{marker::PhantomData, panic::Location};

use crate::{
    AssertThat, AssertionContext, AssertionFailure, Expectation, ExpectationDiagnostics, Mode,
    ValueRenderer,
    failure::{Fact, FailureBuilder, FailureKind},
    renderer::{GroupStyle, RenderedValues, RenderingContext},
    util::matching::match_bipartite,
};

pub(crate) use cardinality::{assert_has_length, assert_is_empty, assert_is_not_empty};
pub(crate) use membership::{assert_contains, assert_contains_all, assert_does_not_contain};
pub(crate) use positional::{
    assert_contains_contiguous, assert_contains_exactly, assert_ends_with, assert_starts_with,
};
pub(crate) use unordered::assert_contains_exactly_in_any_order;

const PREVIEW_CAPACITY: usize = 16;

// Streaming definitions borrow an iterator for one scan. They are execution adapters, not
// reusable expectations over a borrowed subject. The executor below owns the iterator's lifetime.
trait Scan<I: Iterator, R> {
    type Rejection;
    const KIND: FailureKind;

    fn observe(
        &self,
        iterator: &mut I,
        context: &AssertionContext<'_, R>,
    ) -> Result<(), Self::Rejection>;

    fn explain<Target>(
        &self,
        rejection: Self::Rejection,
        failure: FailureBuilder<Target>,
        context: &AssertionContext<'_, R>,
    ) -> FailureBuilder<Target>;
}

#[track_caller]
fn execute<S, I: Iterator, D, M: Mode, R>(
    this: &AssertThat<'_, S, M, R>,
    mut iterator: I,
    definition: &D,
) where
    D: Scan<I, R>,
{
    this.test_once_after_tracking(
        D::KIND,
        Location::caller(),
        |context| {
            definition
                .observe(&mut iterator, context)
                .map_err(|rejection| (iterator, rejection))
        },
        |(iterator, rejection), failure, context| {
            let failure = definition.explain(rejection, failure, context);
            // Diagnostic values now own their contents. Release the iterator before raising,
            // including in panic mode, where unwinding could otherwise poison an owned guard.
            drop(iterator);
            failure
        },
    );
}

struct Preview<Item> {
    items: Vec<Item>,
    consumed: usize,
}

impl<Item> Preview<Item> {
    fn omitted(&self) -> usize {
        self.consumed.saturating_sub(self.items.len())
    }

    /// The retained elements, rendered as the failure's actual value.
    fn rendered<'a, T, R>(
        &'a self,
        rendering: RenderingContext<'a, R>,
    ) -> RenderedValues<'a, T, Vec<Item>, R>
    where
        Item: Borrow<T>,
        R: ValueRenderer<T>,
    {
        rendering.borrowed_values::<T, _>(&self.items, GroupStyle::List)
    }

    /// Attaches what the scan learned about consumption: how many elements were consumed, whether
    /// the preview had to drop earlier ones, and the index of the element that decided the
    /// assertion, if the caller reports positions.
    fn facts<S, R: ValueRenderer<usize>>(
        &self,
        failure: FailureBuilder<S>,
        rendering: RenderingContext<'_, R>,
        decisive_index: Option<usize>,
    ) -> FailureBuilder<S> {
        let mut failure = failure.fact(Fact::labelled(
            "Consumed elements",
            rendering.value(&self.consumed),
        ));
        let omitted = self.omitted();
        if omitted == 1 {
            failure = failure.fact(Fact::note(format_args!(
                "The preview shows the last {} consumed elements. 1 earlier element was omitted.",
                self.items.len()
            )));
        } else if omitted > 1 {
            failure = failure.fact(Fact::note(format_args!(
                "The preview shows the last {} consumed elements. {omitted} earlier elements were omitted.",
                self.items.len()
            )));
        }
        if let Some(index) = decisive_index {
            failure = failure.fact(Fact::labelled("Decisive index", index));
        }
        failure
    }
}

/// Whether the position of an element within the iteration is meaningful to the caller.
///
/// Direct iterator assertions report yield positions. Borrowed `into_iter_*` assertions run over an
/// arbitrary traversal and never mention positions.
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
}

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

/// Flattens the failures of unsatisfied elements into children, each located at its index in yield
/// order. At most `maximum` elements are kept. Returns the children and the number of omitted
/// elements.
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

/// Evaluates equality once and retains the original rejected operands as child evidence.
fn equal_element<T, E, R>(
    context: &AssertionContext<'_, R>,
    element: &T,
    expected: &E,
) -> Result<(), Vec<AssertionFailure>>
where
    T: PartialEq<E>,
    R: ValueRenderer<T> + ValueRenderer<E>,
{
    let definition = EqualToRef(expected);
    definition.evaluate(element, context).map_err(|rejection| {
        alloc::vec![
            definition
                .explain(
                    Some((element, rejection)),
                    FailureBuilder::detached::<T>(FailureKind::Equality),
                    context,
                )
                .build()
        ]
    })
}

/// A child failure for an element that did not match its predicate.
pub(crate) mod matchers;
