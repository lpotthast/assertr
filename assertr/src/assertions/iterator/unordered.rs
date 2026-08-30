use super::{
    AssertThat, AssertrPartialEq, Borrow, Capture, CollectionStyle, Mode, PREVIEW_CAPACITY,
    Preview, String, ValueRenderer, Vec, Write, exact_size_hint, format, join_failures,
    match_bipartite, push_preview_details, writedoc,
};

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

fn push_known_length_detail(
    details: &mut Vec<String>,
    known_length: Option<usize>,
    expected: usize,
) {
    if let Some(actual) = known_length {
        details.push(format!(
            "Iterator reported an exact remaining length of {actual}; expected {expected}."
        ));
    }
}

#[track_caller]
fn fail_unordered<S, T, E, Item, M: Mode, R>(
    this: &AssertThat<'_, S, M, R>,
    mut details: Vec<String>,
    preview: &Preview<Item>,
    expected: &[E],
    summary: &str,
) where
    Item: Borrow<T>,
    R: ValueRenderer<T> + ValueRenderer<E>,
{
    push_preview_details(&mut details, preview, None);
    let actual = this.render_borrowed_values::<T, _>(&preview.items, CollectionStyle::List);
    let expected = this.render_borrowed_values::<E, _>(expected, CollectionStyle::List);
    this.fail_with_details(details, |w: &mut String| {
        writedoc! {w,r"
    Actual: {actual:#?},

    Elements expected: {expected:#?}

    {summary}
"}
    });
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
    let mut details = Vec::new();
    push_known_length_detail(&mut details, captured.known_length, expected.len());
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
        let preview = bounded_preview(captured);
        fail_unordered(
            this,
            details,
            &preview,
            expected,
            "The elements did not match exactly in any order.",
        );
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
    let mut details = Vec::new();
    push_known_length_detail(&mut details, captured.known_length, predicates.len());
    let exact = captured.known_length.is_none()
        && match_bipartite(captured.items.len(), predicates.len(), |a, p| {
            predicates[p](captured.items[a].borrow())
        })
        .is_exact();
    if !exact {
        let preview = bounded_preview(captured);
        push_preview_details(&mut details, &preview, None);
        let actual = this.render_borrowed_values::<T, _>(&preview.items, CollectionStyle::List);
        this.fail_with_details(details, |w: &mut String| {
            writedoc! {w,r"
    Actual: {actual:#?},

    did not exactly match predicates in any order.
"}
        });
    }
}

#[track_caller]
pub(crate) fn assert_contains_exactly_in_any_order_satisfying<S, T, A, I, M: Mode, R>(
    this: &AssertThat<'_, S, M, R>,
    iterator: I,
    assertions: &[A],
) where
    I: Iterator,
    I::Item: Borrow<T>,
    A: for<'a> Fn(AssertThat<'a, T, Capture, R>),
    R: ValueRenderer<T> + Clone,
{
    let captured = capture_unordered(iterator, assertions.len());
    let mut details = Vec::new();
    push_known_length_detail(&mut details, captured.known_length, assertions.len());
    let preview_start = captured.items.len().saturating_sub(PREVIEW_CAPACITY);
    let mut satisfied = Vec::new();
    let mut retained_failures = Vec::new();
    if captured.known_length.is_none() {
        for (index, item) in captured.items.iter().enumerate() {
            let mut row = Vec::new();
            let mut retained_row = (index >= preview_start).then(Vec::new);
            for assertion in assertions {
                let failures = this.collect_element_failures(item.borrow(), assertion);
                row.push(failures.is_empty());
                if let Some(retained_row) = &mut retained_row {
                    retained_row.push(failures);
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
        for index in result
            .unmatched_actual
            .iter()
            .copied()
            .filter(|index| *index >= preview_start)
        {
            if let Some(row) = retained_failures.get(index - preview_start) {
                let joined = row
                    .iter()
                    .filter(|failures| !failures.is_empty())
                    .map(|failures| join_failures(failures))
                    .collect::<Vec<_>>()
                    .join("\n");
                if !joined.is_empty() {
                    details.push(format!(
                        "Element at index {index} did not satisfy any available assertion:\n{joined}"
                    ));
                }
            }
        }
        let preview = bounded_preview(captured);
        push_preview_details(&mut details, &preview, None);
        let actual = this.render_borrowed_values::<T, _>(&preview.items, CollectionStyle::List);
        this.fail_with_details(details, |w: &mut String| {
            writedoc! {w,r"
        Actual: {actual:#?},

        did not exactly satisfy the assertions in any order.
    "}
        });
    }
}
