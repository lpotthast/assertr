//! Shared streaming implementation for direct and borrowed iterator assertions.
//!
//! Each scan consumes only as much of the iterator as it needs and retains a bounded preview
//! or owned child evidence for explanation. The chain executor tracks and raises failures. The
//! equality preview becomes the failure's actual value. Matcher scans share one evidence scope, so
//! they retain leaf evidence for the first rejections within the rendering budget. What the scan
//! learned about consumption becomes its facts.
//!
//! Adapters construct a scan directly and execute it with [`run`].

mod cardinality;
mod membership;
mod positional;
mod unordered;

#[cfg(test)]
mod tests;

use alloc::{collections::VecDeque, vec::Vec};
use core::{borrow::Borrow, marker::PhantomData, panic::Location};

pub(crate) use cardinality::{CountScan, IsExhaustedScan, IsNotExhaustedScan};
pub(crate) use membership::{
    ContainsAllScan, ContainsMatchingScan, ContainsScan, DoesNotContainMatchingScan,
    DoesNotContainScan,
};
pub(crate) use positional::{ElementsEqualScan, ElementsMatchScan};
pub(crate) use unordered::{UnorderedEqualScan, UnorderedMatchScan};

use crate::{
    AssertThat, Mode,
    assertions::{HasLength, collection::Collection},
    expectation::AssertionContext,
    failure::{Fact, FailureBuilder, FailureKind},
    renderer::{CollectionPresentation, Rendered, RenderingContext, RenderingOrder, ValueRenderer},
};

const PREVIEW_CAPACITY: usize = 16;

/// A one-use check over a borrowed iterator.
///
/// Streaming definitions borrow an iterator for one scan. They are execution adapters, not
/// reusable expectations over a borrowed subject. [`run`] owns the iterator's lifetime.
pub(crate) trait Scan<I: Iterator, R> {
    type Rejection;

    fn kind(&self) -> FailureKind;

    fn observe(
        &self,
        iterator: &mut I,
        context: &AssertionContext<'_, R>,
    ) -> Result<(), Self::Rejection>;

    fn explain(
        &self,
        rejection: Self::Rejection,
        failure: FailureBuilder,
        context: &AssertionContext<'_, R>,
    ) -> FailureBuilder;
}

/// Tracks the assertion, then lets `start` create the iterator and the scan, so neither the
/// subject nor the expected operands are accessed before tracking.
///
/// The iterator stays alive until the rejection is explained into owned diagnostic values. It is
/// released before raising, including in panic mode, where unwinding could otherwise poison an
/// owned guard.
#[track_caller]
pub(crate) fn run<S, I, D, M: Mode, R>(
    this: &AssertThat<'_, S, M, R>,
    start: impl FnOnce() -> (I, D),
) where
    I: Iterator,
    D: Scan<I, R>,
{
    this.track_assertion();
    let (mut iterator, scan) = start();
    this.test_once_after_tracking(
        scan.kind(),
        Location::caller(),
        |context| {
            scan.observe(&mut iterator, context)
                .map_err(|rejection| (iterator, rejection))
        },
        |(iterator, rejection), failure, context| {
            let failure = scan.explain(rejection, failure, context);
            drop(iterator);
            failure
        },
    );
}

/// The most recently consumed elements, up to a limit, and the number of consumed elements.
///
/// A zero limit only counts.
pub(crate) struct Tail<Item> {
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
        if self.limit == 0 {
            return;
        }
        if self.items.len() == self.limit {
            let _ = self.items.pop_front();
        }
        self.items.push_back(item);
    }

    /// Trims the retained elements to the preview capacity.
    fn finish(mut self) -> Self {
        let remove = self.items.len().saturating_sub(PREVIEW_CAPACITY);
        self.items.drain(..remove);
        self
    }

    /// The retained elements, rendered as the failure's actual value.
    fn rendered<T: ?Sized, R>(&self, rendering: RenderingContext<'_, R>) -> Rendered
    where
        Item: Borrow<T>,
        R: ValueRenderer<T>,
    {
        rendering.borrowed_values::<T, _>(&self.items, RenderingOrder::PreserveIteration)
    }

    /// Attaches what the scan learned about consumption: how many elements were consumed, whether
    /// the preview had to drop earlier ones, and the index of the element that decided the
    /// assertion, if one did.
    fn facts<R: ValueRenderer<usize>>(
        &self,
        failure: FailureBuilder,
        rendering: RenderingContext<'_, R>,
        decisive_index: Option<usize>,
    ) -> FailureBuilder {
        let failure = self.omission(consumed_fact(failure, rendering, self.consumed));
        match decisive_index {
            Some(index) => failure.fact(Fact::labelled("Decisive index", rendering.value(&index))),
            None => failure,
        }
    }

    /// States how many earlier consumed elements the preview dropped, if any.
    fn omission(&self, failure: FailureBuilder) -> FailureBuilder {
        let shown = self.items.len();
        match self.consumed - shown {
            0 => failure,
            1 => failure.fact(Fact::note(format_args!(
                "The preview shows the last {shown} consumed elements. 1 earlier element was omitted."
            ))),
            omitted => failure.fact(Fact::note(format_args!(
                "The preview shows the last {shown} consumed elements. {omitted} earlier elements were omitted."
            ))),
        }
    }
}

/// How an exact size hint must relate to the expected length.
#[derive(Clone, Copy)]
enum LengthBound {
    Exact,
    AtLeast,
}

/// A length reported exactly by `size_hint` that already rules out the expected length.
///
/// Rejections based on it consume nothing, so their reports show neither an actual value nor a
/// consumption count.
#[derive(Clone, Copy)]
pub(crate) struct KnownLength {
    reported: usize,
    expected: usize,
}

impl KnownLength {
    /// Calls `size_hint` once and returns the mismatch an exact hint establishes, if any.
    fn mismatch<I: Iterator>(iterator: &I, expected: usize, bound: LengthBound) -> Option<Self> {
        let (lower, upper) = iterator.size_hint();
        let reported = (upper == Some(lower)).then_some(lower)?;
        let rejected = match bound {
            LengthBound::Exact => reported != expected,
            LengthBound::AtLeast => reported < expected,
        };
        rejected.then_some(Self { reported, expected })
    }

    /// Attaches the reported length only, for reports showing the expected length as their value.
    fn reported_fact<R: ValueRenderer<usize>>(
        self,
        failure: FailureBuilder,
        rendering: RenderingContext<'_, R>,
    ) -> FailureBuilder {
        failure.fact(Fact::labelled(
            "Reported length",
            rendering.value(&self.reported),
        ))
    }

    /// Attaches the reported and the expected length.
    fn facts<R: ValueRenderer<usize>>(
        self,
        failure: FailureBuilder,
        rendering: RenderingContext<'_, R>,
    ) -> FailureBuilder {
        self.reported_fact(failure, rendering).fact(Fact::labelled(
            "Expected length",
            rendering.value(&self.expected),
        ))
    }
}

/// Buffers the elements of an exact unordered comparison.
///
/// An exact size hint that rules out `expected` rejects without consuming anything. Otherwise this
/// reads at most `expected + 1` elements, enough to prove that the input is longer than expected.
fn buffer_exactly<I: Iterator>(
    iterator: &mut I,
    expected: usize,
) -> Result<Vec<I::Item>, KnownLength> {
    if let Some(known) = KnownLength::mismatch(iterator, expected, LengthBound::Exact) {
        return Err(known);
    }
    let mut items = Vec::new();
    // Push one by one. Collecting would query the size hint again.
    for _ in 0..=expected {
        let Some(item) = iterator.next() else {
            break;
        };
        items.push(item);
    }
    Ok(items)
}

/// Buffered elements viewed as a list collection, so collection expectations can evaluate them.
struct Items<'a, T, Item> {
    items: &'a [Item],
    view: PhantomData<fn() -> T>,
}

impl<'a, T, Item> Items<'a, T, Item> {
    fn new(items: &'a [Item]) -> Self {
        Self {
            items,
            view: PhantomData,
        }
    }
}

impl<T, Item> HasLength for Items<'_, T, Item> {
    fn length(&self) -> usize {
        self.items.len()
    }
}

impl<T, Item: Borrow<T>> Collection for Items<'_, T, Item> {
    type Item = T;
    const PRESENTATION: CollectionPresentation = CollectionPresentation::list();

    fn elements(&self) -> impl Iterator<Item = &T> {
        self.items.iter().map(Borrow::borrow)
    }
}

/// Attaches the number of consumed elements.
fn consumed_fact<R: ValueRenderer<usize>>(
    failure: FailureBuilder,
    rendering: RenderingContext<'_, R>,
    consumed: usize,
) -> FailureBuilder {
    failure.fact(Fact::labelled(
        "Consumed elements",
        rendering.value(&consumed),
    ))
}
