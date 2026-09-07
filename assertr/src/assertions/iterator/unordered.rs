use super::{
    AssertThat, Borrow, FailureBuilder, FailureKind, GroupStyle, Mode, PREVIEW_CAPACITY, Preview,
    ValueRenderer, Vec, exact_size_hint, match_bipartite,
};
use crate::Fact;
use crate::failure::Attached;

struct Captured<Item> {
    items: Vec<Item>,
    known_length: Option<usize>,
}

fn capture_unordered<I: Iterator>(mut iterator: I, expected_len: usize) -> Captured<I::Item> {
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

/// Starts the failure of an unordered exact assertion over the captured elements, with the preview
/// facts and, when the iterator reported a differing length up front, that length.
#[track_caller]
fn unordered_failure<'c, S, T, Item, M: Mode, R>(
    this: &'c AssertThat<'_, S, M, R>,
    captured: Captured<Item>,
    kind: FailureKind,
    relation: &'static str,
    expected_len: usize,
) -> FailureBuilder<Attached<'c>>
where
    Item: Borrow<T>,
    R: ValueRenderer<T> + ValueRenderer<usize>,
{
    let known_length = captured.known_length;
    let preview = bounded_preview(captured);
    let failure = this
        .failure(kind)
        .actual(preview.rendered::<T, _, _, _>(this))
        .relation(relation);
    let failure = preview.facts(failure, this.render(), None);
    match known_length {
        Some(actual) => failure
            .fact(Fact::labelled(
                "Reported length",
                this.render().value(&actual),
            ))
            .fact(Fact::labelled(
                "Expected length",
                this.render().value(&expected_len),
            )),
        None => failure,
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
    let captured = capture_unordered(iterator, expected.len());
    let exact = captured.known_length.is_none()
        && match_bipartite(captured.items.len(), expected.len(), |a, e| {
            crate::matchers::equals(captured.items[a].borrow(), &expected[e])
        })
        .is_exact();
    if !exact {
        unordered_failure::<_, T, _, _, _>(
            this,
            captured,
            FailureKind::Equality,
            "does not contain exactly in any order",
            expected.len(),
        )
        .expected(
            this.render()
                .borrowed_values::<E, _>(expected, GroupStyle::List),
        )
        .raise();
    }
}
