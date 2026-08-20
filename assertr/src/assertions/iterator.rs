//! Shared streaming implementation for direct and borrowed iterator assertions.
//!
//! Per-failure diagnostics are accumulated in local `Vec<String>` buffers and handed to
//! [`AssertThat::fail_with_details`], never stored on the assertion itself.

use alloc::{collections::VecDeque, format, string::String, vec::Vec};
use core::{borrow::Borrow, fmt::Write};
use indoc::writedoc;

use crate::{
    AssertThat, AssertionRenderer, AssertrPartialEq, Mode,
    mode::Capture,
    util::failure::join_failures,
    util::slice::{match_bipartite, match_multiset},
};

const PREVIEW_CAPACITY: usize = 16;

struct Preview<Item> {
    items: Vec<Item>,
    consumed: usize,
}

impl<Item> Preview<Item> {
    fn omitted(&self) -> usize {
        self.consumed.saturating_sub(self.items.len())
    }
    fn start_index(&self) -> usize {
        self.omitted()
    }
}

struct Tail<Item> {
    items: VecDeque<Item>,
    consumed: usize,
}

impl<Item> Tail<Item> {
    fn new() -> Self {
        Self {
            items: VecDeque::new(),
            consumed: 0,
        }
    }
    fn push(&mut self, item: Item) {
        self.consumed += 1;
        if self.items.len() == PREVIEW_CAPACITY {
            let _ = self.items.pop_front();
        }
        self.items.push_back(item);
    }
    fn finish(self) -> Preview<Item> {
        Preview {
            items: self.items.into_iter().collect(),
            consumed: self.consumed,
        }
    }
}

fn exact_size_hint<I: Iterator>(iterator: &I) -> Option<usize> {
    let (lower, upper) = iterator.size_hint();
    (upper == Some(lower)).then_some(lower)
}

fn push_preview_details<Item>(
    details: &mut Vec<String>,
    preview: &Preview<Item>,
    decisive_index: Option<usize>,
) {
    details.push(format!("Consumed {} element(s).", preview.consumed));
    if preview.omitted() > 0 {
        details.push(format!(
            "Actual preview contains the last {} consumed elements; {} earlier element(s) omitted.",
            preview.items.len(),
            preview.omitted()
        ));
    }
    if let Some(index) = decisive_index {
        details.push(format!("Decisive element is at zero-based index {index}."));
    }
}

#[track_caller]
fn fail_membership<S, Item, E: ?Sized, M: Mode, R>(
    this: &AssertThat<'_, S, M, R>,
    preview: &Preview<Item>,
    expected: &E,
    positive: bool,
    decisive_index: Option<usize>,
) where
    R: AssertionRenderer<Vec<Item>> + AssertionRenderer<E>,
{
    let mut details = Vec::new();
    push_preview_details(&mut details, preview, decisive_index);
    let actual = this.render_value(&preview.items);
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
    R: AssertionRenderer<Vec<I::Item>> + AssertionRenderer<E>,
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
pub(crate) fn assert_does_not_contain<S, T, E, I, M: Mode, R>(
    this: &AssertThat<'_, S, M, R>,
    iterator: I,
    expected: &E,
) where
    I: Iterator,
    I::Item: Borrow<T>,
    T: AssertrPartialEq<E, R>,
    R: AssertionRenderer<Vec<I::Item>> + AssertionRenderer<E>,
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
fn fail_predicate_membership<S, Item, M: Mode, R>(
    this: &AssertThat<'_, S, M, R>,
    preview: &Preview<Item>,
    positive: bool,
    decisive_index: Option<usize>,
) where
    R: AssertionRenderer<Vec<Item>>,
{
    let mut details = Vec::new();
    push_preview_details(&mut details, preview, decisive_index);
    let actual = this.render_value(&preview.items);
    this.fail_with_details(details, |w: &mut String| {
        if positive {
            writedoc! {w, r"
            Actual: {actual:#?}

            does not contain an element matching the predicate.
        "}
        } else {
            writedoc! {w, r"
            Actual: {actual:#?}

            contains an element matching the predicate.
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
    R: AssertionRenderer<Vec<I::Item>>,
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
    R: AssertionRenderer<Vec<I::Item>>,
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
fn fail_satisfying_membership<S, Item, M: Mode, R>(
    this: &AssertThat<'_, S, M, R>,
    preview: &Preview<Item>,
    positive: bool,
    decisive_index: Option<usize>,
    failures: &VecDeque<Vec<String>>,
) where
    R: AssertionRenderer<Vec<Item>>,
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
    let actual = this.render_value(&preview.items);
    this.fail_with_details(details, |w: &mut String| {
        if positive {
            writedoc! {w,r"
        Actual: {actual:#?}

        does not contain an element satisfying the assertions.
    "}
        } else {
            writedoc! {w,r"
        Actual: {actual:#?}

        contains an element satisfying the assertions.
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
    R: AssertionRenderer<Vec<I::Item>> + Clone,
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
    R: AssertionRenderer<Vec<I::Item>> + Clone,
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
                    join_failures(failures)
                ));
            }
        }
    }
}

#[track_caller]
pub(crate) fn assert_contains_exactly<S, T, E, I, ER: ?Sized, M: Mode, R>(
    this: &AssertThat<'_, S, M, R>,
    iterator: I,
    expected: &[E],
    rendered_expected: &ER,
) where
    I: Iterator,
    I::Item: Borrow<T>,
    T: AssertrPartialEq<E, R>,
    R: AssertionRenderer<Vec<I::Item>> + AssertionRenderer<ER>,
{
    if let Err((preview, failure)) = evaluate_exact(iterator, expected.len(), |index, item| {
        if AssertrPartialEq::eq(item, &expected[index], Some(&mut this.eq_context())) {
            Ok(())
        } else {
            Err(Vec::new())
        }
    }) {
        let mut details = Vec::new();
        push_exact_details(&mut details, &preview, &failure, expected.len());
        let actual = this.render_value(&preview.items);
        let expected = this.render_value(rendered_expected);
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
    R: AssertionRenderer<Vec<I::Item>>,
{
    if let Err((preview, failure)) = evaluate_exact(iterator, predicates.len(), |index, item| {
        if predicates[index](item) {
            Ok(())
        } else {
            Err(Vec::new())
        }
    }) {
        let mut details = Vec::new();
        push_exact_details(&mut details, &preview, &failure, predicates.len());
        let actual = this.render_value(&preview.items);
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
    R: AssertionRenderer<Vec<I::Item>> + Clone,
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
        push_exact_details(&mut details, &preview, &failure, assertions.len());
        let actual = this.render_value(&preview.items);
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
                join_failures(failures)
            ));
        }
        PrefixFailure::Criterion { .. } => {}
    }
}

#[track_caller]
pub(crate) fn assert_starts_with<S, T, E, I, ER: ?Sized, M: Mode, R>(
    this: &AssertThat<'_, S, M, R>,
    iterator: I,
    expected: &[E],
    rendered_expected: &ER,
) where
    I: Iterator,
    I::Item: Borrow<T>,
    T: AssertrPartialEq<E, R>,
    R: AssertionRenderer<Vec<I::Item>> + AssertionRenderer<ER>,
{
    if let Err((preview, failure)) = evaluate_prefix(iterator, expected.len(), |index, item| {
        if AssertrPartialEq::eq(item, &expected[index], Some(&mut this.eq_context())) {
            Ok(())
        } else {
            Err(Vec::new())
        }
    }) {
        let mut details = Vec::new();
        push_prefix_details(&mut details, &preview, &failure, expected.len());
        let actual = this.render_value(&preview.items);
        let expected = this.render_value(rendered_expected);
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
    R: AssertionRenderer<Vec<I::Item>>,
{
    if let Err((preview, failure)) = evaluate_prefix(iterator, predicates.len(), |index, item| {
        if predicates[index](item) {
            Ok(())
        } else {
            Err(Vec::new())
        }
    }) {
        let mut details = Vec::new();
        push_prefix_details(&mut details, &preview, &failure, predicates.len());
        let actual = this.render_value(&preview.items);
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
    R: AssertionRenderer<Vec<I::Item>> + Clone,
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
        push_prefix_details(&mut details, &preview, &failure, assertions.len());
        let actual = this.render_value(&preview.items);
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
pub(crate) fn assert_ends_with<S, T, E, I, ER: ?Sized, M: Mode, R>(
    this: &AssertThat<'_, S, M, R>,
    iterator: I,
    expected: &[E],
    rendered_expected: &ER,
) where
    I: Iterator,
    I::Item: Borrow<T>,
    T: AssertrPartialEq<E, R>,
    R: AssertionRenderer<Vec<I::Item>> + AssertionRenderer<ER>,
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
        let actual = this.render_value(&preview.items);
        let expected = this.render_value(rendered_expected);
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
    R: AssertionRenderer<Vec<I::Item>>,
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
        let actual = this.render_value(&preview.items);
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
    R: AssertionRenderer<Vec<I::Item>> + Clone,
{
    if assertions.is_empty() {
        return;
    }
    let mut preview = collect_tail(iterator, assertions.len());
    let start = preview.items.len().saturating_sub(assertions.len());
    let mut failures = Vec::new();
    if preview.consumed >= assertions.len() {
        for (item, assertion) in preview.items[start..].iter().zip(assertions) {
            failures.push(this.collect_element_failures(item.borrow(), assertion));
        }
    }
    let matches = preview.consumed >= assertions.len() && failures.iter().all(Vec::is_empty);
    if !matches {
        let mut details = Vec::new();
        for (offset, failures) in failures.iter().enumerate() {
            if !failures.is_empty() {
                details.push(format!(
                    "Suffix element at index {} does not satisfy its assertions:\n{}",
                    preview.consumed - assertions.len() + offset,
                    join_failures(failures)
                ));
            }
        }
        trim_preview(&mut preview);
        push_preview_details(&mut details, &preview, None);
        let actual = this.render_value(&preview.items);
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
pub(crate) fn assert_contains_contiguous<S, T, E, I, ER: ?Sized, M: Mode, R>(
    this: &AssertThat<'_, S, M, R>,
    iterator: I,
    expected: &[E],
    rendered_expected: &ER,
) where
    I: Iterator,
    I::Item: Borrow<T>,
    T: AssertrPartialEq<E, R>,
    R: AssertionRenderer<Vec<I::Item>> + AssertionRenderer<ER>,
{
    if let Err((preview, _)) = find_contiguous::<T, _>(iterator, expected.len(), |window| {
        let matched = window.iter().zip(expected).all(|(item, expected)| {
            AssertrPartialEq::eq(item.borrow(), expected, Some(&mut this.eq_context()))
        });
        (matched, Vec::new())
    }) {
        let mut details = Vec::new();
        push_preview_details(&mut details, &preview, None);
        let actual = this.render_value(&preview.items);
        let expected = this.render_value(rendered_expected);
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
    R: AssertionRenderer<Vec<I::Item>>,
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
        let actual = this.render_value(&preview.items);
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
    R: AssertionRenderer<Vec<I::Item>> + Clone,
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
                join_failures(&failures)
            ));
        }
        push_preview_details(&mut details, &preview, None);
        let actual = this.render_value(&preview.items);
        this.fail_with_details(details, |w: &mut String| {
            writedoc! {w,r"
                Actual: {actual:#?}

                does not contain contiguous elements satisfying the assertions.
            "}
        });
    }
}

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
fn fail_unordered<S, Item, ER: ?Sized, M: Mode, R>(
    this: &AssertThat<'_, S, M, R>,
    mut details: Vec<String>,
    preview: &Preview<Item>,
    expected: &ER,
    summary: &str,
) where
    R: AssertionRenderer<Vec<Item>> + AssertionRenderer<ER>,
{
    push_preview_details(&mut details, preview, None);
    let actual = this.render_value(&preview.items);
    let expected = this.render_value(expected);
    this.fail_with_details(details, |w: &mut String| {
        writedoc! {w,r"
    Actual: {actual:#?},

    Elements expected: {expected:#?}

    {summary}
"}
    });
}

#[track_caller]
pub(crate) fn assert_contains_exactly_in_any_order<S, T, I, ER: ?Sized, M: Mode, R>(
    this: &AssertThat<'_, S, M, R>,
    iterator: I,
    expected: &[T],
    rendered_expected: &ER,
) where
    I: Iterator,
    I::Item: Borrow<T>,
    T: PartialEq,
    R: AssertionRenderer<Vec<I::Item>> + AssertionRenderer<ER>,
{
    let captured = capture_unordered(iterator, expected.len());
    let mut details = Vec::new();
    push_known_length_detail(&mut details, captured.known_length, expected.len());
    let exact = captured.known_length.is_none()
        && match_multiset(captured.items.len(), expected.len(), |a, e| {
            captured.items[a].borrow() == &expected[e]
        })
        .is_exact();
    if !exact {
        let preview = bounded_preview(captured);
        fail_unordered(
            this,
            details,
            &preview,
            rendered_expected,
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
    R: AssertionRenderer<Vec<I::Item>>,
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
        let actual = this.render_value(&preview.items);
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
    R: AssertionRenderer<Vec<I::Item>> + Clone,
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
        let actual = this.render_value(&preview.items);
        this.fail_with_details(details, |w: &mut String| {
            writedoc! {w,r"
        Actual: {actual:#?},

        did not exactly satisfy the assertions in any order.
    "}
        });
    }
}

#[track_caller]
pub(crate) fn assert_is_empty<S, T, I, M: Mode, R>(this: &AssertThat<'_, S, M, R>, mut iterator: I)
where
    I: Iterator,
    I::Item: Borrow<T>,
    R: AssertionRenderer<Vec<I::Item>>,
{
    if let Some(item) = iterator.next() {
        let preview = Preview {
            items: alloc::vec![item],
            consumed: 1,
        };
        let mut details = Vec::new();
        push_preview_details(&mut details, &preview, Some(0));
        let actual = this.render_value(&preview.items);
        this.fail_with_details(details, |w: &mut String| {
            writedoc! {w,r"
    Actual: {actual:#?}

    is not empty.
"}
        });
    }
}

#[track_caller]
pub(crate) fn assert_is_not_empty<S, T, I, M: Mode, R>(
    this: &AssertThat<'_, S, M, R>,
    mut iterator: I,
) where
    I: Iterator,
    I::Item: Borrow<T>,
    R: AssertionRenderer<Vec<I::Item>>,
{
    if iterator.next().is_none() {
        let preview: Vec<I::Item> = Vec::new();
        let actual = this.render_value(&preview);
        this.fail(|w: &mut String| {
            writedoc! {w,r"
    Actual: {actual:#?}

    is empty.
"}
        });
    }
}

#[track_caller]
pub(crate) fn assert_has_length<S, T, I, M: Mode, R>(
    this: &AssertThat<'_, S, M, R>,
    mut iterator: I,
    expected: usize,
) where
    I: Iterator,
    I::Item: Borrow<T>,
    R: AssertionRenderer<Vec<I::Item>>,
{
    if let Some(actual) = exact_size_hint(&iterator) {
        if actual == expected {
            return;
        }
        let details = alloc::vec![format!(
            "Iterator reported an exact remaining length of {actual}; no elements were consumed."
        )];
        let preview: Vec<I::Item> = Vec::new();
        let rendered = this.render_value(&preview);
        this.fail_with_details(details, |w: &mut String| {
            writedoc! {w,r"
                Actual: {rendered:#?}

                does not have the correct length

                Expected: {expected}
                Observed: {actual}
            "}
        });
        return;
    }
    let mut tail = Tail::new();
    let mut exhausted = false;
    for _ in 0..=expected {
        let Some(item) = iterator.next() else {
            exhausted = true;
            break;
        };
        tail.push(item);
    }
    if tail.consumed == expected && exhausted {
        return;
    }
    let preview = tail.finish();
    let mut details = Vec::new();
    push_preview_details(&mut details, &preview, None);
    let actual = this.render_value(&preview.items);
    let observed = if preview.consumed > expected {
        format!("more than {expected}")
    } else {
        format!("{}", preview.consumed)
    };
    this.fail_with_details(details, |w: &mut String| {
        writedoc! {w,r"
        Actual: {actual:#?}

        does not have the correct length

        Expected: {expected}
        Observed: {observed}
    "}
    });
}
