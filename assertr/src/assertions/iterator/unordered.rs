use alloc::boxed::Box;

use super::{
    AssertionContext, Borrow, FailureBuilder, FailureKind, Items, KnownLength, PhantomData,
    RenderingOrder, Scan, ValueRenderer, buffer_exactly, consumed_fact,
};
use crate::{
    assertions::collection::{
        ContainsExactlyInAnyOrder as CollectionContainsExactlyInAnyOrder, elements_are_in_any_order,
    },
    borrow_for::BorrowFor,
    expectation::{Evidence, Expectation},
    failure::AssertionFailure,
    matchers::MatcherList,
};

/// Why unordered equality rejected its input.
pub(crate) enum UnorderedRejection {
    /// An exact size hint ruled out the expected length before consuming anything.
    Reported(KnownLength),
    /// The collection report over the buffered elements. `consumed` is set when the buffer filled
    /// up, because further input may remain unread.
    Compared {
        report: Box<AssertionFailure>,
        consumed: Option<usize>,
    },
}

/// Buffers the input, then delegates to the collection expectation, so both report the same
/// unmatched elements.
pub(crate) struct ContainsExactlyInAnyOrder<'e, T, E> {
    expected: &'e [E],
    item: PhantomData<fn() -> T>,
}

impl<'e, T, E> ContainsExactlyInAnyOrder<'e, T, E> {
    pub(crate) const fn new(expected: &'e [E]) -> Self {
        Self {
            expected,
            item: PhantomData,
        }
    }
}

impl<T, E, I, R> Scan<I, R> for ContainsExactlyInAnyOrder<'_, T, E>
where
    I: Iterator,
    I::Item: Borrow<T>,
    T: PartialEq<E::View>,
    E: BorrowFor<T>,
    R: ValueRenderer<T> + ValueRenderer<E::View> + ValueRenderer<usize>,
{
    type Rejection = UnorderedRejection;

    fn kind(&self) -> FailureKind {
        FailureKind::Equality
    }

    fn observe(
        &self,
        iterator: &mut I,
        context: &AssertionContext<'_, R>,
    ) -> Result<(), Self::Rejection> {
        let items =
            buffer_exactly(iterator, self.expected.len()).map_err(UnorderedRejection::Reported)?;
        let actual = Items::<T, _>::new(&items);
        let expectation = CollectionContainsExactlyInAnyOrder::<E, _>::new(self.expected);
        // The rejection borrows the buffered items, so it is explained before they are released.
        let rejection = match expectation.evaluate(&actual, context) {
            Ok(()) => return Ok(()),
            Err(rejection) => rejection,
        };
        let report = Box::new(
            expectation
                .explain(
                    Some((&actual, rejection)),
                    FailureBuilder::new::<I>(FailureKind::Equality),
                    context,
                )
                .build(),
        );
        Err(UnorderedRejection::Compared {
            report,
            consumed: (items.len() > self.expected.len()).then_some(items.len()),
        })
    }

    fn explain(
        &self,
        rejection: Self::Rejection,
        failure: FailureBuilder,
        context: &AssertionContext<'_, R>,
    ) -> FailureBuilder {
        let render = context.render();
        match rejection {
            UnorderedRejection::Reported(known) => known.facts(
                failure
                    .relation("does not contain exactly in any order")
                    .expected(render.borrowed_values::<E::View, _>(
                        self.expected,
                        RenderingOrder::PreserveIteration,
                    )),
                render,
            ),
            UnorderedRejection::Compared { report, consumed } => {
                let failure = adopt(failure, *report);
                match consumed {
                    Some(consumed) => consumed_fact(failure, context, consumed),
                    None => failure,
                }
            }
        }
    }
}

/// Moves the diagnostic fields of a detached report into the root failure.
fn adopt(failure: FailureBuilder, report: AssertionFailure) -> FailureBuilder {
    let AssertionFailure {
        actual,
        relation,
        expected,
        unexpected,
        facts,
        children,
        omitted_children,
        ..
    } = report;
    let mut failure = failure
        .facts(facts)
        .children(children)
        .omitted_children(omitted_children);
    if let Some(actual) = actual {
        failure = failure.actual(actual);
    }
    if let Some(relation) = relation {
        failure = failure.relation(relation);
    }
    if let Some(expected) = expected {
        failure = failure.expected(expected);
    }
    if let Some(unexpected) = unexpected {
        failure = failure.unexpected(unexpected);
    }
    failure
}

/// Why unordered matching rejected its input.
pub(crate) enum MatchingRejection {
    /// An exact size hint ruled out the expected length before consuming anything.
    Reported(KnownLength),
    Mismatch {
        evidence: Evidence,
        consumed: usize,
    },
}

/// Buffers the input, then evaluates the collection's exact unordered assignment on it.
pub(crate) struct ElementsAreInAnyOrder<T, L> {
    expected: L,
    item: PhantomData<fn() -> T>,
}

impl<T, L> ElementsAreInAnyOrder<T, L> {
    pub(crate) const fn new(expected: L) -> Self {
        Self {
            expected,
            item: PhantomData,
        }
    }
}

impl<T, L, I, R> Scan<I, R> for ElementsAreInAnyOrder<T, L>
where
    I: Iterator,
    I::Item: Borrow<T>,
    L: MatcherList<T, R>,
    R: ValueRenderer<usize>,
{
    type Rejection = MatchingRejection;

    fn kind(&self) -> FailureKind {
        FailureKind::Matching
    }

    fn observe(
        &self,
        iterator: &mut I,
        context: &AssertionContext<'_, R>,
    ) -> Result<(), Self::Rejection> {
        let items =
            buffer_exactly(iterator, self.expected.len()).map_err(MatchingRejection::Reported)?;
        let actual = Items::<T, _>::new(&items);
        let mut child = context.isolated();
        if child.evaluate(&actual, &elements_are_in_any_order(&self.expected)) {
            Ok(())
        } else {
            Err(MatchingRejection::Mismatch {
                evidence: child.into_evidence(),
                consumed: items.len(),
            })
        }
    }

    fn explain(
        &self,
        rejection: Self::Rejection,
        failure: FailureBuilder,
        context: &AssertionContext<'_, R>,
    ) -> FailureBuilder {
        match rejection {
            MatchingRejection::Reported(known) => known.facts(
                failure.relation("does not have the required number of elements"),
                context.render(),
            ),
            MatchingRejection::Mismatch { evidence, consumed } => {
                let failure = failure.relation("does not match exactly in any order");
                evidence.explain(consumed_fact(failure, context, consumed))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use core::cell::Cell;

    use super::*;
    use crate::prelude::*;

    #[test]
    fn buffers_at_most_one_element_beyond_the_expected_length() {
        for length in [0, 1, 3, 100] {
            let consumed = Cell::new(0);
            let mut iterator = (0..length)
                .inspect(|_| consumed.set(consumed.get() + 1))
                .filter(|_| true);
            let scan = ContainsExactlyInAnyOrder::<i32, i32>::new(&[1, 2]);
            assert_that!(
                scan.observe(&mut iterator, &AssertionContext::default())
                    .is_err()
            )
            .is_true();
            assert_that!(consumed.get()).is_equal_to(length.min(3));
        }
    }
}
