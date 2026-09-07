use super::{
    AssertThat, AssertionFailure, Borrow, Fact, FailureBuilder, FailureKind, GroupStyle, Mode,
    PREVIEW_CAPACITY, Preview, Tail, UnsatisfiedElements, ValueRenderer, Vec, VecDeque,
    exact_size_hint, indexed_children, unequal_element,
};
use crate::failure::{Attached, FailureTarget};
use crate::renderer::RenderingContext;

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

    /// Attaches the scan's outcome to the failure: the preview facts, what ended the scan, and the
    /// failures of the decisive element as children located at its index.
    fn apply<S: FailureTarget, Item, R: ValueRenderer<usize>>(
        self,
        failure: FailureBuilder<S>,
        preview: &Preview<Item>,
        rendering: RenderingContext<'_, R>,
        expected_len: usize,
    ) -> FailureBuilder<S> {
        let failure = preview.facts(failure, rendering, self.decisive_index());
        match self {
            Self::KnownLength { actual } => failure
                .fact(Fact::labelled("Reported length", rendering.value(&actual)))
                .fact(Fact::labelled(
                    "Expected length",
                    rendering.value(&expected_len),
                )),
            Self::Exhausted { index } => failure.fact(Fact::labelled("Exhausted at index", index)),
            Self::Extra { index } => failure.fact(Fact::labelled("Extra element at index", index)),
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
    T: PartialEq<E>,
    R: ValueRenderer<T> + ValueRenderer<E> + ValueRenderer<usize>,
{
    if let Err((preview, outcome)) = evaluate_exact(iterator, expected.len(), |index, item| {
        if crate::matchers::equals(item, &expected[index]) {
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
        outcome
            .apply(failure, &preview, this.render(), expected.len())
            .raise();
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

    /// Attaches the scan's outcome to the failure: the preview facts, what ended the scan, and the
    /// failures of the decisive element as children located at its index.
    fn apply<S: FailureTarget, Item, R: ValueRenderer<usize>>(
        self,
        failure: FailureBuilder<S>,
        preview: &Preview<Item>,
        rendering: RenderingContext<'_, R>,
        prefix_len: usize,
    ) -> FailureBuilder<S> {
        let failure = preview.facts(failure, rendering, self.decisive_index());
        match self {
            Self::KnownTooShort { actual } => failure
                .fact(Fact::labelled("Reported length", rendering.value(&actual)))
                .fact(Fact::labelled(
                    "Prefix length",
                    rendering.value(&prefix_len),
                )),
            Self::Exhausted { index } => failure.fact(Fact::labelled("Exhausted at index", index)),
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
    T: PartialEq<E>,
    R: ValueRenderer<T> + ValueRenderer<E> + ValueRenderer<usize>,
{
    if let Err((preview, outcome)) = evaluate_prefix(iterator, expected.len(), |index, item| {
        if crate::matchers::equals(item, &expected[index]) {
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
        outcome
            .apply(failure, &preview, this.render(), expected.len())
            .raise();
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
/// Returns `None` when the iterator yielded fewer elements than the suffix needs. Otherwise returns
/// the failures of the suffix elements that did not satisfy the criterion, each with its index in
/// yield order.
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

/// Starts a suffix failure over the trimmed preview. `unsatisfied` is `None` when the iterator was
/// too short for the suffix and otherwise holds the failing suffix elements.
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
    R: ValueRenderer<T> + ValueRenderer<usize>,
{
    trim_preview(preview);
    let too_short = unsatisfied.is_none();
    let (children, omitted) =
        indexed_children(unsatisfied.unwrap_or_default(), this.render().max_items());
    let failure = this
        .failure(kind)
        .actual(preview.rendered::<T, _, _, _>(this))
        .relation(relation);
    let failure = preview.facts(failure, this.render(), None);
    let failure = if too_short {
        failure.fact(Fact::labelled(
            "Suffix length",
            this.render().value(&suffix_len),
        ))
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
    T: PartialEq<E>,
    R: ValueRenderer<T> + ValueRenderer<E> + ValueRenderer<usize>,
{
    if expected.is_empty() {
        return;
    }
    let mut preview = collect_tail(iterator, expected.len());
    let unsatisfied = check_suffix::<T, _, _>(&preview, expected, |item, expected| {
        if crate::matchers::equals(item, expected) {
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

/// Scans for a window of `pattern_len` consecutive elements satisfying `criterion`, which receives
/// the window and the index of its first element in yield order.
///
/// On failure, returns the preview together with the failing elements of the last candidate window,
/// each with its index in yield order.
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
    T: PartialEq<E>,
    R: ValueRenderer<T> + ValueRenderer<E> + ValueRenderer<usize>,
{
    if let Err((preview, _)) = find_contiguous::<T, _>(iterator, expected.len(), |_, window| {
        let matched = window
            .iter()
            .zip(expected)
            .all(|(item, expected)| crate::matchers::equals(item.borrow(), expected));
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
        preview.facts(failure, this.render(), None).raise();
    }
}
