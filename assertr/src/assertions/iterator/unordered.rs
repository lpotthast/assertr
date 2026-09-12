use super::{
    AssertThat, AssertionContext, Borrow, EqualToRef, Expectation, FailureBuilder, FailureKind,
    GroupStyle, Mode, PREVIEW_CAPACITY, PhantomData, Preview, Scan, ValueRenderer, Vec,
    exact_size_hint, execute, match_bipartite,
};
use crate::Fact;

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
        items: captured.items,
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
    T: PartialEq<E>,
    R: ValueRenderer<T> + ValueRenderer<E> + ValueRenderer<usize>,
{
    type Rejection = Captured<I::Item>;
    fn observe(
        &self,
        iterator: &mut I,
        context: &AssertionContext<'_, R>,
    ) -> Result<(), Self::Rejection> {
        let captured = capture_unordered(iterator, self.expected.len());
        let exact = captured.known_length.is_none()
            && match_bipartite(captured.items.len(), self.expected.len(), |a, e| {
                EqualToRef(&self.expected[e])
                    .evaluate(captured.items[a].borrow(), context)
                    .is_ok()
            })
            .is_exact();
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
        let expected = render.borrowed_values::<E, _>(self.expected, GroupStyle::List);
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
    T: PartialEq<E>,
    R: ValueRenderer<T> + ValueRenderer<E> + ValueRenderer<usize>,
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
