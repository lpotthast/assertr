use super::{
    AssertThat, AssertionContext, AssertionFailure, Borrow, EqualToRef, Expectation, Fact,
    FailureBuilder, FailureKind, GroupStyle, Mode, PREVIEW_CAPACITY, PhantomData, Preview, Scan,
    Tail, UnsatisfiedElements, ValueRenderer, Vec, VecDeque, equal_element, exact_size_hint,
    execute, indexed_children,
};
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
    fn apply<S, Item, R: ValueRenderer<usize>>(
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
    iterator: &mut I,
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

struct ContainsExactly<'e, T, E> {
    expected: &'e [E],
    item: PhantomData<fn() -> T>,
}

impl<T, E, I, R> Scan<I, R> for ContainsExactly<'_, T, E>
where
    I: Iterator,
    I::Item: Borrow<T>,
    T: PartialEq<E>,
    R: ValueRenderer<T> + ValueRenderer<E> + ValueRenderer<usize>,
{
    type Rejection = (Preview<I::Item>, ExactFailure);
    fn observe(
        &self,
        iterator: &mut I,
        context: &AssertionContext<'_, R>,
    ) -> Result<(), Self::Rejection> {
        evaluate_exact(iterator, self.expected.len(), |index, item| {
            equal_element(context, item, &self.expected[index])
        })
    }

    const KIND: FailureKind = FailureKind::Equality;
    fn explain<Target>(
        &self,
        rejection: Self::Rejection,
        failure: FailureBuilder<Target>,
        context: &AssertionContext<'_, R>,
    ) -> FailureBuilder<Target> {
        let render = context.render();
        let expected = render.borrowed_values::<E, _>(self.expected, GroupStyle::List);
        let (preview, outcome) = rejection;
        let failure = failure
            .actual(preview.rendered::<T, _>(render))
            .relation("does not contain exactly")
            .expected(expected);
        outcome.apply(failure, &preview, render, self.expected.len())
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
    T: PartialEq<E>,
    R: ValueRenderer<T> + ValueRenderer<E> + ValueRenderer<usize>,
{
    execute(
        this,
        iterator,
        &ContainsExactly::<T, E> {
            expected,
            item: PhantomData,
        },
    );
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
    fn apply<S, Item, R: ValueRenderer<usize>>(
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
    iterator: &mut I,
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

struct StartsWith<'e, T, E> {
    expected: &'e [E],
    item: PhantomData<fn() -> T>,
}

impl<T, E, I, R> Scan<I, R> for StartsWith<'_, T, E>
where
    I: Iterator,
    I::Item: Borrow<T>,
    T: PartialEq<E>,
    R: ValueRenderer<T> + ValueRenderer<E> + ValueRenderer<usize>,
{
    type Rejection = (Preview<I::Item>, PrefixFailure);
    fn observe(
        &self,
        iterator: &mut I,
        context: &AssertionContext<'_, R>,
    ) -> Result<(), Self::Rejection> {
        evaluate_prefix(iterator, self.expected.len(), |index, item| {
            equal_element(context, item, &self.expected[index])
        })
    }

    const KIND: FailureKind = FailureKind::Membership;
    fn explain<Target>(
        &self,
        rejection: Self::Rejection,
        failure: FailureBuilder<Target>,
        context: &AssertionContext<'_, R>,
    ) -> FailureBuilder<Target> {
        let render = context.render();
        let expected = render.borrowed_values::<E, _>(self.expected, GroupStyle::List);
        let (preview, outcome) = rejection;
        let failure = failure
            .actual(preview.rendered::<T, _>(render))
            .relation("does not start with")
            .expected(expected);
        outcome.apply(failure, &preview, render, self.expected.len())
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
    T: PartialEq<E>,
    R: ValueRenderer<T> + ValueRenderer<E> + ValueRenderer<usize>,
{
    execute(
        this,
        iterator,
        &StartsWith::<T, E> {
            expected,
            item: PhantomData,
        },
    );
}

fn collect_tail<I: Iterator>(iterator: &mut I, required: usize) -> Preview<I::Item> {
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

struct EndsWith<'e, T, E> {
    expected: &'e [E],
    item: PhantomData<fn() -> T>,
}

impl<T, E, I, R> Scan<I, R> for EndsWith<'_, T, E>
where
    I: Iterator,
    I::Item: Borrow<T>,
    T: PartialEq<E>,
    R: ValueRenderer<T> + ValueRenderer<E> + ValueRenderer<usize>,
{
    type Rejection = (Preview<I::Item>, Option<UnsatisfiedElements>);
    fn observe(
        &self,
        iterator: &mut I,
        context: &AssertionContext<'_, R>,
    ) -> Result<(), Self::Rejection> {
        if self.expected.is_empty() {
            return Ok(());
        }
        let preview = collect_tail(iterator, self.expected.len());
        let unsatisfied = check_suffix::<T, _, _>(&preview, self.expected, |item, expected| {
            equal_element(context, item, expected)
                .err()
                .unwrap_or_default()
        });
        if unsatisfied.as_ref().is_some_and(Vec::is_empty) {
            Ok(())
        } else {
            Err((preview, unsatisfied))
        }
    }

    const KIND: FailureKind = FailureKind::Membership;
    fn explain<Target>(
        &self,
        rejection: Self::Rejection,
        failure: FailureBuilder<Target>,
        context: &AssertionContext<'_, R>,
    ) -> FailureBuilder<Target> {
        let render = context.render();
        let expected = render.borrowed_values::<E, _>(self.expected, GroupStyle::List);
        let (mut preview, unsatisfied) = rejection;
        trim_preview(&mut preview);
        let too_short = unsatisfied.is_none();
        let (children, omitted) =
            indexed_children(unsatisfied.unwrap_or_default(), render.max_items());
        let failure = failure
            .actual(preview.rendered::<T, _>(render))
            .relation("does not end with")
            .expected(expected);
        let failure = preview.facts(failure, render, None);
        let failure = if too_short {
            failure.fact(Fact::labelled(
                "Suffix length",
                render.value(&self.expected.len()),
            ))
        } else {
            failure
        };
        failure
            .omitted(omitted, "unsatisfied element")
            .children(children)
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
    T: PartialEq<E>,
    R: ValueRenderer<T> + ValueRenderer<E> + ValueRenderer<usize>,
{
    execute(
        this,
        iterator,
        &EndsWith::<T, E> {
            expected,
            item: PhantomData,
        },
    );
}

/// Scans for a window of `pattern_len` consecutive elements satisfying `criterion`, which receives
/// the window and the index of its first element in yield order.
///
/// On failure, returns the preview together with the failing elements of the last candidate window,
/// each with its index in yield order.
fn find_contiguous<T, I>(
    iterator: &mut I,
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

struct ContainsContiguous<'e, T, E> {
    expected: &'e [E],
    item: PhantomData<fn() -> T>,
}

impl<T, E, I, R> Scan<I, R> for ContainsContiguous<'_, T, E>
where
    I: Iterator,
    I::Item: Borrow<T>,
    T: PartialEq<E>,
    R: ValueRenderer<T> + ValueRenderer<E> + ValueRenderer<usize>,
{
    type Rejection = (Preview<I::Item>, UnsatisfiedElements);
    fn observe(
        &self,
        iterator: &mut I,
        context: &AssertionContext<'_, R>,
    ) -> Result<(), Self::Rejection> {
        find_contiguous::<T, _>(iterator, self.expected.len(), |_, window| {
            let matched = window.iter().zip(self.expected).all(|(item, expected)| {
                EqualToRef(expected)
                    .evaluate(item.borrow(), context)
                    .is_ok()
            });
            if matched { Ok(()) } else { Err(Vec::new()) }
        })
    }

    const KIND: FailureKind = FailureKind::Membership;
    fn explain<Target>(
        &self,
        rejection: Self::Rejection,
        failure: FailureBuilder<Target>,
        context: &AssertionContext<'_, R>,
    ) -> FailureBuilder<Target> {
        let render = context.render();
        let expected = render.borrowed_values::<E, _>(self.expected, GroupStyle::List);
        let (preview, _) = rejection;
        let failure = failure
            .actual(preview.rendered::<T, _>(render))
            .relation("does not contain the contiguous subsequence")
            .expected(expected);
        preview.facts(failure, render, None)
    }
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
    execute(
        this,
        iterator,
        &ContainsContiguous::<T, E> {
            expected,
            item: PhantomData,
        },
    );
}
