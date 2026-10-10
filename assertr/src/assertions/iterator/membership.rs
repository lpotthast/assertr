use super::{
    AssertionContext, Borrow, FailureBuilder, FailureKind, PREVIEW_CAPACITY, PhantomData,
    PositionReporting, RenderingOrder, Scan, Tail, ValueRenderer, Vec, consumed_fact,
};
use crate::{
    assertions::{HasLength, collection::Collection},
    borrow_for::{BorrowFor, borrow_for},
    expectation::{Evidence, Expectation},
    failure::{Fact, PathSegment},
    renderer::CollectionPresentation,
};

/// Borrows only those stored operands selected for display by the rendering context.
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

/// Requires an element equal to a borrowed view, stopping at the first match.
pub(crate) struct Contains<'e, T, E: ?Sized> {
    expected: &'e E,
    item: PhantomData<fn() -> T>,
}

impl<'e, T, E: ?Sized> Contains<'e, T, E> {
    pub(crate) const fn new(expected: &'e E) -> Self {
        Self {
            expected,
            item: PhantomData,
        }
    }
}

impl<T, E: ?Sized, I, R> Scan<I, R> for Contains<'_, T, E>
where
    I: Iterator,
    I::Item: Borrow<T>,
    T: PartialEq<E>,
    R: ValueRenderer<T> + ValueRenderer<E> + ValueRenderer<usize>,
{
    type Rejection = Tail<I::Item>;

    fn kind(&self) -> FailureKind {
        FailureKind::Membership
    }

    fn observe(
        &self,
        iterator: &mut I,
        _context: &AssertionContext<'_, R>,
    ) -> Result<(), Self::Rejection> {
        let mut tail = Tail::new(PREVIEW_CAPACITY);
        for item in iterator {
            let matched = item.borrow().eq(self.expected);
            tail.push(item);
            if matched {
                return Ok(());
            }
        }
        Err(tail.finish())
    }

    fn explain(
        &self,
        tail: Self::Rejection,
        failure: FailureBuilder,
        context: &AssertionContext<'_, R>,
    ) -> FailureBuilder {
        let render = context.render();
        let failure = failure
            .actual(tail.rendered::<T, _>(render))
            .relation("does not contain")
            .expected(render.value(self.expected));
        tail.facts(failure, render, None)
    }
}

/// Requires every expected value to occur, stopping once all have been found.
pub(crate) struct ContainsAll<'e, T, E> {
    expected: &'e [E],
    item: PhantomData<fn() -> T>,
}

impl<'e, T, E> ContainsAll<'e, T, E> {
    pub(crate) const fn new(expected: &'e [E]) -> Self {
        Self {
            expected,
            item: PhantomData,
        }
    }
}

impl<T, E, I, R> Scan<I, R> for ContainsAll<'_, T, E>
where
    I: Iterator,
    I::Item: Borrow<T>,
    T: PartialEq<E::View>,
    E: BorrowFor<T>,
    R: ValueRenderer<T> + ValueRenderer<E::View> + ValueRenderer<usize>,
{
    type Rejection = (Tail<I::Item>, Vec<bool>, usize);

    fn kind(&self) -> FailureKind {
        FailureKind::Membership
    }

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
        let mut tail = Tail::new(PREVIEW_CAPACITY);
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

    fn explain(
        &self,
        (tail, found, remaining): Self::Rejection,
        failure: FailureBuilder,
        context: &AssertionContext<'_, R>,
    ) -> FailureBuilder {
        let render = context.render();
        let missing = Missing {
            expected: self.expected,
            found: &found,
            remaining,
        };
        let failure = failure
            .actual(tail.rendered::<T, _>(render))
            .relation("does not contain all of")
            .expected(
                render.borrowed_values::<E::View, _>(
                    self.expected,
                    RenderingOrder::PreserveIteration,
                ),
            )
            .fact(Fact::labelled(
                "Elements not found",
                render.borrowed_values::<E::View, _>(&missing, RenderingOrder::PreserveIteration),
            ));
        tail.facts(failure, render, None)
    }
}

/// Requires no element equal to a borrowed view, stopping at the first match.
pub(crate) struct DoesNotContain<'e, T, E: ?Sized> {
    expected: &'e E,
    positions: PositionReporting,
    item: PhantomData<fn() -> T>,
}

impl<'e, T, E: ?Sized> DoesNotContain<'e, T, E> {
    pub(crate) const fn new(expected: &'e E, positions: PositionReporting) -> Self {
        Self {
            expected,
            positions,
            item: PhantomData,
        }
    }
}

impl<T, E: ?Sized, I, R> Scan<I, R> for DoesNotContain<'_, T, E>
where
    I: Iterator,
    I::Item: Borrow<T>,
    T: PartialEq<E>,
    R: ValueRenderer<T> + ValueRenderer<E> + ValueRenderer<usize>,
{
    type Rejection = (Tail<I::Item>, usize);

    fn kind(&self) -> FailureKind {
        FailureKind::Membership
    }

    fn observe(
        &self,
        iterator: &mut I,
        _context: &AssertionContext<'_, R>,
    ) -> Result<(), Self::Rejection> {
        let mut tail = Tail::new(PREVIEW_CAPACITY);
        for (index, item) in iterator.enumerate() {
            let matched = item.borrow().eq(self.expected);
            tail.push(item);
            if matched {
                return Err((tail.finish(), index));
            }
        }
        Ok(())
    }

    fn explain(
        &self,
        (tail, index): Self::Rejection,
        failure: FailureBuilder,
        context: &AssertionContext<'_, R>,
    ) -> FailureBuilder {
        let render = context.render();
        let failure = failure
            .actual(tail.rendered::<T, _>(render))
            .relation("contains")
            .unexpected(render.value(self.expected));
        tail.facts(failure, render, self.positions.index(index))
    }
}

/// Requires an element matching an expectation, stopping at the first match.
pub(crate) struct ContainsMatching<T, P> {
    expected: P,
    positions: PositionReporting,
    item: PhantomData<fn() -> T>,
}

impl<T, P> ContainsMatching<T, P> {
    pub(crate) const fn new(expected: P, positions: PositionReporting) -> Self {
        Self {
            expected,
            positions,
            item: PhantomData,
        }
    }
}

impl<T, P, I, R> Scan<I, R> for ContainsMatching<T, P>
where
    I: Iterator,
    I::Item: Borrow<T>,
    P: Expectation<T, R>,
    R: ValueRenderer<usize>,
{
    type Rejection = (Evidence, usize);

    fn kind(&self) -> FailureKind {
        FailureKind::Matching
    }

    fn observe(
        &self,
        iterator: &mut I,
        context: &AssertionContext<'_, R>,
    ) -> Result<(), Self::Rejection> {
        // Candidates share one scope. Once its budget is full, later rejections are only counted,
        // so rendering stays bounded even when a late candidate succeeds.
        let mut candidates = context.isolated();
        let mut consumed = 0;
        for item in iterator {
            let accepted = match self.positions.index(consumed) {
                Some(index) => candidates.scoped(PathSegment::Index(index), |candidate| {
                    candidate.evaluate(item.borrow(), &self.expected)
                }),
                None => candidates.evaluate(item.borrow(), &self.expected),
            };
            consumed += 1;
            if accepted {
                return Ok(());
            }
        }
        candidates
            .finish(false, |candidates| candidates.describe(&self.expected))
            .map_err(|evidence| (evidence, consumed))
    }

    fn explain(
        &self,
        (evidence, consumed): Self::Rejection,
        failure: FailureBuilder,
        context: &AssertionContext<'_, R>,
    ) -> FailureBuilder {
        let failure = failure.relation("does not contain a matching element");
        evidence.explain(consumed_fact(failure, context, consumed))
    }
}

/// Requires no element matching an expectation, stopping at the first match.
pub(crate) struct DoesNotContainMatching<T, P> {
    expected: P,
    positions: PositionReporting,
    item: PhantomData<fn() -> T>,
}

impl<T, P> DoesNotContainMatching<T, P> {
    pub(crate) const fn new(expected: P, positions: PositionReporting) -> Self {
        Self {
            expected,
            positions,
            item: PhantomData,
        }
    }
}

impl<T, P, I, R> Scan<I, R> for DoesNotContainMatching<T, P>
where
    I: Iterator,
    I::Item: Borrow<T>,
    P: Expectation<T, R>,
    R: ValueRenderer<usize> + ValueRenderer<T>,
{
    type Rejection = (Evidence, usize);

    fn kind(&self) -> FailureKind {
        FailureKind::Matching
    }

    fn observe(
        &self,
        iterator: &mut I,
        context: &AssertionContext<'_, R>,
    ) -> Result<(), Self::Rejection> {
        for (index, item) in iterator.enumerate() {
            let mut child = context.isolated();
            if child.probe(item.borrow(), &self.expected) {
                child.record(|child| {
                    FailureBuilder::new::<T>(FailureKind::Membership)
                        .actual(child.render().value(item.borrow()))
                        .relation("matches the unwanted constraint")
                        .constraint(child.describe(&self.expected))
                        .path(self.positions.index(index).map(PathSegment::Index))
                        .build()
                });
                return Err((child.into_evidence(), index + 1));
            }
        }
        Ok(())
    }

    fn explain(
        &self,
        (evidence, consumed): Self::Rejection,
        failure: FailureBuilder,
        context: &AssertionContext<'_, R>,
    ) -> FailureBuilder {
        let failure = failure.relation("contains an unexpected matching element");
        evidence.explain(consumed_fact(failure, context, consumed))
    }
}

#[cfg(test)]
mod tests {
    use core::cell::Cell;

    use super::*;
    use crate::{prelude::*, renderer::RenderedBody};

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
                assert_that!(format!("{value:#}")).is_equal_to((index + 1).to_string());
            }
        }
    }

    #[test]
    fn missing_view_filters_found_operands_in_expected_order() {
        let failures = assert_that!([2]).capture(|it| it.into_iter_contains_all([1, 2, 3]));
        let report = failures[0].to_string();
        assert_that!(report.as_str())
            .contains("Elements not found: [\n        1,\n        3,\n    ]");
    }

    mod matcher_budget {
        use core::cell::Cell;

        use crate::{failure::PathSegment, matchers::eq, prelude::*};

        #[test]
        fn membership_retains_the_first_rejected_candidates_or_the_empty_fallback() {
            for count in [0, 1, 2, 3, 20] {
                for budget in [0, 1, 32] {
                    let failures = assert_that_owned!(0..count)
                        .with_rendering_budget(RenderingBudget::default().with_max_items(budget))
                        .capture(|it| it.contains_matching(eq(99)));
                    let retained = count.max(1).min(budget);
                    assert_that!(failures).has_length(1);
                    assert_that!(failures[0].children).has_length(retained);
                    assert_that!(failures[0].omitted_children).is_equal_to(count.max(1) - retained);
                    if count > 0 {
                        for (index, child) in failures[0].children.iter().enumerate() {
                            assert_that!(child.path).contains_exactly([PathSegment::Index(index)]);
                        }
                    }
                }
            }
        }

        #[test]
        fn rejected_candidates_render_within_the_budget_when_a_later_candidate_matches() {
            use core::fmt;

            struct Counting<'a>(&'a Cell<usize>);
            impl ValueRenderer<i32> for Counting<'_> {
                fn fmt(&self, value: &i32, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                    self.0.set(self.0.get() + 1);
                    write!(f, "{value}")
                }
            }
            impl ValueRenderer<usize> for Counting<'_> {
                fn fmt(&self, value: &usize, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                    self.0.set(self.0.get() + 1);
                    write!(f, "{value}")
                }
            }

            // Each retained window group also renders its start.
            for (budget, window_renders) in [(0, 0), (1, 3), (4, 18)] {
                let renders = Cell::new(0);
                assert_that_owned!(0..10_000)
                    .with_renderer(Counting(&renders))
                    .with_rendering_budget(RenderingBudget::default().with_max_items(budget))
                    .contains_matching(eq(9_999));
                // Each retained equality rejection renders its expected and actual value.
                assert_that!(renders.get()).is_equal_to(2 * budget);

                let renders = Cell::new(0);
                assert_that_owned!(0..10_000)
                    .with_renderer(Counting(&renders))
                    .with_rendering_budget(RenderingBudget::default().with_max_items(budget))
                    .contains_contiguous_matching([eq(9_998), eq(9_999)]);
                assert_that!(renders.get()).is_equal_to(window_renders);
            }
        }
    }
}
