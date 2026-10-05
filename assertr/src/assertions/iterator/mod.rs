//! Shared streaming implementation for direct and borrowed iterator assertions.
//!
//! Each scan consumes only as much of the iterator as it needs and retains a bounded preview
//! or owned child evidence for explanation. The chain executor tracks and raises failures. The
//! equality preview becomes the failure's actual value. Matcher scans share one evidence scope, so
//! they retain leaf evidence for the first rejections within the rendering budget. What the scan
//! learned about consumption becomes its facts.

mod cardinality;
pub(crate) mod matchers;
mod membership;
mod positional;
mod unordered;

#[cfg(test)]
mod tests;

use alloc::{collections::VecDeque, vec::Vec};
use core::borrow::Borrow;
use core::{marker::PhantomData, panic::Location};

use crate::{
    AssertThat, AssertionContext, ExpectationDiagnostics, Mode, ValueRenderer,
    assertions::{HasLength, collection::Collection},
    failure::{Fact, FailureBuilder, FailureKind, PathSegment},
    renderer::{CollectionPresentation, GroupStyle, RenderedValues, RenderingContext},
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
        let failure = failure.fact(Fact::labelled(
            "Consumed elements",
            rendering.value(&self.consumed),
        ));
        let failure = self.omission(failure);
        match decisive_index {
            Some(index) => failure.fact(Fact::labelled("Decisive index", index)),
            None => failure,
        }
    }

    /// States how many earlier consumed elements the preview dropped, if any.
    fn omission<S>(&self, failure: FailureBuilder<S>) -> FailureBuilder<S> {
        let shown = self.items.len();
        match self.omitted() {
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

/// The most recently consumed elements, up to a nonzero limit.
struct Tail<Item> {
    limit: usize,
    items: VecDeque<Item>,
    consumed: usize,
}

impl<Item> Tail<Item> {
    fn new(limit: usize) -> Self {
        debug_assert!(limit > 0, "a tail retains at least one element");
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
        self.items.push_back(item);
    }

    /// Trims the retained elements to the preview capacity.
    fn finish(mut self) -> Preview<Item> {
        let remove = self.items.len().saturating_sub(PREVIEW_CAPACITY);
        self.items.drain(..remove);
        Preview {
            items: self.items,
            consumed: self.consumed,
        }
    }
}

/// The overlapping window [`scan_windows`] passes to its check.
type Window<'a, Item> = core::iter::Skip<alloc::collections::vec_deque::Iter<'a, Item>>;

/// Where [`scan_windows`] checks complete windows.
#[derive(Clone, Copy, PartialEq, Eq)]
enum WindowPlacement {
    /// Checks every complete window and stops at the first success.
    Anywhere,
    /// Reads the whole input and checks only the final window.
    End,
}

/// Searches windows of `pattern_len` consecutive elements, retaining at least `retain` elements.
///
/// The check receives the window and the yield index of its first element. Required storage is
/// independent of the diagnostic budget. An empty pattern succeeds without consuming anything.
/// On rejection, the tail holds the latest elements and the number of elements consumed. An input
/// shorter than the pattern is never checked.
fn scan_windows<I: Iterator>(
    iterator: &mut I,
    pattern_len: usize,
    retain: usize,
    placement: WindowPlacement,
    mut check: impl FnMut(Window<'_, I::Item>, usize) -> bool,
) -> Result<(), Tail<I::Item>> {
    if pattern_len == 0 {
        return Ok(());
    }
    let mut tail = Tail::new(pattern_len.max(retain));
    let mut check_window = |tail: &Tail<I::Item>| {
        tail.consumed >= pattern_len
            && check(
                tail.items.iter().skip(tail.items.len() - pattern_len),
                tail.consumed - pattern_len,
            )
    };
    for item in iterator {
        tail.push(item);
        if placement == WindowPlacement::Anywhere && check_window(&tail) {
            return Ok(());
        }
    }
    if placement == WindowPlacement::End && check_window(&tail) {
        return Ok(());
    }
    Err(tail)
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
struct KnownLength {
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
    fn reported_fact<S, R: ValueRenderer<usize>>(
        self,
        failure: FailureBuilder<S>,
        rendering: RenderingContext<'_, R>,
    ) -> FailureBuilder<S> {
        failure.fact(Fact::labelled(
            "Reported length",
            rendering.value(&self.reported),
        ))
    }

    /// Attaches the reported and the expected length.
    fn facts<S, R: ValueRenderer<usize>>(
        self,
        failure: FailureBuilder<S>,
        rendering: RenderingContext<'_, R>,
    ) -> FailureBuilder<S> {
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
