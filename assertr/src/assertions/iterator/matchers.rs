//! Matcher adapters over bounded, single-pass iterator scans.

use super::{
    AssertThat, Borrow, Mode, PREVIEW_CAPACITY, PositionReporting, Vec, VecDeque, exact_size_hint,
};
use crate::{
    Fact,
    failure::{FailureKind, PathSegment},
    matchers::{AssertrMatcher, MatchContext, MatcherList},
};

#[track_caller]
fn raise<S, M: Mode, R>(
    this: &AssertThat<'_, S, M, R>,
    context: MatchContext<'_, R>,
    relation: &'static str,
    consumed: usize,
) where
    R: crate::ValueRenderer<usize>,
{
    this.failure(FailureKind::Matching)
        .relation(relation)
        .fact(Fact::labelled("Consumed", this.render().value(&consumed)))
        .fact(Fact::labelled(
            "Preview starts at",
            consumed.saturating_sub(PREVIEW_CAPACITY),
        ))
        .omitted_children(context.omitted_children())
        .children(context.into_failures())
        .raise();
}

#[track_caller]
pub(crate) fn membership<S, T, P, I, M: Mode, R>(
    this: &AssertThat<'_, S, M, R>,
    iterator: I,
    matcher: &P,
    positive: bool,
    positions: PositionReporting,
) where
    I: Iterator,
    I::Item: Borrow<T>,
    P: AssertrMatcher<T, R>,
    R: crate::ValueRenderer<usize>,
{
    let mut retained: VecDeque<MatchContext<'_, R>> = VecDeque::new();
    let mut consumed = 0;
    let mut discarded = 0;
    for item in iterator {
        let mut context = MatchContext::for_assertion(this);
        context.set_positive(positive);
        let accepted = if let Some(index) = positions.index(consumed) {
            context
                .scoped(PathSegment::Index(index), |context| {
                    matcher.evaluate(item.borrow(), context)
                })
                .matched
        } else {
            matcher.evaluate(item.borrow(), &mut context).matched
        };
        consumed += 1;
        if accepted {
            if !positive {
                raise(
                    this,
                    context,
                    "contains an unexpected matching element",
                    consumed,
                );
            }
            return;
        }
        if retained.len() == PREVIEW_CAPACITY
            && let Some(old) = retained.pop_front()
        {
            discarded += old.evidence.len() + old.omitted;
        }
        retained.push_back(context);
    }
    if positive {
        let mut context = MatchContext::for_assertion(this);
        context.omitted = discarded;
        for child in retained {
            context.append(child);
        }
        if context.evidence.is_empty() {
            context.outcome(false, |context| matcher.describe(context));
        }
        raise(
            this,
            context,
            "does not contain a matching element",
            consumed,
        );
    }
}

#[track_caller]
pub(crate) fn exact_or_prefix<S, T, L, I, M: Mode, R>(
    this: &AssertThat<'_, S, M, R>,
    mut iterator: I,
    list: &L,
    exact: bool,
) where
    I: Iterator,
    I::Item: Borrow<T>,
    L: MatcherList<T, R>,
    R: crate::ValueRenderer<usize>,
{
    let expected_length = list.len();
    let mut context = MatchContext::for_assertion(this);
    if let Some(actual) = exact_size_hint(&iterator)
        && ((exact && actual != expected_length) || (!exact && actual < expected_length))
    {
        this.failure(FailureKind::Matching)
            .relation("does not have the required sequence length")
            .fact(Fact::labelled(
                "Reported length",
                this.render().value(&actual),
            ))
            .fact(Fact::labelled(
                "Expected length",
                this.render().value(&expected_length),
            ))
            .raise();
        return;
    }
    for index in 0..expected_length {
        let Some(item) = iterator.next() else {
            raise(this, context, "is missing a matching position", index);
            return;
        };
        if !context
            .scoped(PathSegment::Index(index), |context| {
                list.evaluate_at(index, item.borrow(), context)
            })
            .matched
        {
            raise(
                this,
                context,
                "does not match the required position",
                index + 1,
            );
            return;
        }
    }
    if exact && iterator.next().is_some() {
        raise(this, context, "has an extra element", expected_length + 1);
    }
}

#[track_caller]
pub(crate) fn suffix_or_contiguous<S, T, L, I, M: Mode, R>(
    this: &AssertThat<'_, S, M, R>,
    iterator: I,
    list: &L,
    suffix: bool,
) where
    I: Iterator,
    I::Item: Borrow<T>,
    L: MatcherList<T, R>,
    R: crate::ValueRenderer<usize>,
{
    let expected_length = list.len();
    if expected_length == 0 {
        return;
    }
    let mut window = VecDeque::new();
    let mut consumed = 0;
    let mut final_window = MatchContext::for_assertion(this);
    for item in iterator {
        if window.len() == expected_length {
            window.pop_front();
        }
        window.push_back(item);
        consumed += 1;
        if !suffix && window.len() == expected_length {
            let mut context = MatchContext::for_assertion(this);
            let mut matched = true;
            for (slot, item) in window.iter().enumerate() {
                matched &= context
                    .scoped(
                        PathSegment::Index(consumed - expected_length + slot),
                        |context| list.evaluate_at(slot, item.borrow(), context),
                    )
                    .matched;
            }
            if matched {
                return;
            }
            final_window = context;
        }
    }
    if suffix && window.len() == expected_length {
        let mut matched = true;
        for (slot, item) in window.iter().enumerate() {
            matched &= final_window
                .scoped(
                    PathSegment::Index(consumed - expected_length + slot),
                    |context| list.evaluate_at(slot, item.borrow(), context),
                )
                .matched;
        }
        if matched {
            return;
        }
    }
    raise(
        this,
        final_window,
        if suffix {
            "does not end with matching positions"
        } else {
            "does not contain matching contiguous positions"
        },
        consumed,
    );
}
struct Items<'a, T, I> {
    items: &'a [I],
    view: core::marker::PhantomData<T>,
}

impl<T, I> crate::assertions::HasLength for Items<'_, T, I> {
    fn length(&self) -> usize {
        self.items.len()
    }
}

impl<T, I: Borrow<T>> crate::assertions::collection::Collection for Items<'_, T, I> {
    type Item = T;
    const PRESENTATION: crate::renderer::CollectionPresentation =
        crate::renderer::CollectionPresentation::list();

    fn elements(&self) -> impl Iterator<Item = &T> {
        self.items.iter().map(Borrow::borrow)
    }
}

#[track_caller]
pub(crate) fn unordered<S, T, L, I, M: Mode, R>(
    this: &AssertThat<'_, S, M, R>,
    iterator: I,
    list: &L,
) where
    I: Iterator,
    I::Item: Borrow<T>,
    L: MatcherList<T, R>,
    R: crate::ValueRenderer<usize>,
{
    let expected_length = list.len();
    if let Some(actual) = exact_size_hint(&iterator)
        && actual != expected_length
    {
        this.failure(FailureKind::Matching)
            .relation("does not have the required number of elements")
            .fact(Fact::labelled(
                "Reported length",
                this.render().value(&actual),
            ))
            .fact(Fact::labelled(
                "Expected length",
                this.render().value(&expected_length),
            ))
            .raise();
        return;
    }
    let items = iterator
        .take(expected_length.saturating_add(1))
        .collect::<Vec<_>>();
    let actual = Items {
        items: &items,
        view: core::marker::PhantomData::<T>,
    };
    let mut context = MatchContext::for_assertion(this);
    if !crate::matchers::elements_are_in_any_order(list)
        .evaluate(&actual, &mut context)
        .matched
    {
        raise(
            this,
            context,
            "does not match exactly in any order",
            items.len(),
        );
    }
}
