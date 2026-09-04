use super::{
    AssertThat, AssertionFailure, AssertrPartialEq, Borrow, Capture, FailureKind, GroupStyle, Mode,
    PREVIEW_CAPACITY, PositionReporting, Preview, Reference, Tail, ValueRenderer, Vec, VecDeque,
};

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
    R: ValueRenderer<T> + ValueRenderer<E>,
{
    let failure = this
        .failure(kind)
        .actual(preview.rendered::<T, _, _, _>(this))
        .relation(relation);
    let failure = match reference {
        Reference::Expected(expected) => failure.expected(this.render().value(expected)),
        Reference::Unexpected(unexpected) => failure.unexpected(this.render().value(unexpected)),
    };
    preview.facts(failure, decisive_index).raise();
}

#[track_caller]
pub(crate) fn assert_contains<S, T, E, I, M: Mode, R>(
    this: &AssertThat<'_, S, M, R>,
    iterator: I,
    expected: &E,
) where
    I: Iterator,
    I::Item: Borrow<T>,
    T: AssertrPartialEq<E, R>,
    R: ValueRenderer<T> + ValueRenderer<E>,
{
    let mut tail = Tail::new();
    for item in iterator {
        let matches = AssertrPartialEq::eq(item.borrow(), expected, Some(&mut this.eq_context()));
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
    T: AssertrPartialEq<E, R>,
    R: ValueRenderer<T> + ValueRenderer<E>,
{
    if expected.is_empty() {
        return;
    }

    let mut found = alloc::vec![false; expected.len()];
    let mut remaining = expected.len();
    let mut tail = Tail::new();

    for item in iterator {
        for (index, expected) in expected.iter().enumerate() {
            if !found[index]
                && AssertrPartialEq::eq(item.borrow(), expected, Some(&mut this.eq_context()))
            {
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
        .fact(
            "Elements not found",
            format_args!(
                "{:#?}",
                this.render()
                    .borrowed_values::<E, _>(not_found.as_slice(), GroupStyle::List)
            ),
        );
    preview.facts(failure, None).raise();
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
    T: AssertrPartialEq<E, R>,
    R: ValueRenderer<T> + ValueRenderer<E>,
{
    let mut tail = Tail::new();
    for (index, item) in iterator.enumerate() {
        let matches =
            AssertrPartialEq::eq(item.borrow(), not_expected, Some(&mut this.eq_context()));
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

#[track_caller]
fn fail_predicate_membership<S, T, Item, M: Mode, R>(
    this: &AssertThat<'_, S, M, R>,
    preview: &Preview<Item>,
    kind: FailureKind,
    relation: &'static str,
    decisive_index: Option<usize>,
) where
    Item: Borrow<T>,
    R: ValueRenderer<T>,
{
    let failure = this
        .failure(kind)
        .actual(preview.rendered::<T, _, _, _>(this))
        .relation(relation);
    preview.facts(failure, decisive_index).raise();
}

#[track_caller]
pub(crate) fn assert_contains_matching<S, T, P, I, M: Mode, R>(
    this: &AssertThat<'_, S, M, R>,
    iterator: I,
    predicate: &P,
) where
    I: Iterator,
    I::Item: Borrow<T>,
    P: Fn(&T) -> bool,
    R: ValueRenderer<T>,
{
    let mut tail = Tail::new();
    for item in iterator {
        let matches = predicate(item.borrow());
        tail.push(item);
        if matches {
            return;
        }
    }
    let preview = tail.finish();
    fail_predicate_membership(
        this,
        &preview,
        FailureKind::Predicate,
        "does not contain an element matching the predicate",
        None,
    );
}

#[track_caller]
pub(crate) fn assert_does_not_contain_matching<S, T, P, I, M: Mode, R>(
    this: &AssertThat<'_, S, M, R>,
    iterator: I,
    predicate: &P,
    positions: PositionReporting,
) where
    I: Iterator,
    I::Item: Borrow<T>,
    P: Fn(&T) -> bool,
    R: ValueRenderer<T>,
{
    let mut tail = Tail::new();
    for (index, item) in iterator.enumerate() {
        let matches = predicate(item.borrow());
        tail.push(item);
        if matches {
            let preview = tail.finish();
            fail_predicate_membership(
                this,
                &preview,
                FailureKind::Predicate,
                "contains an element matching the predicate",
                positions.index(index),
            );
            return;
        }
    }
}

/// Raises a `_satisfying` membership failure.
///
/// `unsatisfied` holds the failures of the last retained elements, in yield order, so that the
/// element at offset `n` is the element at index `preview.start_index() + n`. They become the
/// failure's children, tagged with their index when `positions` reports it. At most the
/// rendering budget's item count of elements is kept.
#[track_caller]
fn fail_satisfying_membership<S, T, Item, M: Mode, R>(
    this: &AssertThat<'_, S, M, R>,
    preview: &Preview<Item>,
    kind: FailureKind,
    relation: &'static str,
    positions: PositionReporting,
    decisive_index: Option<usize>,
    unsatisfied: VecDeque<Vec<AssertionFailure>>,
) where
    Item: Borrow<T>,
    R: ValueRenderer<T>,
{
    let maximum = this.render().max_items();
    let rendered = unsatisfied.len().min(maximum);
    let omitted = if unsatisfied.is_empty() {
        0
    } else {
        preview.consumed.saturating_sub(rendered)
    };
    let start = preview.start_index();
    let children =
        unsatisfied
            .into_iter()
            .take(maximum)
            .enumerate()
            .flat_map(|(offset, failures)| {
                failures
                    .into_iter()
                    .map(move |failure| positions.locate(failure, start + offset))
            });

    let failure = this
        .failure(kind)
        .actual(preview.rendered::<T, _, _, _>(this))
        .relation(relation);
    preview
        .facts(failure, decisive_index)
        .omitted(omitted, "unsatisfied element")
        .children(children)
        .raise();
}

#[track_caller]
pub(crate) fn assert_contains_satisfying<S, T, A, I, M: Mode, R>(
    this: &AssertThat<'_, S, M, R>,
    iterator: I,
    assertions: &A,
    positions: PositionReporting,
) where
    I: Iterator,
    I::Item: Borrow<T>,
    A: for<'a> Fn(AssertThat<'a, T, Capture, R>),
    R: ValueRenderer<T> + Clone,
{
    let mut tail = Tail::new();
    let mut retained = VecDeque::new();
    for item in iterator {
        let failures = this.collect_element_failures(item.borrow(), assertions);
        tail.push(item);
        if failures.is_empty() {
            return;
        }
        if retained.len() == PREVIEW_CAPACITY {
            let _ = retained.pop_front();
        }
        retained.push_back(failures);
    }
    let preview = tail.finish();
    fail_satisfying_membership(
        this,
        &preview,
        FailureKind::Predicate,
        "does not contain an element satisfying the assertions",
        positions,
        None,
        retained,
    );
}

#[track_caller]
pub(crate) fn assert_does_not_contain_satisfying<S, T, A, I, M: Mode, R>(
    this: &AssertThat<'_, S, M, R>,
    iterator: I,
    assertions: &A,
    positions: PositionReporting,
) where
    I: Iterator,
    I::Item: Borrow<T>,
    A: for<'a> Fn(AssertThat<'a, T, Capture, R>),
    R: ValueRenderer<T> + Clone,
{
    let mut tail = Tail::new();
    for (index, item) in iterator.enumerate() {
        let matches = this
            .collect_element_failures(item.borrow(), assertions)
            .is_empty();
        tail.push(item);
        if matches {
            let preview = tail.finish();
            fail_satisfying_membership(
                this,
                &preview,
                FailureKind::Predicate,
                "contains an element satisfying the assertions",
                positions,
                positions.index(index),
                VecDeque::new(),
            );
            return;
        }
    }
}
