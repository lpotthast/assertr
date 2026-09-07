use super::{
    AssertThat, Borrow, FailureKind, GroupStyle, Mode, PositionReporting, Preview, Reference, Tail,
    ValueRenderer, Vec,
};
use crate::Fact;

#[track_caller]
fn fail_membership<S, T, Item, E: ?Sized, M: Mode, R>(
    this: &AssertThat<'_, S, M, R>,
    preview: &Preview<Item>,
    kind: FailureKind,
    relation: &'static str,
    reference: Reference<'_, E>,
    decisive_index: Option<usize>,
) where
    Item: Borrow<T>,
    R: ValueRenderer<T> + ValueRenderer<E> + ValueRenderer<usize>,
{
    let failure = this
        .failure(kind)
        .actual(preview.rendered::<T, _, _, _>(this))
        .relation(relation);
    let failure = match reference {
        Reference::Expected(expected) => failure.expected(this.render().value(expected)),
        Reference::Unexpected(unexpected) => failure.unexpected(this.render().value(unexpected)),
    };
    preview
        .facts(failure, this.render(), decisive_index)
        .raise();
}

#[track_caller]
pub(crate) fn assert_contains<S, T, E, I, M: Mode, R>(
    this: &AssertThat<'_, S, M, R>,
    iterator: I,
    expected: &E,
) where
    I: Iterator,
    I::Item: Borrow<T>,
    T: PartialEq<E>,
    R: ValueRenderer<T> + ValueRenderer<E> + ValueRenderer<usize>,
{
    let mut tail = Tail::new();
    for item in iterator {
        let matches = crate::matchers::equals(item.borrow(), expected);
        tail.push(item);
        if matches {
            return;
        }
    }
    let preview = tail.finish();
    fail_membership(
        this,
        &preview,
        FailureKind::Membership,
        "does not contain",
        Reference::Expected(expected),
        None,
    );
}

#[track_caller]
pub(crate) fn assert_contains_all<S, T, E, I, M: Mode, R>(
    this: &AssertThat<'_, S, M, R>,
    iterator: I,
    expected: &[E],
) where
    I: Iterator,
    I::Item: Borrow<T>,
    T: PartialEq<E>,
    R: ValueRenderer<T> + ValueRenderer<E> + ValueRenderer<usize>,
{
    if expected.is_empty() {
        return;
    }

    let mut found = alloc::vec![false; expected.len()];
    let mut remaining = expected.len();
    let mut tail = Tail::new();

    for item in iterator {
        for (index, expected) in expected.iter().enumerate() {
            if !found[index] && crate::matchers::equals(item.borrow(), expected) {
                found[index] = true;
                remaining -= 1;
            }
        }
        tail.push(item);
        if remaining == 0 {
            return;
        }
    }

    let not_found = expected
        .iter()
        .zip(found)
        .filter_map(|(expected, found)| (!found).then_some(expected))
        .collect::<Vec<_>>();
    let preview = tail.finish();
    let failure = this
        .failure(FailureKind::Membership)
        .actual(preview.rendered::<T, _, _, _>(this))
        .relation("does not contain all of")
        .expected(
            this.render()
                .borrowed_values::<E, _>(expected, GroupStyle::List),
        )
        .fact(Fact::labelled(
            "Elements not found",
            this.render()
                .borrowed_values::<E, _>(not_found.as_slice(), GroupStyle::List),
        ));
    preview.facts(failure, this.render(), None).raise();
}

#[track_caller]
pub(crate) fn assert_does_not_contain<S, T, E, I, M: Mode, R>(
    this: &AssertThat<'_, S, M, R>,
    iterator: I,
    not_expected: &E,
    positions: PositionReporting,
) where
    I: Iterator,
    I::Item: Borrow<T>,
    T: PartialEq<E>,
    R: ValueRenderer<T> + ValueRenderer<E> + ValueRenderer<usize>,
{
    let mut tail = Tail::new();
    for (index, item) in iterator.enumerate() {
        let matches = crate::matchers::equals(item.borrow(), not_expected);
        tail.push(item);
        if matches {
            let preview = tail.finish();
            fail_membership(
                this,
                &preview,
                FailureKind::Membership,
                "contains",
                Reference::Unexpected(not_expected),
                positions.index(index),
            );
            return;
        }
    }
}
