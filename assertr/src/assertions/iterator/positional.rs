use super::{
    AssertThat, AssertionFailure, AssertrPartialEq, Borrow, Capture, Fact, FailureBuilder,
    FailureKind, GroupStyle, Mode, PREVIEW_CAPACITY, Preview, Tail, UnsatisfiedElements,
    ValueRenderer, Vec, VecDeque, exact_size_hint, indexed_children, unequal_element,
    unmatched_element,
};
use crate::failure::{Attached, FailureTarget};

/// What ended an exact positional scan before it could succeed.
enum ExactFailure {
    KnownLength {
        actual: usize,
    },
    Exhausted {
        index: usize,
    },
    Criterion {
        index: usize,
        failures: Vec<AssertionFailure>,
    },
    Extra {
        index: usize,
    },
}

impl ExactFailure {
    fn decisive_index(&self) -> Option<usize> {
        match self {
            Self::Criterion { index, .. } | Self::Extra { index } => Some(*index),
            Self::KnownLength { .. } | Self::Exhausted { .. } => None,
        }
    }

    /// Attaches the scan's outcome to the failure: the preview facts, what ended the scan, and
    /// the failures of the decisive element as children located at its index.
    fn apply<S: FailureTarget, Item>(
        self,
        failure: FailureBuilder<S>,
        preview: &Preview<Item>,
        expected_len: usize,
    ) -> FailureBuilder<S> {
        let failure = preview.facts(failure, self.decisive_index());
        match self {
            Self::KnownLength { actual } => failure
                .fact("Reported length", actual)
                .fact("Expected length", expected_len),
            Self::Exhausted { index } => failure.fact("Exhausted at index", index),
            Self::Extra { index } => failure.fact("Extra element at index", index),
            Self::Criterion { index, failures } => failure.children(
                failures
                    .into_iter()
                    .map(|failure| failure.located_at(Fact::index(index))),
            ),
        }
    }
}

fn evaluate_exact<T, I>(
    mut iterator: I,
    expected_len: usize,
    mut criterion: impl FnMut(usize, &T) -> Result<(), Vec<AssertionFailure>>,
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
    if let Err((preview, outcome)) = evaluate_exact(iterator, expected.len(), |index, item| {
        if AssertrPartialEq::eq(item, &expected[index], Some(&mut this.eq_context())) {
            Ok(())
        } else {
            Err(alloc::vec![unequal_element(this, item, &expected[index])])
        }
    }) {
        let failure = this
            .failure(FailureKind::Equality)
            .actual(preview.rendered::<T, _, _, _>(this))
            .relation("does not contain exactly")
            .expected(
                this.render()
                    .borrowed_values::<E, _>(expected, GroupStyle::List),
            );
        outcome.apply(failure, &preview, expected.len()).raise();
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
    if let Err((preview, outcome)) = evaluate_exact(iterator, predicates.len(), |index, item| {
        if predicates[index](item) {
            Ok(())
        } else {
            Err(alloc::vec![unmatched_element(this, item)])
        }
    }) {
        let failure = this
            .failure(FailureKind::Predicate)
            .actual(preview.rendered::<T, _, _, _>(this))
            .relation("does not exactly match the predicates");
        outcome.apply(failure, &preview, predicates.len()).raise();
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
    if let Err((preview, outcome)) = evaluate_exact(iterator, assertions.len(), |index, item| {
        let failures = this.collect_element_failures(item, &assertions[index]);
        if failures.is_empty() {
            Ok(())
        } else {
            Err(failures)
        }
    }) {
        let failure = this
            .failure(FailureKind::Predicate)
            .actual(preview.rendered::<T, _, _, _>(this))
            .relation("does not exactly satisfy the assertions");
        outcome.apply(failure, &preview, assertions.len()).raise();
    }
}

/// What ended a prefix scan before it could succeed.
enum PrefixFailure {
    KnownTooShort {
        actual: usize,
    },
    Exhausted {
        index: usize,
    },
    Criterion {
        index: usize,
        failures: Vec<AssertionFailure>,
    },
}

impl PrefixFailure {
    fn decisive_index(&self) -> Option<usize> {
        match self {
            Self::Criterion { index, .. } => Some(*index),
            Self::KnownTooShort { .. } | Self::Exhausted { .. } => None,
        }
    }

    /// Attaches the scan's outcome to the failure: the preview facts, what ended the scan, and
    /// the failures of the decisive element as children located at its index.
    fn apply<S: FailureTarget, Item>(
        self,
        failure: FailureBuilder<S>,
        preview: &Preview<Item>,
        prefix_len: usize,
    ) -> FailureBuilder<S> {
        let failure = preview.facts(failure, self.decisive_index());
        match self {
            Self::KnownTooShort { actual } => failure
                .fact("Reported length", actual)
                .fact("Prefix length", prefix_len),
            Self::Exhausted { index } => failure.fact("Exhausted at index", index),
            Self::Criterion { index, failures } => failure.children(
                failures
                    .into_iter()
                    .map(|failure| failure.located_at(Fact::index(index))),
            ),
        }
    }
}

fn evaluate_prefix<T, I>(
    mut iterator: I,
    expected_len: usize,
    mut criterion: impl FnMut(usize, &T) -> Result<(), Vec<AssertionFailure>>,
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
    if let Err((preview, outcome)) = evaluate_prefix(iterator, expected.len(), |index, item| {
        if AssertrPartialEq::eq(item, &expected[index], Some(&mut this.eq_context())) {
            Ok(())
        } else {
            Err(alloc::vec![unequal_element(this, item, &expected[index])])
        }
    }) {
        let failure = this
            .failure(FailureKind::Membership)
            .actual(preview.rendered::<T, _, _, _>(this))
            .relation("does not start with")
            .expected(
                this.render()
                    .borrowed_values::<E, _>(expected, GroupStyle::List),
            );
        outcome.apply(failure, &preview, expected.len()).raise();
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
    if let Err((preview, outcome)) = evaluate_prefix(iterator, predicates.len(), |index, item| {
        if predicates[index](item) {
            Ok(())
        } else {
            Err(alloc::vec![unmatched_element(this, item)])
        }
    }) {
        let failure = this
            .failure(FailureKind::Predicate)
            .actual(preview.rendered::<T, _, _, _>(this))
            .relation("does not start with elements matching the predicates");
        outcome.apply(failure, &preview, predicates.len()).raise();
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
    if let Err((preview, outcome)) = evaluate_prefix(iterator, assertions.len(), |index, item| {
        let failures = this.collect_element_failures(item, &assertions[index]);
        if failures.is_empty() {
            Ok(())
        } else {
            Err(failures)
        }
    }) {
        let failure = this
            .failure(FailureKind::Predicate)
            .actual(preview.rendered::<T, _, _, _>(this))
            .relation("does not start with elements satisfying the assertions");
        outcome.apply(failure, &preview, assertions.len()).raise();
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

/// Checks the retained tail against a per-element suffix criterion.
///
/// Returns `None` when the iterator yielded fewer elements than the suffix needs. Otherwise
/// returns the failures of the suffix elements that did not satisfy the criterion, each with its
/// index in yield order.
fn check_suffix<T, Item, C>(
    preview: &Preview<Item>,
    criteria: &[C],
    mut criterion: impl FnMut(&T, &C) -> Vec<AssertionFailure>,
) -> Option<UnsatisfiedElements>
where
    Item: Borrow<T>,
{
    if preview.consumed < criteria.len() {
        return None;
    }
    let start = preview.items.len().saturating_sub(criteria.len());
    let first_index = preview.consumed - criteria.len();
    let unsatisfied = preview.items[start..]
        .iter()
        .zip(criteria)
        .enumerate()
        .filter_map(|(offset, (item, criterion_of_element))| {
            let failures = criterion(item.borrow(), criterion_of_element);
            (!failures.is_empty()).then_some((first_index + offset, failures))
        })
        .collect();
    Some(unsatisfied)
}

/// Starts a suffix failure over the trimmed preview. `unsatisfied` is `None` when the iterator
/// was too short for the suffix and otherwise holds the failing suffix elements.
#[track_caller]
fn suffix_failure<'c, S, T, Item, M: Mode, R>(
    this: &'c AssertThat<'_, S, M, R>,
    preview: &mut Preview<Item>,
    kind: FailureKind,
    relation: &'static str,
    suffix_len: usize,
    unsatisfied: Option<UnsatisfiedElements>,
) -> FailureBuilder<Attached<'c>>
where
    Item: Borrow<T>,
    R: ValueRenderer<T>,
{
    trim_preview(preview);
    let too_short = unsatisfied.is_none();
    let (children, omitted) =
        indexed_children(unsatisfied.unwrap_or_default(), this.render().max_items());
    let failure = this
        .failure(kind)
        .actual(preview.rendered::<T, _, _, _>(this))
        .relation(relation);
    let failure = preview.facts(failure, None);
    let failure = if too_short {
        failure.fact("Suffix length", suffix_len)
    } else {
        failure
    };
    failure
        .omitted(omitted, "unsatisfied element")
        .children(children)
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
    let unsatisfied = check_suffix::<T, _, _>(&preview, expected, |item, expected| {
        if AssertrPartialEq::eq(item, expected, Some(&mut this.eq_context())) {
            Vec::new()
        } else {
            alloc::vec![unequal_element(this, item, expected)]
        }
    });
    if !unsatisfied.as_ref().is_some_and(Vec::is_empty) {
        suffix_failure(
            this,
            &mut preview,
            FailureKind::Membership,
            "does not end with",
            expected.len(),
            unsatisfied,
        )
        .expected(
            this.render()
                .borrowed_values::<E, _>(expected, GroupStyle::List),
        )
        .raise();
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
    let unsatisfied = check_suffix::<T, _, _>(&preview, predicates, |item, predicate| {
        if predicate(item) {
            Vec::new()
        } else {
            alloc::vec![unmatched_element(this, item)]
        }
    });
    if !unsatisfied.as_ref().is_some_and(Vec::is_empty) {
        suffix_failure(
            this,
            &mut preview,
            FailureKind::Predicate,
            "does not end with elements matching the predicates",
            predicates.len(),
            unsatisfied,
        )
        .raise();
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
    let unsatisfied = check_suffix::<T, _, _>(&preview, assertions, |item, assertion| {
        this.collect_element_failures(item, assertion)
    });
    if !unsatisfied.as_ref().is_some_and(Vec::is_empty) {
        suffix_failure(
            this,
            &mut preview,
            FailureKind::Predicate,
            "does not end with elements satisfying the assertions",
            assertions.len(),
            unsatisfied,
        )
        .raise();
    }
}

/// Scans for a window of `pattern_len` consecutive elements satisfying `criterion`, which
/// receives the window and the index of its first element in yield order.
///
/// On failure, returns the preview together with the failing elements of the last candidate
/// window, each with its index in yield order.
fn find_contiguous<T, I>(
    iterator: I,
    pattern_len: usize,
    mut criterion: impl FnMut(usize, &[I::Item]) -> Result<(), UnsatisfiedElements>,
) -> Result<(), (Preview<I::Item>, UnsatisfiedElements)>
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
    let mut last_unsatisfied = Vec::new();
    for item in iterator {
        consumed += 1;
        if window.len() == capacity {
            let _ = window.pop_front();
        }
        window.push_back(item);
        if window.len() >= pattern_len {
            let contiguous = window.make_contiguous();
            let start = contiguous.len() - pattern_len;
            match criterion(consumed - pattern_len, &contiguous[start..]) {
                Ok(()) => return Ok(()),
                Err(unsatisfied) => last_unsatisfied = unsatisfied,
            }
        }
    }
    let mut preview = Preview {
        items: window.into_iter().collect(),
        consumed,
    };
    trim_preview(&mut preview);
    Err((preview, last_unsatisfied))
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
    if let Err((preview, _)) = find_contiguous::<T, _>(iterator, expected.len(), |_, window| {
        let matched = window.iter().zip(expected).all(|(item, expected)| {
            AssertrPartialEq::eq(item.borrow(), expected, Some(&mut this.eq_context()))
        });
        if matched { Ok(()) } else { Err(Vec::new()) }
    }) {
        let failure = this
            .failure(FailureKind::Membership)
            .actual(preview.rendered::<T, _, _, _>(this))
            .relation("does not contain the contiguous subsequence")
            .expected(
                this.render()
                    .borrowed_values::<E, _>(expected, GroupStyle::List),
            );
        preview.facts(failure, None).raise();
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
    if let Err((preview, _)) = find_contiguous::<T, _>(iterator, predicates.len(), |_, window| {
        let matched = window
            .iter()
            .zip(predicates)
            .all(|(item, predicate)| predicate(item.borrow()));
        if matched { Ok(()) } else { Err(Vec::new()) }
    }) {
        let failure = this
            .failure(FailureKind::Predicate)
            .actual(preview.rendered::<T, _, _, _>(this))
            .relation("does not contain a contiguous subsequence matching the predicates");
        preview.facts(failure, None).raise();
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
    if let Err((preview, unsatisfied)) =
        find_contiguous::<T, _>(iterator, assertions.len(), |start, window| {
            let unsatisfied = window
                .iter()
                .zip(assertions)
                .enumerate()
                .filter_map(|(offset, (item, assertion))| {
                    let failures = this.collect_element_failures(item.borrow(), assertion);
                    (!failures.is_empty()).then_some((start + offset, failures))
                })
                .collect::<Vec<_>>();
            if unsatisfied.is_empty() {
                Ok(())
            } else {
                Err(unsatisfied)
            }
        })
    {
        let (children, omitted) = indexed_children(unsatisfied, this.render().max_items());
        let failure = this
            .failure(FailureKind::Predicate)
            .actual(preview.rendered::<T, _, _, _>(this))
            .relation("does not contain a contiguous subsequence satisfying the assertions");
        preview
            .facts(failure, None)
            .omitted(omitted, "unsatisfied element")
            .children(children)
            .raise();
    }
}
