use super::{
    AssertThat, AssertionContext, Borrow, EqualToRef, Expectation, FailureBuilder, FailureKind,
    GroupStyle, Mode, PhantomData, PositionReporting, Preview, Scan, Tail, ValueRenderer, Vec,
    execute,
};
use crate::Fact;

struct Contains<'e, T, E> {
    expected: &'e E,
    item: PhantomData<fn() -> T>,
}

impl<T, E, I, R> Scan<I, R> for Contains<'_, T, E>
where
    I: Iterator,
    I::Item: Borrow<T>,
    T: PartialEq<E>,
    R: ValueRenderer<T> + ValueRenderer<E> + ValueRenderer<usize>,
{
    type Rejection = Preview<I::Item>;
    fn observe(
        &self,
        iterator: &mut I,
        context: &AssertionContext<'_, R>,
    ) -> Result<(), Self::Rejection> {
        let mut tail = Tail::new();
        for item in iterator {
            let matched = EqualToRef(self.expected)
                .evaluate(item.borrow(), context)
                .is_ok();
            tail.push(item);
            if matched {
                return Ok(());
            }
        }
        Err(tail.finish())
    }

    const KIND: FailureKind = FailureKind::Membership;
    fn explain<Target>(
        &self,
        rejection: Self::Rejection,
        failure: FailureBuilder<Target>,
        context: &AssertionContext<'_, R>,
    ) -> FailureBuilder<Target> {
        let expected = context.render().value(self.expected);
        let preview = rejection;
        let failure = failure
            .actual(preview.rendered::<T, _>(context.render()))
            .relation("does not contain")
            .expected(expected);
        preview.facts(failure, context.render(), None)
    }
}

struct ContainsAll<'e, T, E> {
    expected: &'e [E],
    item: PhantomData<fn() -> T>,
}

impl<T, E, I, R> Scan<I, R> for ContainsAll<'_, T, E>
where
    I: Iterator,
    I::Item: Borrow<T>,
    T: PartialEq<E>,
    R: ValueRenderer<T> + ValueRenderer<E> + ValueRenderer<usize>,
{
    type Rejection = (Preview<I::Item>, Vec<usize>);
    fn observe(
        &self,
        iterator: &mut I,
        context: &AssertionContext<'_, R>,
    ) -> Result<(), Self::Rejection> {
        if self.expected.is_empty() {
            return Ok(());
        }
        let mut found = alloc::vec![false; self.expected.len()];
        let mut remaining = self.expected.len();
        let mut tail = Tail::new();
        for item in iterator {
            for (index, expected) in self.expected.iter().enumerate() {
                if !found[index]
                    && EqualToRef(expected)
                        .evaluate(item.borrow(), context)
                        .is_ok()
                {
                    found[index] = true;
                    remaining -= 1;
                }
            }
            tail.push(item);
            if remaining == 0 {
                return Ok(());
            }
        }
        let missing = found
            .into_iter()
            .enumerate()
            .filter_map(|(index, found)| (!found).then_some(index))
            .collect();
        Err((tail.finish(), missing))
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
        let (preview, missing) = rejection;
        let missing = missing
            .into_iter()
            .map(|index| &self.expected[index])
            .collect::<Vec<_>>();
        let failure = failure
            .actual(preview.rendered::<T, _>(render))
            .relation("does not contain all of")
            .expected(expected)
            .fact(Fact::labelled(
                "Elements not found",
                render.borrowed_values::<E, _>(missing.as_slice(), GroupStyle::List),
            ));
        preview.facts(failure, render, None)
    }
}

struct DoesNotContain<'e, T, E> {
    expected: &'e E,
    item: PhantomData<fn() -> T>,
    positions: PositionReporting,
}

impl<T, E, I, R> Scan<I, R> for DoesNotContain<'_, T, E>
where
    I: Iterator,
    I::Item: Borrow<T>,
    T: PartialEq<E>,
    R: ValueRenderer<T> + ValueRenderer<E> + ValueRenderer<usize>,
{
    type Rejection = (Preview<I::Item>, usize);
    fn observe(
        &self,
        iterator: &mut I,
        context: &AssertionContext<'_, R>,
    ) -> Result<(), Self::Rejection> {
        let mut tail = Tail::new();
        for (index, item) in iterator.enumerate() {
            let matched = EqualToRef(self.expected)
                .evaluate(item.borrow(), context)
                .is_ok();
            tail.push(item);
            if matched {
                return Err((tail.finish(), index));
            }
        }
        Ok(())
    }

    const KIND: FailureKind = FailureKind::Membership;
    fn explain<Target>(
        &self,
        rejection: Self::Rejection,
        failure: FailureBuilder<Target>,
        context: &AssertionContext<'_, R>,
    ) -> FailureBuilder<Target> {
        let render = context.render();
        let unexpected = render.value(self.expected);
        let (preview, index) = rejection;
        let failure = failure
            .actual(preview.rendered::<T, _>(render))
            .relation("contains")
            .unexpected(unexpected);
        preview.facts(failure, render, self.positions.index(index))
    }
}

#[track_caller]
pub(crate) fn assert_contains<S, T, E, I, M: Mode, R>(
    this: &AssertThat<'_, S, M, R>,
    iterator: I,
    expected: &E,
) where
    I: Iterator,
    I::Item: Borrow<T>,
    T: PartialEq<E>,
    R: ValueRenderer<T> + ValueRenderer<E> + ValueRenderer<usize>,
{
    execute(
        this,
        iterator,
        &Contains::<T, E> {
            expected,
            item: PhantomData,
        },
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
    T: PartialEq<E>,
    R: ValueRenderer<T> + ValueRenderer<E> + ValueRenderer<usize>,
{
    execute(
        this,
        iterator,
        &ContainsAll::<T, E> {
            expected,
            item: PhantomData,
        },
    );
}

#[track_caller]
pub(crate) fn assert_does_not_contain<S, T, E, I, M: Mode, R>(
    this: &AssertThat<'_, S, M, R>,
    iterator: I,
    expected: &E,
    positions: PositionReporting,
) where
    I: Iterator,
    I::Item: Borrow<T>,
    T: PartialEq<E>,
    R: ValueRenderer<T> + ValueRenderer<E> + ValueRenderer<usize>,
{
    execute(
        this,
        iterator,
        &DoesNotContain::<T, E> {
            expected,
            item: PhantomData,
            positions,
        },
    );
}
