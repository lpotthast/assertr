use super::{
    AssertThat, AssertionContext, Borrow, FailureBuilder, FailureKind, GroupStyle, Mode,
    PREVIEW_CAPACITY, PhantomData, Preview, Scan, ValueRenderer, Vec, exact_size_hint, execute,
};
use crate::borrow_for::{BorrowFor, borrow_for};
use crate::{Fact, util::matching::matches_exactly};

struct Captured<Item> {
    items: Vec<Item>,
    known_length: Option<usize>,
}

fn capture_unordered<I: Iterator>(iterator: &mut I, expected_len: usize) -> Captured<I::Item> {
    if let Some(actual) = exact_size_hint(&iterator)
        && actual != expected_len
    {
        return Captured {
            items: Vec::new(),
            known_length: Some(actual),
        };
    }
    let mut items = Vec::new();
    for _ in 0..=expected_len {
        if let Some(item) = iterator.next() {
            items.push(item);
        } else {
            break;
        }
    }
    Captured {
        items,
        known_length: None,
    }
}

fn bounded_preview<Item>(mut captured: Captured<Item>) -> Preview<Item> {
    let consumed = captured.items.len();
    if captured.items.len() > PREVIEW_CAPACITY {
        let remove = captured.items.len() - PREVIEW_CAPACITY;
        captured.items.drain(..remove);
    }
    Preview {
        items: captured.items.into(),
        consumed,
    }
}

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
    type Rejection = Captured<I::Item>;
    fn observe(
        &self,
        iterator: &mut I,
        _context: &AssertionContext<'_, R>,
    ) -> Result<(), Self::Rejection> {
        let captured = capture_unordered(iterator, self.expected.len());
        let exact = captured.known_length.is_none()
            && matches_exactly(captured.items.len(), self.expected.len(), |a, e| {
                captured.items[a]
                    .borrow()
                    .eq(borrow_for::<T, _>(&self.expected[e]))
            });
        if exact { Ok(()) } else { Err(captured) }
    }

    const KIND: FailureKind = FailureKind::Equality;
    fn explain<Target>(
        &self,
        rejection: Self::Rejection,
        failure: FailureBuilder<Target>,
        context: &AssertionContext<'_, R>,
    ) -> FailureBuilder<Target> {
        let render = context.render();
        let expected = render.borrowed_values::<E::View, _>(self.expected, GroupStyle::List);
        let captured = rejection;
        let known_length = captured.known_length;
        let preview = bounded_preview(captured);
        let failure = failure
            .actual(preview.rendered::<T, _>(render))
            .relation("does not contain exactly in any order")
            .expected(expected);
        let failure = preview.facts(failure, render, None);
        match known_length {
            Some(actual) => failure
                .fact(Fact::labelled("Reported length", render.value(&actual)))
                .fact(Fact::labelled(
                    "Expected length",
                    render.value(&self.expected.len()),
                )),
            None => failure,
        }
    }
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

    #[derive(Debug)]
    struct Compared<'a>(&'a Cell<usize>);
    impl BorrowFor<Compared<'_>> for i32 {
        type View = i32;
    }
    impl PartialEq<i32> for Compared<'_> {
        fn eq(&self, _: &i32) -> bool {
            self.0.set(self.0.get() + 1);
            true
        }
    }

    #[test]
    fn unequal_buffered_lengths_skip_comparisons_and_keep_the_consumption_limit() {
        for length in [0, 1, 3, 100] {
            let comparisons = Cell::new(0);
            let consumed = Cell::new(0);
            let mut iterator = (0..length)
                .map(|_| {
                    consumed.set(consumed.get() + 1);
                    Compared(&comparisons)
                })
                .filter(|_| true);
            let scan = ContainsExactlyInAnyOrder::<Compared<'_>, i32> {
                expected: &[1, 2],
                item: PhantomData,
            };
            assert_that!(
                scan.observe(&mut iterator, &AssertionContext::default())
                    .is_err()
            )
            .is_true();
            assert_that!(comparisons.get()).is_equal_to(0);
            assert_that!(consumed.get()).is_equal_to(length.min(3));
        }
    }
}
