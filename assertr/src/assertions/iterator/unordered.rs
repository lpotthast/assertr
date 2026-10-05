use super::{
    AssertThat, AssertionContext, Borrow, ExpectationDiagnostics, FailureBuilder, FailureKind,
    GroupStyle, Items, KnownLength, Mode, PhantomData, Scan, ValueRenderer, buffer_exactly,
    execute,
};
use alloc::boxed::Box;

use crate::{
    AssertionFailure, Expectation, Fact,
    assertions::collection::ContainsExactlyInAnyOrder as CollectionContainsExactlyInAnyOrder,
    borrow_for::BorrowFor,
};

/// Why unordered equality rejected its input.
enum UnorderedRejection {
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
struct ContainsExactlyInAnyOrder<'e, T, E> {
    expected: &'e [E],
    item: PhantomData<fn() -> T>,
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
                    FailureBuilder::detached::<I>(FailureKind::Equality),
                    context,
                )
                .build(),
        );
        Err(UnorderedRejection::Compared {
            report,
            consumed: (items.len() > self.expected.len()).then_some(items.len()),
        })
    }

    const KIND: FailureKind = FailureKind::Equality;
    fn explain<Target>(
        &self,
        rejection: Self::Rejection,
        failure: FailureBuilder<Target>,
        context: &AssertionContext<'_, R>,
    ) -> FailureBuilder<Target> {
        let render = context.render();
        match rejection {
            UnorderedRejection::Reported(known) => known.facts(
                failure
                    .relation("does not contain exactly in any order")
                    .expected(
                        render.borrowed_values::<E::View, _>(self.expected, GroupStyle::List),
                    ),
                render,
            ),
            UnorderedRejection::Compared { report, consumed } => {
                let failure = adopt(failure, *report);
                match consumed {
                    Some(consumed) => {
                        failure.fact(Fact::labelled("Consumed elements", render.value(&consumed)))
                    }
                    None => failure,
                }
            }
        }
    }
}

/// Moves the diagnostic fields of a detached report into the root failure.
fn adopt<Target>(
    failure: FailureBuilder<Target>,
    report: AssertionFailure,
) -> FailureBuilder<Target> {
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

#[track_caller]
pub(crate) fn assert_contains_exactly_in_any_order<S, T, E, I, M: Mode, R>(
    this: &AssertThat<'_, S, M, R>,
    iterator: I,
    expected: &[E],
) where
    I: Iterator,
    I::Item: Borrow<T>,
    T: PartialEq<E::View>,
    E: BorrowFor<T>,
    R: ValueRenderer<T> + ValueRenderer<E::View> + ValueRenderer<usize>,
{
    execute(
        this,
        iterator,
        &ContainsExactlyInAnyOrder::<T, E> {
            expected,
            item: PhantomData,
        },
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prelude::*;
    use core::cell::Cell;

    #[test]
    fn buffers_at_most_one_element_beyond_the_expected_length() {
        for length in [0, 1, 3, 100] {
            let consumed = Cell::new(0);
            let mut iterator = (0..length)
                .inspect(|_| consumed.set(consumed.get() + 1))
                .filter(|_| true);
            let scan = ContainsExactlyInAnyOrder::<i32, i32> {
                expected: &[1, 2],
                item: PhantomData,
            };
            assert_that!(
                scan.observe(&mut iterator, &AssertionContext::default())
                    .is_err()
            )
            .is_true();
            assert_that!(consumed.get()).is_equal_to(length.min(3));
        }
    }
}
