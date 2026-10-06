use super::{
    AssertThat, AssertionContext, Borrow, FailureBuilder, FailureKind, GroupStyle, KnownLength,
    LengthBound, Mode, PREVIEW_CAPACITY, PhantomData, Preview, Scan, Tail, ValueRenderer,
    WindowPlacement, equal_element, execute, scan_windows,
};
use crate::borrow_for::{BorrowFor, borrow_for};
use crate::expectation::Evidence;
use crate::failure::Fact;

/// What ended an equality prefix or exact scan before it could succeed.
enum SequenceFailure {
    Exhausted,
    Criterion { index: usize, evidence: Evidence },
    Extra { index: usize },
}

impl SequenceFailure {
    fn decisive_index(&self) -> Option<usize> {
        match self {
            Self::Criterion { index, .. } => Some(*index),
            // The extra element's own fact already names its index.
            Self::Extra { .. } | Self::Exhausted => None,
        }
    }
}

/// Why an equality prefix or exact scan rejected its input.
enum SequenceRejection<Item> {
    /// An exact size hint ruled out the expected length before consuming anything.
    Reported(KnownLength),
    Scanned(Preview<Item>, SequenceFailure),
}

// The policy is constant because exact equality and prefix membership have different failure kinds.
struct ElementsEqual<'e, T, E, const EXACT: bool> {
    expected: &'e [E],
    item: PhantomData<fn() -> T>,
}

impl<T, E, I, R, const EXACT: bool> Scan<I, R> for ElementsEqual<'_, T, E, EXACT>
where
    I: Iterator,
    I::Item: Borrow<T>,
    T: PartialEq<E::View>,
    E: BorrowFor<T>,
    R: ValueRenderer<T> + ValueRenderer<E::View> + ValueRenderer<usize>,
{
    type Rejection = SequenceRejection<I::Item>;
    fn observe(
        &self,
        iterator: &mut I,
        context: &AssertionContext<'_, R>,
    ) -> Result<(), Self::Rejection> {
        let expected_len = self.expected.len();
        let bound = if EXACT {
            LengthBound::Exact
        } else {
            LengthBound::AtLeast
        };
        if let Some(known) = KnownLength::mismatch(iterator, expected_len, bound) {
            return Err(SequenceRejection::Reported(known));
        }
        let mut tail = Tail::new(PREVIEW_CAPACITY);
        let mut child = context.isolated();
        for (index, expected) in self.expected.iter().enumerate() {
            let Some(item) = iterator.next() else {
                return Err(SequenceRejection::Scanned(
                    tail.finish(),
                    SequenceFailure::Exhausted,
                ));
            };
            let matched = equal_element(
                &mut child,
                index,
                item.borrow(),
                borrow_for::<T, _>(expected),
            );
            tail.push(item);
            if !matched {
                return Err(SequenceRejection::Scanned(
                    tail.finish(),
                    SequenceFailure::Criterion {
                        index,
                        evidence: child.into_evidence(),
                    },
                ));
            }
        }
        if EXACT && let Some(item) = iterator.next() {
            tail.push(item);
            return Err(SequenceRejection::Scanned(
                tail.finish(),
                SequenceFailure::Extra {
                    index: expected_len,
                },
            ));
        }
        Ok(())
    }

    const KIND: FailureKind = if EXACT {
        FailureKind::Equality
    } else {
        FailureKind::Membership
    };

    fn explain<Target>(
        &self,
        rejection: Self::Rejection,
        failure: FailureBuilder<Target>,
        context: &AssertionContext<'_, R>,
    ) -> FailureBuilder<Target> {
        let render = context.render();
        let failure = failure
            .relation(if EXACT {
                "does not contain exactly"
            } else {
                "does not start with"
            })
            .expected(render.borrowed_values::<E::View, _>(self.expected, GroupStyle::List));
        let (preview, outcome) = match rejection {
            SequenceRejection::Reported(known) => return known.facts(failure, render),
            SequenceRejection::Scanned(preview, outcome) => (preview, outcome),
        };
        let failure = failure.actual(preview.rendered::<T, _>(render));
        let failure = preview.facts(failure, render, outcome.decisive_index());
        match outcome {
            SequenceFailure::Exhausted => failure,
            SequenceFailure::Extra { index } => failure.fact(Fact::labelled(
                "Extra element at index",
                render.value(&index),
            )),
            SequenceFailure::Criterion { evidence, .. } => evidence.explain(failure),
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
    T: PartialEq<E::View>,
    E: BorrowFor<T>,
    R: ValueRenderer<T> + ValueRenderer<E::View> + ValueRenderer<usize>,
{
    execute(
        this,
        iterator,
        &ElementsEqual::<T, E, true> {
            expected,
            item: PhantomData,
        },
    );
}

#[track_caller]
pub(crate) fn assert_starts_with<S, T, E, I, M: Mode, R>(
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
        &ElementsEqual::<T, E, false> {
            expected,
            item: PhantomData,
        },
    );
}

enum SuffixFailure {
    Short,
    Mismatch(Evidence),
}

struct EndsWith<'e, T, E> {
    expected: &'e [E],
    item: PhantomData<fn() -> T>,
}

impl<T, E, I, R> Scan<I, R> for EndsWith<'_, T, E>
where
    I: Iterator,
    I::Item: Borrow<T>,
    T: PartialEq<E::View>,
    E: BorrowFor<T>,
    R: ValueRenderer<T> + ValueRenderer<E::View> + ValueRenderer<usize>,
{
    type Rejection = (Preview<I::Item>, SuffixFailure);
    fn observe(
        &self,
        iterator: &mut I,
        context: &AssertionContext<'_, R>,
    ) -> Result<(), Self::Rejection> {
        let mut child = context.isolated();
        let scanned = scan_windows(
            iterator,
            self.expected.len(),
            PREVIEW_CAPACITY,
            WindowPlacement::End,
            |window, first_index| {
                let mut matched = true;
                for (offset, (item, expected)) in window.zip(self.expected).enumerate() {
                    matched &= equal_element(
                        &mut child,
                        first_index + offset,
                        item.borrow(),
                        borrow_for::<T, _>(expected),
                    );
                }
                matched
            },
        );
        scanned.map_err(|tail| {
            let outcome = if tail.consumed < self.expected.len() {
                SuffixFailure::Short
            } else {
                SuffixFailure::Mismatch(child.into_evidence())
            };
            (tail.finish(), outcome)
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
        let expected = render.borrowed_values::<E::View, _>(self.expected, GroupStyle::List);
        let (preview, outcome) = rejection;
        let failure = failure
            .actual(preview.rendered::<T, _>(render))
            .relation("does not end with")
            .expected(expected);
        let failure = preview.facts(failure, render, None);
        match outcome {
            SuffixFailure::Short => failure.fact(Fact::labelled(
                "Suffix length",
                render.value(&self.expected.len()),
            )),
            SuffixFailure::Mismatch(evidence) => evidence.explain(failure),
        }
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
    T: PartialEq<E::View>,
    E: BorrowFor<T>,
    R: ValueRenderer<T> + ValueRenderer<E::View> + ValueRenderer<usize>,
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

struct ContainsContiguous<'e, T, E> {
    expected: &'e [E],
    item: PhantomData<fn() -> T>,
}

impl<T, E, I, R> Scan<I, R> for ContainsContiguous<'_, T, E>
where
    I: Iterator,
    I::Item: Borrow<T>,
    T: PartialEq<E::View>,
    E: BorrowFor<T>,
    R: ValueRenderer<T> + ValueRenderer<E::View> + ValueRenderer<usize>,
{
    type Rejection = Preview<I::Item>;
    fn observe(
        &self,
        iterator: &mut I,
        _context: &AssertionContext<'_, R>,
    ) -> Result<(), Self::Rejection> {
        scan_windows(
            iterator,
            self.expected.len(),
            PREVIEW_CAPACITY,
            WindowPlacement::Anywhere,
            |window, _| {
                window
                    .zip(self.expected)
                    .all(|(item, expected)| item.borrow().eq(borrow_for::<T, _>(expected)))
            },
        )
        .map_err(Tail::finish)
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
        let preview = rejection;
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
    T: PartialEq<E::View>,
    E: BorrowFor<T>,
    R: ValueRenderer<T> + ValueRenderer<E::View> + ValueRenderer<usize>,
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::failure::PathSegment;
    use crate::{
        prelude::*,
        test_support::{CustomValueRenderer, assert_custom_fact},
    };
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
                let report = ToHumanReadableText.render(&failures[0]);
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
                let scan = ContainsContiguous::<usize, _> {
                    expected: &pattern,
                    item: PhantomData,
                };
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
            for suffix in [false, true] {
                let mut yielded = [Some(1), None, Some(2)].into_iter();
                let mut iterator = core::iter::from_fn(|| yielded.next().flatten());
                let expected = [1, 2];
                let context = AssertionContext::default();
                if suffix {
                    let scan = EndsWith::<i32, _> {
                        expected: &expected,
                        item: PhantomData,
                    };
                    let (preview, outcome) = scan.observe(&mut iterator, &context).err().unwrap();
                    assert_that!(preview.consumed).is_equal_to(1);
                    assert_that!(matches!(outcome, SuffixFailure::Short)).is_true();
                } else {
                    let scan = ContainsContiguous::<i32, _> {
                        expected: &expected,
                        item: PhantomData,
                    };
                    let preview = scan.observe(&mut iterator, &context).err().unwrap();
                    assert_that!(preview.consumed).is_equal_to(1);
                }
                assert_that!(iterator.next()).is_equal_to(Some(2));
            }
        }

        #[test]
        fn empty_windows_do_not_advance_infinite_inputs() {
            let mut iterator = 0..;
            let context = AssertionContext::default();
            let suffix = EndsWith::<i32, i32> {
                expected: &[],
                item: PhantomData,
            };
            let contiguous = ContainsContiguous::<i32, i32> {
                expected: &[],
                item: PhantomData,
            };
            assert_that!(suffix.observe(&mut iterator, &context).is_ok()).is_true();
            assert_that!(contiguous.observe(&mut iterator, &context).is_ok()).is_true();
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
            assert_that_panic_by(|| {
                let it = assert_that_owned!([1, 2, 3].into_iter()).with_location(false);
                if exact {
                    it.contains_exactly([1, 9, 3]);
                } else {
                    it.starts_with([1, 9, 3]);
                }
            })
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

    #[test]
    fn known_lengths_reject_without_consumption_and_render_numeric_evidence() {
        for (exact, relation, kind) in [
            (false, "does not start with", FailureKind::Membership),
            (true, "does not contain exactly", FailureKind::Equality),
        ] {
            let failures = assert_that_owned!([1, 2].into_iter())
                .with_renderer(CustomValueRenderer)
                .with_location(false)
                .capture(|it| {
                    if exact {
                        it.contains_exactly([1, 2, 3])
                    } else {
                        it.starts_with([1, 2, 3])
                    }
                });
            assert_that!(failures).contains_exactly_satisfying([
                |failure: AssertThat<AssertionFailure, Capture>| {
                    failure
                        .derive_owned(AssertionFailure::kind)
                        .is_equal_to(kind);
                    assert_custom_fact(failure.actual(), "Reported length", 2);
                    assert_custom_fact(failure.actual(), "Expected length", 3);
                    failure.has_text_report(formatdoc! {"
                        -------- assertr --------
                        Expression: `[1, 2].into_iter()`

                        {relation}

                        Expected: [
                            custom(1),
                            custom(2),
                            custom(3),
                        ]

                        Details:
                          - Reported length: custom(2)
                          - Expected length: custom(3)
                        -------- assertr --------
                    "});
                },
            ]);
        }
    }

    #[test]
    fn positional_facts_render_indexes_through_the_active_renderer() {
        let capture = |expected: &'static [i32]| {
            assert_that_owned!([1, 2, 3].into_iter().filter(|_| true))
                .with_renderer(CustomValueRenderer)
                .capture(|it| it.contains_exactly(expected))
        };

        let extra = capture(&[1, 2]);
        assert_custom_fact(&extra[0], "Extra element at index", 2);
        let labels = extra[0]
            .facts
            .iter()
            .map(|fact| fact.label.as_ref())
            .collect::<Vec<_>>();
        assert_that!(labels).contains_exactly(["Consumed elements", "Extra element at index"]);

        let mismatch = capture(&[1, 9, 3]);
        assert_custom_fact(&mismatch[0], "Decisive index", 1);
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
        fn check<const EXACT: bool>() {
            let comparisons = Cell::new(0);
            let item = |value| Compared {
                value,
                comparisons: &comparisons,
            };
            let expected: Vec<_> = (0..19).chain([99]).map(item).collect();
            let scan = ElementsEqual::<Compared<'_>, _, EXACT> {
                expected: &expected,
                item: PhantomData,
            };
            let context = AssertionContext::new(&DebugRenderer, RenderingBudget::default());
            let mut iterator = (0..).map(item);
            let SequenceRejection::Scanned(preview, outcome) =
                scan.observe(&mut iterator, &context).unwrap_err()
            else {
                panic!("an unknown length cannot reject before scanning")
            };
            assert_that!(preview.consumed).is_equal_to(20);
            assert_that!(
                preview
                    .items
                    .iter()
                    .map(|item| item.value)
                    .collect::<Vec<_>>()
            )
            .contains_exactly((4..20).collect::<Vec<_>>());
            assert_that!(outcome.decisive_index()).is_equal_to(Some(19));
            assert_that!(comparisons.get()).is_equal_to(20);
            assert_that!(iterator.next().map(|item| item.value)).is_equal_to(Some(20));
        }
        check::<false>();
        check::<true>();
    }
}
