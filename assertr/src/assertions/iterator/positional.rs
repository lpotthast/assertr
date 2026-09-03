use super::{
    AssertThat, AssertrPartialEq, Borrow, Capture, Mode, PREVIEW_CAPACITY, Preview, String, Tail,
    ValueRenderer, Vec, VecDeque, Write, exact_size_hint, format, join_failures,
    push_preview_details, writedoc,
};

use crate::renderer::{GroupStyle, omission};
enum ExactFailure {
    KnownLength { actual: usize },
    Exhausted { index: usize },
    Criterion { index: usize, failures: Vec<String> },
    Extra { index: usize },
}

fn evaluate_exact<T, I>(
    mut iterator: I,
    expected_len: usize,
    mut criterion: impl FnMut(usize, &T) -> Result<(), Vec<String>>,
) -> Result<(), (Preview<I::Item>, ExactFailure)>
where
    I: Iterator,
    I::Item: Borrow<T>,
{
    if let Some(actual) = exact_size_hint(&iterator)
        && actual != expected_len
    {
        return Err((Tail::new().finish(), ExactFailure::KnownLength { actual }));
    }
    let mut tail = Tail::new();
    for index in 0..expected_len {
        let Some(item) = iterator.next() else {
            return Err((tail.finish(), ExactFailure::Exhausted { index }));
        };
        let result = criterion(index, item.borrow());
        tail.push(item);
        if let Err(failures) = result {
            return Err((tail.finish(), ExactFailure::Criterion { index, failures }));
        }
    }
    if let Some(item) = iterator.next() {
        tail.push(item);
        return Err((
            tail.finish(),
            ExactFailure::Extra {
                index: expected_len,
            },
        ));
    }
    Ok(())
}

fn push_exact_details<Item>(
    details: &mut Vec<String>,
    preview: &Preview<Item>,
    failure: &ExactFailure,
    expected_len: usize,
    maximum: usize,
) {
    let decisive = match failure {
        ExactFailure::Criterion { index, .. } | ExactFailure::Extra { index } => Some(*index),
        _ => None,
    };
    push_preview_details(details, preview, decisive);
    match failure {
        ExactFailure::KnownLength { actual } => details.push(format!(
            "Iterator reported an exact remaining length of {actual}; expected {expected_len}."
        )),
        ExactFailure::Exhausted { index } => details.push(format!(
            "Iterator was exhausted before expected element at index {index}."
        )),
        ExactFailure::Extra { index } => details.push(format!(
            "Iterator produced an unexpected extra element at index {index}."
        )),
        ExactFailure::Criterion { index, failures } => {
            if !failures.is_empty() {
                details.push(format!(
                    "Element at index {index} does not satisfy its assertions:\n{}",
                    join_failures(failures, maximum)
                ));
            }
        }
    }
}

#[track_caller]
pub(crate) fn assert_contains_exactly<S, T, E, I, M: Mode, R>(
    this: &AssertThat<'_, S, M, R>,
    iterator: I,
    expected: &[E],
) where
    I: Iterator,
    I::Item: Borrow<T>,
    T: AssertrPartialEq<E, R>,
    R: ValueRenderer<T> + ValueRenderer<E>,
{
    if let Err((preview, failure)) = evaluate_exact(iterator, expected.len(), |index, item| {
        if AssertrPartialEq::eq(item, &expected[index], Some(&mut this.eq_context())) {
            Ok(())
        } else {
            Err(Vec::new())
        }
    }) {
        let mut details = Vec::new();
        push_exact_details(
            &mut details,
            &preview,
            &failure,
            expected.len(),
            this.render().max_items(),
        );
        let actual = this
            .render()
            .borrowed_values::<T, _>(&preview.items, GroupStyle::List);
        let expected = this
            .render()
            .borrowed_values::<E, _>(expected, GroupStyle::List);
        this.fail_with_details(details, |w: &mut String| {
            writedoc! {w,r"
        Actual: {actual:#?},

        did not exactly match

        Expected: {expected:#?}
    "}
        });
    }
}

#[track_caller]
pub(crate) fn assert_contains_exactly_matching<S, T, P, I, M: Mode, R>(
    this: &AssertThat<'_, S, M, R>,
    iterator: I,
    predicates: &[P],
) where
    I: Iterator,
    I::Item: Borrow<T>,
    P: Fn(&T) -> bool,
    R: ValueRenderer<T>,
{
    if let Err((preview, failure)) = evaluate_exact(iterator, predicates.len(), |index, item| {
        if predicates[index](item) {
            Ok(())
        } else {
            Err(Vec::new())
        }
    }) {
        let mut details = Vec::new();
        push_exact_details(
            &mut details,
            &preview,
            &failure,
            predicates.len(),
            this.render().max_items(),
        );
        let actual = this
            .render()
            .borrowed_values::<T, _>(&preview.items, GroupStyle::List);
        this.fail_with_details(details, |w: &mut String| {
            writedoc! {w,r"
        Actual: {actual:#?},

        did not exactly match predicates.
    "}
        });
    }
}

#[track_caller]
pub(crate) fn assert_contains_exactly_satisfying<S, T, A, I, M: Mode, R>(
    this: &AssertThat<'_, S, M, R>,
    iterator: I,
    assertions: &[A],
) where
    I: Iterator,
    I::Item: Borrow<T>,
    A: for<'a> Fn(AssertThat<'a, T, Capture, R>),
    R: ValueRenderer<T> + Clone,
{
    if let Err((preview, failure)) = evaluate_exact(iterator, assertions.len(), |index, item| {
        let failures = this.collect_element_failures(item, &assertions[index]);
        if failures.is_empty() {
            Ok(())
        } else {
            Err(failures)
        }
    }) {
        let mut details = Vec::new();
        push_exact_details(
            &mut details,
            &preview,
            &failure,
            assertions.len(),
            this.render().max_items(),
        );
        let actual = this
            .render()
            .borrowed_values::<T, _>(&preview.items, GroupStyle::List);
        this.fail_with_details(details, |w: &mut String| {
            writedoc! {w,r"
        Actual: {actual:#?},

        did not exactly satisfy the assertions.
    "}
        });
    }
}

enum PrefixFailure {
    KnownTooShort { actual: usize },
    Exhausted { index: usize },
    Criterion { index: usize, failures: Vec<String> },
}

fn evaluate_prefix<T, I>(
    mut iterator: I,
    expected_len: usize,
    mut criterion: impl FnMut(usize, &T) -> Result<(), Vec<String>>,
) -> Result<(), (Preview<I::Item>, PrefixFailure)>
where
    I: Iterator,
    I::Item: Borrow<T>,
{
    if let Some(actual) = exact_size_hint(&iterator)
        && actual < expected_len
    {
        return Err((
            Tail::new().finish(),
            PrefixFailure::KnownTooShort { actual },
        ));
    }
    let mut tail = Tail::new();
    for index in 0..expected_len {
        let Some(item) = iterator.next() else {
            return Err((tail.finish(), PrefixFailure::Exhausted { index }));
        };
        let result = criterion(index, item.borrow());
        tail.push(item);
        if let Err(failures) = result {
            return Err((tail.finish(), PrefixFailure::Criterion { index, failures }));
        }
    }
    Ok(())
}

fn push_prefix_details<Item>(
    details: &mut Vec<String>,
    preview: &Preview<Item>,
    failure: &PrefixFailure,
    expected_len: usize,
    maximum: usize,
) {
    let decisive = match failure {
        PrefixFailure::Criterion { index, .. } => Some(*index),
        _ => None,
    };
    push_preview_details(details, preview, decisive);
    match failure {
        PrefixFailure::KnownTooShort { actual } => details.push(format!(
            "Iterator reported an exact remaining length of {actual}, shorter than prefix length {expected_len}."
        )),
        PrefixFailure::Exhausted { index } => details.push(format!(
            "Iterator was exhausted before prefix element at index {index}."
        )),
        PrefixFailure::Criterion { index, failures } if !failures.is_empty() => {
            details.push(format!(
                "Element at index {index} does not satisfy its prefix assertions:\n{}",
                join_failures(failures, maximum)
            ));
        }
        PrefixFailure::Criterion { .. } => {}
    }
}

#[track_caller]
pub(crate) fn assert_starts_with<S, T, E, I, M: Mode, R>(
    this: &AssertThat<'_, S, M, R>,
    iterator: I,
    expected: &[E],
) where
    I: Iterator,
    I::Item: Borrow<T>,
    T: AssertrPartialEq<E, R>,
    R: ValueRenderer<T> + ValueRenderer<E>,
{
    if let Err((preview, failure)) = evaluate_prefix(iterator, expected.len(), |index, item| {
        if AssertrPartialEq::eq(item, &expected[index], Some(&mut this.eq_context())) {
            Ok(())
        } else {
            Err(Vec::new())
        }
    }) {
        let mut details = Vec::new();
        push_prefix_details(
            &mut details,
            &preview,
            &failure,
            expected.len(),
            this.render().max_items(),
        );
        let actual = this
            .render()
            .borrowed_values::<T, _>(&preview.items, GroupStyle::List);
        let expected = this
            .render()
            .borrowed_values::<E, _>(expected, GroupStyle::List);
        this.fail_with_details(details, |w: &mut String| {
            writedoc! {w,r"
                Actual: {actual:#?}

                does not start with expected prefix: {expected:#?}
            "}
        });
    }
}

#[track_caller]
pub(crate) fn assert_starts_with_matching<S, T, P, I, M: Mode, R>(
    this: &AssertThat<'_, S, M, R>,
    iterator: I,
    predicates: &[P],
) where
    I: Iterator,
    I::Item: Borrow<T>,
    P: Fn(&T) -> bool,
    R: ValueRenderer<T>,
{
    if let Err((preview, failure)) = evaluate_prefix(iterator, predicates.len(), |index, item| {
        if predicates[index](item) {
            Ok(())
        } else {
            Err(Vec::new())
        }
    }) {
        let mut details = Vec::new();
        push_prefix_details(
            &mut details,
            &preview,
            &failure,
            predicates.len(),
            this.render().max_items(),
        );
        let actual = this
            .render()
            .borrowed_values::<T, _>(&preview.items, GroupStyle::List);
        this.fail_with_details(details, |w: &mut String| {
            writedoc! {w,r"
                Actual: {actual:#?}

                does not start with elements matching the predicates.
            "}
        });
    }
}

#[track_caller]
pub(crate) fn assert_starts_with_satisfying<S, T, A, I, M: Mode, R>(
    this: &AssertThat<'_, S, M, R>,
    iterator: I,
    assertions: &[A],
) where
    I: Iterator,
    I::Item: Borrow<T>,
    A: for<'a> Fn(AssertThat<'a, T, Capture, R>),
    R: ValueRenderer<T> + Clone,
{
    if let Err((preview, failure)) = evaluate_prefix(iterator, assertions.len(), |index, item| {
        let failures = this.collect_element_failures(item, &assertions[index]);
        if failures.is_empty() {
            Ok(())
        } else {
            Err(failures)
        }
    }) {
        let mut details = Vec::new();
        push_prefix_details(
            &mut details,
            &preview,
            &failure,
            assertions.len(),
            this.render().max_items(),
        );
        let actual = this
            .render()
            .borrowed_values::<T, _>(&preview.items, GroupStyle::List);
        this.fail_with_details(details, |w: &mut String| {
            writedoc! {w,r"
                Actual: {actual:#?}

                does not start with elements satisfying the assertions.
            "}
        });
    }
}

fn collect_tail<I: Iterator>(iterator: I, required: usize) -> Preview<I::Item> {
    let capacity = core::cmp::max(required, PREVIEW_CAPACITY);
    let mut items = VecDeque::new();
    let mut consumed = 0;
    for item in iterator {
        consumed += 1;
        if items.len() == capacity {
            let _ = items.pop_front();
        }
        items.push_back(item);
    }
    Preview {
        items: items.into_iter().collect(),
        consumed,
    }
}

fn trim_preview<Item>(preview: &mut Preview<Item>) {
    if preview.items.len() > PREVIEW_CAPACITY {
        let remove = preview.items.len() - PREVIEW_CAPACITY;
        preview.items.drain(..remove);
    }
}

#[track_caller]
pub(crate) fn assert_ends_with<S, T, E, I, M: Mode, R>(
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
    let mut preview = collect_tail(iterator, expected.len());
    let start = preview.items.len().saturating_sub(expected.len());
    let matches = preview.consumed >= expected.len()
        && preview.items[start..]
            .iter()
            .zip(expected)
            .all(|(item, expected)| {
                AssertrPartialEq::eq(item.borrow(), expected, Some(&mut this.eq_context()))
            });
    if !matches {
        trim_preview(&mut preview);
        let mut details = Vec::new();
        push_preview_details(&mut details, &preview, None);
        let actual = this
            .render()
            .borrowed_values::<T, _>(&preview.items, GroupStyle::List);
        let expected = this
            .render()
            .borrowed_values::<E, _>(expected, GroupStyle::List);
        this.fail_with_details(details, |w: &mut String| {
            writedoc! {w,r"
                Actual: {actual:#?}

                does not end with expected suffix: {expected:#?}
            "}
        });
    }
}

#[track_caller]
pub(crate) fn assert_ends_with_matching<S, T, P, I, M: Mode, R>(
    this: &AssertThat<'_, S, M, R>,
    iterator: I,
    predicates: &[P],
) where
    I: Iterator,
    I::Item: Borrow<T>,
    P: Fn(&T) -> bool,
    R: ValueRenderer<T>,
{
    if predicates.is_empty() {
        return;
    }
    let mut preview = collect_tail(iterator, predicates.len());
    let start = preview.items.len().saturating_sub(predicates.len());
    let matches = preview.consumed >= predicates.len()
        && preview.items[start..]
            .iter()
            .zip(predicates)
            .all(|(item, predicate)| predicate(item.borrow()));
    if !matches {
        trim_preview(&mut preview);
        let mut details = Vec::new();
        push_preview_details(&mut details, &preview, None);
        let actual = this
            .render()
            .borrowed_values::<T, _>(&preview.items, GroupStyle::List);
        this.fail_with_details(details, |w: &mut String| {
            writedoc! {w,r"
                Actual: {actual:#?}

                does not end with elements matching the predicates.
            "}
        });
    }
}

#[track_caller]
pub(crate) fn assert_ends_with_satisfying<S, T, A, I, M: Mode, R>(
    this: &AssertThat<'_, S, M, R>,
    iterator: I,
    assertions: &[A],
) where
    I: Iterator,
    I::Item: Borrow<T>,
    A: for<'a> Fn(AssertThat<'a, T, Capture, R>),
    R: ValueRenderer<T> + Clone,
{
    if assertions.is_empty() {
        return;
    }
    let mut preview = collect_tail(iterator, assertions.len());
    let start = preview.items.len().saturating_sub(assertions.len());
    let maximum = this.render().max_items();
    let mut unsatisfied = Vec::new();
    let mut number_of_unsatisfied_elements = 0_usize;
    if preview.consumed >= assertions.len() {
        for (offset, (item, assertion)) in preview.items[start..].iter().zip(assertions).enumerate()
        {
            let failures = this.collect_element_failures(item.borrow(), assertion);
            if !failures.is_empty() {
                number_of_unsatisfied_elements += 1;
                if unsatisfied.len() < maximum {
                    unsatisfied.push((offset, failures));
                }
            }
        }
    }
    let matches = preview.consumed >= assertions.len() && number_of_unsatisfied_elements == 0;
    if !matches {
        let mut details = Vec::new();
        for (offset, failures) in unsatisfied {
            details.push(format!(
                "Suffix element at index {} does not satisfy its assertions:\n{}",
                preview.consumed - assertions.len() + offset,
                join_failures(&failures, maximum)
            ));
        }
        let omitted = number_of_unsatisfied_elements.saturating_sub(maximum);
        if omitted != 0 {
            details.push(omission(omitted, "unsatisfied suffix element"));
        }
        trim_preview(&mut preview);
        push_preview_details(&mut details, &preview, None);
        let actual = this
            .render()
            .borrowed_values::<T, _>(&preview.items, GroupStyle::List);
        this.fail_with_details(details, |w: &mut String| {
            writedoc! {w,r"
                Actual: {actual:#?}

                does not end with elements satisfying the assertions.
            "}
        });
    }
}

fn find_contiguous<T, I>(
    iterator: I,
    pattern_len: usize,
    mut criterion: impl FnMut(&[I::Item]) -> (bool, Vec<String>),
) -> Result<(), (Preview<I::Item>, Vec<String>)>
where
    I: Iterator,
    I::Item: Borrow<T>,
{
    if pattern_len == 0 {
        return Ok(());
    }
    let capacity = core::cmp::max(pattern_len, PREVIEW_CAPACITY);
    let mut window = VecDeque::new();
    let mut consumed = 0;
    let mut last_failures = Vec::new();
    for item in iterator {
        consumed += 1;
        if window.len() == capacity {
            let _ = window.pop_front();
        }
        window.push_back(item);
        if window.len() >= pattern_len {
            let contiguous = window.make_contiguous();
            let start = contiguous.len() - pattern_len;
            let (is_match, failures) = criterion(&contiguous[start..]);
            if is_match {
                return Ok(());
            }
            last_failures = failures;
        }
    }
    let mut preview = Preview {
        items: window.into_iter().collect(),
        consumed,
    };
    trim_preview(&mut preview);
    Err((preview, last_failures))
}

#[track_caller]
pub(crate) fn assert_contains_contiguous<S, T, E, I, M: Mode, R>(
    this: &AssertThat<'_, S, M, R>,
    iterator: I,
    expected: &[E],
) where
    I: Iterator,
    I::Item: Borrow<T>,
    T: AssertrPartialEq<E, R>,
    R: ValueRenderer<T> + ValueRenderer<E>,
{
    if let Err((preview, _)) = find_contiguous::<T, _>(iterator, expected.len(), |window| {
        let matched = window.iter().zip(expected).all(|(item, expected)| {
            AssertrPartialEq::eq(item.borrow(), expected, Some(&mut this.eq_context()))
        });
        (matched, Vec::new())
    }) {
        let mut details = Vec::new();
        push_preview_details(&mut details, &preview, None);
        let actual = this
            .render()
            .borrowed_values::<T, _>(&preview.items, GroupStyle::List);
        let expected = this
            .render()
            .borrowed_values::<E, _>(expected, GroupStyle::List);
        this.fail_with_details(details, |w: &mut String| {
            writedoc! {w,r"
                Actual: {actual:#?}

                does not contain contiguous expected elements: {expected:#?}
            "}
        });
    }
}

#[track_caller]
pub(crate) fn assert_contains_contiguous_matching<S, T, P, I, M: Mode, R>(
    this: &AssertThat<'_, S, M, R>,
    iterator: I,
    predicates: &[P],
) where
    I: Iterator,
    I::Item: Borrow<T>,
    P: Fn(&T) -> bool,
    R: ValueRenderer<T>,
{
    if let Err((preview, _)) = find_contiguous::<T, _>(iterator, predicates.len(), |window| {
        (
            window
                .iter()
                .zip(predicates)
                .all(|(item, predicate)| predicate(item.borrow())),
            Vec::new(),
        )
    }) {
        let mut details = Vec::new();
        push_preview_details(&mut details, &preview, None);
        let actual = this
            .render()
            .borrowed_values::<T, _>(&preview.items, GroupStyle::List);
        this.fail_with_details(details, |w: &mut String| {
            writedoc! {w,r"
                Actual: {actual:#?}

                does not contain contiguous elements matching the predicates.
            "}
        });
    }
}

#[track_caller]
pub(crate) fn assert_contains_contiguous_satisfying<S, T, A, I, M: Mode, R>(
    this: &AssertThat<'_, S, M, R>,
    iterator: I,
    assertions: &[A],
) where
    I: Iterator,
    I::Item: Borrow<T>,
    A: for<'a> Fn(AssertThat<'a, T, Capture, R>),
    R: ValueRenderer<T> + Clone,
{
    if let Err((preview, failures)) =
        find_contiguous::<T, _>(iterator, assertions.len(), |window| {
            let failures = window
                .iter()
                .zip(assertions)
                .map(|(item, assertion)| this.collect_element_failures(item.borrow(), assertion))
                .collect::<Vec<_>>();
            (failures.iter().all(Vec::is_empty), failures.concat())
        })
    {
        let mut details = Vec::new();
        if !failures.is_empty() {
            details.push(format!(
                "The final contiguous candidate did not satisfy the assertions:\n{}",
                join_failures(&failures, this.render().max_items())
            ));
        }
        push_preview_details(&mut details, &preview, None);
        let actual = this
            .render()
            .borrowed_values::<T, _>(&preview.items, GroupStyle::List);
        this.fail_with_details(details, |w: &mut String| {
            writedoc! {w,r"
                Actual: {actual:#?}

                does not contain contiguous elements satisfying the assertions.
            "}
        });
    }
}
