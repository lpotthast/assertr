//! Positional scans: an expected sequence at the start, as the whole input, at the end, or in
//! any contiguous window.
//!
//! Equality and matcher scans share one engine, which checks each expected slot through a
//! family-specific closure. Each family keeps its own report.

use super::{
    AssertionContext, Borrow, FailureBuilder, FailureKind, KnownLength, LengthBound,
    PREVIEW_CAPACITY, PhantomData, RenderingOrder, Scan, Tail, ValueRenderer, consumed_fact,
};
use crate::{
    assertions::collection::Placement,
    borrow_for::{BorrowFor, borrow_for},
    expectation::{Evidence, context::unsatisfied},
    failure::Fact,
    failure::PathSegment,
    matchers::MatcherList,
};

/// Checks one expected slot against the element at a yield position, recording any rejection
/// with that position as its path.
trait SlotCheck<T: ?Sized, R>: FnMut(usize, usize, &T, &mut AssertionContext<'_, R>) -> bool {}

impl<T: ?Sized, R, F> SlotCheck<T, R> for F where
    F: FnMut(usize, usize, &T, &mut AssertionContext<'_, R>) -> bool
{
}

/// What decided a rejected positional scan.
pub(crate) enum End {
    /// The input ended before the expected sequence, or a first complete window.
    Short,
    /// A prefix or exact scan rejected the element at this position.
    Mismatch(usize),
    /// An exact scan found an element after the expected ones.
    Extra,
    /// Every complete window was rejected.
    Windows,
}

/// Why a positional scan rejected its input.
pub(crate) enum Rejection<Item> {
    /// An exact size hint ruled out the expected length before consuming anything.
    Reported(KnownLength),
    Scanned {
        tail: Tail<Item>,
        end: End,
        evidence: Evidence,
    },
}

/// Scans for `len` expected slots at `placement`, retaining at least `retain` of the latest
/// elements.
///
/// Prefix and exact scans compare each visited element once and stop at the first rejection. An
/// exact scan reads at most one element beyond the list. Suffix and contiguous scans evaluate
/// every position of each complete window, with storage independent of the diagnostic budget. An
/// empty window pattern succeeds without consuming anything. A contiguous scan stops at the first
/// matching window. A suffix scan reads the whole input and checks only the final window.
fn scan<T, I, R>(
    iterator: &mut I,
    len: usize,
    placement: Placement,
    retain: usize,
    context: &AssertionContext<'_, R>,
    mut check: impl SlotCheck<T, R>,
) -> Result<(), Rejection<I::Item>>
where
    I: Iterator,
    I::Item: Borrow<T>,
    R: ValueRenderer<usize>,
{
    let mut scope = context.isolated();
    let (tail, end) = match placement {
        Placement::Exact | Placement::Prefix => {
            let bound = if placement == Placement::Exact {
                LengthBound::Exact
            } else {
                LengthBound::AtLeast
            };
            if let Some(known) = KnownLength::mismatch(iterator, len, bound) {
                return Err(Rejection::Reported(known));
            }
            let mut tail = Tail::new(retain);
            let end = 'scan: {
                for index in 0..len {
                    let Some(item) = iterator.next() else {
                        break 'scan End::Short;
                    };
                    let matched = check(index, index, item.borrow(), &mut scope);
                    tail.push(item);
                    if !matched {
                        break 'scan End::Mismatch(index);
                    }
                }
                if placement == Placement::Exact
                    && let Some(item) = iterator.next()
                {
                    tail.push(item);
                    break 'scan End::Extra;
                }
                return Ok(());
            };
            (tail, end)
        }
        Placement::Suffix | Placement::Contiguous => {
            if len == 0 {
                return Ok(());
            }
            let mut tail = Tail::new(len.max(retain));
            let mut window = |tail: &Tail<I::Item>, scope: &mut AssertionContext<'_, R>| {
                tail.consumed >= len
                    && window::<T, I, _>(
                        placement,
                        tail.items.iter().skip(tail.items.len() - len),
                        tail.consumed - len,
                        scope,
                        &mut check,
                    )
            };
            for item in iterator {
                tail.push(item);
                if placement == Placement::Contiguous && window(&tail, &mut scope) {
                    return Ok(());
                }
            }
            if placement == Placement::Suffix && window(&tail, &mut scope) {
                return Ok(());
            }
            let end = if tail.consumed < len {
                End::Short
            } else {
                End::Windows
            };
            (tail, end)
        }
    };
    Err(Rejection::Scanned {
        tail: tail.finish(),
        end,
        evidence: scope.into_evidence(),
    })
}

/// Evaluates every position of one complete window starting at yield index `start`.
///
/// Suffix evidence joins the scan's evidence directly. A rejected contiguous window instead takes
/// one slot as a group, so evidence of overlapping windows does not interleave. Windows beyond the
/// budget are still evaluated, but only counted as omitted. A probe stops at the first rejection.
fn window<'a, T, I, R>(
    placement: Placement,
    items: impl Iterator<Item = &'a I::Item>,
    start: usize,
    windows: &mut AssertionContext<'_, R>,
    check: &mut impl SlotCheck<T, R>,
) -> bool
where
    I: Iterator<Item: 'a>,
    I::Item: Borrow<T>,
    R: ValueRenderer<usize>,
{
    let mut candidate = windows.isolated();
    let mut matched = true;
    for (slot, item) in items.enumerate() {
        matched &= check(slot, start + slot, item.borrow(), &mut candidate);
        if !matched && candidate.is_probe() {
            break;
        }
    }
    if !matched {
        let evidence = candidate.into_evidence();
        if placement == Placement::Suffix {
            windows.append(evidence);
        } else {
            windows.record(|windows| {
                evidence
                    .explain(
                        FailureBuilder::new::<I>(FailureKind::Matching)
                            .relation("does not match in this window")
                            .fact(Fact::labelled(
                                "Window start",
                                windows.render().value(&start),
                            )),
                    )
                    .build()
            });
        }
    }
    matched
}

/// Compares the input with expected values at a placement, retaining an equality preview.
pub(crate) struct ElementsEqual<'e, T, E> {
    expected: &'e [E],
    placement: Placement,
    item: PhantomData<fn() -> T>,
}

impl<'e, T, E> ElementsEqual<'e, T, E> {
    pub(crate) const fn new(expected: &'e [E], placement: Placement) -> Self {
        Self {
            expected,
            placement,
            item: PhantomData,
        }
    }
}

impl<T, E, I, R> Scan<I, R> for ElementsEqual<'_, T, E>
where
    I: Iterator,
    I::Item: Borrow<T>,
    T: PartialEq<E::View>,
    E: BorrowFor<T>,
    R: ValueRenderer<T> + ValueRenderer<E::View> + ValueRenderer<usize>,
{
    type Rejection = Rejection<I::Item>;

    fn kind(&self) -> FailureKind {
        if self.placement == Placement::Exact {
            FailureKind::Equality
        } else {
            FailureKind::Membership
        }
    }

    fn observe(
        &self,
        iterator: &mut I,
        context: &AssertionContext<'_, R>,
    ) -> Result<(), Self::Rejection> {
        let probe;
        let context = if self.placement == Placement::Contiguous {
            // The report shows no window evidence, so windows are only probed.
            probe = context.isolated().with_diagnostics(false);
            &probe
        } else {
            context
        };
        let check = |slot: usize, position, actual: &T, context: &mut AssertionContext<'_, R>| {
            let expected = borrow_for::<T, _>(&self.expected[slot]);
            let matched = actual.eq(expected);
            if !matched {
                // Construct indexed evidence only for retained rejections.
                context.record(|context| {
                    let render = context.render();
                    FailureBuilder::new::<T>(FailureKind::Equality)
                        .actual(render.value(actual))
                        .expected(render.value(expected))
                        .path([PathSegment::Index(position)])
                        .build()
                });
            }
            matched
        };
        let len = self.expected.len();
        scan(
            iterator,
            len,
            self.placement,
            PREVIEW_CAPACITY,
            context,
            check,
        )
    }

    fn explain(
        &self,
        rejection: Self::Rejection,
        failure: FailureBuilder,
        context: &AssertionContext<'_, R>,
    ) -> FailureBuilder {
        let render = context.render();
        let len = self.expected.len();
        let failure =
            failure
                .relation(match self.placement {
                    Placement::Exact => "does not contain exactly",
                    Placement::Prefix => "does not start with",
                    Placement::Suffix => "does not end with",
                    Placement::Contiguous => "does not contain the contiguous subsequence",
                })
                .expected(render.borrowed_values::<E::View, _>(
                    self.expected,
                    RenderingOrder::PreserveIteration,
                ));
        let (tail, end, evidence) = match rejection {
            Rejection::Reported(known) => return known.facts(failure, render),
            Rejection::Scanned {
                tail,
                end,
                evidence,
            } => (tail, end, evidence),
        };
        let decisive = match end {
            End::Mismatch(index) => Some(index),
            _ => None,
        };
        let failure = tail.facts(
            failure.actual(tail.rendered::<T, _>(render)),
            render,
            decisive,
        );
        match end {
            End::Short if self.placement == Placement::Suffix => {
                failure.fact(Fact::labelled("Suffix length", render.value(&len)))
            }
            End::Extra => {
                failure.fact(Fact::labelled("Extra element at index", render.value(&len)))
            }
            _ => evidence.explain(failure),
        }
    }
}

/// Matches the input against a matcher list at a placement, without retaining a preview.
pub(crate) struct ElementsMatch<T, L> {
    expected: L,
    placement: Placement,
    item: PhantomData<fn() -> T>,
}

impl<T, L> ElementsMatch<T, L> {
    pub(crate) const fn new(expected: L, placement: Placement) -> Self {
        Self {
            expected,
            placement,
            item: PhantomData,
        }
    }
}

impl<T, L, I, R> Scan<I, R> for ElementsMatch<T, L>
where
    I: Iterator,
    I::Item: Borrow<T>,
    L: MatcherList<T, R>,
    R: ValueRenderer<usize>,
{
    type Rejection = Rejection<I::Item>;

    fn kind(&self) -> FailureKind {
        FailureKind::Matching
    }

    fn observe(
        &self,
        iterator: &mut I,
        context: &AssertionContext<'_, R>,
    ) -> Result<(), Self::Rejection> {
        let check = |slot, position, actual: &T, context: &mut AssertionContext<'_, R>| {
            context.scoped(PathSegment::Index(position), |context| {
                self.expected.evaluate_at(slot, actual, context)
            })
        };
        let len = self.expected.len();
        scan(iterator, len, self.placement, 0, context, check)
    }

    fn explain(
        &self,
        rejection: Self::Rejection,
        failure: FailureBuilder,
        context: &AssertionContext<'_, R>,
    ) -> FailureBuilder {
        let render = context.render();
        let len = self.expected.len();
        let (tail, end, evidence) = match rejection {
            Rejection::Reported(known) => {
                let failure = failure.relation("does not have the required sequence length");
                return known.facts(failure, render);
            }
            Rejection::Scanned {
                tail,
                end,
                evidence,
            } => (tail, end, evidence),
        };
        let consumed = tail.consumed;
        let sequence = matches!(self.placement, Placement::Exact | Placement::Prefix);
        let suffix = self.placement == Placement::Suffix;
        let (relation, evidence) = match end {
            End::Short if sequence => {
                // Describe the missing position without evaluating its matcher.
                let mut missing = context.isolated();
                missing.scoped(PathSegment::Index(consumed), |slot| {
                    slot.record(|slot| unsatisfied(self.expected.describe_at(consumed, slot)));
                });
                ("is missing a matching position", missing.into_evidence())
            }
            End::Mismatch(_) => ("does not match the required position", evidence),
            End::Extra => ("has an extra element", evidence),
            End::Short | End::Windows if suffix => {
                ("does not end with matching elements", evidence)
            }
            End::Short | End::Windows => ("does not contain these elements contiguously", evidence),
        };
        let failure =
            evidence.explain(consumed_fact(failure.relation(relation), context, consumed));
        match end {
            End::Short if sequence => {
                failure.fact(Fact::labelled("Expected length", render.value(&len)))
            }
            End::Extra => {
                failure.fact(Fact::labelled("Extra element at index", render.value(&len)))
            }
            // Describe incomplete windows without evaluating their matchers or inventing positions.
            End::Short => failure
                .fact(Fact::labelled("Expected length", render.value(&len)))
                .constraint(
                    context
                        .describe_list::<T, _>(
                            &self.expected,
                            FailureBuilder::new::<()>(FailureKind::Matching).relation(if suffix {
                                "ends with these elements"
                            } else {
                                "contains these elements contiguously"
                            }),
                        )
                        .build(),
                ),
            End::Mismatch(_) | End::Windows => failure,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::failure::PathSegment;
    use crate::prelude::*;
    use alloc::vec::Vec;
    use core::cell::Cell;
    use indoc::formatdoc;

    mod evidence_budget {
        use super::*;
        use core::fmt;

        struct Renderer<'a>(&'a Cell<usize>);
        impl ValueRenderer<Compared<'_>> for Renderer<'_> {
            fn fmt(&self, value: &Compared<'_>, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                self.0.set(self.0.get() + 1);
                write!(f, "{}", value.value)
            }
        }
        impl ValueRenderer<usize> for Renderer<'_> {
            fn fmt(&self, value: &usize, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{value}")
            }
        }

        #[test]
        fn suffix_compares_every_position_and_renders_only_retained_evidence() {
            for (length, alternating, budget) in [
                (10_000, false, RenderingBudget::default().with_max_items(0)),
                (10_000, false, RenderingBudget::default().with_max_items(1)),
                (20, true, RenderingBudget::default().with_max_items(1)),
                (20, true, RenderingBudget::unlimited()),
            ] {
                let comparisons = Cell::new(0);
                let renders = Cell::new(0);
                let item = |value| Compared {
                    value,
                    comparisons: &comparisons,
                };
                let expected = (0..length).map(|_| item(9)).collect::<Vec<_>>();
                let actual = (0..length)
                    .map(|index| item(if alternating && index % 2 == 0 { 9 } else { 0 }));
                let failures = assert_that_owned!(actual)
                    .with_renderer(Renderer(&renders))
                    .with_rendering_budget(budget)
                    .capture(|it| it.ends_with(&expected));
                let rejected = if alternating { length / 2 } else { length };
                let retained = rejected.min(budget.max_items());
                assert_that!(comparisons.get()).is_equal_to(length);
                assert_that!(renders.get()).is_equal_to(
                    16.min(budget.max_items()) + length.min(budget.max_items()) + retained * 2,
                );
                assert_that!(failures).has_length(1);
                let failure = &failures[0];
                assert_that!(failure.kind).is_equal_to(FailureKind::Membership);
                assert_that!(failure.children).has_length(retained);
                assert_that!(failure.omitted_children).is_equal_to(rejected - retained);
                for (slot, child) in failure.children.iter().enumerate() {
                    let index = if alternating { slot * 2 + 1 } else { slot };
                    assert_that!(child.path).contains_exactly([PathSegment::Index(index)]);
                    assert_that!(child.kind).is_equal_to(FailureKind::Equality);
                }
            }
        }

        #[test]
        fn prefix_and_exact_suppress_mismatch_children_without_changing_the_observation() {
            for exact in [false, true] {
                let comparisons = Cell::new(0);
                let renders = Cell::new(0);
                let item = |value| Compared {
                    value,
                    comparisons: &comparisons,
                };
                let expected = [item(0), item(9), item(2)];
                let actual = (0..).map(item);
                let failures = assert_that_owned!(actual)
                    .with_renderer(Renderer(&renders))
                    .with_rendering_budget(RenderingBudget::default().with_max_items(0))
                    .capture(|it| {
                        if exact {
                            it.contains_exactly(expected)
                        } else {
                            it.starts_with(expected)
                        }
                    });
                assert_that!(comparisons.get()).is_equal_to(2);
                assert_that!(renders.get()).is_equal_to(0);
                assert_that!(failures).has_length(1);
                assert_that!(failures[0].children).is_empty();
                assert_that!(failures[0].omitted_children).is_equal_to(1);
                assert_that!(failures[0].kind).is_equal_to(if exact {
                    FailureKind::Equality
                } else {
                    FailureKind::Membership
                });
                let report = failures[0].to_string();
                assert_that!(report.as_str())
                    .contains("Consumed elements: 2")
                    .contains("Decisive index: 1");
            }
        }
    }

    mod windows {
        use super::*;

        #[test]
        fn deque_wraparound_preserves_overlaps_and_first_success() {
            for length in [1, 3, 16, 17, 40] {
                let pattern = (0..length).collect::<Vec<_>>();
                let mut iterator = core::iter::repeat_n(99, 77).chain(0..).inspect(|_| {});
                let scan = ElementsEqual::<usize, _>::new(&pattern, Placement::Contiguous);
                assert_that!(
                    scan.observe(&mut iterator, &AssertionContext::default())
                        .is_ok()
                )
                .is_true();
                assert_that!(iterator.next()).is_equal_to(Some(length));
            }
            assert_that_owned!([1, 1, 1, 2].into_iter()).contains_contiguous([1, 1, 2]);
        }

        #[test]
        fn suffix_windows_retain_patterns_longer_than_the_preview() {
            for length in [1, 3, 16, 17, 40] {
                let expected = (80 - length..80).collect::<Vec<_>>();
                assert_that_owned!(0..80).ends_with(expected);
            }
        }

        #[test]
        fn short_and_non_fused_inputs_stop_at_first_exhaustion() {
            for placement in [Placement::Suffix, Placement::Contiguous] {
                for matcher in [false, true] {
                    let mut yielded = [Some(1), None, Some(2)].into_iter();
                    let mut iterator = core::iter::from_fn(|| yielded.next().flatten());
                    let context = AssertionContext::default();
                    let rejection = if matcher {
                        let expected = [crate::matchers::eq(1), crate::matchers::eq(2)];
                        ElementsMatch::<i32, _>::new(expected, placement)
                            .observe(&mut iterator, &context)
                    } else {
                        ElementsEqual::<i32, _>::new(&[1, 2], placement)
                            .observe(&mut iterator, &context)
                    };
                    let Err(Rejection::Scanned {
                        tail,
                        end: End::Short,
                        ..
                    }) = rejection
                    else {
                        panic!("a short input is rejected after scanning")
                    };
                    assert_that!(tail.consumed).is_equal_to(1);
                    assert_that!(iterator.next()).is_equal_to(Some(2));
                }
            }
        }

        #[test]
        fn empty_windows_do_not_advance_infinite_inputs() {
            let mut iterator = 0..;
            let context = AssertionContext::default();
            for placement in [Placement::Suffix, Placement::Contiguous] {
                let scan = ElementsEqual::<i32, i32>::new(&[], placement);
                assert_that!(scan.observe(&mut iterator, &context).is_ok()).is_true();
            }
            assert_that!(iterator.next()).is_equal_to(Some(0));
        }
    }

    #[test]
    fn equality_scans_locate_child_evidence_by_yield_position() {
        let prefix = assert_that_owned!([1, 2, 3].into_iter()).capture(|it| it.starts_with([1, 9]));
        let exact =
            assert_that_owned!([1, 2, 3].into_iter()).capture(|it| it.contains_exactly([1, 9, 3]));
        let suffix = assert_that_owned!([1, 2, 3].into_iter()).capture(|it| it.ends_with([9, 3]));
        for failures in [prefix, exact, suffix] {
            assert_that!(failures).has_length(1);
            assert_that!(failures[0].children).has_length(1);
            let child = &failures[0].children[0];
            assert_that!(child.path).contains_exactly([PathSegment::Index(1)]);
            assert_that!(child.facts).is_empty();
        }
    }

    #[test]
    fn prefix_and_exact_mismatches_retain_the_decisive_element() {
        for (exact, relation) in [
            (false, "does not start with"),
            (true, "does not contain exactly"),
        ] {
            assert_that!(|| {
                let it = assert_that_owned!([1, 2, 3].into_iter()).with_location(false);
                if exact {
                    it.contains_exactly([1, 9, 3]);
                } else {
                    it.starts_with([1, 9, 3]);
                }
            })
            .panics()
            .has_type::<String>()
            .is_equal_to(formatdoc! {"
                -------- assertr --------
                Expression: `[1, 2, 3].into_iter()`

                Actual: [
                    1,
                    2,
                ]

                {relation}

                Expected: [
                    1,
                    9,
                    3,
                ]

                Details:
                  - Consumed elements: 2
                  - Decisive index: 1
                Nested failures:
                  - At [1]:
                    Expected: 9

                      Actual: 2
                -------- assertr --------
            "});
        }
    }

    #[derive(Debug)]
    struct Compared<'a> {
        value: i32,
        comparisons: &'a Cell<usize>,
    }

    impl PartialEq for Compared<'_> {
        fn eq(&self, other: &Self) -> bool {
            self.comparisons.set(self.comparisons.get() + 1);
            self.value == other.value
        }
    }

    #[test]
    fn prefix_and_exact_scans_bound_the_preview_and_compare_each_item_once() {
        for placement in [Placement::Prefix, Placement::Exact] {
            let comparisons = Cell::new(0);
            let item = |value| Compared {
                value,
                comparisons: &comparisons,
            };
            let expected: Vec<_> = (0..19).chain([99]).map(item).collect();
            let scan = ElementsEqual::<Compared<'_>, _>::new(&expected, placement);
            let context = AssertionContext::new(&DebugRenderer, RenderingBudget::default());
            let mut iterator = (0..).map(item);
            let Err(Rejection::Scanned {
                tail,
                end: End::Mismatch(index),
                ..
            }) = scan.observe(&mut iterator, &context)
            else {
                panic!("an unknown length cannot reject before scanning")
            };
            assert_that!(tail.consumed).is_equal_to(20);
            assert_that!(tail.items.iter().map(|item| item.value).collect::<Vec<_>>())
                .contains_exactly((4..20).collect::<Vec<_>>());
            assert_that!(index).is_equal_to(19);
            assert_that!(comparisons.get()).is_equal_to(20);
            assert_that!(iterator.next().map(|item| item.value)).is_equal_to(Some(20));
        }
    }

    mod matchers {
        use crate::{
            expectation::Expectation,
            prelude::*,
            renderer::{Rendered, RenderedBody},
            test_support::CustomValueRenderer,
        };
        use core::cell::Cell;

        use crate::failure::{FailureBuilder, FailureKind};

        mod evidence_budget {
            use super::*;
            use crate::{failure::PathSegment, matchers::eq};

            #[test]
            fn contiguous_retains_the_first_rejected_windows_and_keeps_evaluating_after_budget_exhaustion()
             {
                let failures = assert_that_owned!(0..30)
                    .with_rendering_budget(RenderingBudget::default().with_max_items(1))
                    .capture(|it| it.contains_contiguous_matching([eq(99), eq(99)]));
                // The first of 29 rejected windows is retained as one group. The others are
                // counted.
                assert_that!(failures[0].children).has_length(1);
                let window = &failures[0].children[0];
                assert_that!(window.path).is_empty();
                assert_that!(window.children).has_length(1);
                assert_that!(window.children[0].path).contains_exactly([PathSegment::Index(0)]);
                assert_that!(window.omitted_children).is_equal_to(1);
                assert_that!(failures[0].omitted_children).is_equal_to(28);
                assert_that_owned!(0..)
                    .with_rendering_budget(RenderingBudget::default().with_max_items(0))
                    .contains_contiguous_matching([eq(20), eq(21)]);
            }

            #[test]
            fn suffix_limits_repeated_position_evidence() {
                let failures = assert_that_owned!([1, 2, 3].into_iter())
                    .with_rendering_budget(RenderingBudget::default().with_max_items(1))
                    .with_location(false)
                    .capture(|it| it.ends_with_matching([eq(0), eq(0), eq(0)]));

                assert_that!(failures[0].children).has_length(1);
                assert_that!(failures[0].children[0].path)
                    .contains_exactly([PathSegment::Index(0)]);
                assert_that!(failures[0].omitted_children).is_equal_to(2);
            }
        }

        mod windows {
            use super::*;

            #[test]
            fn wraparound_and_overlapping_windows_preserve_matching_and_stopping_points() {
                for length in [1, 3, 16, 17, 40] {
                    let expected = (0..length).map(matchers::eq).collect::<Vec<_>>();
                    let consumed = Cell::new(0);
                    let actual = core::iter::repeat_n(99, 77)
                        .chain(0..)
                        .inspect(|_| consumed.set(consumed.get() + 1));
                    assert_that_owned!(actual).contains_contiguous_matching(&expected);
                    assert_that!(consumed.get()).is_equal_to(77 + length);
                    assert_that_owned!(core::iter::repeat_n(99, 77).chain(0..length))
                        .ends_with_matching(expected);
                }
                assert_that_owned!([1, 1, 1, 2].into_iter()).contains_contiguous_matching([
                    matchers::eq(1),
                    matchers::eq(1),
                    matchers::eq(2),
                ]);
            }
        }

        struct DescriptionOnly<'a> {
            expected: i32,
            descriptions: &'a Cell<usize>,
        }

        impl<R: ValueRenderer<i32>> Expectation<i32, R> for DescriptionOnly<'_> {
            type Success<'a>
                = ()
            where
                Self: 'a,
                i32: 'a;
            type Rejection<'a>
                = ()
            where
                Self: 'a,
                i32: 'a;
            fn evaluate(&self, _: &i32, _: &AssertionContext<'_, R>) -> Result<(), ()> {
                panic!("an incomplete window must not evaluate matchers")
            }

            const KIND: FailureKind = FailureKind::Matching;
            fn explain(
                &self,
                rejected: Option<(&i32, ())>,
                failure: FailureBuilder,
                context: &AssertionContext<'_, R>,
            ) -> FailureBuilder {
                let render = context.render();
                match rejected {
                    None => {
                        self.descriptions.set(self.descriptions.get() + 1);
                        failure
                            .relation("is equal to")
                            .expected(render.value(&self.expected))
                    }
                    Some((_, ())) => {
                        unreachable!("the test cannot return a rejection")
                    }
                }
            }
        }

        fn assert_truncated_value(value: &Rendered, type_name: &str, omitted_characters: usize) {
            assert_that!(value.type_name).is_equal_to(Some(type_name));
            assert_that!(value.body).is_equal_to(RenderedBody::Text {
                text: "cus".into(),
                omitted_characters,
            });
        }

        fn assert_truncated_lengths(failure: &AssertionFailure) {
            for label in ["Consumed elements", "Expected length"] {
                let fact = failure
                    .facts
                    .iter()
                    .find(|fact| fact.label == label)
                    .unwrap();
                assert_truncated_value(&fact.value, "usize", 6);
            }
        }

        #[test]
        fn missing_positions_respect_renderer_and_budget_without_evaluating_matchers() {
            for exact in [true, false] {
                for maximum in 0..=2 {
                    let evaluations = Cell::new(0);
                    let descriptions = Cell::new(0);
                    let later_descriptions = Cell::new(0);
                    let matchers = crate::matchers![
                        matchers::predicate(|actual: &i32| {
                            evaluations.set(evaluations.get() + 1);
                            *actual == 1
                        }),
                        DescriptionOnly {
                            expected: 987_654,
                            descriptions: &descriptions,
                        },
                        DescriptionOnly {
                            expected: 10,
                            descriptions: &later_descriptions,
                        },
                    ];
                    let failures = assert_that_owned!([1].into_iter().filter(|_| true))
                        .with_renderer(CustomValueRenderer)
                        .with_rendering_budget(
                            RenderingBudget::default()
                                .with_max_items(maximum)
                                .with_max_leaf_characters(3),
                        )
                        .capture(|it| {
                            if exact {
                                it.contains_exactly_matching(matchers)
                            } else {
                                it.starts_with_matching(matchers)
                            }
                        });

                    let retained = maximum.min(1);
                    assert_that!(evaluations.get()).is_equal_to(1);
                    assert_that!(descriptions.get()).is_equal_to(retained);
                    assert_that!(later_descriptions.get()).is_equal_to(0);
                    assert_that!(failures).has_length(1);
                    let failure = &failures[0];
                    assert_that!(failure.omitted_children).is_equal_to(1 - retained);
                    assert_that!(failure.children).has_length(retained);
                    for child in &failure.children {
                        assert_that!(child.path)
                            .is_equal_to([crate::failure::PathSegment::Index(1)]);
                        let constraint = child.constraint.as_deref().unwrap();
                        assert_that!(constraint.relation.as_deref())
                            .is_equal_to(Some("is equal to"));
                        assert_truncated_value(constraint.expected.as_ref().unwrap(), "i32", 11);
                    }
                    assert_truncated_lengths(failure);
                }
            }
        }

        #[test]
        fn short_windows_respect_renderer_and_budget_without_evaluating_matchers() {
            for suffix in [true, false] {
                for maximum in 0..=2 {
                    let descriptions = Cell::new(0);
                    let matchers = [9, 10].map(|expected| DescriptionOnly {
                        expected,
                        descriptions: &descriptions,
                    });
                    let failures = assert_that_owned!([1].into_iter().filter(|_| true))
                        .with_renderer(CustomValueRenderer)
                        .with_rendering_budget(
                            RenderingBudget::default()
                                .with_max_items(maximum)
                                .with_max_leaf_characters(3),
                        )
                        .capture(|it| {
                            if suffix {
                                it.ends_with_matching(matchers)
                            } else {
                                it.contains_contiguous_matching(matchers)
                            }
                        });

                    assert_that!(descriptions.get()).is_equal_to(maximum);
                    assert_that!(failures).has_length(1);
                    let failure = &failures[0];
                    let constraint = failure.constraint.as_deref().unwrap();
                    assert_that!(constraint.omitted_children).is_equal_to(2 - maximum);
                    assert_that!(constraint.children).has_length(maximum);
                    for (index, child) in constraint.children.iter().enumerate() {
                        assert_truncated_value(child.expected.as_ref().unwrap(), "i32", 6 + index);
                    }
                    assert_truncated_lengths(failure);
                }
            }
        }

        #[test]
        fn short_scans_describe_the_missing_constraints_and_observed_length() {
            use crate::test_support::assert_custom_fact;

            // Prefix/exact retain the first missing slot. Suffix/contiguous describe the full
            // missing window. Both cases use the same diagnostics for empty and short iterators.
            for consumed in 0..=1 {
                for operation in 0..4 {
                    let expected = [matchers::eq(1), matchers::eq(987_654)];
                    let mut values = (0..consumed).map(|_| 1);
                    let iterator = core::iter::from_fn(move || values.next());
                    let failures = assert_that_owned!(iterator)
                        .with_renderer(CustomValueRenderer)
                        .capture(|it| match operation {
                            0 => it.starts_with_matching(expected),
                            1 => it.contains_exactly_matching(expected),
                            2 => it.ends_with_matching(expected),
                            _ => it.contains_contiguous_matching(expected),
                        });
                    assert_that!(failures).has_length(1);
                    let failure = &failures[0];
                    assert_custom_fact(failure, "Consumed elements", consumed);
                    assert_custom_fact(failure, "Expected length", 2);
                    if operation < 2 {
                        assert_that!(failure.relation.as_deref())
                            .is_equal_to(Some("is missing a matching position"));
                        assert_that!(failure.children).has_length(1);
                        let child = &failure.children[0];
                        assert_that!(child.path)
                            .is_equal_to([crate::failure::PathSegment::Index(consumed)]);
                        crate::test_support::assert_custom_value(
                            child
                                .constraint
                                .as_deref()
                                .unwrap()
                                .expected
                                .as_ref()
                                .unwrap(),
                            &if consumed == 0 { 1_i32 } else { 987_654_i32 },
                        );
                    } else {
                        let constraint = failure.constraint.as_deref().unwrap();
                        assert_that!(constraint.relation.as_deref()).is_equal_to(Some(
                            if operation == 2 {
                                "ends with these elements"
                            } else {
                                "contains these elements contiguously"
                            },
                        ));
                        assert_that!(constraint.children).has_length(2);
                        for (child, value) in constraint.children.iter().zip([1_i32, 987_654]) {
                            crate::test_support::assert_custom_value(
                                child.expected.as_ref().unwrap(),
                                &value,
                            );
                        }
                    }
                }
            }
        }
    }
}
