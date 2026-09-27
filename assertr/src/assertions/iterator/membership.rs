use super::{
    AssertThat, AssertionContext, Borrow, FailureBuilder, FailureKind, GroupStyle, Mode,
    PhantomData, PositionReporting, Preview, Scan, Tail, ValueRenderer, Vec, execute,
};
use crate::{
    Fact,
    assertions::{HasLength, collection::Collection},
    renderer::CollectionPresentation,
};

/// Borrows only those stored operands selected for display by the rendering adapter.
struct Missing<'a, E> {
    expected: &'a [E],
    found: &'a [bool],
    remaining: usize,
}

impl<E> HasLength for Missing<'_, E> {
    fn length(&self) -> usize {
        self.remaining
    }
}

impl<E> Collection for Missing<'_, E> {
    type Item = E;
    const PRESENTATION: CollectionPresentation = <[E] as Collection>::PRESENTATION;
    fn elements(&self) -> impl Iterator<Item = &E> {
        self.expected
            .iter()
            .zip(self.found)
            .filter_map(|(expected, found)| (!found).then_some(expected))
    }
}
use crate::borrow_for::{BorrowFor, borrow_for};

struct Contains<'e, T, E: ?Sized> {
    expected: &'e E,
    item: PhantomData<fn() -> T>,
}

impl<T, E: ?Sized, I, R> Scan<I, R> for Contains<'_, T, E>
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
        _context: &AssertionContext<'_, R>,
    ) -> Result<(), Self::Rejection> {
        let mut tail = Tail::new(super::PREVIEW_CAPACITY);
        for item in iterator {
            let matched = item.borrow().eq(self.expected);
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
    T: PartialEq<E::View>,
    E: BorrowFor<T>,
    R: ValueRenderer<T> + ValueRenderer<E::View> + ValueRenderer<usize>,
{
    type Rejection = (Preview<I::Item>, Vec<bool>, usize);
    fn observe(
        &self,
        iterator: &mut I,
        _context: &AssertionContext<'_, R>,
    ) -> Result<(), Self::Rejection> {
        if self.expected.is_empty() {
            return Ok(());
        }
        let mut found = alloc::vec![false; self.expected.len()];
        let mut remaining = self.expected.len();
        let mut tail = Tail::new(super::PREVIEW_CAPACITY);
        for item in iterator {
            for (index, expected) in self.expected.iter().enumerate() {
                if !found[index] && item.borrow().eq(borrow_for::<T, _>(expected)) {
                    found[index] = true;
                    remaining -= 1;
                }
            }
            tail.push(item);
            if remaining == 0 {
                return Ok(());
            }
        }
        Err((tail.finish(), found, remaining))
    }

    const KIND: FailureKind = FailureKind::Membership;
    fn explain<Target>(
        &self,
        rejection: Self::Rejection,
        failure: FailureBuilder<Target>,
        context: &AssertionContext<'_, R>,
    ) -> FailureBuilder<Target> {
        let render = context.render();
        let expected = render.borrowed_values::<E::View, _>(self.expected, GroupStyle::List);
        let (preview, found, remaining) = rejection;
        let missing = Missing {
            expected: self.expected,
            found: &found,
            remaining,
        };
        let failure = failure
            .actual(preview.rendered::<T, _>(render))
            .relation("does not contain all of")
            .expected(expected)
            .fact(Fact::labelled(
                "Elements not found",
                render.borrowed_values::<E::View, _>(&missing, GroupStyle::List),
            ));
        preview.facts(failure, render, None)
    }
}

struct DoesNotContain<'e, T, E: ?Sized> {
    expected: &'e E,
    item: PhantomData<fn() -> T>,
    positions: PositionReporting,
}

impl<T, E: ?Sized, I, R> Scan<I, R> for DoesNotContain<'_, T, E>
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
        _context: &AssertionContext<'_, R>,
    ) -> Result<(), Self::Rejection> {
        let mut tail = Tail::new(super::PREVIEW_CAPACITY);
        for (index, item) in iterator.enumerate() {
            let matched = item.borrow().eq(self.expected);
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
    T: PartialEq<E::View>,
    E: BorrowFor<T>,
    R: ValueRenderer<T> + ValueRenderer<E::View> + ValueRenderer<usize>,
{
    let expected = borrow_for::<T, _>(expected);
    execute(
        this,
        iterator,
        &Contains::<T, E::View> {
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
    T: PartialEq<E::View>,
    E: BorrowFor<T>,
    R: ValueRenderer<T> + ValueRenderer<E::View> + ValueRenderer<usize>,
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
    T: PartialEq<E::View>,
    E: BorrowFor<T>,
    R: ValueRenderer<T> + ValueRenderer<E::View> + ValueRenderer<usize>,
{
    let expected = borrow_for::<T, _>(expected);
    execute(
        this,
        iterator,
        &DoesNotContain::<T, E::View> {
            expected,
            item: PhantomData,
            positions,
        },
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{prelude::*, renderer::RenderedBody};
    use core::cell::Cell;

    struct Operand<'a> {
        value: i32,
        borrows: &'a Cell<usize>,
    }
    impl Borrow<i32> for Operand<'_> {
        fn borrow(&self) -> &i32 {
            self.borrows.set(self.borrows.get() + 1);
            &self.value
        }
    }
    impl BorrowFor<i32> for Operand<'_> {
        type View = i32;
    }

    #[test]
    fn missing_operands_are_borrowed_only_when_displayed() {
        for budget in [0, 1, 2, 4] {
            let borrows = Cell::new(0);
            let expected = [1, 2, 3].map(|value| Operand {
                value,
                borrows: &borrows,
            });
            let failures = assert_that!([] as [i32; 0])
                .with_rendering_budget(RenderingBudget::default().with_max_items(budget))
                .capture(|it| it.into_iter_contains_all(expected));
            assert_that!(borrows.get()).is_equal_to(2 * budget.min(3));
            let missing = failures[0]
                .facts
                .iter()
                .find(|fact| fact.label == "Elements not found")
                .unwrap();
            let RenderedBody::Group { items, omitted, .. } = &missing.value.body else {
                panic!("missing values are a group")
            };
            assert_that!(items).has_length(budget.min(3));
            assert_that!(*omitted).is_equal_to(3 - budget.min(3));
            for (index, value) in items.iter().enumerate() {
                assert_that!(rendered_text(value)).is_equal_to((index + 1).to_string());
            }
        }
    }

    #[test]
    fn missing_view_filters_found_operands_in_expected_order() {
        let failures = assert_that!([2]).capture(|it| it.into_iter_contains_all([1, 2, 3]));
        let report = ToHumanReadableText.render(&failures[0]);
        assert_that!(report.as_str())
            .contains("Elements not found: [\n        1,\n        3,\n    ]");
    }
}
