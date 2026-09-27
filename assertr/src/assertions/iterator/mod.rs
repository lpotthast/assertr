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

use crate::assertions::core::partial_eq::EqualTo;
use alloc::{collections::VecDeque, vec::Vec};
use core::borrow::Borrow;
use core::{marker::PhantomData, panic::Location};

use crate::{
    AssertThat, AssertionContext, Expectation, ExpectationDiagnostics, Mode, ValueRenderer,
    failure::{Fact, FailureBuilder, FailureKind, PathSegment},
    renderer::{GroupStyle, RenderedValues, RenderingContext},
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
    items: VecDeque<Item>,
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
    ) -> RenderedValues<'a, T, VecDeque<Item>, R>
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

struct Tail<Item> {
    limit: usize,
    items: VecDeque<Item>,
    consumed: usize,
}

impl<Item> Tail<Item> {
    fn new(limit: usize) -> Self {
        Self {
            limit,
            items: VecDeque::new(),
            consumed: 0,
        }
    }
    fn push(&mut self, item: Item) {
        self.consumed += 1;
        if self.items.len() == self.limit {
            let _ = self.items.pop_front();
        }
        if self.limit > 0 {
            self.items.push_back(item);
        }
    }
    fn finish(mut self) -> Preview<Item> {
        let remove = self.items.len().saturating_sub(PREVIEW_CAPACITY);
        self.items.drain(..remove);
        Preview {
            items: self.items,
            consumed: self.consumed,
        }
    }
}

fn exact_size_hint<I: Iterator>(iterator: &I) -> Option<usize> {
    let (lower, upper) = iterator.size_hint();
    (upper == Some(lower)).then_some(lower)
}

/// Compares once, constructing indexed equality evidence only for retained rejections.
fn equal_element<T, E: ?Sized, R>(
    context: &mut AssertionContext<'_, R>,
    index: usize,
    element: &T,
    expected: &E,
) -> bool
where
    T: PartialEq<E>,
    R: ValueRenderer<T> + ValueRenderer<E>,
{
    let matched = element.eq(expected);
    if !matched {
        context.record_with(|context| {
            let render = context.render();
            FailureBuilder::detached::<T>(FailureKind::Equality)
                .actual(render.value(element))
                .expected(render.value(expected))
                .path([PathSegment::Index(index)])
                .build()
        });
    }
    matched
}

/// A child failure for an element that did not match its predicate.
pub(crate) mod matchers;
