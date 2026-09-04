use super::{
    AssertThat, AssertionFailure, AssertrPartialEq, Borrow, Capture, FailureBuilder, FailureKind,
    GroupStyle, Mode, PREVIEW_CAPACITY, PositionReporting, Preview, ValueRenderer, Vec,
    exact_size_hint, match_bipartite,
};
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

/// Starts the failure of an unordered exact assertion over the captured elements, with the
/// preview facts and, when the iterator reported a differing length up front, that length.
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
    R: ValueRenderer<T>,
{
    let known_length = captured.known_length;
    let preview = bounded_preview(captured);
    let failure = this
        .failure(kind)
        .actual(preview.rendered::<T, _, _, _>(this))
        .relation(relation);
    let failure = preview.facts(failure, None);
    match known_length {
        Some(actual) => failure
            .fact("Reported length", actual)
            .fact("Expected length", expected_len),
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
    T: AssertrPartialEq<E, R>,
    R: ValueRenderer<T> + ValueRenderer<E>,
{
    let captured = capture_unordered(iterator, expected.len());
    let exact = captured.known_length.is_none()
        && match_bipartite(captured.items.len(), expected.len(), |a, e| {
            AssertrPartialEq::eq(
                captured.items[a].borrow(),
                &expected[e],
                Some(&mut this.eq_context()),
            )
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

#[track_caller]
pub(crate) fn assert_contains_exactly_in_any_order_matching<S, T, P, I, M: Mode, R>(
    this: &AssertThat<'_, S, M, R>,
    iterator: I,
    predicates: &[P],
) where
    I: Iterator,
    I::Item: Borrow<T>,
    P: Fn(&T) -> bool,
    R: ValueRenderer<T>,
{
    let captured = capture_unordered(iterator, predicates.len());
    let exact = captured.known_length.is_none()
        && match_bipartite(captured.items.len(), predicates.len(), |a, p| {
            predicates[p](captured.items[a].borrow())
        })
        .is_exact();
    if !exact {
        unordered_failure::<_, T, _, _, _>(
            this,
            captured,
            FailureKind::Predicate,
            "does not exactly match the predicates in any order",
            predicates.len(),
        )
        .raise();
    }
}

#[track_caller]
pub(crate) fn assert_contains_exactly_in_any_order_satisfying<S, T, A, I, M: Mode, R>(
    this: &AssertThat<'_, S, M, R>,
    iterator: I,
    assertions: &[A],
    positions: PositionReporting,
) where
    I: Iterator,
    I::Item: Borrow<T>,
    A: for<'a> Fn(AssertThat<'a, T, Capture, R>),
    R: ValueRenderer<T> + Clone,
{
    let captured = capture_unordered(iterator, assertions.len());
    // Failures are retained only for the elements the preview will show.
    let preview_start = captured.items.len().saturating_sub(PREVIEW_CAPACITY);
    let mut satisfied = Vec::new();
    let mut retained_failures: Vec<Vec<AssertionFailure>> = Vec::new();
    if captured.known_length.is_none() {
        for (index, item) in captured.items.iter().enumerate() {
            let mut row = Vec::new();
            let mut retained_row = (index >= preview_start).then(Vec::new);
            for assertion in assertions {
                let failures = this.collect_element_failures(item.borrow(), assertion);
                row.push(failures.is_empty());
                if let Some(retained_row) = &mut retained_row {
                    retained_row.extend(failures);
                }
            }
            satisfied.push(row);
            if let Some(retained_row) = retained_row {
                retained_failures.push(retained_row);
            }
        }
    }
    let result = match_bipartite(captured.items.len(), assertions.len(), |a, p| {
        satisfied
            .get(a)
            .and_then(|row| row.get(p))
            .copied()
            .unwrap_or(false)
    });
    if captured.known_length.is_some() || !result.is_exact() {
        let maximum = this.render().max_items();
        let mut unsatisfied = result
            .unmatched_actual
            .iter()
            .copied()
            .filter(|index| *index >= preview_start)
            .filter_map(|index| {
                let failures = core::mem::take(retained_failures.get_mut(index - preview_start)?);
                (!failures.is_empty()).then_some((index, failures))
            })
            .collect::<Vec<_>>();
        let omitted = unsatisfied.len().saturating_sub(maximum);
        unsatisfied.truncate(maximum);
        let children = unsatisfied
            .into_iter()
            .flat_map(|(index, failures)| {
                failures
                    .into_iter()
                    .map(move |failure| positions.locate(failure, index))
            })
            .collect::<Vec<_>>();
        unordered_failure::<_, T, _, _, _>(
            this,
            captured,
            FailureKind::Predicate,
            "does not exactly satisfy the assertions in any order",
            assertions.len(),
        )
        .omitted(omitted, "unsatisfied element")
        .children(children)
        .raise();
    }
}
