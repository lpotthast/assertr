use super::{
    AssertThat, AssertrPartialEq, Borrow, Capture, CollectionStyle, Mode, PREVIEW_CAPACITY,
    Preview, String, Tail, ValueRenderer, Vec, VecDeque, Write, format, join_failures,
    push_preview_details, writedoc,
};

#[track_caller]
fn fail_membership<S, T, Item, E: ?Sized, M: Mode, R>(
    this: &AssertThat<'_, S, M, R>,
    preview: &Preview<Item>,
    expected: &E,
    positive: bool,
    decisive_index: Option<usize>,
) where
    Item: Borrow<T>,
    R: ValueRenderer<T> + ValueRenderer<E>,
{
    let mut details = Vec::new();
    push_preview_details(&mut details, preview, decisive_index);
    let actual = this.render_borrowed_values::<T, _>(&preview.items, CollectionStyle::List);
    let expected = this.render_value(expected);
    this.fail_with_details(details, |w: &mut String| {
        if positive {
            writedoc! {w, r"
                Actual: {actual:#?}

                does not contain expected: {expected:#?}
            "}
        } else {
            writedoc! {w, r"
                Actual: {actual:#?}

                contains unexpected: {expected:#?}
            "}
        }
    });
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
    fail_membership(this, &preview, expected, true, None);
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
    let mut details = Vec::new();
    push_preview_details(&mut details, &preview, None);
    let actual = this.render_borrowed_values::<T, _>(&preview.items, CollectionStyle::List);
    let expected = this.render_borrowed_values::<E, _>(expected, CollectionStyle::List);
    let not_found = this.render_values(not_found.as_slice(), CollectionStyle::List);
    this.fail_with_details(details, |w: &mut String| {
        writedoc! {w, r"
            Actual: {actual:#?}

            does not contain all expected elements

            Expected: {expected:#?}

            Elements not found: {not_found:#?}
        "}
    });
}

#[track_caller]
pub(crate) fn assert_does_not_contain<S, T, E, I, M: Mode, R>(
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
    for (index, item) in iterator.enumerate() {
        let matches = AssertrPartialEq::eq(item.borrow(), expected, Some(&mut this.eq_context()));
        tail.push(item);
        if matches {
            let preview = tail.finish();
            fail_membership(this, &preview, expected, false, Some(index));
            return;
        }
    }
}

#[track_caller]
fn fail_predicate_membership<S, T, Item, M: Mode, R>(
    this: &AssertThat<'_, S, M, R>,
    preview: &Preview<Item>,
    positive: bool,
    decisive_index: Option<usize>,
) where
    Item: Borrow<T>,
    R: ValueRenderer<T>,
{
    let mut details = Vec::new();
    push_preview_details(&mut details, preview, decisive_index);
    let actual = this.render_borrowed_values::<T, _>(&preview.items, CollectionStyle::List);
    this.fail_with_details(details, |w: &mut String| {
        if positive {
            writedoc! {w, r"
            Actual: {actual:#?}

            does not contain an element matching the predicate.
        "}
        } else {
            writedoc! {w, r"
            Actual: {actual:#?}

            unexpectedly contains an element matching the predicate.
        "}
        }
    });
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
    fail_predicate_membership(this, &preview, true, None);
}

#[track_caller]
pub(crate) fn assert_does_not_contain_matching<S, T, P, I, M: Mode, R>(
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
    for (index, item) in iterator.enumerate() {
        let matches = predicate(item.borrow());
        tail.push(item);
        if matches {
            let preview = tail.finish();
            fail_predicate_membership(this, &preview, false, Some(index));
            return;
        }
    }
}

#[track_caller]
fn fail_satisfying_membership<S, T, Item, M: Mode, R>(
    this: &AssertThat<'_, S, M, R>,
    preview: &Preview<Item>,
    positive: bool,
    decisive_index: Option<usize>,
    failures: &VecDeque<Vec<String>>,
) where
    Item: Borrow<T>,
    R: ValueRenderer<T>,
{
    let mut details = Vec::new();
    push_preview_details(&mut details, preview, decisive_index);
    if positive {
        for (offset, failures) in failures.iter().enumerate() {
            details.push(format!(
                "Element at index {} does not satisfy the assertions:\n{}",
                preview.start_index() + offset,
                join_failures(failures)
            ));
        }
    }
    let actual = this.render_borrowed_values::<T, _>(&preview.items, CollectionStyle::List);
    this.fail_with_details(details, |w: &mut String| {
        if positive {
            writedoc! {w,r"
        Actual: {actual:#?}

        does not contain an element satisfying the assertions.
    "}
        } else {
            writedoc! {w,r"
        Actual: {actual:#?}

        unexpectedly contains an element satisfying the assertions.
    "}
        }
    });
}

#[track_caller]
pub(crate) fn assert_contains_satisfying<S, T, A, I, M: Mode, R>(
    this: &AssertThat<'_, S, M, R>,
    iterator: I,
    assertions: &A,
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
    fail_satisfying_membership(this, &preview, true, None, &retained);
}

#[track_caller]
pub(crate) fn assert_does_not_contain_satisfying<S, T, A, I, M: Mode, R>(
    this: &AssertThat<'_, S, M, R>,
    iterator: I,
    assertions: &A,
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
            fail_satisfying_membership(this, &preview, false, Some(index), &VecDeque::new());
            return;
        }
    }
}
